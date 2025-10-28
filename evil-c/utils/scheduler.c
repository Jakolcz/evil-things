#include "scheduler.h"
#include "../features/feature.h"
#include <stdio.h>
#include <stdlib.h>

#define INITIAL_CAPACITY 10

// Callback wrapper for threadpool timers
static VOID CALLBACK threadpool_timer_callback(PTP_CALLBACK_INSTANCE instance,
                                               PVOID context,
                                               PTP_TIMER timer) {
    SchedulerTask *task = context;
    if (!task || !task->is_active || !task->callback) {
        return;
    }

    task->callback(task->user_data);
}

// Initialize scheduler
bool scheduler_init(Scheduler *scheduler) {
    scheduler->tasks = (SchedulerTask *) malloc(sizeof(SchedulerTask) * INITIAL_CAPACITY);
    if (!scheduler->tasks) {
        return false;
    }

    scheduler->task_count = 0;
    scheduler->capacity = INITIAL_CAPACITY;
    InitializeCriticalSection(&scheduler->lock);

    return true;
}

// Helper: Expand task array if needed
static bool ensure_capacity(Scheduler *scheduler) {
    if (scheduler->task_count >= scheduler->capacity) {
        int new_capacity = scheduler->capacity * 2;
        SchedulerTask *new_tasks = realloc(scheduler->tasks,
                                           sizeof(SchedulerTask) * new_capacity);
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
    return scheduler_add_interval_task(scheduler,
                                       feature->execute,
                                       params,
                                       interval_ms);
}

// Add interval-based task
int scheduler_add_interval_task(Scheduler *scheduler,
                                SchedulerCallback callback,
                                void *user_data,
                                DWORD interval_ms) {
    EnterCriticalSection(&scheduler->lock);

    if (!ensure_capacity(scheduler)) {
        LeaveCriticalSection(&scheduler->lock);
        return -1;
    }

    const int task_id = scheduler->task_count;
    SchedulerTask *task = &scheduler->tasks[task_id];

    task->type = TASK_INTERVAL;
    task->parent_scheduler = scheduler;
    task->callback = callback;
    task->user_data = user_data;
    task->interval_ms = interval_ms;
    task->is_active = true;
    task->timer_handle = NULL;

    // Create threadpool timer (most efficient for Windows)
    task->threadpool_timer = CreateThreadpoolTimer(threadpool_timer_callback, task, NULL);
    if (!task->threadpool_timer) {
        LeaveCriticalSection(&scheduler->lock);
        return -1;
    }

    // Convert milliseconds to 100-nanosecond intervals (negative for relative time)
    FILETIME due_time;
    ULARGE_INTEGER ulDueTime;
    ulDueTime.QuadPart = (ULONGLONG) -(interval_ms * 10000LL);
    due_time.dwHighDateTime = ulDueTime.HighPart;
    due_time.dwLowDateTime = ulDueTime.LowPart;

    // Set the timer to fire periodically
    SetThreadpoolTimer(task->threadpool_timer, &due_time, interval_ms, 0);

    scheduler->task_count++;

    LeaveCriticalSection(&scheduler->lock);
    return task_id;
}

// Add time-based task
int scheduler_add_time_task(Scheduler *scheduler,
                            SchedulerCallback callback,
                            void *user_data,
                            SYSTEMTIME *target_time) {
    EnterCriticalSection(&scheduler->lock);

    if (!ensure_capacity(scheduler)) {
        LeaveCriticalSection(&scheduler->lock);
        return -1;
    }

    const int task_id = scheduler->task_count;
    SchedulerTask *task = &scheduler->tasks[task_id];

    task->type = TASK_AT_TIME;
    task->parent_scheduler = scheduler;
    task->callback = callback;
    task->user_data = user_data;
    task->target_time = *target_time;
    task->is_active = true;
    task->timer_handle = NULL;

    // Create threadpool timer
    task->threadpool_timer = CreateThreadpoolTimer(threadpool_timer_callback, task, NULL);
    if (!task->threadpool_timer) {
        LeaveCriticalSection(&scheduler->lock);
        return -1;
    }

    // Convert SYSTEMTIME to FILETIME
    FILETIME ft;
    SystemTimeToFileTime(target_time, &ft);

    // Set the timer to fire once at the specified time
    SetThreadpoolTimer(task->threadpool_timer, &ft, 0, 0);

    scheduler->task_count++;

    LeaveCriticalSection(&scheduler->lock);
    return task_id;
}

// Remove a task
bool scheduler_remove_task(Scheduler *scheduler, int task_id) {
    EnterCriticalSection(&scheduler->lock);

    if (task_id < 0 || task_id >= scheduler->task_count) {
        LeaveCriticalSection(&scheduler->lock);
        return false;
    }

    SchedulerTask *task = &scheduler->tasks[task_id];
    task->is_active = false;

    if (task->threadpool_timer) {
        SetThreadpoolTimer(task->threadpool_timer, NULL, 0, 0); // Cancel timer
        WaitForThreadpoolTimerCallbacks(task->threadpool_timer, TRUE);
        CloseThreadpoolTimer(task->threadpool_timer);
        task->threadpool_timer = NULL;
    }

    LeaveCriticalSection(&scheduler->lock);
    return true;
}

// Wait for tasks
void scheduler_wait(Scheduler *scheduler, DWORD timeout_ms) {
    Sleep(timeout_ms);
}

// Cleanup scheduler
void scheduler_cleanup(Scheduler *scheduler) {
    EnterCriticalSection(&scheduler->lock);

    // Cancel and cleanup all tasks
    for (int i = 0; i < scheduler->task_count; i++) {
        SchedulerTask *task = &scheduler->tasks[i];
        if (task->threadpool_timer) {
            SetThreadpoolTimer(task->threadpool_timer, NULL, 0, 0);
            WaitForThreadpoolTimerCallbacks(task->threadpool_timer, TRUE);
            CloseThreadpoolTimer(task->threadpool_timer);
        }
    }

    free(scheduler->tasks);
    scheduler->tasks = NULL;
    scheduler->task_count = 0;

    LeaveCriticalSection(&scheduler->lock);
    DeleteCriticalSection(&scheduler->lock);
}
