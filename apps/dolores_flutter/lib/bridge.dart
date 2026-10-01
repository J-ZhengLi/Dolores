import 'dart:async';
import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'dart:isolate';

import 'package:ffi/ffi.dart';
import 'package:path/path.dart' as path;

typedef _CallNative = Pointer<Utf8> Function(Pointer<Uint8>, UintPtr);
typedef _Call = Pointer<Utf8> Function(Pointer<Uint8>, int);
typedef _FreeNative = Void Function(Pointer<Utf8>);
typedef _Free = void Function(Pointer<Utf8>);

String libraryPath() {
  final directory = path.dirname(Platform.resolvedExecutable);
  if (Platform.isWindows) {
    return path.join(directory, 'dolores_flutter_bridge.dll');
  }
  if (Platform.isMacOS) {
    return path.normalize(
      path.join(
        directory,
        '..',
        'Frameworks',
        'libdolores_flutter_bridge.dylib',
      ),
    );
  }
  return path.join(directory, 'lib', 'libdolores_flutter_bridge.so');
}

void _worker(List<Object> arguments) {
  final parent = arguments[0] as SendPort;
  try {
    final library = DynamicLibrary.open(arguments[1] as String);
    final call = library.lookupFunction<_CallNative, _Call>('dolores_call');
    final free = library.lookupFunction<_FreeNative, _Free>('dolores_free');
    final requests = ReceivePort();
    parent.send(requests.sendPort);
    requests.listen((message) {
      final request = message as List<Object>;
      final id = request[0] as int;
      final json = request[1] as String;
      final input = json.toNativeUtf8();
      Pointer<Utf8>? result;
      try {
        result = call(input.cast(), utf8.encode(json).length);
        parent.send([id, result.toDartString()]);
      } catch (_) {
        parent.send([id, '{"ok":false,"error":"Native bridge failed."}']);
      } finally {
        calloc.free(input);
        if (result != null) free(result);
      }
    });
  } catch (_) {
    parent.send(
      'Could not load the Rust chat library. Rebuild the Flutter app.',
    );
  }
}

/// All native requests, including SQLite, execute outside the UI isolate.
abstract interface class ChatBridge {
  Future<void> open();
  Future<dynamic> call(Map<String, dynamic> command);
  Future<void> close();
}

class NativeBridge implements ChatBridge {
  final _responses = ReceivePort();
  final _errors = ReceivePort();
  final _pending = <int, Completer<dynamic>>{};
  final _ready = Completer<SendPort>();
  Isolate? _isolate;
  int _next = 0;
  bool _closed = false;
  NativeBridge() {
    _ready.future.ignore();
    _responses.listen((message) {
      if (message is SendPort) {
        if (!_ready.isCompleted) _ready.complete(message);
      } else if (message is String) {
        _fail(message);
      } else {
        final response = message as List<dynamic>;
        final completer = _pending.remove(response[0]);
        if (completer == null) return;
        final envelope = jsonDecode(response[1] as String);
        if (envelope['ok'] == true) {
          completer.complete(envelope['result']);
        } else {
          completer.completeError(envelope['error'] as String);
        }
      }
    });
    _errors.listen((_) => _fail('The chat worker stopped. Restart Dolores.'));
  }
  void _fail(String message) {
    if (!_ready.isCompleted) _ready.completeError(message);
    for (final request in _pending.values) {
      request.completeError(message);
    }
    _pending.clear();
    _closed = true;
  }

  @override
  Future<void> open({String? testLibrary}) async {
    _isolate = await Isolate.spawn(
      _worker,
      [_responses.sendPort, testLibrary ?? libraryPath()],
      onError: _errors.sendPort,
      onExit: _errors.sendPort,
    );
    await _ready.future;
  }

  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    if (_closed) throw 'The chat worker is closed.';
    final port = await _ready.future;
    final id = ++_next;
    final result = Completer<dynamic>();
    _pending[id] = result;
    port.send([id, jsonEncode(command)]);
    return result.future;
  }

  @override
  Future<void> close() async {
    if (!_closed && _ready.isCompleted) {
      try {
        await call({'command': 'shutdown'});
      } catch (_) {
        /* Already closed. */
      }
    }
    _closed = true;
    _isolate?.kill(priority: Isolate.immediate);
    _responses.close();
    _errors.close();
    _fail('Dolores closed.');
  }
}
