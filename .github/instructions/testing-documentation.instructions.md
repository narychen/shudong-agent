---
description: "Use when setting up testing, writing tests, or creating documentation. Covers test structure, coverage expectations, documentation standards, and CI/CD considerations."
---

# Testing & Documentation Standards

## Test Organization
- **Unit tests**: `test/` directory, follow Dart testing conventions
- **Widget tests**: `test/*_test.dart` files
- **Integration tests**: `integration_test/` directory (Flutter integration tests)
- **Rust tests**: `core/src/lib.rs` with `#[cfg(test)]` modules

## Dart Unit Testing
```dart
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('MyWidget', () {
    testWidgets('renders correctly', (WidgetTester tester) async {
      await tester.pumpWidget(const MyWidget());
      expect(find.byType(Text), findsOneWidget);
    });
  });
}
```

- Use descriptive test names: `'should return error when input is empty'`
- Test edge cases: empty strings, null values, invalid input
- Mock dependencies: `mockito`, `mocktail` for FFI bindings

## Dart Code Coverage
- Target: **80%+ coverage** for public APIs
- Run tests with coverage: `flutter test --coverage`
- View coverage: `genhtml coverage/lcov.report -o coverage/html`
- Exclude generated code and test files from coverage

## Rust Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_c_string_with_null() {
        unsafe {
            let result = read_c_string(std::ptr::null());
            assert_eq!(result, String::new());
        }
    }
}
```

- Test memory safety: null pointers, string overflow
- Use `#[ignore]` for slow tests
- Run: `cargo test` or `cargo test -- --nocapture` for output

## FFI Integration Testing
- Test Dart-Rust round-trip: call Rust, verify output, free memory
- Test error paths: invalid input, panic scenarios
- Mock Rust library for Dart unit tests

Example:
```dart
test('greet returns formatted string from Rust', () {
  final result = RustCore.instance.greet('Alice');
  expect(result, contains('Alice'));
});
```

## Documentation

### README
- **Purpose**: Project overview, architecture, setup instructions
- **Sections**: 
  - Project Description
  - Architecture (Dart UI + Rust Core)
  - Prerequisites (Flutter version, Rust toolchain)
  - Setup & Build Instructions
  - Running Tests
  - Platform-Specific Notes (macOS, Linux, Windows)

### Code Documentation
- **Public APIs**: Add Dart doc comments (`///`)
  ```dart
  /// Greets a person with a message from Rust core.
  /// 
  /// [name] The name of the person to greet.
  /// Returns a greeting string from Rust.
  Future<String> greet(String name) async { ... }
  ```

- **Rust public functions**: Document with `///` and safety notes
  ```rust
  /// Processes a task and returns the result as a C string.
  /// 
  /// # Safety
  /// Caller must free returned pointer using `shudong_string_free()`.
  #[no_mangle]
  pub extern "C" fn shudong_process_task(task: *const c_char) -> *mut c_char { ... }
  ```

- **Complex logic**: Explain intent, not implementation
- **FFI boundaries**: Mark unsafe code and explain why it's safe

### Module Documentation
- `lib/main.dart`: App entry point and routing
- `lib/rust_core.dart`: FFI bindings and initialization
- `core/src/lib.rs`: Core business logic and FFI exports

## Changelog
- Maintain `CHANGELOG.md` for each release
- Format: **[Version]** - **[Date]**
  - Added: New features
  - Changed: Modifications
  - Fixed: Bug fixes
  - Removed: Deprecated features

## CI/CD Considerations
- Run tests on every commit
- Lint Dart code: `flutter analyze`
- Lint Rust code: `cargo clippy`
- Build for all platforms (macOS, Linux, Windows)
- Generate coverage reports
