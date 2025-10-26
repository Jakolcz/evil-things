#include "mouse.h"
#include "../utils/logger.h"
#include <windows.h>

static int get_current_trail_length() {
    int trailLength = 0;
    SystemParametersInfo(SPI_GETMOUSETRAILS, 0, &trailLength, 0);
    return trailLength;
}

/// Set mouse trail length
/// @param length Length of the mouse trail, must be 0-16 (0 or 1 to disable)
static void set_mouse_trail(const int length) {
    SystemParametersInfo(SPI_SETMOUSETRAILS, length, 0, SPIF_SENDCHANGE);
}

static void disable_mouse_trail(void) {
    set_mouse_trail(0);
}

void execute_mouse_feature(void *params) {
    LOG_DEBUG("Executing mouse feature, current trail length: %d", get_current_trail_length());
    const int trail_length = rand() % 15 + 2; // Random length between 2 and 16
    set_mouse_trail(trail_length);
    // since this will be called from a scheduler, we can safely use Sleep here
    Sleep(750);
    disable_mouse_trail();
}

static Feature mouse_feature = {
    .name = "mouse",
    .execute = execute_mouse_feature,
    .initialize = NULL,
    .cleanup = NULL,
};

Feature *get_mouse_feature(void) {
    return &mouse_feature;
}
