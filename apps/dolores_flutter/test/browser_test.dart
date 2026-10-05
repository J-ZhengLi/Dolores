import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/browser_settings.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/tool_activity.dart';

class BrowserBridge implements ChatBridge {
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  int captures = 0;
  bool fail = true;
  @override
  Future<dynamic> call(Map<String, dynamic> value) async {
    if (value['command'] == 'browserSettings') {
      return {
        'available': false,
        'reason': 'Install the optional adapter.',
        'adapter': 'Pinned runtime',
        'profile': 'Fresh profile',
        'bounds': 'Bounded operations',
        'setup': 'Browser setup in the user guide',
      };
    }
    if (value['command'] == 'browserCapture') {
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
  testWidgets(
    'missing adapter has setup/refresh and scrolls in compact themes',
    (tester) async {
      tester.view.physicalSize = const Size(420, 480);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final chat = ChatController(BrowserBridge());
      addTearDown(chat.dispose);
      for (final brightness in Brightness.values) {
        await tester.pumpWidget(
          MaterialApp(
            theme: ThemeData(brightness: brightness),
            home: BrowserSettingsInspector(chat: chat),
          ),
        );
        await tester.pumpAndSettle();
        expect(find.text('Needs setup'), findsOneWidget);
        await tester.tap(find.text('Setup'));
        await tester.pumpAndSettle();
        expect(find.text('Install the optional adapter.'), findsOneWidget);
        await tester.tap(find.text('Refresh'));
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
      }
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
