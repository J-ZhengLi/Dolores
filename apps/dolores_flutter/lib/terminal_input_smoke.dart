/// Non-executing native keyboard fixture using the production terminal view.
library;

import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:window_manager/window_manager.dart';

import 'bridge.dart';
import 'desktop_frame.dart';
import 'terminal_host.dart';
import 'terminal_page.dart';
import 'theme.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final directory = Directory(
    Platform.environment['DOLORES_TERMINAL_INPUT_DIR'] ?? '',
  ).absolute;
  if (!directory.path.replaceAll('\\', '/').contains('/output/')) {
    throw StateError('An isolated output directory is required.');
  }
  await directory.create(recursive: true);
  await initializeDesktopFrame();
  await windowManager.setTitle(
    'Dolores non-executing terminal keyboard fixture',
  );
  final bridge = InputFixtureBridge(directory);
  final host = TerminalHost(bridge);
  bridge.host = host;
  await host.enter('fixture');
  host.sessions.values.single.terminal.write(
    'Keyboard fixture only. No shell or command execution.\r\nINPUT> ',
  );
  HardwareKeyboard.instance.addHandler((event) {
    bridge.keys.add({
      'type': '${event.runtimeType}',
      'key': event.logicalKey.debugName,
      'character': event.character,
    });
    bridge.publish();
    return false;
  });
  runApp(
    MaterialApp(
      theme: doloresTheme(false),
      debugShowCheckedModeBanner: false,
      builder: (context, child) => DesktopFrame(child: child!),
      home: Scaffold(
        body: TerminalPage(host: host, session: 'fixture', chooseFolder: () {}),
      ),
    ),
  );
  await bridge.publish();
}

class InputFixtureBridge implements ChatBridge {
  final Directory directory;
  late TerminalHost host;
  final keys = <Map<String, dynamic>>[];
  final inputs = <String>[];
  InputFixtureBridge(this.directory);
  Future<void> publish() async {
    final file = File('${directory.path}/input.json');
    // Serialize small public evidence writes rather than concurrent renames.
    _writing = _writing
        .then(
          (_) => file.writeAsString(
            jsonEncode({
              'keys': keys,
              'inputs': inputs,
              'focus': host.sessions.values.firstOrNull?.focus.hasFocus,
              'shellProcesses': 0,
              'modelRequests': 0,
            }),
          ),
        )
        .then((_) {});
    await _writing;
  }

  Future<void> _writing = Future<void>.value();
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> value) async {
    final request = value['request'] as Map;
    switch (request['action']) {
      case 'restore':
        return null;
      case 'create':
        return {
          'id': 'keyboard-fixture',
          'cwd': directory.path,
          'shell': 'Keyboard fixture',
          'state': 'running',
        };
      case 'input':
        inputs.add(request['text'] as String);
        host.sessions.values.single.terminal.write(
          request['text'] == '\r' ? '\r\nINPUT> ' : request['text'],
        );
        await publish();
        return null;
      case 'poll':
        return {
          'session': {'state': 'running'},
          'bytes': '',
          'outputError': false,
        };
      default:
        return null;
    }
  }
}
