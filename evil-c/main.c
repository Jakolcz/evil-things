#include "utils/logger.h"
#include "utils/scheduler.h"
#include "features/clipboard.h"

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

    Feature *clipboard_feature = get_clipboard_feature();
    clipboard_feature->execute(NULL);

    scheduler_wait(&scheduler, 1000);
    scheduler_cleanup(&scheduler);
    logger_close();

    return 0;
}
