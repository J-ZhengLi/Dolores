import 'package:dolores_flutter/mcp_import.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test(
    'Import preserves literal arguments and credential bindings without launch',
    () {
      final data = parseMcpImport(
        r'{"mcpServers":{"Local":{"command":"C:/node.exe","args":["a b.mjs","$HOME"],"env":{"API_KEY":"fixture"}}}}',
      );
      expect(data.single['args'], ['a b.mjs', r'$HOME']);
      expect(data.single['env'], {'API_KEY': 'fixture'});
    },
  );
  test(
    'Malformed and remote imports cannot substitute launch configuration',
    () {
      for (final text in [
        '{',
        '{"mcpServers":{"x":{"url":"https://example.org"}}}',
        '{"mcpServers":{"x":{"command":"node","args":[2]}}}',
        '{"mcpServers":{"x":{"command":"node","env":{"BAD-NAME":"key"}}}}',
        '{"mcpServers":{"x":{"command":"node","unknown":true}}}',
        'x' * 65537,
      ]) {
        expect(() => parseMcpImport(text), throwsFormatException);
      }
    },
  );
  testWidgets(
    'Import error retains editable text and supports cancel without accepting data',
    (t) async {
      Map<String, dynamic>? imported;
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) => TextButton(
                onPressed: () async {
                  imported = await showMcpImport(context);
                },
                child: const Text('Open'),
              ),
            ),
          ),
        ),
      );
      await t.tap(find.text('Open'));
      await t.pumpAndSettle();
      await t.enterText(find.byKey(const Key('mcp-import-json')), '{broken');
      await t.tap(find.byKey(const Key('mcp-import-review')));
      await t.pumpAndSettle();
      expect(find.textContaining('Nothing was imported'), findsOneWidget);
      expect(
        t
            .widget<TextField>(find.byKey(const Key('mcp-import-json')))
            .controller!
            .text,
        '{broken',
      );
      await t.tap(find.text('Cancel'));
      await t.pumpAndSettle();
      expect(imported, isNull);
    },
  );
}
