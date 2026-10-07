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
    'plus snapshots folder; moves retain one shell; failed plus retains tabs',
    () async {
      final b = PtyBridge();
      final h = TerminalHost(b);
      final a = (await h.create('A'))!, other = (await h.create('B'))!;
      expect(a.cwd, 'A');
      expect(other.cwd, 'B');
      final original = h.layout.activeGroup;
      expect(
        h.layout.split(original, a.id, Axis.horizontal, source: original),
        true,
      );
      final target = h.layout.activeGroup;
      final page = TerminalPage(host: h, session: 'B', chooseFolder: () {});
      page.drop(TerminalTabDrag(h, other.id, original), target, null);
      expect(h.sessions.length, 2);
      expect(h.sessions[a.id], same(a));
      expect(b.calls.where((c) => c['action'] == 'create').length, 2);
      page.drop(TerminalTabDrag(h, 'missing', original), target, Axis.vertical);
      expect(h.layout.groups.length, 2);
      b.fail = true;
      expect(await h.create('C'), null);
      expect(h.sessions.length, 2);
      expect(h.layout.active.tabs, contains(a.id));
      h.layout.closeGroup(original);
      expect(h.layout.groups.length, 1);
      expect(h.layout.active.tabs.toSet(), {a.id, other.id});
      h.dispose();
    },
  );
  testWidgets(
    'split creates a new shell, close can cancel, compact groups remain reachable',
    (t) async {
      t.view.physicalSize = const Size(1100, 700);
      t.view.devicePixelRatio = 1;
      addTearDown(t.view.resetPhysicalSize);
      addTearDown(t.view.resetDevicePixelRatio);
      final b = PtyBridge(), h = TerminalHost(PtyBridge());
      h.dispose();
      final host = TerminalHost(b);
      await host.create('A');
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: TerminalPage(host: host, session: 'B', chooseFolder: () {}),
          ),
        ),
      );
      await t.tap(find.byTooltip('Terminal actions'));
      await t.pumpAndSettle();
      await t.tap(find.text('Split terminal right'));
      await t.pumpAndSettle();
      expect(host.sessions.length, 2);
      expect(host.sessions.values.last.cwd, 'B');
      expect(host.layout.groups.length, 2);
      await t.tap(find.byTooltip('Close terminal').first);
      await t.pumpAndSettle();
      await t.tap(find.text('Keep open'));
      await t.pumpAndSettle();
      expect(host.sessions.length, 2);
      t.view.physicalSize = const Size(420, 480);
      await t.pump();
      await t.tap(find.byTooltip('Terminal actions'));
      await t.pumpAndSettle();
      expect(find.textContaining('Focus group-'), findsOneWidget);
      expect(t.takeException(), isNull);
      await t.pumpWidget(const SizedBox());
      host.dispose();
    },
  );
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
