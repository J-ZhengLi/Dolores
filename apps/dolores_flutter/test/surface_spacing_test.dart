import 'dart:convert';

import 'package:dolores_flutter/theme.dart';
import 'package:dolores_flutter/tool_activity.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'frost_design_test.dart' show capture, viewport;
import 'support/render_fonts.dart';

void main() {
  for (final dark in [false, true]) {
    for (final width in [420.0, 1000.0]) {
      testWidgets('tool and panel insets dark=$dark width=$width', (t) async {
        viewport(t, Size(width, 1000));
        await t.runAsync(loadReviewFonts);
        final key = GlobalKey();
        final records = List.generate(
          4,
          (i) => {
            'callId': 'cli-$i',
            'name': 'run_command',
            'target': i == 3
                ? 'project-local-tool-with-a-long-name.exe'
                : 'gitee',
            'status': 'completed',
            'command': {
              'program': 'gitee',
              'args': ['--version'],
            },
            'content': jsonEncode({
              'exitCode': i == 2 ? 4 : 0,
              'stdout': 'Retained command output',
              'stderr': '',
              'reason': 'completed',
              'truncated': false,
            }),
          },
        );
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
                          const Text(
                            'I’ll check the CLI and inspect the result.',
                          ),
                          const SizedBox(height: 8),
                          ToolRecords(records: records, compact: true),
                          const SizedBox(height: 24),
                          const Text('Saved evidence'),
                          ToolRecords(records: [records.first]),
                          const SizedBox(height: 16),
                          const Card(
                            child: ListTile(
                              leading: Icon(Icons.terminal_outlined),
                              title: Text('Command sessions'),
                              trailing: Icon(Icons.chevron_right),
                            ),
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
        final rows = find.byType(ExpansionTile);
        expect(rows, findsNWidgets(5));
        for (var i = 0; i < 4; i++) {
          final row = rows.at(i);
          final rect = t.getRect(row);
          final icons = find.descendant(of: row, matching: find.byType(Icon));
          expect(
            t.getRect(icons.first).left - rect.left,
            greaterThanOrEqualTo(12),
          );
          expect(
            rect.right - t.getRect(icons.last).right,
            greaterThanOrEqualTo(12),
          );
          expect(rect.height, lessThanOrEqualTo(48));
          if (i > 0) {
            expect(
              rect.top - t.getRect(rows.at(i - 1)).bottom,
              greaterThanOrEqualTo(8),
            );
          }
        }
        final commandTitle = find.descendant(
          of: rows.first,
          matching: find.text('Command · gitee'),
        );
        final status = find.descendant(
          of: rows.first,
          matching: find.text('· Exited 0'),
        );
        expect(
          t.getRect(status).left - t.getRect(commandTitle).right,
          lessThanOrEqualTo(8),
        );
        final savedRow = rows.last;
        final savedIcon = find.descendant(
          of: savedRow,
          matching: find.byIcon(Icons.description_outlined),
        );
        expect(
          t.getRect(savedIcon).left - t.getRect(savedRow).left,
          greaterThanOrEqualTo(16),
        );
        final list = find.byType(ListTile).last;
        final listIcon = find.descendant(
          of: list,
          matching: find.byIcon(Icons.terminal_outlined),
        );
        expect(
          t.getRect(listIcon).left - t.getRect(list).left,
          greaterThanOrEqualTo(16),
        );
        expect(t.takeException(), isNull);
        final name =
            'surface-spacing-${dark ? 'dark' : 'light'}-${width.toInt()}';
        await capture(t, key, name);
        await t.tap(rows.first);
        await t.pumpAndSettle();
        final output = find.textContaining('Retained command output');
        expect(output, findsOneWidget);
        expect(
          t.getRect(output).left - t.getRect(rows.first).left,
          closeTo(12, .1),
        );
        await capture(t, key, '$name-expanded');
        await t.pumpWidget(
          MaterialApp(
            builder: (context, child) => MediaQuery(
              data: MediaQuery.of(context)
                  .copyWith(textScaler: const TextScaler.linear(1.5)),
              child: child!,
            ),
            theme: doloresTheme(dark),
            home: Scaffold(body: ToolRecords(records: records, compact: true)),
          ),
        );
        await t.pumpAndSettle();
        expect(t.takeException(), isNull);
      });
    }
  }
}
