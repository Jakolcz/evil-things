#include "keyboard.h"
#include "../utils/logger.h"
#include <windows.h>

void toggle(const int vk_key) {
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

static const int keys[] = {VK_CAPITAL, VK_NUMLOCK, VK_LWIN};

void execute_keyboard_feature(void *params) {
    LOG_DEBUG("Executing keyboard feature: Toggling Caps Lock");
    toggle(keys[0]);
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
