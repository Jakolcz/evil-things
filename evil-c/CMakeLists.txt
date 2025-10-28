cmake_minimum_required(VERSION 4.0)
project(evil_c C)

set(CMAKE_C_STANDARD 11)

# Add version resource
enable_language(RC)
set(RESOURCE_FILE ${CMAKE_CURRENT_SOURCE_DIR}/version.rc)

add_executable(evil_c main.c
        utils/logger.c
        utils/logger.h
        utils/scheduler.c
        utils/scheduler.h
        features/feature.h
        features/clipboard.c
        features/clipboard.h
        features/keyboard.c
        features/keyboard.h
        features/mouse.c
        features/mouse.h
        ${RESOURCE_FILE})
