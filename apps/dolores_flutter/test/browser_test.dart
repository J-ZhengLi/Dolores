import 'support/render_fonts.dart';

import 'package:dolores_flutter/theme.dart';

import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/rendering.dart';

import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/browser_settings.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/tool_activity.dart';

class BrowserBridge implements ChatBridge {
  final commands = <Map<String, dynamic>>[];
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  int captures = 0;
  bool fail = true;
  bool ready = false;
  String phase = '';
  Map prerequisites = {'nodeReady': true, 'browserReady': true};
  @override
  Future<dynamic> call(Map<String, dynamic> value) async {
    commands.add(Map.of(value));
    if (value['command'] == 'browserSettings') {
      return {
        'available': ready,
        'prerequisites': prerequisites,
        'setupState': {'phase': phase},
        'reason': 'Install the optional adapter.',
        'adapter': 'Pinned runtime',
        'profile': 'Fresh profile',
        'bounds': 'Bounded operations',
        'setup': 'Browser setup in the user guide',
      };
    }
    if (value['command'] == 'browserSetup') {
      phase = value['action'] == 'start'
          ? 'installing'
          : value['action'] == 'cancel'
          ? 'stopped'
          : phase;
      return {
        'phase': phase,
        'note': phase == 'stopped' ? 'Setup cancelled. Try again.' : '',
      };
    }
    if (value['command'] == 'browserPrerequisite') return null;
    if (value['command'] == 'browserCapture' ||
        value['command'] == 'desktopPreview') {
      captures++;
      if (fail) {
        throw 'Local capture is missing.';
      }
      return {'data': base64Encode(const [])};
    }
    return null;
  }
}

void main() {
  setUpAll(loadReviewFonts);
  for (final name in ['browser', 'inspect_desktop_capture']) {
    testWidgets(
      'missing $name capture stays readable inside its saved receipt',
      (tester) async {
        final bridge = BrowserBridge();
        const capture = '00000000-0000-0000-0000-000000000001';
        final content = name == 'browser'
            ? {
                'capture': capture,
                'outcome': 'completed',
                'text': 'Retained public page state.',
                'note': 'Public browser receipt.',
              }
            : {
                'capture': {
                  'id': capture,
                  'session': 'public-side-chat',
                  'observation': {
                    'target': {'title': 'Public fixture window'},
                    'width': 1,
                    'height': 1,
                    'dpi': 96,
                  },
                },
                'freshness': 'Historical screenshot.',
              };
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: SingleChildScrollView(
                child: ToolRecords(
                  bridge: bridge,
                  records: [
                    {
                      'callId': 'public-capture',
                      'name': name,
                      'target': 'Public capture receipt',
                      'status': 'completed',
                      'content': jsonEncode(content),
                    },
                  ],
                ),
              ),
            ),
          ),
        );
        await tester.tap(find.text('Public capture receipt'));
        await tester.pumpAndSettle();
        expect(bridge.captures, 0);
        await tester.tap(find.text('View local screenshot'));
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
        expect(
          find.textContaining('Local capture is missing.'),
          findsOneWidget,
        );
        expect(
          find.textContaining('does not repeat the action'),
          findsOneWidget,
        );
        await tester.tap(find.text('View local screenshot'));
        await tester.pumpAndSettle();
        expect(bridge.captures, 2);
        expect(
          bridge.commands.map((command) => command['command']).toList(),
          List.filled(
            2,
            name == 'browser' ? 'browserCapture' : 'desktopPreview',
          ),
        );
        expect(
          bridge.commands.every((command) => command['capture'] == capture),
          isTrue,
        );
        expect(
          find.textContaining('Local capture is missing.'),
          findsOneWidget,
        );
        expect(tester.takeException(), isNull);
      },
    );
  }

  testWidgets(
    'missing adapter has setup/refresh and scrolls in compact themes',
    (tester) async {
      tester.view.physicalSize = const Size(420, 480);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final chat = ChatController(BrowserBridge());
      final renderKey = GlobalKey();
      addTearDown(chat.dispose);
      for (final brightness in Brightness.values) {
        await tester.pumpWidget(
          MaterialApp(
            theme: doloresTheme(brightness == Brightness.dark),
            home: RepaintBoundary(
              key: renderKey,
              child: BrowserSettingsInspector(chat: chat),
            ),
          ),
        );
        await tester.pumpAndSettle();
        expect(find.text('Set up Browser'), findsOneWidget);
        expect(find.text('Set up browser'), findsOneWidget);
        expect(find.text('Install the optional adapter.'), findsNothing);
        await tester.tap(find.text('Check again'));
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
        final output = Platform.environment['DOLORES_RENDER_DIR'];
        if (output != null) {
          await tester.runAsync(() async {
            final image =
                await (renderKey.currentContext!.findRenderObject()
                        as RenderRepaintBoundary)
                    .toImage();
            final data = await image.toByteData(format: ui.ImageByteFormat.png);
            image.dispose();
            final file = File('$output/browser-${brightness.name}.png');
            await file.parent.create(recursive: true);
            await file.writeAsBytes(data!.buffer.asUint8List());
          });
        }
      }
    },
  );
  testWidgets(
    'explicit setup can be cancelled and retried; missing prerequisites have actions',
    (t) async {
      final bridge = BrowserBridge();
      final chat = ChatController(bridge);
      await t.pumpWidget(
        MaterialApp(home: BrowserSettingsInspector(chat: chat)),
      );
      await t.pumpAndSettle();
      await t.tap(find.text('Set up browser'));
      await t.pump();
      expect(find.text('Setting up browser…'), findsOneWidget);
      await t.tap(find.text('Cancel setup'));
      await t.pumpAndSettle();
      expect(find.text('Setup cancelled. Try again.'), findsOneWidget);
      expect(find.text('Set up browser'), findsOneWidget);
      bridge.prerequisites = {'nodeReady': false, 'browserReady': false};
      await t.tap(find.text('Check again'));
      await t.pumpAndSettle();
      expect(find.text('Install Node.js'), findsOneWidget);
      expect(find.text('Install Microsoft Edge'), findsOneWidget);
      await t.tap(find.text('Download').first);
      await t.pump();
      expect(bridge.commands.last['command'], 'browserPrerequisite');
      expect(bridge.commands.last['kind'], 'node');
      await t.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'screenshot failure preserves receipt and retries viewing only on demand',
    (tester) async {
      final bridge = BrowserBridge();
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: BrowserCapturePreview(
              bridge: bridge,
              capture: '00000000-0000-0000-0000-000000000001',
            ),
          ),
        ),
      );
      expect(bridge.captures, 0);
      await tester.tap(find.text('View local screenshot'));
      await tester.pumpAndSettle();
      expect(find.textContaining('Local capture is missing.'), findsOneWidget);
      expect(find.text('View local screenshot'), findsOneWidget);
      await tester.tap(find.text('View local screenshot'));
      await tester.pumpAndSettle();
      expect(bridge.captures, 2);
      expect(tester.takeException(), isNull);
    },
  );
  test('stale/uncertain evidence and unsafe screenshot IDs stay honest', () {
    expect(
      toolStatus({'name': 'browser', 'content': '{"outcome":"stale"}'}),
      contains('no action ran'),
    );
    expect(
      toolStatus({'name': 'browser', 'content': '{"outcome":"uncertain"}'}),
      contains('uncertain'),
    );
    expect(browserCaptureId({'content': '{"capture":"../../secret"}'}), isNull);
    expect(
      toolResultText({
        'name': 'browser',
        'status': 'completed',
        'content': '{"outcome":"stale","recovery":"Inspect fresh state","text":"Retained evidence"}',
      }),
      contains('Retained evidence'),
    );
  });
}
