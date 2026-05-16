import 'dart:convert';
import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

typedef _NativeInitFn = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _DartInitFn = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _NativeStringFn = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _DartStringFn = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _NativeStatusFn = Pointer<Utf8> Function();
typedef _DartStatusFn = Pointer<Utf8> Function();

typedef _NativeFreeFn = Void Function(Pointer<Utf8>);
typedef _DartFreeFn = void Function(Pointer<Utf8>);

class RustCore {
  RustCore._(
    this._init,
    this._processTask,
    this._getStatus,
    this._getLastError,
    this._clearHistory,
    this._freeString,
  );

  final _DartInitFn _init;
  final _DartStringFn _processTask;
  final _DartStatusFn _getStatus;
  final _DartStatusFn _getLastError;
  final _DartStatusFn _clearHistory;
  final _DartFreeFn _freeString;

  static RustCore? _instance;

  static RustCore get instance {
    final existing = _instance;
    if (existing != null) {
      return existing;
    }

    final libraryPath = Platform.environment['SHUDONG_CORE_LIB'];
    if (libraryPath == null || libraryPath.isEmpty) {
      throw StateError(
        'Set SHUDONG_CORE_LIB to the compiled Rust library path before running the app.',
      );
    }

    final dylib = DynamicLibrary.open(libraryPath);
    final core = RustCore._(
      dylib.lookupFunction<_NativeInitFn, _DartInitFn>('shudong_init'),
      dylib.lookupFunction<_NativeStringFn, _DartStringFn>('shudong_process_task'),
      dylib.lookupFunction<_NativeStatusFn, _DartStatusFn>('shudong_get_status'),
      dylib.lookupFunction<_NativeStatusFn, _DartStatusFn>('shudong_get_last_error'),
      dylib.lookupFunction<_NativeStatusFn, _DartStatusFn>('shudong_clear_history'),
      dylib.lookupFunction<_NativeFreeFn, _DartFreeFn>('shudong_string_free'),
    );
    _instance = core;
    return core;
  }

  Map<String, dynamic> init(Map<String, dynamic> config) {
    final configJson = jsonEncode(config);
    final input = configJson.toNativeUtf8();
    final output = _init(input);
    calloc.free(input);

    try {
      final result = output.toDartString();
      return jsonDecode(result) as Map<String, dynamic>;
    } finally {
      _freeString(output);
    }
  }

  Map<String, dynamic> processTask(String task) {
    final input = task.toNativeUtf8();
    final output = _processTask(input);
    calloc.free(input);

    try {
      final result = output.toDartString();
      return jsonDecode(result) as Map<String, dynamic>;
    } finally {
      _freeString(output);
    }
  }

  Map<String, dynamic> getStatus() {
    final output = _getStatus();
    try {
      final result = output.toDartString();
      return jsonDecode(result) as Map<String, dynamic>;
    } finally {
      _freeString(output);
    }
  }

  String getLastError() {
    final output = _getLastError();
    try {
      return output.toDartString();
    } finally {
      _freeString(output);
    }
  }

  Map<String, dynamic> clearHistory() {
    final output = _clearHistory();
    try {
      final result = output.toDartString();
      return jsonDecode(result) as Map<String, dynamic>;
    } finally {
      _freeString(output);
    }
  }
}
