#include "scheduler.h"

#include "../features/feature.h"
#include <stdlib.h>

#define INITIAL_CAPACITY 10
#define SCHEDULER_CLOSING ((LONG) 0x80000000u)
#define SCHEDULER_REF_MASK 0x7fffffffL

static _Thread_local Scheduler *current_callback_scheduler;
static _Thread_local SchedulerTask *current_callback_task;

static bool scheduler_acquire(Scheduler *scheduler) {
    LONG state = InterlockedCompareExchange(&scheduler->operation_state, 0, 0);
    while (!(state & SCHEDULER_CLOSING)) {
        if ((state & SCHEDULER_REF_MASK) == SCHEDULER_REF_MASK) return false;
        LONG previous = InterlockedCompareExchange(&scheduler->operation_state,
                                                   state + 1,
                                                   state);
        if (previous == state) return true;
        state = previous;
    }
    return false;
}

static void scheduler_close_admission(Scheduler *scheduler) {
    LONG state = InterlockedCompareExchange(&scheduler->operation_state, 0, 0);
    while (!(state & SCHEDULER_CLOSING)) {
        LONG previous = InterlockedCompareExchange(&scheduler->operation_state,
                                                   state | SCHEDULER_CLOSING,
                                                   state);
        if (previous == state) return;
        state = previous;
    }
}

// Called with scheduler->lock held. Releases the operation and the lock.
static void scheduler_unlock(Scheduler *scheduler) {
    LONG state = InterlockedDecrement(&scheduler->operation_state);
    WakeAllConditionVariable(&scheduler->task_done_cond);
    LeaveCriticalSection(&scheduler->lock);
    if ((state & SCHEDULER_CLOSING) &&
        (state & SCHEDULER_REF_MASK) == 0) {
        DeleteCriticalSection(&scheduler->lock);
    }
}

static void scheduler_release(Scheduler *scheduler) {
    EnterCriticalSection(&scheduler->lock);
    scheduler_unlock(scheduler);
}

static void detach_task_locked(Scheduler *scheduler, SchedulerTask *task) {
    for (int i = 0; i < scheduler->task_count; i++) {
        if (scheduler->tasks[i] == task) {
            scheduler->tasks[i] = scheduler->tasks[scheduler->task_count - 1];
            scheduler->task_count--;
            return;
        }
    }
}

static FILETIME relative_due_time(DWORD interval_ms) {
    ULARGE_INTEGER due_time;
    due_time.QuadPart = (ULONGLONG) -((LONGLONG)interval_ms * 10000LL);

    FILETIME result;
    result.dwHighDateTime = due_time.HighPart;
    result.dwLowDateTime = due_time.LowPart;
    return result;
}

// Callback wrapper for threadpool timers
static VOID CALLBACK threadpool_timer_callback(PTP_CALLBACK_INSTANCE instance,
                                               PVOID context,
                                               PTP_TIMER timer) {
    (void)instance;
    (void)timer;

    SchedulerTask *task = context;
    if (!task) return;

    Scheduler *scheduler = task->parent_scheduler;
    if (!scheduler_acquire(scheduler)) return;

    EnterCriticalSection(&scheduler->lock);
    if (scheduler->shutting_down || !task->is_active) {
        scheduler_unlock(scheduler);
        return;
    }
    task->callback_thread_id = GetCurrentThreadId();
    scheduler->callbacks_running++;
    SchedulerCallback callback = task->callback;
    void *user_data = task->user_data;
    LeaveCriticalSection(&scheduler->lock);

    Scheduler *previous_scheduler = current_callback_scheduler;
    SchedulerTask *previous_task = current_callback_task;
    current_callback_scheduler = scheduler;
    current_callback_task = task;
    callback(user_data);
    current_callback_scheduler = previous_scheduler;
    current_callback_task = previous_task;

    EnterCriticalSection(&scheduler->lock);
    task->callback_thread_id = 0;
    if (task->is_active) {
        if (task->type == TASK_AT_TIME) {
            task->is_active = false;
            scheduler->active_task_count--;
            task->deferred_free = true;
            detach_task_locked(scheduler, task);
        } else if (!scheduler->shutting_down) {
            FILETIME due_time = relative_due_time(task->interval_ms);
            SetThreadpoolTimer(task->threadpool_timer, &due_time, 0, 0);
        }
    }
    bool deferred_free = task->deferred_free;
    HANDLE timer_handle = task->threadpool_timer;
    if (deferred_free) {
        LeaveCriticalSection(&scheduler->lock);
        if (timer_handle) CloseThreadpoolTimer(timer_handle);
        free(task);
        EnterCriticalSection(&scheduler->lock);
        scheduler->callbacks_running--;
        WakeAllConditionVariable(&scheduler->task_done_cond);
        scheduler_unlock(scheduler);
        return;
    }
    scheduler->callbacks_running--;
    scheduler_unlock(scheduler);
}

// Initialize scheduler
bool scheduler_init(Scheduler *scheduler) {
    if (!scheduler) return false;

    scheduler->tasks = (SchedulerTask **) malloc(sizeof(SchedulerTask *) * INITIAL_CAPACITY);
    if (!scheduler->tasks) {
        return false;
    }

    scheduler->task_count = 0;
    scheduler->active_task_count = 0;
    scheduler->callbacks_running = 0;
    scheduler->removals_pending = 0;
    scheduler->capacity = INITIAL_CAPACITY;
    scheduler->next_id = 0;
    scheduler->shutting_down = false;
    scheduler->operation_state = 0;
    InitializeCriticalSection(&scheduler->lock);
    InitializeConditionVariable(&scheduler->task_done_cond);

    return true;
}

static bool ensure_capacity(Scheduler *scheduler) {
    if (scheduler->task_count >= scheduler->capacity) {
        int new_capacity = scheduler->capacity * 2;
        SchedulerTask **new_tasks = realloc(scheduler->tasks,
                                            sizeof(SchedulerTask *) * new_capacity);
        if (!new_tasks) {
            return false;
        }
        scheduler->tasks = new_tasks;
        scheduler->capacity = new_capacity;
    }
    return true;
}

int scheduler_add_interval_feature(Scheduler *scheduler,
                                   Feature *feature,
                                   void *params,
                                   DWORD interval_ms) {
    if (!feature || !feature->execute) return -1;
    return scheduler_add_interval_task(scheduler,
                                       feature->execute,
                                       params,
                                       interval_ms);
}

static int add_task_common(Scheduler *scheduler, SchedulerTask *task) {
    // MUST be called with scheduler->lock held.
    if (scheduler->shutting_down) {
        free(task);
        return -1;
    }

    if (!ensure_capacity(scheduler)) {
        free(task);
        return -1;
    }

    int task_id = scheduler->next_id++;
    task->id = task_id;
    task->is_active = true;
    
    // Create threadpool timer
    task->threadpool_timer = CreateThreadpoolTimer(threadpool_timer_callback, task, NULL);
    if (!task->threadpool_timer) {
        free(task);
        return -1;
    }

    scheduler->tasks[scheduler->task_count] = task;
    scheduler->task_count++;
    scheduler->active_task_count++;
    
    return task_id;
}

int scheduler_add_interval_task(Scheduler *scheduler,
                                SchedulerCallback callback,
                                void *user_data,
                                DWORD interval_ms) {
    if (!scheduler || !callback || interval_ms == 0 || !scheduler_acquire(scheduler)) return -1;

    EnterCriticalSection(&scheduler->lock);
    if (scheduler->shutting_down) {
        scheduler_unlock(scheduler);
        return -1;
    }

    SchedulerTask *task = calloc(1, sizeof(SchedulerTask));
    if (!task) {
        scheduler_unlock(scheduler);
        return -1;
    }

    task->type = TASK_INTERVAL;
    task->parent_scheduler = scheduler;
    task->callback = callback;
    task->user_data = user_data;
    task->interval_ms = interval_ms;

    int task_id = add_task_common(scheduler, task);
    if (task_id == -1) {
        scheduler_unlock(scheduler);
        return -1;
    }

    FILETIME due_time = relative_due_time(interval_ms);

    SetThreadpoolTimer(task->threadpool_timer, &due_time, 0, 0);

    scheduler_unlock(scheduler);
    return task_id;
}

int scheduler_add_time_task(Scheduler *scheduler,
                            SchedulerCallback callback,
                            void *user_data,
                            SYSTEMTIME *target_time) {
    if (!scheduler || !callback || !target_time || !scheduler_acquire(scheduler)) return -1;

    FILETIME ft;
    if (!SystemTimeToFileTime(target_time, &ft)) {
        scheduler_release(scheduler);
        return -1;
    }

    EnterCriticalSection(&scheduler->lock);
    if (scheduler->shutting_down) {
        scheduler_unlock(scheduler);
        return -1;
    }

    SchedulerTask *task = calloc(1, sizeof(SchedulerTask));
    if (!task) {
        scheduler_unlock(scheduler);
        return -1;
    }

    task->type = TASK_AT_TIME;
    task->parent_scheduler = scheduler;
    task->callback = callback;
    task->user_data = user_data;
    task->target_time = *target_time;

    int task_id = add_task_common(scheduler, task);
    if (task_id == -1) {
        scheduler_unlock(scheduler);
        return -1;
    }

    SetThreadpoolTimer(task->threadpool_timer, &ft, 0, 0);

    scheduler_unlock(scheduler);
    return task_id;
}

bool scheduler_remove_task(Scheduler *scheduler, int task_id) {
    if (!scheduler || !scheduler_acquire(scheduler)) return false;

    EnterCriticalSection(&scheduler->lock);
    if (scheduler->shutting_down) {
        scheduler_unlock(scheduler);
        return false;
    }

    SchedulerTask *found = NULL;
    for (int i = 0; i < scheduler->task_count; i++) {
        if (scheduler->tasks[i]->id == task_id) {
            found = scheduler->tasks[i];
            // Immediately remove from array by swapping with the last element
            detach_task_locked(scheduler, found);
            if (found->is_active) {
                found->is_active = false;
                scheduler->active_task_count--;
                WakeAllConditionVariable(&scheduler->task_done_cond);
            }
            // Cancel future timer firings while still under the lock (non-blocking)
            if (found->threadpool_timer) {
                SetThreadpoolTimer(found->threadpool_timer, NULL, 0, 0);
            }
            if (found->callback_thread_id == GetCurrentThreadId()) {
                found->deferred_free = true;
            } else {
                found->removal_pending = true;
                scheduler->removals_pending++;
            }
            break;
        }
    }

    LeaveCriticalSection(&scheduler->lock);

    // Wait outside the lock; self-removal is freed at the callback's tail.
    if (found) {
        if (found->deferred_free) {
            scheduler_release(scheduler);
            return true;
        }
        if (found->threadpool_timer) {
            WaitForThreadpoolTimerCallbacks(found->threadpool_timer, TRUE);
            CloseThreadpoolTimer(found->threadpool_timer);
        }
        free(found);
        EnterCriticalSection(&scheduler->lock);
        scheduler->removals_pending--;
        scheduler_unlock(scheduler);
        return true;
    }
    scheduler_release(scheduler);
    return false;
}

void scheduler_wait(Scheduler *scheduler, DWORD timeout_ms) {
    if (!scheduler || !scheduler_acquire(scheduler)) return;

    ULONGLONG deadline = GetTickCount64() + timeout_ms;
    EnterCriticalSection(&scheduler->lock);
    while (!scheduler->shutting_down &&
           (scheduler->active_task_count > 0 || scheduler->callbacks_running > 0 ||
            scheduler->removals_pending > 0)) {
        DWORD remaining = INFINITE;
        if (timeout_ms != INFINITE) {
            ULONGLONG now = GetTickCount64();
            if (now >= deadline) break;
            ULONGLONG time_left = deadline - now;
            remaining = time_left >= INFINITE ? INFINITE - 1 : (DWORD) time_left;
        }
        if (!SleepConditionVariableCS(&scheduler->task_done_cond, &scheduler->lock, remaining)) {
            break;
        }
    }
    scheduler_unlock(scheduler);
}

void scheduler_cleanup(Scheduler *scheduler) {
    if (!scheduler || !scheduler_acquire(scheduler)) return;

    EnterCriticalSection(&scheduler->lock);
    if (scheduler->shutting_down) {
        scheduler_unlock(scheduler);
        return;
    }

    // Cancel future firings and detach the task array.
    scheduler_close_admission(scheduler);
    scheduler->shutting_down = true;
    int count = scheduler->task_count;
    SchedulerTask **old_tasks = scheduler->tasks;
    SchedulerTask *current_task = current_callback_scheduler == scheduler
                                      ? current_callback_task
                                      : NULL;
    for (int i = 0; i < count; i++) {
        SchedulerTask *task = old_tasks[i];
        if (task) {
            task->is_active = false;
            if (task == current_task && !task->removal_pending) task->deferred_free = true;
            if (task->threadpool_timer) {
                SetThreadpoolTimer(task->threadpool_timer, NULL, 0, 0);
            }
        }
    }
    scheduler->tasks = NULL;
    scheduler->task_count = 0;
    scheduler->active_task_count = 0;
    scheduler->capacity = 0;
    WakeAllConditionVariable(&scheduler->task_done_cond);

    LeaveCriticalSection(&scheduler->lock);

    // Wait outside the lock so callbacks can finish their locked bookkeeping.
    for (int i = 0; i < count; i++) {
        SchedulerTask *task = old_tasks[i];
        if (task && task != current_task) {
            if (task->threadpool_timer) {
                WaitForThreadpoolTimerCallbacks(task->threadpool_timer, TRUE);
                CloseThreadpoolTimer(task->threadpool_timer);
            }
            free(task);
        }
    }
    free(old_tasks);

    // A callback cannot wait for its own operation reference. Its tail will
    // release the final reference and destroy the lock when it returns.
    if (current_task) {
        scheduler_release(scheduler);
        return;
    }

    EnterCriticalSection(&scheduler->lock);
    while ((InterlockedCompareExchange(&scheduler->operation_state, 0, 0) &
            SCHEDULER_REF_MASK) > 1) {
        SleepConditionVariableCS(&scheduler->task_done_cond, &scheduler->lock, INFINITE);
    }
    scheduler_unlock(scheduler);
}
