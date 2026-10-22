import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/comparison.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:dolores_flutter/form_field_layout.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'history_test.dart';

// Public deterministic receipts reopened through the native bridge, not live
// model answers. Keep unknown provider usage unknown.
class SavedComparisonBridge extends HistoryBridge {
  final receipts = (jsonDecode(
    File('test/fixtures/comparison_receipts.json').readAsStringSync(),
  ) as List).cast<Map<String, dynamic>>();

  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    commands.add(command);
    switch (command['command']) {
      case 'comparisonSources':
        return {
          'items': [],
          'model': 'fixture',
          'settings': {'maxOutputTokens': 1024, 'timeoutSeconds': 20},
        };
      case 'comparisonsPage':
        return {
          'items': receipts.map((r) => r['summary']).toList(),
          'hasOlder': false,
        };
      case 'comparison':
        return receipts.singleWhere((r) => r['id'] == command['comparisonId']);
      default:
        throw StateError(
          'Viewing a receipt must not mutate or start a run: $command',
        );
    }
  }
}

Future<void> reveal(WidgetTester tester, Finder finder) async {
  await tester.scrollUntilVisible(
    finder,
    200,
    scrollable: find.byType(Scrollable).first,
    maxScrolls: 60,
  );
  await tester.ensureVisible(finder);
  await tester.pumpAndSettle();
}

void main() {
  const renderDirectory = String.fromEnvironment('COMPARISON_RENDER_DIR');
  setUpAll(() async {
    if (renderDirectory.isEmpty) return;
    final font = ByteData.sublistView(
      await File(
        '${Platform.environment['WINDIR'] ?? 'C:/Windows'}/Fonts/segoeui.ttf',
      ).readAsBytes(),
    );
    for (final family in ['Ahem', 'Segoe UI', 'Roboto']) {
      await (FontLoader(family)..addFont(Future.value(font))).load();
    }
    await ui.loadFontFromList(
      await File(
        '../../output/toolchains/flutter/bin/cache/artifacts/material_fonts/MaterialIcons-Regular.otf',
      ).readAsBytes(),
      fontFamily: 'MaterialIcons',
    );
  });
  for (final dark in [false, true]) {
    testWidgets(
      'saved receipts retain exact evidence without activation ($dark)',
      (tester) async {
        tester.view.physicalSize = const Size(700, 780);
        tester.view.devicePixelRatio = 1;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);
        final bridge = SavedComparisonBridge();
        final chat = ChatController(bridge)
          ..session = 'public-comparison-audit'
          ..loading = false;
        addTearDown(chat.dispose);
        final boundary = GlobalKey();
        await tester.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark).copyWith(
              textButtonTheme: TextButtonThemeData(
                style: TextButton.styleFrom(
                  textStyle: const TextStyle(
                    fontFamily: 'Segoe UI',
                    fontSize: 14,
                  ),
                ),
              ),
            ),
            home: RepaintBoundary(
              key: boundary,
              child: ComparisonInspector(chat: chat),
            ),
          ),
        );
        await tester.pumpAndSettle();
        for (final receipt in bridge.receipts) {
          final row = find.widgetWithText(
            ListTile,
            'fixture · ${receipt['status']} · #${receipt['id']}',
          );
          await reveal(tester, row);
          await tester.tap(row);
          await tester.pumpAndSettle();
          await tester.drag(
            find.byType(ListView).first,
            const Offset(0, 10000),
          );
          await tester.pumpAndSettle();
          expect(
            find.text('${receipt['draft']['title']} · ${receipt['status']}'),
            findsOneWidget,
          );
          expect(
            find.textContaining(
              receipt['summary']['improved'] == true
                  ? 'no memory or skill was activated'
                  : 'No complete strict improvement',
            ),
            findsOneWidget,
          );
          if (receipt['status'] == 'running') {
            expect(
              find.textContaining('no automatic resumption'),
              findsOneWidget,
            );
          }
          for (final (index, result) in (receipt['results'] as List).indexed) {
            final tile = find.widgetWithText(
              ExpansionTile,
              'Test ${index ~/ 2 + 1} · ${index.isEven ? 'Baseline' : 'Candidate'} · ${result['outcome']}',
            );
            await reveal(tester, tile);
            // Switching saved receipts can retain an expanded row at this position.
            if (find
                .descendant(of: tile, matching: find.byType(SelectableText))
                .evaluate()
                .isEmpty) {
              await tester.tap(
                find
                    .descendant(of: tile, matching: find.byType(ListTile))
                    .first,
              );
              await tester.pumpAndSettle();
            }
            expect(
              find.descendant(of: tile, matching: find.text(result['output'])),
              findsOneWidget,
            );
            expect(
              find.descendant(
                of: tile,
                matching: find.text(
                  'Elapsed: ${result['elapsedMs']} ms · Reported usage: null',
                ),
              ),
              findsOneWidget,
            );
            if (result['detail'] != null) {
              expect(
                find.descendant(
                  of: tile,
                  matching: find.text(result['detail']),
                ),
                findsOneWidget,
              );
            }
          }
          final frozen = find.widgetWithText(
            ExpansionTile,
            'Exact frozen requests and checks',
          );
          await reveal(tester, frozen);
          if (find
              .descendant(of: frozen, matching: find.byType(SelectableText))
              .evaluate()
              .isEmpty) {
            await tester.tap(
              find
                  .descendant(of: frozen, matching: find.byType(ListTile))
                  .first,
            );
            await tester.pumpAndSettle();
          }
          final exact = tester
              .widget<SelectableText>(
                find.descendant(
                  of: frozen,
                  matching: find.byType(SelectableText),
                ),
              )
              .data!;
          expect(jsonDecode(exact), {
            'draft': receipt['draft'],
            'baselineMessages': receipt['baselineMessages'],
            'candidateMessages': receipt['candidateMessages'],
          });
          expect(find.textContaining('Activate'), findsNothing);
          expect(tester.takeException(), isNull);
          if (renderDirectory.isNotEmpty && receipt['id'] == 3) {
            await tester.drag(
              find.byType(ListView).first,
              const Offset(0, 10000),
            );
            await tester.pumpAndSettle();
            await tester.runAsync(() async {
              final image =
                  await (boundary.currentContext!.findRenderObject()
                          as RenderRepaintBoundary)
                      .toImage();
              final bytes = await image.toByteData(
                format: ui.ImageByteFormat.png,
              );
              final file = File(
                '$renderDirectory/comparison-${dark ? 'dark' : 'light'}.png',
              );
              await file.parent.create(recursive: true);
              await file.writeAsBytes(bytes!.buffer.asUint8List());
              image.dispose();
            });
          }
        }
        expect(
          bridge.commands.every(
            (c) => [
              'comparisonSources',
              'comparisonsPage',
              'comparison',
            ].contains(c['command']),
          ),
          isTrue,
        );
      },
    );
  }

  testWidgets('Use as draft retains saved output and time limits', (
    tester,
  ) async {
    final bridge = SavedComparisonBridge();
    final chat = ChatController(bridge)
      ..session = 'public-comparison-audit'
      ..loading = false;
    addTearDown(chat.dispose);
    await tester.pumpWidget(MaterialApp(home: ComparisonInspector(chat: chat)));
    await tester.pumpAndSettle();
    final row = find.widgetWithText(ListTile, 'fixture · completed · #1');
    await reveal(tester, row);
    await tester.tap(row);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Use as draft'));
    await tester.pumpAndSettle();
    final tokens = find.byKey(const Key('comparison-tokens'));
    await reveal(tester, tokens);
    expect(tester.widget<TextField>(tokens).controller!.text, '512');
    final timeout = find.descendant(
      of: find.widgetWithText(FieldLabel, 'Timeout per response · up to 60 seconds'),
      matching: find.byType(TextField),
    );
    await reveal(tester, timeout);
    expect(tester.widget<TextField>(timeout).controller!.text, '5');
    expect(
      bridge.commands.any((c) => c['command'] == 'startComparison'),
      isFalse,
    );
    expect(tester.takeException(), isNull);
  });
}
