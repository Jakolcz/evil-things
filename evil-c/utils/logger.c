#include "logger.h"
#include <time.h>
#include <stdio.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>
#include <stdarg.h>
#include <windows.h>

// Color codes (ANSI escape sequences)
#define COLOR_RESET   "\x1b[0m"
#define COLOR_GRAY    "\x1b[90m"
#define COLOR_GREEN   "\x1b[32m"
#define COLOR_YELLOW  "\x1b[33m"
#define COLOR_RED     "\x1b[31m"
#define MSG_BUFFER_SIZE 1024
#define LOG_BUFFER_SIZE (MSG_BUFFER_SIZE + 128)

static FILE *log_file = NULL;
static bool log_to_console = true;
const char *level_strings[] = {"DEBUG", "INFO", "WARN", "ERROR"};
static bool has_color_support = false;
static CRITICAL_SECTION log_mutex;

static bool supports_color_output(void) {
    DWORD mode;
    HANDLE hOut = GetStdHandle(STD_OUTPUT_HANDLE);
    if (hOut == INVALID_HANDLE_VALUE) return false;
    if (!GetConsoleMode(hOut, &mode)) return false;
    // Try to enable virtual terminal processing
    mode |= 0x0004; // ENABLE_VIRTUAL_TERMINAL_PROCESSING
    SetConsoleMode(hOut, mode);
    return true;
}

void logger_init(const char *filename, const bool to_console) {
    log_to_console = to_console;

    if (!filename || strlen(filename) <= 0) {
        exit(EXIT_FAILURE);
    }

    log_file = fopen(filename, "a");
    if (!log_file) {
        fprintf(stderr, "Failed to open log file: %s\n", filename);
    }

    has_color_support = supports_color_output();
    InitializeCriticalSection(&log_mutex);
}

void logger_close(void) {
    if (log_file) {
        fflush(log_file);
        fclose(log_file);
        log_file = NULL;
    }
    DeleteCriticalSection(&log_mutex);
}

static const char *get_color(const LogLevel level) {
    if (!has_color_support) return "";

    switch (level) {
        case LOG_LEVEL_DEBUG: return COLOR_GRAY;
        case LOG_LEVEL_INFO: return COLOR_GREEN;
        case LOG_LEVEL_WARN: return COLOR_YELLOW;
        case LOG_LEVEL_ERROR: return COLOR_RED;
        default: return COLOR_RESET;
    }
}

static const char *basename_c(const char *path) {
    const char *slash = strrchr(path, '/');
    const char *backslash = strrchr(path, '\\');
    const char *p = slash > backslash ? slash : backslash;
    return p ? p + 1 : path;
}

void logger_log(const LogLevel level, const char *file, const int line, const char *func, const char *fmt, ...) {
    EnterCriticalSection(&log_mutex);

    SYSTEMTIME st;
    GetLocalTime(&st);

    char timebuf[32];
    snprintf(timebuf, sizeof(timebuf), "%04d-%02d-%02d %02d:%02d:%02d.%03d",
             st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute, st.wSecond, st.wMilliseconds);

    const char *base = basename_c(file);

    char msgbuf[MSG_BUFFER_SIZE];
    va_list args;
    va_start(args, fmt);
    vsnprintf(msgbuf, sizeof(msgbuf), fmt, args);
    va_end(args);

    if (log_to_console) {
        FILE *stream = LOG_LEVEL_WARN <= level ? stderr : stdout;
        char outbuf[LOG_BUFFER_SIZE];
        snprintf(outbuf, sizeof(outbuf), "%s[%s] %s (%s:%d %s): %s%s\n",
                 get_color(level), timebuf, level_strings[level], base, line, func, msgbuf, COLOR_RESET);
        fputs(outbuf, stream);
    }

    if (log_file) {
        char outbuf[LOG_BUFFER_SIZE];
        snprintf(outbuf, sizeof(outbuf), "[%s] %s (%s:%d %s): %s\n",
                 timebuf, level_strings[level], base, line, func, msgbuf);
        fputs(outbuf, log_file);
    }

    LeaveCriticalSection(&log_mutex);
}
