import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:flutter/material.dart';

import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/terminal_host.dart';
import 'package:dolores_flutter/terminal_page.dart';

class PtyBridge implements ChatBridge {
  final calls = <Map<String, dynamic>>[];
  final chunks = <List<int>>[];
  bool fail = false;
  int next = 0;
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    final r = Map<String, dynamic>.from(command['request']);
    calls.add(r);
    if (fail) throw 'Folder or terminal unavailable. Retry.';
    switch (r['action']) {
      case 'create':
        return {
          'id': 'pty-${++next}',
          'cwd': r['home'] == true ? 'HOME' : r['session'] ?? 'HOME',
          'shell': 'powershell.exe',
          'state': 'running',
        };
      case 'poll':
        return {
          'session': {'state': 'running'},
          'bytes': base64Encode(chunks.isEmpty ? [] : chunks.removeAt(0)),
          'outputError': false,
        };
      case 'stop':
        return {'state': 'stopped'};
      default:
        return null;
    }
  }
}

void main() {
  test(
    'terminal entry is lazy, reuses owner and retains Unicode across reads',
    () async {
      final bridge = PtyBridge(), host = TerminalHost(PtyBridge());
      host.dispose();
      final h = TerminalHost(bridge);
      expect(bridge.calls, isEmpty);
      await h.enter('A');
      final s = h.sessions.values.single;
      await h.enter('B');
      expect(h.sessions.length, 1);
      expect(s.cwd, 'A');
      final bytes = utf8.encode('终😀');
      bridge.chunks.addAll([bytes.sublist(0, 2), bytes.sublist(2)]);
      await h.poll(s);
      await h.poll(s);
      expect(s.terminal.buffer.getText(), contains('终😀'));
      h.dispose();
    },
  );
  testWidgets(
    'missing spawn keeps recovery usable; stopped shell keeps output',
    (tester) async {
      final b = PtyBridge()..fail = true;
      final h = TerminalHost(b);
      await h.enter('missing');
      expect(h.sessions, isEmpty);
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: TerminalPage(
              host: h,
              session: 'missing',
              chooseFolder: () {},
            ),
          ),
        ),
      );
      expect(find.text('Open at home'), findsOneWidget);
      b.fail = false;
      await tester.tap(find.text('Open at home'));
      await tester.pump();
      final s = h.sessions.values.single;
      expect(s.cwd, 'HOME');
      s.terminal.write('retained work');
      await h.stop(s);
      await tester.pump();
      expect(s.terminal.buffer.getText(), contains('retained work'));
      b.fail = true;
      await h.poll(s);
      expect(s.suspended, true);
      await tester.pump();
      expect(find.text('Retry connection'), findsOneWidget);
      b.fail = false;
      h.retry(s);
      await h.poll(s);
      expect(s.suspended, false);
      await tester.pumpWidget(const SizedBox());
      h.dispose();
    },
  );
}
