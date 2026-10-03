import 'dart:async';
import 'dart:io';

import 'package:flutter/services.dart';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/task_feedback.dart';
import 'package:dolores_flutter/theme.dart';

import 'history_test.dart';

class FeedbackBridge extends HistoryBridge {
  bool failWrites = false;
  Completer<Map<String, dynamic>>? pending;
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    commands.add(command);
    if (failWrites) {
      throw 'Feedback changed elsewhere. Close and reopen to review it.';
    }
    return pending?.future ??
        {
          'revision': 1,
          'updatedAt': 42,
          'outcome': command['draft']['outcome'],
          'note': command['draft']['note'],
        };
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async {
    final fontPath = Platform.environment['DOLORES_TEST_FONT'];
    if (fontPath != null) {
      for (final family in ['Segoe UI']) {
        final loader = FontLoader(family);
        loader.addFont(
          File(fontPath)
              .readAsBytes()
              .then((bytes) => ByteData.sublistView(bytes)),
        );
        await loader.load();
      }
    }
  });
  for (final dark in [false, true]) {
    testWidgets(
      'feedback failure keeps note, retry binds reply, compact ${dark ? 'dark' : 'light'}',
      (tester) async {
        tester.view.physicalSize = const Size(620, 700);
        tester.view.devicePixelRatio = 1;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);
        final bridge = FeedbackBridge()..failWrites = true;
        final chat = ChatController(bridge)
          ..session = 'task'
          ..loading = false;
        addTearDown(chat.dispose);
        await tester.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark),
            home: RepaintBoundary(
              key: const Key('capture'),
              child: TaskFeedbackDialog(
                chat: chat,
                message: {
                  'id': 2,
                  'content': 'claim',
                  'metadata': {
                    'model': 'original',
                    'paused': {'reason': 'outputLimit'},
                  },
                },
              ),
            ),
          ),
        );
        await tester.tap(find.text('Needs work'));
        await tester.enterText(
          find.byKey(const Key('feedback-note')),
          'retained correction',
        );
        await tester.tap(find.text('Save feedback'));
        await tester.pumpAndSettle();
        expect(
          find.textContaining('Feedback changed elsewhere'),
          findsOneWidget,
        );
        expect(find.text('retained correction'), findsOneWidget);
        expect(find.text('Save feedback'), findsOneWidget);
        expect(tester.takeException(), isNull);
        final capture = Platform.environment['DOLORES_CAPTURE_DIR'];
        if (capture != null) {
          await tester.pump();
          await expectLater(
            find.byKey(const Key('capture')),
            matchesGoldenFile(
              Uri.file('$capture/feedback-${dark ? 'dark' : 'light'}.png'),
            ),
          );
        }
        expect(bridge.commands.single['draft']['expectedContent'], 'claim');
        expect(
          bridge.commands.single['draft']['expectedMetadata']['model'],
          'original',
        );
        expect(bridge.commands.single['draft']['outcome'], 'needsWork');
        bridge.failWrites = false;
        await tester.tap(find.text('Save feedback'));
        await tester.pumpAndSettle();
        expect(bridge.commands.length, 2);
      },
    );
  }
  testWidgets(
    'pending feedback blocks duplicates and close; clear uses saved revision',
    (tester) async {
      final bridge = FeedbackBridge()..pending = Completer();
      final chat = ChatController(bridge)
        ..session = 'task'
        ..loading = false;
      addTearDown(chat.dispose);
      await tester.pumpWidget(
        MaterialApp(
          home: TaskFeedbackDialog(
            chat: chat,
            message: {
              'id': 2,
              'content': 'claim',
              'feedback': {'revision': 4, 'outcome': 'worked', 'note': 'old'},
            },
          ),
        ),
      );
      await tester.tap(find.text('Clear feedback'));
      await tester.pump();
      expect(bridge.commands.single['draft']['revision'], 4);
      expect(bridge.commands.single['draft']['outcome'], isNull);
      expect(bridge.commands.single['draft']['note'], '');
      expect(
        tester
            .widget<TextButton>(find.widgetWithText(TextButton, 'Close'))
            .onPressed,
        isNull,
      );
      expect(
        tester
            .widget<FilledButton>(find.widgetWithText(FilledButton, 'Saving…'))
            .onPressed,
        isNull,
      );
      bridge.pending!.complete({'revision': 5, 'outcome': null, 'note': ''});
      await tester.pumpAndSettle();
    },
  );
}
