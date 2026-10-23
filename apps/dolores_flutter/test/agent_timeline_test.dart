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
          expect(rows, findsNWidgets(5));
          expect(
            t.getTopLeft(prose.at(0)).dy,
            lessThan(t.getTopLeft(rows.at(0)).dy),
          );
          expect(
            t.getTopLeft(rows.at(0)).dy,
            lessThan(t.getTopLeft(prose.at(1)).dy),
          );
          expect(
            t.getTopLeft(rows.at(2)).dy,
            lessThan(t.getTopLeft(prose.at(2)).dy),
          );
          expect(
            t.getTopLeft(rows.at(4)).dy,
            lessThan(t.getTopLeft(prose.at(4)).dy),
          );
          expect(t.getSize(rows.first).height, lessThanOrEqualTo(48));
          await capture(
            t,
            key,
            'timeline-${dark ? 'dark' : 'light'}-${width.toInt()}',
          );
          await t.ensureVisible(rows.at(2));
          await t.tap(rows.at(2));
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
        expect(find.byType(ExpansionTile), findsNWidgets(2));
        expect(t.takeException(), isNull);
      }
    },
  );
}
