import 'dart:convert';
import 'dart:io';

import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets('real SQLite write contention cannot veto the app close path', (
    t,
  ) async {
    final directory = Directory(Platform.environment['DOLORES_DATA_DIR']!);
    final root = directory.parent;
    final output = Directory('../../output').absolute
        .resolveSymbolicLinksSync();
    expect(
      root.resolveSymbolicLinksSync(),
      startsWith('$output${Platform.pathSeparator}'),
    );
    final bridge = (await t.runAsync(() async {
      final bridge = NativeBridge();
      await bridge
          .open(testLibrary: Platform.environment['DOLORES_AUDIT_LIBRARY'])
          .timeout(const Duration(seconds: 10));
      return bridge;
    }))!;
    AppHost? host;
    Process? locker;
    addTearDown(() async {
      locker?.kill();
      host?.dispose();
      await t.runAsync(() async {
        await locker?.exitCode;
        await bridge.close();
      });
    });
    final session =
        (await t.runAsync(
              () => bridge.call({
                'command': 'createSession',
                'kind': 'temporary',
              }),
            ))['session']['id']
            as String;
    await t.runAsync(
      () => bridge.call({
        'command': 'saveDraft',
        'session': session,
        'text': 'Public retained draft',
      }),
    );
    final chat = ChatController(bridge, ownsBridge: false)
      ..loading = false
      ..session = session
      ..draft = 'Public retained draft';
    host = AppHost(chat);
    await t.pump(const Duration(seconds: 1));
    await t.runAsync(
      () => Future<void>.delayed(const Duration(milliseconds: 100)),
    );
    await t.pump();
    final ready = File('${root.path}/write-lock-ready');
    final python = Platform.environment['DOLORES_TEST_PYTHON'] ?? 'python';
    locker = await t.runAsync(
      () => Process.start(python, [
        '-c',
        '''
import sqlite3, sys, time
from pathlib import Path
db = sqlite3.connect(sys.argv[1], timeout=1)
db.execute('BEGIN IMMEDIATE')
Path(sys.argv[2]).write_text('locked')
time.sleep(30)
db.rollback()
''',
        '${directory.path}/dolores.db',
        ready.path,
      ]),
    );
    await t.runAsync(() async {
      final deadline = DateTime.now().add(const Duration(seconds: 3));
      while (!ready.existsSync() && DateTime.now().isBefore(deadline)) {
        await Future<void>.delayed(const Duration(milliseconds: 20));
      }
    });
    expect(
      ready.existsSync(),
      true,
      reason: 'The real SQLite write lock must be held before closing.',
    );
    final watch = Stopwatch()..start();
    final close = await t.runAsync(host.requestClose);
    watch.stop();
    expect(close, true);
    expect(watch.elapsedMilliseconds, lessThan(2300));
    expect(
      host.error,
      isNotNull,
      reason: 'The blocked checkpoint must exercise failure recovery.',
    );
    locker!.kill();
    await t.runAsync(() async {
      await locker!.exitCode;
    });
    final checked = await t.runAsync(
      () => Process.run(python, [
        '-c',
        '''
import sqlite3, sys
db = sqlite3.connect(sys.argv[1])
assert db.execute('PRAGMA integrity_check').fetchone()[0] == 'ok'
assert db.execute('SELECT text FROM session_drafts WHERE session_id = ?', (sys.argv[2],)).fetchone()[0] == 'Public retained draft'
''',
        '${directory.path}/dolores.db',
        session,
      ]),
    );
    expect(checked!.exitCode, 0, reason: checked.stderr.toString());
    File('${root.path}/close-receipt.json').writeAsStringSync(
      jsonEncode({
        'scope': 'Real Rust/SQLite bridge and AppHost close path; no physical window/input claim',
        'sqliteWriteLockHeld': true,
        'closeAllowed': close,
        'elapsedMs': watch.elapsedMilliseconds,
        'checkpointFailureObserved': host.error != null,
        'sqliteIntegrity': 'ok',
        'priorSavedDraftRetained': true,
        'modelRequests': 0,
      }),
    );
  });
}
