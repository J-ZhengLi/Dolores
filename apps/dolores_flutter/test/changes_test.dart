import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/changes.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

class JournalBridge implements ChatBridge {
  final commands = <Map<String, dynamic>>[];
  bool conflict = false, reverted = false;
  String status = 'applied';
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    commands.add(command);
    final change = {
      'id': 1,
      'createdAt': 1,
      'target': 'notes/世界.txt',
      'status': reverted ? 'reverted' : status,
      'reverts': null,
      'bytesBefore': 10,
      'bytesAfter': 9,
    };
    switch (command['command']) {
      case 'changesPage':
        return {
          'items': [change],
          'hasOlder': command['cursor'] == null,
          'hasNewer': false,
        };
      case 'changeDetails':
        return {
          'change': change,
          'diff': '--- before\n+++ after\n-before 世界\n+after 世界',
        };
      case 'previewRevert':
        return {
          'token': 'one-use',
          'target': change['target'],
          'diff': '--- before\n+++ after\n-after 世界\n+before 世界',
        };
      case 'applyRevert':
        if (conflict) {
          throw Exception('File changed since preview. No edit was applied.');
        }
        reverted = true;
        return {'applied': true, 'journalStatus': 'applied'};
      default:
        return null;
    }
  }
}

Future<void> open(
  WidgetTester tester,
  JournalBridge bridge, {
  bool dark = false,
}) async {
  tester.view.physicalSize = const Size(390, 700);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.resetPhysicalSize);
  addTearDown(tester.view.resetDevicePixelRatio);
  await tester.pumpWidget(
    MaterialApp(
      theme: doloresTheme(dark),
      home: Scaffold(
        body: Builder(
          builder: (context) => TextButton(
            onPressed: () => showDialog<void>(
              context: context,
              builder: (_) => ChangesInspector(bridge: bridge, session: 'chat'),
            ),
            child: const Text('Open'),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.text('Open'));
  await tester.pumpAndSettle();
}

Future<void> select(WidgetTester tester) async {
  await tester.tap(find.byKey(const Key('change-1')));
  await tester.pumpAndSettle();
  await tester.scrollUntilVisible(
    find.byKey(const Key('review-revert')),
    150,
    scrollable: find
        .descendant(
          of: find.byType(ChangesInspector),
          matching: find.byType(Scrollable),
        )
        .first,
  );
}

void main() {
  for (final dark in [false, true]) {
    testWidgets(
      'compact ${dark ? 'dark' : 'light'} review requires a separate revert decision',
      (tester) async {
        final bridge = JournalBridge();
        await open(tester, bridge, dark: dark);
        await select(tester);
        expect(
          bridge.commands.where((c) => c['command'] == 'applyRevert'),
          isEmpty,
        );
        await tester.tap(find.byKey(const Key('review-revert')));
        await tester.pumpAndSettle();
        expect(find.text('Revert this change?'), findsOneWidget);
        expect(
          bridge.commands.where((c) => c['command'] == 'applyRevert'),
          isEmpty,
        );
        await tester.tap(find.text('Cancel'));
        await tester.pumpAndSettle();
        expect(bridge.commands.last['command'], 'cancelRevert');
        await tester.tap(find.byKey(const Key('review-revert')));
        await tester.pumpAndSettle();
        await tester.tap(find.byKey(const Key('apply-revert')));
        await tester.pumpAndSettle();
        expect(
          bridge.commands
              .where((c) => c['command'] == 'applyRevert')
              .single['token'],
          'one-use',
        );
        expect(find.byKey(const Key('review-revert')), findsNothing);
        expect(tester.takeException(), isNull);
      },
    );
  }
  testWidgets(
    'pending receipt is explicit and conflict consumes preview without retry',
    (tester) async {
      final bridge = JournalBridge()
        ..conflict = true
        ..status = 'pending';
      await open(tester, bridge);
      await select(tester);
      expect(find.textContaining('Needs check'), findsOneWidget);
      expect(find.textContaining('An intent was saved'), findsOneWidget);
      await tester.tap(find.byKey(const Key('review-revert')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('apply-revert')));
      await tester.pumpAndSettle();
      expect(find.textContaining('File changed since preview'), findsOneWidget);
      expect(find.byKey(const Key('apply-revert')), findsNothing);
      expect(
        bridge.commands.where((c) => c['command'] == 'applyRevert').length,
        1,
      );
      expect(tester.takeException(), isNull);
    },
  );
  testWidgets(
    'closing a reviewed preview cancels and paging replaces the page',
    (tester) async {
      final bridge = JournalBridge();
      await open(tester, bridge);
      await tester.tap(find.text('Older changes'));
      await tester.pumpAndSettle();
      expect(bridge.commands.last['cursor'], 1);
      expect(find.byKey(const Key('change-1')), findsOneWidget);
      await select(tester);
      await tester.tap(find.byKey(const Key('review-revert')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Close'));
      await tester.pumpAndSettle();
      expect(bridge.commands.last['command'], 'cancelRevert');
      expect(
        bridge.commands.where((c) => c['command'] == 'applyRevert'),
        isEmpty,
      );
    },
  );
  test(
    'inspection excludes sending and folder changes until it closes',
    () async {
      final bridge = JournalBridge();
      final chat = ChatController(bridge)
        ..loading = false
        ..session = 'chat'
        ..workspaceRoot = 'chosen';
      await chat.inspectLocalChanges(() async {
        expect(chat.changing, isTrue);
        await chat.openProject('another');
        expect(bridge.commands, isEmpty);
      });
      expect(chat.changing, isFalse);
      chat.dispose();
    },
  );
}
