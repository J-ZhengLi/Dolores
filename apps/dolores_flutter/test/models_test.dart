import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/models.dart';

import 'widget_test.dart' show ConnectionBridge;

class DetailsBridge extends ConnectionBridge {
  Map<String, dynamic> details = {
    'contextWindowTokens': null,
    'imageInput': false,
    'requestSettings': null,
  };
  bool stale = false, failRefresh = false;
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    if (command['command'] == 'modelDetails') return Map.of(details);
    if (command['command'] == 'setModelDetails') {
      commands.add(command);
      if (stale) {
        stale = false;
        details = {...details, 'contextWindowTokens': 32768};
      }
      if (jsonEncode(command['expected']) != jsonEncode(details)) {
        throw StateError(
          'Model settings changed. Refresh, keep your edits and review before saving again.',
        );
      }
      details = (command['details'] as Map).cast<String, dynamic>();
      return details;
    }
    if (command['command'] == 'bootstrap' && failRefresh) {
      throw StateError('Refresh unavailable');
    }
    return super.call(command);
  }
}

void main() {
  testWidgets(
    'Model output supports provider default, rejects invalid input and keeps larger saved limits',
    (tester) async {
      final bridge = DetailsBridge();
      final chat = ChatController(bridge)
        ..loading = false
        ..draft = 'Keep my draft';
      await tester.pumpWidget(
        MaterialApp(
          home: ModelDetailsEditor(chat: chat, model: 'fixture'),
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('model-detail-advanced')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Customize responses'));
      await tester.pumpAndSettle();
      for (final value in ['0', 'invalid', '', '65536']) {
        await tester.ensureVisible(
          find.byKey(const Key('model-detail-output')),
        );
        await tester.enterText(
          find.byKey(const Key('model-detail-output')),
          value,
        );
        await tester.tap(find.byKey(const Key('save-model-details')));
        await tester.pumpAndSettle();
        if (value == '0' || value == 'invalid') {
          await tester.drag(find.byType(ListView), const Offset(0, 800));
          await tester.pumpAndSettle();
          expect(bridge.details['requestSettings'], isNull);
          expect(find.textContaining('Use whole numbers'), findsOneWidget);
        } else {
          expect(
            bridge.details['requestSettings']['maxOutputTokens'],
            value.isEmpty ? null : 65536,
          );
          await tester.tap(find.text('Refresh · keep edits'));
          await tester.pumpAndSettle();
          expect(
            tester
                .widget<TextField>(find.byKey(const Key('model-detail-output')))
                .controller!
                .text,
            value,
          );
        }
        expect(chat.draft, 'Keep my draft');
        expect(tester.takeException(), isNull);
      }
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  for (final dark in [false, true]) {
    testWidgets(
      'Model details fit 420x480, default to 128K and acknowledge the saved values ($dark)',
      (tester) async {
        tester.view.physicalSize = const Size(420, 480);
        tester.view.devicePixelRatio = 1;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);
        final bridge = DetailsBridge()..failRefresh = true;
        final chat = ChatController(bridge)
          ..loading = false
          ..baseUrl = 'http://localhost:1234/v1'
          ..model = 'fixture'
          ..draft = 'Unrelated draft';
        await tester.pumpWidget(
          MaterialApp(
            theme: ThemeData(
              brightness: dark ? Brightness.dark : Brightness.light,
            ),
            home: ModelDetailsEditor(chat: chat, model: 'fixture'),
          ),
        );
        await tester.pumpAndSettle();
        expect(find.text('Blank uses 128K'), findsOneWidget);
        await tester.enterText(
          find.byKey(const Key('model-detail-context')),
          '65536',
        );
        await tester.ensureVisible(
          find.byKey(const Key('model-detail-images')),
        );
        await tester.pumpAndSettle();
        await tester.tap(find.byKey(const Key('model-detail-images')));
        await tester.pumpAndSettle();
        await tester.tap(find.byKey(const Key('save-model-details')));
        await tester.pumpAndSettle();
        expect(bridge.details['contextWindowTokens'], 65536);
        expect(bridge.details['imageInput'], true);
        expect(find.textContaining('Saved. Reopen'), findsOneWidget);
        expect(chat.draft, 'Unrelated draft');
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
        chat.dispose();
      },
    );
  }
  testWidgets(
    'Conflicting model save keeps edits through refresh and Close offers a decision',
    (tester) async {
      final bridge = DetailsBridge()..stale = true;
      final chat = ChatController(bridge)..loading = false;
      await tester.pumpWidget(
        MaterialApp(
          home: ModelDetailsEditor(chat: chat, model: 'fixture'),
        ),
      );
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byKey(const Key('model-detail-context')),
        '8192',
      );
      await tester.tap(find.byKey(const Key('save-model-details')));
      await tester.pumpAndSettle();
      expect(find.textContaining('Model settings changed'), findsOneWidget);
      await tester.tap(find.text('Refresh · keep edits'));
      await tester.pumpAndSettle();
      expect(
        tester
            .widget<TextField>(find.byKey(const Key('model-detail-context')))
            .controller!
            .text,
        '8192',
      );
      await tester.tap(find.byTooltip('Close model details'));
      await tester.pumpAndSettle();
      expect(find.text('Unsaved changes'), findsOneWidget);
      await tester.tap(find.text('Keep editing'));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('save-model-details')));
      await tester.pumpAndSettle();
      expect(bridge.details['contextWindowTokens'], 8192);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
}
