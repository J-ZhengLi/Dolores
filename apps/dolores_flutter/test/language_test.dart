import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/language_host.dart';
import 'package:dolores_flutter/file_host.dart';
import 'package:dolores_flutter/language_setup.dart';
import 'package:flutter/material.dart';

class LanguageBridge implements ChatBridge {
  final finished = Completer<void>();
  int version = 0;
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> v) async {
    final r = v['request'];
    if (r['action'] == 'feature') {
      version = r['version'];
      return {'id': 'job'};
    }
    if (r['action'] == 'poll') {
      await finished.future;
      return {
        'state': 'done',
        'document': 'doc',
        'version': version,
        'result': {'value': 'result'},
      };
    }
    return null;
  }
}

class SetupBridge implements ChatBridge {
  final calls = <String>[];
  bool cancel = false;
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> v) async {
    final action = v['request']['action'] as String;
    calls.add(action);
    switch (action) {
      case 'installInfo':
        return {
          'typescript': {'compiler': '6.0.3'},
          'rust': {'version': '2026-10-05'},
          'note': 'Pinned verified tools',
        };
      case 'install':
        return {'id': 'one'};
      case 'cancelInstall':
        cancel = true;
        return null;
      case 'installPoll':
        return cancel
            ? {
                'state': 'failed',
                'error': 'Installation canceled. Previous tools remain.',
              }
            : {'state': 'downloading', 'bytes': 1024};
      default:
        return null;
    }
  }
}

void main() {
  testWidgets(
    'setup is explicit; cancel returns a usable dialog with old tools retained',
    (t) async {
      final b = SetupBridge(), h = LanguageHost(SetupBridge());
      final host = LanguageHost(b);
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(body: LanguageSetup(host: host)),
        ),
      );
      await t.pump();
      expect(b.calls, ['installInfo']);
      await t.tap(find.text('Install TypeScript 6.0.3'));
      await t.pump();
      await t.pump(const Duration(milliseconds: 400));
      expect(find.text('Cancel installation'), findsOneWidget);
      await t.tap(find.text('Cancel installation'));
      await t.pump(const Duration(milliseconds: 400));
      await t.pump();
      expect(
        find.text('Installation canceled. Previous tools remain.'),
        findsOneWidget,
      );
      expect(b.calls.where((c) => c == 'install').length, 1);
      await t.pumpWidget(const SizedBox());
      await h.stop();
    },
  );
  test('language ranges respect UTF-16 and reject overlapping edits', () {
    expect(LanguageHost.offset('a😀b\n世界', {'line': 0, 'character': 3}), 3);
    expect(
      () => LanguageHost.offset('a😀b', {'line': 0, 'character': 2}),
      throwsStateError,
    );
    expect(
      LanguageHost.apply('a😀b', [
        {
          'range': {
            'start': {'line': 0, 'character': 1},
            'end': {'line': 0, 'character': 3},
          },
          'newText': '世界',
        },
      ]),
      'a世界b',
    );
    expect(
      () => LanguageHost.apply('abc', [
        for (var i = 0; i < 2; i++)
          {
            'range': {
              'start': {'line': 0, 'character': 0},
              'end': {'line': 0, 'character': 1},
            },
            'newText': 'x',
          },
      ]),
      throwsStateError,
    );
  });
  test(
    'late result after typing or project change cannot enter the editor',
    () async {
      final b = LanguageBridge(), files = FileHost(LanguageBridge());
      final w = FileWorkspace('p', 'root', 'chat');
      files.workspaces['p'] = w;
      files.selected = w;
      final d = FileDocument({
        'document': 'doc',
        'project': 'p',
        'version': 0,
        'text': 'before',
        'snapshot': {'path': 'a.ts', 'text': 'before', 'readonly': false},
      });
      final h = LanguageHost(b);
      final result = h.feature(files, w, d, 'hover', {
        'line': 0,
        'character': 0,
      });
      await Future<void>.delayed(Duration.zero);
      d.text = 'typed later';
      b.finished.complete();
      await expectLater(result, throwsStateError);
      expect(d.text, 'typed later');
      files.dispose();
      d.dispose();
    },
  );
}
