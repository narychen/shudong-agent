# shudong-agent

## Current status
- Flutter desktop shell is wired to a Rust core via `dart:ffi`.
- Rust library exports `greet` and `processTask` entry points.
- Tests pass on this machine.

## Run locally
1. Build the Rust library:
   `cd core && cargo build`
2. Point Flutter to the built dylib:
   `export SHUDONG_CORE_LIB=/absolute/path/to/core/target/debug/libshudong_core.dylib`
3. Run the app (recommended, env will be inherited):
   `cd app && flutter run -d macos`

   If you `open app.app`, macOS may not pass env vars into the app process. For that case, launch the binary directly:
   `env SHUDONG_CORE_LIB=/absolute/path/to/core/target/debug/libshudong_core.dylib \
   /absolute/path/to/app/build/macos/Build/Products/Release/app.app/Contents/MacOS/app`

## Notes
- This is the minimal desktop bridge first.
- Mobile and web support will need a different packaging strategy.
