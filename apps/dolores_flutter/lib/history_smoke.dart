// Diagnostic entry point, excluded from the normal release.
import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:path/path.dart' as path;

import 'bridge.dart';
import 'chat.dart';
import 'main.dart';
import 'smoke.dart' as helpers;

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  final output = Platform.environment['DOLORES_SMOKE_DIR'];
  final data = Platform.environment['DOLORES_DATA_DIR'];
  if (output == null ||
      data == null ||
      !path.isAbsolute(output) ||
      !path.isAbsolute(data)) {
    throw StateError('History checks require isolated absolute directories.');
  }
  final chat = ChatController(NativeBridge());
  final capture = GlobalKey();
  runApp(
    DoloresApp(chat: chat, captureKey: capture, themeMode: ThemeMode.light),
  );
  unawaited(runChecks(chat, capture, Directory(output)));
}

Future<void> runChecks(
  ChatController chat,
  GlobalKey capture,
  Directory output,
) async {
  final checks = <String>[];
  void check(bool result, String name) => helpers.check(result, name, checks);
  try {
    await chat.initialize();
    check(
      chat.error == null &&
          chat.session == 'long' &&
          chat.sessions.length == 50 &&
          chat.messages.length == 80,
      'Legacy history migration opens bounded latest pages',
    );
    final ids = chat.sessions.map((s) => s['id']).toSet();
    while (chat.sessionsOlder) {
      await chat.browseSessions(newer: false);
      check(
        chat.error == null && chat.sessions.length <= 50,
        'Sidebar paging remains bounded',
      );
      for (final item in chat.sessions) {
        if (!ids.add(item['id'])) {
          throw StateError('Duplicate session');
        }
      }
    }
    check(
      ids.length == 137,
      'Every conversation beyond the previous 100-session cap is reachable',
    );
    await chat.browseSessions(newer: true);
    check(
      chat.sessions.length == 50 && chat.sessionsNewer,
      'Newer sidebar page preserves stable ordering',
    );
    final messages = chat.messages.map((m) => m['id']).toSet();
    while (chat.messagesOlder) {
      await chat.browseMessages(newer: false);
      if (chat.error != null || chat.messages.length > 80) {
        throw StateError('Message page failed');
      }
      for (final item in chat.messages) {
        if (!messages.add(item['id'])) throw StateError('Duplicate message');
      }
    }
    check(
      messages.length == 246 && chat.messages.first['content'] == '你好 0',
      'All 123 complete turns are reachable without accumulating pages',
    );
    chat.draft = 'Keep this draft';
    chat.rememberScroll(120);
    await chat.select('fixture-000');
    chat.draft = 'Other draft';
    await chat.select('long');
    check(
      chat.draft == 'Keep this draft' &&
          chat.scrollOffset == 120 &&
          chat.messages.first['content'] == '你好 0',
      'Switching restores the draft, older page and scroll position',
    );
    for (final format in ['json', 'markdown']) {
      final extension = format == 'json' ? 'json' : 'md';
      final filename = path.join(output.path, 'complete.$extension');
      final count = await chat.exportConversation(format, () async => filename);
      check(
        count == 246 && chat.draft == 'Keep this draft',
        'Complete $format export preserves the draft',
      );
    }
    final exported = jsonDecode(
      await File(path.join(output.path, 'complete.json')).readAsString(),
    );
    check(
      exported['messages'].length == 246 &&
          exported['messages'].first['content'] == '你好 0' &&
          exported['messages'].last['content'].contains('answer 122'),
      'Export includes both oldest and newest saved messages',
    );
    final cancelled = await chat.exportConversation('json', () async => null);
    check(
      cancelled == null && chat.error == null,
      'Cancelled export makes no file request',
    );
    final denied = await chat.exportConversation(
      'json',
      () async => path.join(output.path, 'complete.json'),
    );
    check(
      denied == null && chat.error!.contains('already exists'),
      'Existing export is never overwritten',
    );
    await chat.configure('http://127.0.0.1:19421/v1', 'dolores-mock', '');
    chat.draft = 'hello';
    await chat.send();
    await helpers.waitUntil(() => !chat.busy && !chat.changing);
    check(
      chat.error == null &&
          !chat.messagesNewer &&
          chat.messages.length == 80 &&
          chat.messages.last['content'].contains('你好！'),
      'Sending from an earlier page returns to latest with bounded provider context',
    );
    await helpers.screenshot(capture, output, 'history-light');
    await chat.browseMessages(newer: false);
    runApp(
      DoloresApp(chat: chat, captureKey: capture, themeMode: ThemeMode.dark),
    );
    await helpers.screenshot(capture, output, 'history-dark');
    await File(path.join(output.path, 'report.json')).writeAsString(
      jsonEncode({
        'ok': true,
        'checks': checks,
        'sessionsReached': ids.length,
        'messagesExported': 246,
        'visibleMessages': chat.messages.length,
        'scope': 'Release renderer, controller, FFI, legacy SQLite migration and actual export files; native Save dialog interaction is not exercised.',
      }),
    );
  } catch (failure) {
    await File(path.join(output.path, 'report.json')).writeAsString(
      jsonEncode({'ok': false, 'checks': checks, 'error': failure.toString()}),
    );
  }
}
