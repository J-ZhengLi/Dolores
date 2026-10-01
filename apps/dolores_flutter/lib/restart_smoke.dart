// Diagnostic build only. Each phase runs in a separate process and isolated DB.
import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:path/path.dart' as path;

import 'bridge.dart';
import 'chat.dart';
import 'main.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  final output = Platform.environment['DOLORES_SMOKE_DIR'];
  final data = Platform.environment['DOLORES_DATA_DIR'];
  final phase = int.tryParse(
    Platform.environment['DOLORES_RESTART_PHASE'] ?? '',
  );
  if (output == null ||
      data == null ||
      !path.isAbsolute(output) ||
      !path.isAbsolute(data) ||
      phase == null ||
      phase < 1 ||
      phase > 4) {
    throw StateError(
      'Restart testing requires isolated absolute directories and phase 1–4.',
    );
  }
  final chat = ChatController(NativeBridge());
  runApp(DoloresApp(chat: chat));
  unawaited(runPhase(chat, Directory(output), phase));
}

void require(bool condition, String message) {
  if (!condition) throw StateError(message);
}

Future<void> runPhase(ChatController chat, Directory output, int phase) async {
  String? error;
  try {
    await output.create(recursive: true);
    await chat.initialize();
    require(chat.error == null, chat.error ?? 'Initialization failed');
    switch (phase) {
      case 1:
        require(
          chat.sessions.isEmpty && !chat.configured,
          'Use fresh test data',
        );
        await chat.configure(
          'http://127.0.0.1:19421/v1',
          'dolores-mock',
          'dolores-generated-restart-test',
          remember: true,
          models: ['dolores-mock', 'dolores-fast'],
        );
        await chat.selectModel('dolores-fast');
        require(
          chat.configured && chat.rememberConnection && chat.hasSavedKey,
          'Saved connection is ready',
        );
      case 2:
        require(
          chat.model == 'dolores-fast' && chat.enabledModels.length == 2,
          'Model choices and active model survive restart',
        );
        final models = await chat.listModels(chat.baseUrl, null);
        require(
          models.contains('dolores-fast'),
          'Saved key authorizes model discovery',
        );
        require(
          chat.configured && chat.rememberConnection && chat.hasSavedKey,
          'Restart restores saved key',
        );
        chat.draft = 'credential-check';
        await chat.send();
        final deadline = DateTime.now().add(const Duration(seconds: 10));
        while (chat.busy && DateTime.now().isBefore(deadline)) {
          await Future<void>.delayed(const Duration(milliseconds: 25));
        }
        require(
          !chat.busy && chat.error == null && chat.messages.length == 2,
          'Restored key authorizes a full streamed turn',
        );
      case 3:
        final count = chat.messages.length;
        await chat.forgetConnection();
        require(
          !chat.configured &&
              !chat.hasSavedKey &&
              !chat.rememberConnection &&
              chat.messages.length == count,
          'Forget removes connection and preserves history',
        );
      case 4:
        require(
          !chat.configured &&
              !chat.hasSavedKey &&
              !chat.rememberConnection &&
              chat.messages.length == 2,
          'Forgotten connection stays forgotten after restart',
        );
    }
    final state = await chat.bridge.call({'command': 'bootstrap'});
    require(
      !jsonEncode(state).contains('dolores-generated-restart-test'),
      'Bootstrap never returns the key',
    );
  } catch (failure) {
    error = failure.toString();
  }
  await File(path.join(output.path, 'phase-$phase.json')).writeAsString(
    jsonEncode({
      'ok': error == null,
      'phase': phase,
      'error': ?error,
      'configured': chat.configured,
      'rememberConnection': chat.rememberConnection,
      'hasSavedKey': chat.hasSavedKey,
      'messages': chat.messages.length,
    }),
  );
}
