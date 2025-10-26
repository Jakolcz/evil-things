#include "utils/logger.h"
#include "utils/scheduler.h"
#include "features/mouse.h"
#include <windows.h>

int main(void) {
    logger_init("test.log", true);

    // Seed random number generator
    srand((unsigned int) GetTickCount64());
    LOG_INFO("Starting the application");
    Scheduler scheduler;
    if (!scheduler_init(&scheduler)) {
        LOG_ERROR("Failed to initialize scheduler");
        logger_close();
        exit(EXIT_FAILURE);
    }
    LOG_DEBUG("Scheduler initialized successfully");

    Feature *test_feature = get_mouse_feature();
    test_feature->execute(NULL);

    scheduler_wait(&scheduler, 1000);
    scheduler_cleanup(&scheduler);
    logger_close();

    return 0;
}
