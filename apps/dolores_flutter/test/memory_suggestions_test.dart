import 'package:dolores_flutter/memory.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'memory_test.dart' as manual;

class SuggestionBridge extends manual.MemoryBridge {
  bool slow = false, stopped = false, generationFailure = false, empty = false;
  int generations = 0;
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    switch (command['command']) {
      case 'reviewMemorySources':
        commands.add(command);
        return {
          'token': 'sources',
          'model': 'fixture',
          'folderAvailable': folder,
          'hasOlder': true,
          'items': [
            {'messageId': 1, 'text': 'I prefer concise examples.'},
            {'messageId': 3, 'text': 'Fix this temporary task.'},
          ],
        };
      case 'suggestMemories':
        commands.add(command);
        generations++;
        return null;
      case 'poll':
        commands.add(command);
        if (slow && !stopped) return [];
        return [
          {
            'type': 'done',
            'id': command['id'],
            if (stopped || generationFailure)
              'error': 'Suggestions failed or stopped. Nothing was saved.'
            else
              'memorySuggestions': {
                'token': 'suggestions',
                'model': 'fixture',
                'usage': {'inputTokens': 100, 'outputTokens': 20},
                'items': empty
                    ? []
                    : [
                        {
                          'title': 'Response style',
                          'text': 'Prefer concise examples.',
                          'messageId': 1,
                          'quote': 'I prefer concise examples.',
                        },
                      ],
              },
          },
        ];
      case 'cancel':
        commands.add(command);
        stopped = true;
        return null;
      case 'discardMemoryReview':
        commands.add(command);
        return null;
      case 'saveMemorySuggestion':
        commands.add(command);
        if (this.fail) {
          throw 'Storage failed. Nothing was saved.';
        }
        final item = {
          'id': 'reviewed',
          'revision': 1,
          'scope': command['scope'],
          'title': command['title'],
          'text': command['text'],
          'enabled': command['enabled'],
          'source': 'conversation',
          'updatedAt': 1,
          'createdAt': 1,
          'origin': {
            'messageId': 1,
            'session': 'chat',
            'quote': 'I prefer concise examples.',
            'model': 'fixture',
            'reviewedAt': 1,
          },
          'originAvailable': false,
        };
        items.add(item);
        return item;
    }
    return super.call(command);
  }
}

Future<void> open(
  WidgetTester tester,
  SuggestionBridge bridge, {
  bool dark = true,
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
              builder: (_) => MemoryInspector(bridge: bridge, session: 'chat'),
            ),
            child: const Text('Open'),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.text('Open'));
  await tester.pumpAndSettle();
  await tester.tap(find.byKey(const Key('suggest-from-chat')));
  await tester.pumpAndSettle();
}

Future<void> select(WidgetTester tester) async {
  final finder = find.byKey(const Key('memory-source-1'));
  await tester.scrollUntilVisible(
    finder,
    150,
    scrollable: find.byType(Scrollable).first,
  );
  await tester.ensureVisible(finder);
  await tester.pumpAndSettle();
  await tester.tap(
    find.descendant(of: finder, matching: find.byType(Checkbox)),
  );
  await tester.pumpAndSettle();
}

Future<void> generate(WidgetTester tester) async {
  await select(tester);
  await tester.tap(find.byKey(const Key('generate-memory-suggestions')));
  await tester.pumpAndSettle();
}

void main() {
  testWidgets(
    'source sharing is explicit; compact review correction is saved once with origin',
    (tester) async {
      final bridge = SuggestionBridge();
      await open(tester, bridge);
      expect(bridge.generations, 0);
      expect(bridge.items, isEmpty);
      expect(
        tester
            .widget<FilledButton>(
              find.byKey(const Key('generate-memory-suggestions')),
            )
            .onPressed,
        isNull,
      );
      await tester.scrollUntilVisible(
        find.textContaining('Earlier history excluded'),
        150,
        scrollable: find.byType(Scrollable).first,
      );
      expect(find.textContaining('Earlier history excluded'), findsOneWidget);
      await generate(tester);
      expect(bridge.generations, 1);
      expect(bridge.items, isEmpty);
      expect(
        bridge.commands.singleWhere(
          (c) => c['command'] == 'suggestMemories',
        )['messageIds'],
        [1],
      );
      await tester.scrollUntilVisible(
        find.byKey(const Key('review-suggestion-0')),
        150,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.tap(find.byKey(const Key('review-suggestion-0')));
      await tester.pumpAndSettle();
      await manual.enter(tester, 'memory-text', 'Prefer one short example.');
      await tester.tap(find.byKey(const Key('save-memory')));
      await tester.pumpAndSettle();
      expect(bridge.items.single['text'], 'Prefer one short example.');
      expect(bridge.items.single['scope'], 'folder');
      expect(bridge.items.single['origin']['messageId'], 1);
      await tester.scrollUntilVisible(
        find.byKey(const Key('review-suggestion-0')),
        150,
        scrollable: find.byType(Scrollable).first,
      );
      expect(
        tester
            .widget<TextButton>(find.byKey(const Key('review-suggestion-0')))
            .onPressed,
        isNull,
      );
      await tester.tap(find.byKey(const Key('discard-memory-suggestions')));
      await tester.pumpAndSettle();
      await tester.scrollUntilVisible(
        find.textContaining('no longer available'),
        150,
        scrollable: find.byType(Scrollable).first,
      );
      expect(find.textContaining('no longer available'), findsOneWidget);
      expect(tester.takeException(), isNull);
    },
  );
  testWidgets(
    'failed activation retains edited draft and Cancel Discard never save',
    (tester) async {
      final bridge = SuggestionBridge()..fail = true;
      await open(tester, bridge, dark: false);
      await generate(tester);
      await tester.scrollUntilVisible(
        find.byKey(const Key('review-suggestion-0')),
        150,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.tap(find.byKey(const Key('review-suggestion-0')));
      await tester.pumpAndSettle();
      await manual.enter(tester, 'memory-text', 'Corrected draft');
      await tester.tap(find.byKey(const Key('save-memory')));
      await tester.pumpAndSettle();
      expect(
        tester
            .widget<TextField>(find.byKey(const Key('memory-text')))
            .controller!
            .text,
        'Corrected draft',
      );
      expect(bridge.items, isEmpty);
      await tester.tap(find.byKey(const Key('cancel-memory-edit')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('discard-memory-suggestions')));
      await tester.pumpAndSettle();
      expect(bridge.items, isEmpty);
      expect(bridge.generations, 1);
      expect(tester.takeException(), isNull);
    },
  );
  testWidgets(
    'Stop remains available and never saves or automatically retries',
    (tester) async {
      final bridge = SuggestionBridge()..slow = true;
      await open(tester, bridge);
      await select(tester);
      await tester.tap(find.byKey(const Key('generate-memory-suggestions')));
      await tester.pump(const Duration(milliseconds: 100));
      expect(
        tester
            .widget<TextButton>(
              find.byKey(const Key('stop-memory-suggestions')),
            )
            .onPressed,
        isNotNull,
      );
      await tester.tap(find.byKey(const Key('stop-memory-suggestions')));
      await tester.pumpAndSettle();
      expect(bridge.generations, 1);
      expect(bridge.items, isEmpty);
      expect(
        tester
            .widget<CheckboxListTile>(find.byKey(const Key('memory-source-1')))
            .value,
        true,
      );
      expect(tester.takeException(), isNull);
    },
  );
  testWidgets('empty response saves nothing and Close discards the review', (
    tester,
  ) async {
    final bridge = SuggestionBridge()..empty = true;
    await open(tester, bridge);
    await generate(tester);
    await tester.scrollUntilVisible(
      find.textContaining('No stable preferences found'),
      150,
      scrollable: find.byType(Scrollable).first,
    );
    expect(find.textContaining('No stable preferences found'), findsOneWidget);
    await tester.tap(find.text('Close'));
    await tester.pumpAndSettle();
    expect(bridge.items, isEmpty);
    expect(
      bridge.commands.where((c) => c['command'] == 'discardMemoryReview'),
      isNotEmpty,
    );
    expect(tester.takeException(), isNull);
  });
}
