# Repository Guidelines

## Project Structure

This is a Windows-focused C11 application built from the repository root. `main.c` contains the entry point; `features/` holds keyboard, mouse, and clipboard feature modules plus their shared `Feature` interface; `utils/` contains the logger and scheduler. There is no separate test or asset directory. Keep generated CMake output in a build directory such as `build/` rather than adding it to source folders.

## Build and Development

Use CMake 4.0 or newer with a Windows toolchain that provides the Windows API headers:

```powershell
cmake -S . -B build -DCMAKE_BUILD_TYPE=Debug
cmake --build build --config Debug
```

The first command configures the `evil_c` executable; the second compiles it. Never run the generated executable on this machine, under any circumstances. It may only be run inside a separate virtual machine. No automated test target is currently configured.

## Coding Style

Follow the existing C style: four spaces for indentation, braces on the same line as function declarations, and `snake_case` for functions and variables. Use `PascalCase` for struct type names and uppercase names for macros and constants. Keep feature implementations in matching `.c` and `.h` files under `features/`, and shared support modules under `utils/`. Prefer the existing C11 standard and include the declarations a module uses through its headers.

## Testing Guidelines

There is no test framework or coverage requirement in the repository. Build with the commands above, then perform any runtime checks only inside a separate virtual machine. Never execute the generated program on this machine. The program changes system input settings and clipboard contents, so keep those effects within the VM.

## Commits and Pull Requests

The available checkout did not expose usable Git history, so no repository-specific commit pattern could be confirmed. Use a short, imperative commit subject such as `Add clipboard transformation`. Pull requests should explain the behavior changed, note the build or manual checks performed, and include reproduction steps for user-visible issues. Add screenshots only when a visual change makes them useful.

## Configuration and Generated Files

Keep machine-specific IDE settings and generated build artifacts out of commits unless a change explicitly requires them. Do not commit logs, local binaries, or secrets.
