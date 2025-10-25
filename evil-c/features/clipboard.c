#include "clipboard.h"
#include "../utils/logger.h"
#include <windows.h>

char *get_text_from_clipboard(void) {
    if (!OpenClipboard(NULL)) {
        LOG_ERROR("Failed to open clipboard");
        return NULL;
    }

    HANDLE hData = GetClipboardData(CF_TEXT);
    if (hData == NULL) {
        LOG_WARN("No text data in clipboard");
        CloseClipboard();
        return NULL;
    }

    // Lock the handle to get pointer to the data
    const char *pszText = GlobalLock(hData);
    if (pszText == NULL) {
        LOG_ERROR("Failed to lock clipboard data\n");
        CloseClipboard();
        return NULL;
    }

    // Copy the text to our buffer
    const size_t len = strlen(pszText);
    char *text = malloc(len + 1);
    if (text != NULL) {
        strcpy(text, pszText);
    }

    // Unlock and close
    GlobalUnlock(hData);
    CloseClipboard();

    return text;
}

/// Replace all semicolons in the input string with Greek question marks (;).
///
/// @return A newly allocated string with replacements, or NULL if no replacements were made or something went wrong.
char *replace_semicolon_with_greek_question_mark(const char *input) {
    if (!input) {
        return NULL;
    }

    // Count semicolons to determine new length
    const char *p = input;
    size_t len = 0;
    size_t semicolon_count = 0;
    while (*p) {
        if (*p == ';') {
            semicolon_count++;
        }
        len++;
        p++;
    }

    // If there are no semicolons, do not return anything
    if (semicolon_count == 0) {
        return NULL;
    }
    // Allocate exact size: original length + extra byte per semicolon (since ; is 2 bytes in UTF-8) + null terminator
    char *output = malloc(len + semicolon_count + 1);
    if (!output) {
        return NULL;
    }

    // The separate out pointer serves as a write cursor that moves forward independently from the output base pointer.
    // Preserve the base pointer: output must remain unchanged because:
    // It's needed to return the start of the allocated string
    // If you modify output directly with output++, you lose the reference to the beginning and can't return or free it properly
    char *out = output;
    p = input;
    while (*p) {
        if (*p == ';') {
            // Replace with Greek question mark (;)
            *out++ = (char) 0xCD; // First byte of ; in UTF-8
            *out++ = (char) 0xBE; // Second byte of ; in UTF-8
        } else {
            *out++ = *p;
        }
        p++;
    }
    *out = '\0';

    return output;
}

void execute_clipboard_feature(void *ignored) {
    LOG_DEBUG("Executing clipboard feature");
    char *text = get_text_from_clipboard();
    if (!text) {
        LOG_DEBUG("No text retrieved from clipboard");
        return;
    }
    char *modified_text = replace_semicolon_with_greek_question_mark(text);
    if (!modified_text) {
        LOG_DEBUG("Modified text is NULL, no modifications made or error occurred");
        free(text);
        return;
    }

    LOG_DEBUG("Clipboard text: %s", text ? text : "(null)");
    free(text);
    LOG_DEBUG("Modified clipboard text: %s", modified_text);
    free(modified_text);
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
