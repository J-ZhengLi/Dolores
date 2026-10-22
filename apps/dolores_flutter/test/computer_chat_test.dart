import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:dolores_flutter/tool_activity.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/render_fonts.dart';

class ComputerBridge implements ChatBridge {
  final calls = <Map<String, dynamic>>[];
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> value) async {
    calls.add(value);
    if (value['command'] == 'approveTool') return null;
    throw StateError('Unexpected ${value['command']}');
  }
}

void main() {
  test('computer receipts identify windows and actual screenshot evidence', () {
    expect(toolLabel('computer'), 'Computer use');
    expect(toolQueryText({'name': 'computer', 'query': '{}'}), 'Operation: {}');
    expect(
      toolResultText({
        'name': 'computer',
        'status': 'completed',
        'content': jsonEncode({'windows': []}),
      }),
      '0 visible windows · No screenshot yet',
    );
    expect(
      toolResultText({
        'name': 'computer',
        'status': 'error',
        'content': 'Window closed. List again.',
      }),
      'Window closed. List again.',
    );
    final screenshot = {
      'name': 'computer',
      'status': 'completed',
      'content': jsonEncode({
        'capture': {
          'id': '11111111-1111-4111-8111-111111111111',
          'session': 'fixture',
          'observation': {
            'target': {'title': 'Local form'},
            'width': 640,
            'height': 480,
          },
        },
      }),
    };
    expect(desktopCaptureReference(screenshot)?['session'], 'fixture');
    expect(
      toolResultText(screenshot),
      contains('Fresh screenshot · Local form'),
    );
  });

  for (final brightness in Brightness.values) {
    testWidgets(
      'computer input review retains exact action and draft in $brightness',
      (tester) async {
        tester.view.physicalSize = const Size(420, 480);
        tester.view.devicePixelRatio = 1;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);
        final renderDirectory = Platform.environment['DOLORES_RENDER_DIR'];
        if (renderDirectory != null) await tester.runAsync(loadReviewFonts);
        final bridge = ComputerBridge();
        final chat = ChatController(bridge)
          ..busy = true
          ..draft = 'Keep my unrelated draft'
          ..toolApproval = {
            'callId': 'fixture-input',
            'name': 'computer',
            'target': 'Local form',
            'query': jsonEncode({
              'operation': 'type',
              'capture': 'public-capture',
              'text': 'Verified ordinary chat',
            }),
          };
        addTearDown(chat.dispose);
        final boundary = GlobalKey();
        await tester.pumpWidget(
          MaterialApp(
            theme: doloresTheme(brightness == Brightness.dark),
            home: RepaintBoundary(
              key: boundary,
              child: Scaffold(
                body: Padding(
                  padding: const EdgeInsets.all(12),
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.end,
                    children: [ToolApprovalCard(chat: chat)],
                  ),
                ),
              ),
            ),
          ),
        );
        await tester.pumpAndSettle();
        expect(find.text('Allow this desktop input?'), findsOneWidget);
        expect(find.text('Allow a file read?'), findsNothing);
        expect(find.byKey(const Key('allow-tool')), findsOneWidget);
        expect(find.byKey(const Key('deny-tool')), findsOneWidget);
        expect(tester.takeException(), isNull);
        if (renderDirectory != null) {
          await tester.runAsync(() async {
            final image =
                await (boundary.currentContext!.findRenderObject()!
                        as RenderRepaintBoundary)
                    .toImage();
            final bytes = await image.toByteData(
              format: ui.ImageByteFormat.png,
            );
            await Directory(renderDirectory).create(recursive: true);
            await File(
              '$renderDirectory/computer-review-${brightness.name}.png',
            ).writeAsBytes(bytes!.buffer.asUint8List());
            image.dispose();
          });
        }
        await tester.tap(find.byKey(const Key('deny-tool')));
        await tester.pumpAndSettle();
        expect(bridge.calls.single['allow'], false);
        expect(bridge.calls.single['callId'], 'fixture-input');
        expect(chat.draft, 'Keep my unrelated draft');
      },
    );
  }
}
