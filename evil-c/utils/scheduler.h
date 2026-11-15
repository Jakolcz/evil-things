#ifndef EVIL_C_SCHEDULER_H
#define EVIL_C_SCHEDULER_H
#include "../features/feature.h"
#include <windows.h>
#include <stdbool.h>

// Callback function type
typedef void (*SchedulerCallback)(void *user_data);

// Forward declaration
typedef struct Scheduler Scheduler;

// Task types
typedef enum {
    TASK_INTERVAL, // Repeating task at intervals
    TASK_AT_TIME // One-time task at specific time
} TaskType;

// Task structure
typedef struct {
    TaskType type;
    Scheduler *parent_scheduler;
    SchedulerCallback callback;
    SchedulerCallback initializer;
    SchedulerCallback cleanup;
    void *user_data;
    HANDLE timer_handle;
    PTP_TIMER threadpool_timer;
    bool is_active;
    DWORD callback_thread_id;
    bool deferred_free;
    bool removal_pending;
    int id; // Stable ID

    // For interval tasks
    DWORD interval_ms;

    // For time-based tasks
    SYSTEMTIME target_time;
} SchedulerTask;

// Scheduler structure
typedef struct Scheduler {
    SchedulerTask **tasks;
    int task_count;
    int active_task_count;
    int callbacks_running;
    int removals_pending;
    int capacity;
    int next_id;
    bool shutting_down;
    // High bit closes admission; remaining bits count operations using the lock.
    volatile LONG operation_state;
    CRITICAL_SECTION lock;
    CONDITION_VARIABLE task_done_cond;
} Scheduler;

// Initialize scheduler
bool scheduler_init(Scheduler *scheduler);

// Repeats interval_ms after each callback completes.
int scheduler_add_interval_task(Scheduler *scheduler,
                                SchedulerCallback callback,
                                void *user_data,
                                DWORD interval_ms);

// Add a one-time task for a UTC time.
int scheduler_add_time_task(Scheduler *scheduler,
                            SchedulerCallback callback,
                            void *user_data,
                            SYSTEMTIME *target_time);

int scheduler_add_interval_feature(Scheduler *scheduler,
                                   Feature *feature,
                                   void *params,
                                   DWORD interval_ms);

// Remove a task by ID
bool scheduler_remove_task(Scheduler *scheduler, int task_id);

// Cleanup scheduler
void scheduler_cleanup(Scheduler *scheduler);

// Wait for all tasks to complete (or timeout)
void scheduler_wait(Scheduler *scheduler, DWORD timeout_ms);

#endif //EVIL_C_SCHEDULER_H
