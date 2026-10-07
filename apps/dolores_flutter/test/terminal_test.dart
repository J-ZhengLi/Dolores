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
  dynamic saved;
  bool failSave = false;
  final states = <String,String>{};
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
      case 'restore': return saved;
      case 'checkpoint':
        if (failSave) throw StateError('Disk full; previous checkpoint remains.');
        saved = r['value']; return null;
      case 'create':
        states['pty-${next+1}'] = 'running';
        return {
          'id': 'pty-${++next}',
          'cwd': r['home'] == true ? 'HOME' : r['session'] ?? 'HOME',
          'shell': 'powershell.exe',
          'state': 'running',
        };
      case 'poll':
        return {
          'session': {'state': states[r['id']] ?? 'stopped'},
          'bytes': base64Encode(chunks.isEmpty ? [] : chunks.removeAt(0)),
          'outputError': false,
        };
      case 'stop':
        states[r['id']] = 'stopped';
        return {'state': 'stopped'};
      default:
        return null;
    }
  }
}

void main() {
  test('cold recovery retains split output without creating shells; failed checkpoint keeps owners', () async {
    final b = PtyBridge(); final first = TerminalHost(b);
    final a = (await first.create('A'))!, other = (await first.create('B'))!;
    a.terminal.write('kept 世界'); other.terminal.write('second result');
    final g = first.layout.activeGroup;
    first.layout.split(g,a.id,Axis.horizontal,source:g);
    await first.checkpoint(); first.dispose();
    final calls = b.next;
    final second = TerminalHost(b); await second.enter('C');
    expect(b.next,calls); expect(second.live,isEmpty);
    expect(second.layout.groups.length,2);
    expect(second.sessions[a.id]!.terminal.buffer.getText(),contains('kept 世界'));
    b.failSave = true;
    expect(second.checkpoint(),throwsStateError);
    expect(second.sessions.length,2);
    await Future<void>.delayed(Duration.zero);
    expect(second.checkpointFailed,true);
    b.failSave = false; await second.checkpoint();
    expect(second.checkpointFailed,false);
    expect(await second.close(second.sessions[a.id]!),true);
    second.dispose();
    expect(utf8.encode(TerminalHost.tail('😀'*5000)).length,lessThanOrEqualTo(8192));
  });
  test('malformed recovery preserves checkpoint and offers a fresh shell', () async {
    final b = PtyBridge()..saved = {'version':999};
    final h = TerminalHost(b); await h.enter('A');
    expect(h.sessions,isEmpty); expect(h.error,contains('previous checkpoint remains'));
    expect(b.saved,{'version':999});
    await h.create('A'); expect(h.live.length,1); h.dispose();
  });
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
