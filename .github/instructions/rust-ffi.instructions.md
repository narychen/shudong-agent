---
description: "Use when writing Rust code, especially FFI bindings and C interop. Covers memory safety, C string handling, unsafe block patterns, and FFI best practices."
applyTo: "core/src/**/*.rs"
---

# Rust & FFI Code Standards

## Naming Conventions
- **Modules & Functions**: snake_case (`shudong_greet`, `read_c_string`)
- **Types & Traits**: PascalCase (`CString`, `RustCore`)
- **Constants**: UPPER_SNAKE_CASE
- **FFI exported functions**: Use `#[no_mangle]` and `pub extern "C"`
- **Prefix exported functions** with module name: `shudong_*`

## FFI & C Interop Safety

### String Handling (Critical)
- **Always use `CStr::from_ptr()` with null checks**:
  ```rust
  fn read_c_string(ptr: *const c_char) -> String {
      if ptr.is_null() {
          return String::new();
      }
      CStr::from_ptr(ptr).to_string_lossy().into_owned()
  }
  ```

- **Return strings as owned pointers**:
  ```rust
  fn into_raw_string(value: String) -> *mut c_char {
      CString::new(value).unwrap().into_raw()
  }
  ```

- **Always provide memory cleanup function**:
  ```rust
  #[no_mangle]
  pub extern "C" fn shudong_string_free(ptr: *mut c_char) {
      if ptr.is_null() { return; }
      unsafe { drop(CString::from_raw(ptr)); }
  }
  ```

### Unsafe Block Patterns
- **Document why unsafe is needed**: Add a comment above each unsafe block
- **Keep unsafe blocks small**: Extract unsafe code into named functions
- **Validate all pointer arguments**: Check for null pointers and dangling references
- **Never assume Dart follows memory safety**: Validate all inputs from FFI calls

Example:
```rust
/// SAFETY: Assumes Dart provides a valid C string or null pointer.
/// Caller must free returned pointer using shudong_string_free().
#[no_mangle]
pub extern "C" fn shudong_process(input: *const c_char) -> *mut c_char {
    let input = unsafe { read_c_string(input) };
    // ... process safely ...
    into_raw_string(result)
}
```

## Memory Management
- **No memory leaks**: Every allocated pointer from Rust must have a corresponding free function
- **Clear ownership**: Rust owns memory it allocates; Dart is responsible for calling free
- **Avoid static/global mutable state**: Use message passing or thread-safe wrappers if needed

## Error Handling
- **Use Result<T, E> for fallible operations**: `CString::new()` can fail
- **Return safe defaults when FFI calls fail**: Don't panic across FFI boundaries
- **Log errors**: Use `eprintln!()` or logging crate for debugging

## Dependencies
- Keep dependencies minimal in FFI code
- Use std library functions when possible
- Document any external crate's FFI-safety implications

## Testing
- Write unit tests for C string handling
- Test null pointer cases
- Test round-trip Dart → Rust → Dart for strings

## Build Configuration
- Set `crate-type = ["cdylib", "rlib"]` in Cargo.toml
- Platform-specific builds for macOS, Linux, Windows
