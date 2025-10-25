#include "clipboard.h"
#include "../utils/logger.h"
#include <windows.h>
#include <stdbool.h>

char *get_text_from_clipboard(void) {
    if (!OpenClipboard(NULL)) {
        LOG_ERROR("Failed to open clipboard (Error: %lu)", GetLastError());
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
        LOG_ERROR("Failed to lock clipboard data (Error: %lu)", GetLastError());
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

bool write_text_to_clipboard(const char *text) {
    if (!text) {
        return false;
    }

    // Convert UTF-8 to UTF-16
    const int wlen = MultiByteToWideChar(CP_UTF8, 0, text, -1, NULL, 0);
    if (wlen == 0) {
        LOG_ERROR("MultiByteToWideChar size calculation failed (Error: %lu)", GetLastError());
        return false;
    }

    HGLOBAL hMem = GlobalAlloc(GMEM_MOVEABLE, wlen * sizeof(wchar_t));
    if (!hMem) {
        LOG_ERROR("GlobalAlloc failed (Error: %lu)", GetLastError());
        return false;
    }

    wchar_t *pMem = GlobalLock(hMem);
    if (!pMem) {
        LOG_ERROR("GlobalLock failed (Error: %lu)", GetLastError());
        GlobalFree(hMem);
        return false;
    }

    if (MultiByteToWideChar(CP_UTF8, 0, text, -1, pMem, wlen) == 0) {
        LOG_ERROR("MultiByteToWideChar conversion failed (Error: %lu)", GetLastError());
        GlobalUnlock(hMem);
        GlobalFree(hMem);
        return false;
    }
    GlobalUnlock(hMem);

    int retry_count = 0;
    while (!OpenClipboard(NULL) && retry_count < 5) {
        Sleep(10);
        retry_count++;
    }

    if (retry_count >= 5) {
        LOG_ERROR("Failed to open clipboard after %d retries", retry_count);
        GlobalFree(hMem);
        return false;
    }

    if (!EmptyClipboard()) {
        LOG_ERROR("EmptyClipboard failed (Error: %lu)", GetLastError());
        CloseClipboard();
        GlobalFree(hMem);
        return false;
    }

    if (SetClipboardData(CF_UNICODETEXT, hMem) == NULL) {
        LOG_ERROR("SetClipboardData failed (Error: %lu)", GetLastError());
        CloseClipboard();
        GlobalFree(hMem);
        return false;
    }

    // Close clipboard (don't free hMem - clipboard owns it now)
    CloseClipboard();
    return true;
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
    free(text);
    if (!modified_text) {
        LOG_DEBUG("Modified text is NULL, no modifications made or error occurred");
        return;
    }

    if (!write_text_to_clipboard(modified_text)) {
        LOG_ERROR("Failed to write modified text to clipboard");
    } else {
        LOG_DEBUG("Clipboard text modified successfully");
    }
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
