#include "utils/logger.h"
#include "utils/scheduler.h"

int main(void) {
    logger_init("test.log", true);
    LOG_INFO("Starting the application");
    Scheduler scheduler;
    if (!scheduler_init(&scheduler)) {
        LOG_ERROR("Failed to initialize scheduler");
        logger_close();
        exit(EXIT_FAILURE);
    }
    LOG_DEBUG("Scheduler initialized successfully");

    scheduler_wait(&scheduler, 10000);
    scheduler_cleanup(&scheduler);
    logger_close();

    return 0;
}
