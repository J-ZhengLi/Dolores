import 'package:dolores_flutter/agent_timeline.dart';
import 'package:dolores_flutter/reply_content.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'frost_design_test.dart' show capture, viewport;
import 'support/render_fonts.dart';

Map<String, dynamic> receipt(int n, {bool failed = false}) => {
  'callId': 'call-$n',
  'name': n == 2 ? 'run_command' : 'computer',
  'target': n == 2 ? 'node --version' : 'Dolores capture $n',
  'status': failed ? 'error' : 'completed',
  'content': failed
      ? 'Window changed. Inspect a fresh capture before retrying.'
      : 'Receipt $n',
};

void main() {
  testWidgets(
    'silent model calls share one tool group until new public output',
    (t) async {
      Widget view({bool next = false}) => MaterialApp(
        home: Scaffold(
          body: SingleChildScrollView(
            child: AgentTimeline(
              steps: [
                {'number': 1, 'text': 'Inspecting the project.'},
                {'number': 2, 'text': ''},
                {'number': 3, 'text': '   '},
                if (next)
                  {'number': 4, 'text': 'Now validating the implementation.'},
              ],
              records: [
                receipt(1),
                receipt(2),
                receipt(3),
                if (next) receipt(4),
              ],
              toolSteps: [1, 2, 3, if (next) 4],
            ),
          ),
        ),
      );
      await t.pumpWidget(view());
      expect(find.byType(ToolExecutionGroup), findsOneWidget);
      expect(find.text('3 tool executions'), findsOneWidget);
      expect(find.byType(ReplyContent), findsOneWidget);
      expect(find.byType(ExpansionTile), findsOneWidget);
      await t.tap(find.byType(ExpansionTile));
      await t.pumpAndSettle();
      expect(find.byType(ExpansionTile), findsNWidgets(4));
      await t.pumpWidget(view(next: true));
      await t.pumpAndSettle();
      expect(find.byType(ToolExecutionGroup), findsNWidgets(2));
      expect(find.byType(ReplyContent), findsNWidgets(2));
      expect(find.byType(ExpansionTile), findsNWidgets(5));
      expect(
        t.getTopLeft(find.byKey(const ValueKey('timeline-tools-call-1'))).dy,
        lessThan(
          t.getTopLeft(find.text('Now validating the implementation.')).dy,
        ),
      );
      expect(t.takeException(), isNull);
    },
  );
  for (final dark in [false, true]) {
    for (final width in [420.0, 1000.0]) {
      testWidgets(
        'interleaves progress and receipts, dark=$dark width=$width',
        (t) async {
          viewport(t, Size(width, 900));
          await t.runAsync(loadReviewFonts);
          final key = GlobalKey();
          await t.pumpWidget(
            MaterialApp(
              theme: doloresTheme(dark),
              home: Scaffold(
                body: RepaintBoundary(
                  key: key,
                  child: ColoredBox(
                    color: Palette(dark).bg,
                    child: SingleChildScrollView(
                      child: Padding(
                        padding: const EdgeInsets.all(24),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            AgentTimeline(
                              steps: const [
                                {
                                  'number': 1,
                                  'text': 'I’ll inspect the window and find Settings.',
                                },
                                {
                                  'number': 2,
                                  'text': 'Settings is open. I’ll check the available controls.',
                                },
                                {
                                  'number': 3,
                                  'text': 'The window changed. I’ll inspect it again before continuing.',
                                },
                                {
                                  'number': 4,
                                  'text': 'Light mode is active. I’ll inspect the composer dock.',
                                },
                              ],
                              records: [
                                receipt(1),
                                receipt(2),
                                receipt(3, failed: true),
                                receipt(4),
                                receipt(5),
                              ],
                              toolSteps: const [1, 2, 2, 3, 4],
                            ),
                            const ReplyContent(
                              text: 'The dock remains readable in Light mode.',
                            ),
                          ],
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            ),
          );
          await t.pumpAndSettle();
          final prose = find.byType(ReplyContent);
          final rows = find.byType(ExpansionTile);
          expect(prose, findsNWidgets(5));
          expect(rows, findsNWidgets(4));
          expect(
            t.getTopLeft(prose.at(0)).dy,
            lessThan(t.getTopLeft(rows.at(0)).dy),
          );
          expect(
            t.getTopLeft(rows.at(0)).dy,
            lessThan(t.getTopLeft(prose.at(1)).dy),
          );
          expect(
            t.getTopLeft(rows.at(1)).dy,
            lessThan(t.getTopLeft(prose.at(2)).dy),
          );
          expect(
            t.getTopLeft(rows.at(3)).dy,
            lessThan(t.getTopLeft(prose.at(4)).dy),
          );
          expect(t.getSize(rows.first).height, lessThanOrEqualTo(48));
          final leading = find.descendant(
            of: rows.first,
            matching: find.byIcon(Icons.terminal_outlined),
          );
          expect(
            t.getRect(leading).left - t.getRect(rows.first).left,
            greaterThanOrEqualTo(12),
          );
          expect(
            t.getRect(rows.at(2)).top - t.getRect(rows.at(1)).bottom,
            greaterThanOrEqualTo(8),
          );
          await capture(
            t,
            key,
            'timeline-${dark ? 'dark' : 'light'}-${width.toInt()}',
          );
          await t.ensureVisible(rows.at(1));
          await t.tap(rows.at(1));
          await t.pumpAndSettle();
          final failed = find.byKey(
            const PageStorageKey(
              'tool-record-call-3-computer-Dolores capture 3',
            ),
          );
          await t.ensureVisible(failed);
          await t.tap(failed);
          await t.pumpAndSettle();
          expect(
            find.text(
              'Window changed. Inspect a fresh capture before retrying.',
            ),
            findsOneWidget,
          );
          expect(t.takeException(), isNull);
        },
      );
    }
  }
  testWidgets(
    'legacy and malformed ordering retain every receipt without guessing',
    (t) async {
      for (final ordering in [
        <dynamic>[],
        <dynamic>[2],
        <dynamic>[2, 'bad'],
        <dynamic>[2, 1],
      ]) {
        await t.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: AgentTimeline(
                steps: const [
                  {'number': 1, 'text': 'Earlier public progress'},
                ],
                records: [receipt(1), receipt(2)],
                toolSteps: ordering,
                unsaved: true,
              ),
            ),
          ),
        );
        expect(find.text('Earlier activity'), findsOneWidget);
        expect(find.text('Progress · Not saved'), findsOneWidget);
        expect(find.byType(ToolExecutionGroup), findsOneWidget);
        await t.tap(find.byType(ExpansionTile));
        await t.pumpAndSettle();
        expect(find.byType(ExpansionTile), findsNWidgets(3));
        await t.pumpWidget(const SizedBox());
        expect(t.takeException(), isNull);
      }
    },
  );
}
