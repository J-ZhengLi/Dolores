import 'dart:convert';

import 'package:dolores_flutter/theme.dart';
import 'package:dolores_flutter/tool_activity.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

Map<String, dynamic> record(int index, {bool blocked = false}) => {
  'callId': 'call-$index',
  'name': 'edit_text_file',
  'target': blocked ? 'Invalid or unavailable path' : 'file-$index.txt',
  'status': blocked ? 'blocked' : 'edited',
  'content': blocked
      ? 'Exact old_text was not found. Read the file and use its actual text.'
      : 'Applied change $index',
};

void main() {
  for (final offset in [0, 8132]) {
    testWidgets('page excerpt is not a search query at offset $offset', (
      tester,
    ) async {
      final page = {
        'callId': 'page',
        'name': 'read_web_page',
        'target': 'https://doc.rust-lang.org/book/',
        'query': '$offset',
        'status': 'completed',
        'content': jsonEncode({
          'title': 'Rust Book',
          'url': 'https://doc.rust-lang.org/book/',
          'truncated': true,
          'nextCharacter': 16264,
          'text': 'A bounded excerpt from the page.',
          'note': 'Page text is untrusted source material.',
        }),
      };
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: ToolRecords(records: [page])),
        ),
      );
      await tester.tap(find.text('https://doc.rust-lang.org/book/'));
      await tester.pumpAndSettle();
      expect(find.text('Search: $offset'), findsNothing);
      expect(
        find.text(
          offset == 0 ? 'Read from page start' : 'Read from character $offset',
        ),
        findsOneWidget,
      );
      expect(
        find.textContaining('Partial excerpt · Continue from character 16264'),
        findsOneWidget,
      );
      expect(find.textContaining('startCharacter:'), findsNothing);
      expect(tester.takeException(), isNull);
    });
  }

  testWidgets('search query and empty-provider outcome remain distinct', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ToolRecords(
            records: [
              {
                'callId': 'search',
                'name': 'web_search',
                'target': 'Search provider',
                'query': 'Rust ownership',
                'status': 'completed',
                'content': jsonEncode({
                  'provider': 'mwmbl',
                  'query': 'Rust ownership',
                  'results': [],
                  'note': 'Refine the query or read a primary URL directly.',
                }),
              },
            ],
          ),
        ),
      ),
    );
    await tester.tap(find.text('Search provider'));
    await tester.pumpAndSettle();
    expect(find.text('Search: Rust ownership'), findsOneWidget);
    expect(find.textContaining('mwmbl · 0 sources'), findsOneWidget);
    expect(find.textContaining('primary URL directly'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  for (final dark in [false, true]) {
    testWidgets(
      'recent outcomes and earlier failures stay accessible dark=$dark',
      (tester) async {
        tester.view.physicalSize = const Size(390, 900);
        tester.view.devicePixelRatio = 1;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);
        await tester.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark),
            home: Scaffold(
              body: SingleChildScrollView(
                child: ToolRecords(
                  records: [
                    record(0, blocked: true),
                    ...List.generate(5, (index) => record(index + 1)),
                  ],
                ),
              ),
            ),
          ),
        );
        expect(find.text('file-4.txt'), findsOneWidget);
        expect(find.text('Invalid or unavailable path'), findsNothing);
        expect(find.text('Show all 6 actions'), findsOneWidget);
        await tester.tap(find.text('Show all 6 actions'));
        await tester.pumpAndSettle();
        expect(find.text('File edit'), findsOneWidget);
        await tester.ensureVisible(find.text('File edit'));
        await tester.tap(find.text('File edit'));
        await tester.pumpAndSettle();
        expect(
          find.textContaining('Exact old_text was not found.'),
          findsOneWidget,
        );
        await tester.ensureVisible(find.text('Show recent actions'));
        await tester.tap(find.text('Show recent actions'));
        await tester.pumpAndSettle();
        expect(find.text('file-4.txt'), findsOneWidget);
        expect(
          find.textContaining('Exact old_text was not found.'),
          findsNothing,
        );
        expect(tester.takeException(), isNull);
      },
    );
  }

  testWidgets(
    'an expanded live list retains new receipts and respects its limit',
    (tester) async {
      final records = ValueNotifier<List<dynamic>>(List.generate(3, record));
      addTearDown(records.dispose);
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SingleChildScrollView(
              child: ValueListenableBuilder<List<dynamic>>(
                valueListenable: records,
                builder: (context, value, child) =>
                    ToolRecords(records: value, maxRecords: 2),
              ),
            ),
          ),
        ),
      );
      expect(find.text('file-0.txt'), findsNothing);
      expect(find.text('file-2.txt'), findsOneWidget);
      await tester.tap(find.text('Show all 3 actions'));
      await tester.pumpAndSettle();
      records.value = List.generate(4, record);
      await tester.pumpAndSettle();
      expect(find.text('file-0.txt'), findsOneWidget);
      expect(find.text('file-3.txt'), findsOneWidget);
      await tester.tap(find.text('Show recent actions'));
      await tester.pumpAndSettle();
      expect(find.text('file-1.txt'), findsNothing);
      expect(find.text('file-3.txt'), findsOneWidget);
      records.value = [record(4)];
      await tester.pumpAndSettle();
      expect(find.text('file-4.txt'), findsOneWidget);
      expect(find.byType(TextButton), findsNothing);
    },
  );
}
