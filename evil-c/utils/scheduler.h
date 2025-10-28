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

    // For interval tasks
    DWORD interval_ms;

    // For time-based tasks
    SYSTEMTIME target_time;
} SchedulerTask;

// Scheduler structure
typedef struct Scheduler {
    SchedulerTask *tasks;
    int task_count;
    int capacity;
    CRITICAL_SECTION lock;
} Scheduler;

// Initialize scheduler
bool scheduler_init(Scheduler *scheduler);

// Add interval-based task (repeats every interval_ms)
int scheduler_add_interval_task(Scheduler *scheduler,
                                SchedulerCallback callback,
                                void *user_data,
                                DWORD interval_ms);

// Add time-based task (executes once at specific time)
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