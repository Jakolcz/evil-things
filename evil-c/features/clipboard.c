#include "clipboard.h"
#include "../utils/logger.h"
#include <windows.h>

static void execute_clipboard_feature(void *ignored) {
    LOG_DEBUG("Executing clipboard feature");
    if (!OpenClipboard(NULL)) {
        LOG_ERROR("Failed to open clipboard");
        return;
    }

    HANDLE hData = GetClipboardData(CF_TEXT);
    if (hData == NULL) {
        LOG_WARN("No text data in clipboard");
        CloseClipboard();
        return;
    }

    // Lock the handle to get pointer to the data
    const char *pszText = GlobalLock(hData);
    if (pszText == NULL) {
        LOG_ERROR("Failed to lock clipboard data\n");
        CloseClipboard();
        return;
    }

    // Copy the text to our buffer
    const size_t len = strlen(pszText);
    char *text = malloc(len + 1);
    if (text != NULL) {
        strcpy(text, pszText);
    }

    LOG_DEBUG("Clipboard text: %s", text ? text : "(null)");
    free(text);

    // Unlock and close
    GlobalUnlock(hData);
    CloseClipboard();
}

static Feature clipboard_feature = {
    .name = "clipboard",
    .execute = execute_clipboard_feature,
    .initialize = NULL,
    .cleanup = NULL,
};

Feature *get_clipboard_feature(void) {
    return &clipboard_feature;
}
