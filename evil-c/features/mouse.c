#include "mouse.h"
#include "../utils/logger.h"
#include <windows.h>

static unsigned int call_count = 0;

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

static int get_mouse_sensitivity() {
    int sensitivity = 0;
    SystemParametersInfo(SPI_GETMOUSESPEED, 0, &sensitivity, 0);
    return sensitivity;
}

static void set_mouse_sensitivity(const int sensitivity) {
    if (sensitivity < 1 || sensitivity > 20) {
        LOG_WARN("Attempted to set invalid mouse sensitivity: %d", sensitivity);
        return;
    }
    SystemParametersInfo(SPI_SETMOUSESPEED, 0, (PVOID) (intptr_t) sensitivity, SPIF_SENDCHANGE | SPIF_UPDATEINIFILE);
}

static void disable_mouse_trail(void) {
    set_mouse_trail(0);
}

static void mouse_sensitivity_feature(void) {
    static int sensitivity_direction = 1; // 1 for increasing, -1 for decreasing
    int sensitivity = get_mouse_sensitivity();
    LOG_DEBUG("Current mouse sensitivity: %d", sensitivity);

    // Change direction at bounds
    if (sensitivity >= 20) {
        sensitivity_direction = -1;
    } else if (sensitivity <= 1) {
        sensitivity_direction = 1;
    }

    sensitivity += sensitivity_direction;
    set_mouse_sensitivity(sensitivity);
}

void mouse_trail_feature(void) {
    const int trail_length = rand() % 15 + 2; // Random length between 2 and 16
    set_mouse_trail(trail_length);
    // since this will be called from a scheduler, we can safely use Sleep here
    Sleep(750);
    disable_mouse_trail();
}

void execute_mouse_feature(void *params) {
    LOG_DEBUG("Executing mouse feature, current sensitivity: %d", get_mouse_sensitivity());

    call_count++;

    // TODO maybe run just once every X calls?
    mouse_trail_feature();
    mouse_sensitivity_feature();
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
