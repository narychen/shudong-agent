---
description: "Use when writing Dart code in Flutter widgets, services, or utilities. Covers naming conventions, immutability patterns, proper lifecycle management, and Material Design 3 best practices."
applyTo: "**/*.dart"
---

# Dart & Flutter Code Standards

## Naming Conventions
- **Classes & Types**: PascalCase (`MyHomePageState`, `RustCore`)
- **Variables & Functions**: camelCase (`_controller`, `_tryWireCore()`)
- **Private members**: Leading underscore (`_status`, `_busy`)
- **Constants**: camelCase with const prefix (`const Color(0xFF2F6BFF)`)
- **Widgets**: End with Widget for stateless/stateful (`MyHomePage`, `MyHomePageState`)

## Immutability & const
- Use `const` constructors whenever possible (e.g., `const MyApp({super.key})`)
- Use `final` for immutable variables: `final TextEditingController _controller = ...`
- Avoid `var` unless type is obvious from context
- Prefer immutable data structures; use `@immutable` annotation for custom classes

## Widget Best Practices
- **Prefer StatelessWidget** over StatefulWidget when possible
- Use `super.key` in widget constructors: `const MyApp({super.key})`
- Override `runtimeType` or use `key` for widget identification
- Proper lifecycle: `initState()` → setup, `dispose()` → cleanup
- Always check `if (!mounted)` before `setState()` in async callbacks

## Material Design 3
- Use `ColorScheme.fromSeed()` for theme generation
- Set `useMaterial3: true` in ThemeData
- Use Material 3 widgets: `Scaffold`, `AppBar`, `FloatingActionButton`, etc.
- Prefer `Theme.of(context)` over hardcoded colors

## Error Handling
- Use try-catch for FFI calls and async operations
- Provide meaningful error messages to users
- Log errors for debugging (consider adding a logging package)

## Code Organization
- One widget per file (unless small helper widgets)
- Services in `lib/services/` directory
- Models in `lib/models/` directory
- Place FFI bindings in `lib/rust_core.dart` or similar isolated module

## Testing
- Write unit tests in `test/` directory
- Test widget behavior with `testWidgets()`
- Mock external dependencies (including Rust FFI)

## Example Structure
```dart
import 'package:flutter/material.dart';
import 'rust_core.dart';  // FFI bindings

class MyWidget extends StatelessWidget {
  const MyWidget({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Title')),
      body: const Center(child: Text('Hello')),
    );
  }
}
```
