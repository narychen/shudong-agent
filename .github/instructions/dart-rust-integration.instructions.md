---
description: "Use when implementing Dart-Rust integration, FFI bindings, or calling native code from Dart. Covers binding generation, error handling across boundaries, and platform-specific considerations."
---

# Dart-Rust Integration Patterns

## FFI Module Organization
- Create a dedicated FFI module: `lib/rust_core.dart` (or `lib/ffi/rust_bindings.dart`)
- Use lazy initialization: `static final instance = RustCore._init()`
- Separate concerns: Keep FFI calls away from UI layer

## Binding Structure
```dart
import 'dart:ffi' as ffi;

final DynamicLibrary _dylib = ffi.DynamicLibrary.open(
  'path/to/libshudong_core.so', // Platform-specific
);

typedef ShudongGreetNative = ffi.Pointer<ffi.Char> Function(ffi.Pointer<ffi.Char>);
typedef ShudongGreetDart = ffi.Pointer<ffi.Char> Function(ffi.Pointer<ffi.Char>);

final ShudongGreetDart shudongGreet = _dylib
    .lookup<ffi.NativeFunction<ShudongGreetNative>>('shudong_greet')
    .asFunction();
```

## String Marshalling
- **C → Dart**: Use `toDartString()` helper:
  ```dart
  extension CStringExtension on ffi.Pointer<ffi.Char> {
    String toDartString() {
      return ffi.Utf8.fromUtf8(cast<ffi.Utf8>());
    }
  }
  ```

- **Dart → C**: Use `toNativeUtf8()`:
  ```dart
  final cString = 'Hello'.toNativeUtf8();
  // Use cString...
  malloc.free(cString);
  ```

- **Always free C strings from Rust**: Call the Rust cleanup function
  ```dart
  final result = shudongGreet(nativeString);
  final dartString = result.toDartString();
  shudongStringFree(result); // Don't forget!
  malloc.free(nativeString);
  ```

## Error Handling Across Boundaries
- **Wrap FFI calls in try-catch**: Rust panics → Dart exceptions
- **Return null or default values on error**: Don't propagate Rust errors directly
- **Log full error context**: Device logs, not just user-facing messages

Example:
```dart
Future<String> greet(String name) async {
  try {
    final nativeString = name.toNativeUtf8();
    final result = shudongGreet(nativeString.cast());
    final dartString = result.toDartString();
    shudongStringFree(result);
    malloc.free(nativeString);
    return dartString;
  } catch (e) {
    debugPrint('Rust FFI error: $e');
    return 'Failed to greet';
  }
}
```

## Platform-Specific Builds
- **macOS**: Ensure library is linked in `macos/Runner.xcodeproj`
- **Linux**: Update `CMakeLists.txt` to build Rust core
- **Windows**: Use Rust MSVC toolchain, link in `windows/CMakeLists.txt`

## Initialization & Lifecycle
- Initialize FFI module once at app startup: `main()` or `initState()`
- Cache native functions after lookup (don't re-lookup per call)
- Use singleton pattern for RustCore instance:
  ```dart
  class RustCore {
    static final RustCore instance = RustCore._init();
    RustCore._init() { /* load library */ }
  }
  ```

## Concurrency Considerations
- FFI calls are synchronous—don't block the UI thread
- Use `compute()` for heavy Rust calls: `await compute(rustHeavyWork, arg)`
- Consider isolates for truly parallel work

## Testing
- Mock FFI bindings with `mockito` or similar
- Test round-trip marshalling: Dart → C → Rust → C → Dart
- Test error cases: null pointers, invalid input, Rust panics
