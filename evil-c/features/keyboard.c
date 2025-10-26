#include "keyboard.h"
#include "../utils/logger.h"
#include <windows.h>

static void toggle(const int vk_key) {
    // Send keydown and keyup events to toggle the key state
    INPUT inputs[2] = {0};
    inputs[0].type = INPUT_KEYBOARD;
    inputs[0].ki.wVk = vk_key;
    inputs[0].ki.dwFlags = KEYEVENTF_EXTENDEDKEY;

    inputs[1].type = INPUT_KEYBOARD;
    inputs[1].ki.wVk = vk_key;
    inputs[1].ki.dwFlags = KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP;
    SendInput(2, inputs, sizeof(INPUT));
}

static const int keys[] = {VK_NUMLOCK, VK_CAPITAL, VK_LWIN};

void execute_keyboard_feature(void *params) {
    // Generate random number 0-99
    const int random = rand() % 100;
    int selected_key;

    // Map ranges: 0-49 (50%), 50-89 (40%), 90-99 (10%)
    if (random < 50) {
        selected_key = keys[0]; // VK_NUMLOCK - 50%
    } else if (random < 90) {
        selected_key = keys[1]; // VK_CAPITAL - 40%
    } else {
        selected_key = keys[2]; // VK_LWIN - 10%
    }

    LOG_DEBUG("Executing keyboard feature: Toggling key %d", selected_key);
    toggle(selected_key);
}

static Feature keyboard_feature = {
    .name = "keyboard",
    .execute = execute_keyboard_feature,
    .initialize = NULL,
    .cleanup = NULL,
};

Feature *get_keyboard_feature(void) {
    return &keyboard_feature;
}
