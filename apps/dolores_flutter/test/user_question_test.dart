import 'dart:async';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/user_question.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'tools_test.dart' show ToolBridge, ready, compact;

class QuestionBridge extends ToolBridge {
  bool fail = false;
  Completer<void>? answerPending;
  final answers = <Map<String, dynamic>>[];
  static final request = {
    'callId': 'question-one',
    'questions': [
      {
        'id': 'date',
        'question': 'Which report should I prepare?',
        'options': [
          {'label': 'Today', 'description': 'Write a new daily report'},
          {
            'label': 'Previous entry',
            'description': 'Check and complete the saved report',
          },
        ],
      },
      {
        'id': 'language',
        'question': 'Which language?',
        'options': [
          {'label': 'English'},
          {'label': '中文'},
        ],
      },
    ],
  };
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    if (command['command'] == 'start') {
      commands.add(command);
      input = command['input'];
      queue.addAll([
        {'type': 'started', 'id': command['id'], 'session': 'run'},
        {'type': 'userQuestion', 'id': command['id'], 'request': request},
      ]);
      return null;
    }
    if (command['command'] == 'answerQuestion') {
      answers.add(command);
      if (answerPending != null) await answerPending!.future;
      if (fail) throw StateError('Connection interrupted');
      messages = [
        {'id': 1, 'role': 'user', 'content': input},
        {'id': 2, 'role': 'assistant', 'content': 'Report prepared.'},
      ];
      queue.add({
        'type': 'done',
        'id': command['id'],
        'answer': 'Report prepared.',
      });
      return null;
    }
    return super.call(command);
  }
}

Future<void> start(WidgetTester tester, ChatController chat) async {
  await tester.pumpWidget(DoloresApp(chat: chat));
  await chat.send();
  await tester.pump(const Duration(milliseconds: 100));
  await tester.pumpAndSettle();
}

void main() {
  setUpAll(() async {
    const directory = String.fromEnvironment('QUESTION_RENDER_DIR');
    if (directory.isEmpty) return;
    final fonts = '${Platform.environment['WINDIR'] ?? 'C:/Windows'}/Fonts';
    for (final family in ['Ahem', 'Segoe UI', 'Roboto']) {
      final loader = FontLoader(family);
      loader.addFont(
        File('$fonts/segoeui.ttf').readAsBytes().then(ByteData.sublistView),
      );
      await loader.load();
    }
    await ui.loadFontFromList(
      await File(
        '../../output/toolchains/flutter/bin/cache/artifacts/material_fonts/MaterialIcons-Regular.otf',
      ).readAsBytes(),
      fontFamily: 'MaterialIcons',
    );
  });
  testWidgets(
    'Real chat chooser waits for explicit answers, custom input and back retain choices, then continues',
    (tester) async {
      compact(tester);
      final bridge = QuestionBridge();
      final chat = ready(bridge);
      await start(tester, chat);
      expect(find.byType(UserQuestionCard), findsOneWidget);
      expect(chat.modelActivityLabel, 'Waiting for your answer');
      expect(
        tester
            .widget<FilledButton>(
              find.byKey(const Key('submit-question-answer')),
            )
            .onPressed,
        isNull,
      );
      expect(bridge.answers, isEmpty);
      await tester.tap(find.byKey(const Key('question-option-1')));
      await tester.pump();
      await tester.tap(find.text('Next'));
      await tester.pump();
      await tester.enterText(
        find.byKey(const Key('question-custom-answer')),
        '简体中文，保留引用',
      );
      await tester.pump();
      await tester.tap(find.text('Back'));
      await tester.pump();
      expect(chat.questionAnswers.first, 'Previous entry');
      await tester.tap(find.text('Next'));
      await tester.pump();
      expect(
        tester
            .widget<TextField>(find.byKey(const Key('question-custom-answer')))
            .controller!
            .text,
        '简体中文，保留引用',
      );
      await tester.pump();
      await tester.tap(find.text('Send answer'));
      await tester.pump(const Duration(milliseconds: 250));
      await tester.pumpAndSettle();
      expect(bridge.answers.single['answers'], ['Previous entry', '简体中文，保留引用']);
      expect(
        bridge.commands.where((c) => c['command'] == 'approveTool'),
        isEmpty,
      );
      expect(chat.busy, isFalse);
      expect(find.byType(UserQuestionCard), findsNothing);
      expect(find.text('Report prepared.'), findsOneWidget);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'Failed submission keeps drafts, pending submission cannot duplicate, Stop restores original task',
    (tester) async {
      final bridge = QuestionBridge()..fail = true;
      final chat = ready(bridge);
      await start(tester, chat);
      await tester.tap(find.byKey(const Key('question-option-0')));
      await tester.pump();
      await tester.tap(find.text('Next'));
      await tester.pump();
      await tester.enterText(
        find.byKey(const Key('question-custom-answer')),
        'Short English',
      );
      await tester.pump();
      await tester.tap(find.text('Send answer'));
      await tester.pump();
      expect(chat.userQuestion, isNotNull);
      expect(chat.questionAnswers, ['Today', 'Short English']);
      expect(find.textContaining('Could not send your answer'), findsOneWidget);
      bridge.fail = false;
      bridge.answerPending = Completer<void>();
      unawaited(chat.answerQuestion());
      await tester.pump();
      await chat.answerQuestion();
      expect(bridge.answers.length, 2);
      expect(
        tester
            .widget<TextField>(find.byKey(const Key('question-custom-answer')))
            .enabled,
        isFalse,
      );
      await chat.stop();
      await tester.pump(const Duration(milliseconds: 250));
      bridge.answerPending!.complete();
      await tester.pumpAndSettle();
      expect(chat.userQuestion, isNull);
      expect(chat.draft, 'Read readme');
      expect(chat.questionAnswers, ['Today', 'Short English']);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  for (final dark in [false, true]) {
    testWidgets('failed answer can be retried from the compact UI ($dark)', (
      tester,
    ) async {
      compact(tester);
      final bridge = QuestionBridge()..fail = true;
      final chat = ready(bridge);
      final capture = GlobalKey();
      await tester.pumpWidget(
        DoloresApp(
          chat: chat,
          captureKey: capture,
          themeMode: dark ? ThemeMode.dark : ThemeMode.light,
        ),
      );
      await chat.send();
      await tester.pump(const Duration(milliseconds: 100));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('question-option-1')));
      await tester.pump();
      await tester.tap(find.text('Next'));
      await tester.pump();
      await tester.enterText(
        find.byKey(const Key('question-custom-answer')),
        '简体中文，保留引用',
      );
      await tester.pump();
      await tester.tap(find.text('Send answer'));
      await tester.pumpAndSettle();
      final failure = find.textContaining('Could not send your answer');
      expect(failure, findsOneWidget);
      expect(failure.hitTestable(), findsOneWidget);
      expect(chat.questionAnswers, ['Previous entry', '简体中文，保留引用']);
      expect(bridge.answers.length, 1);
      const directory = String.fromEnvironment('QUESTION_RENDER_DIR');
      if (directory.isNotEmpty) {
        await tester.runAsync(() async {
          final image =
              await (capture.currentContext!.findRenderObject()
                      as RenderRepaintBoundary)
                  .toImage();
          final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
          final file = File(
            '$directory/question-retry-${dark ? 'dark' : 'light'}.png',
          );
          await file.parent.create(recursive: true);
          await file.writeAsBytes(bytes!.buffer.asUint8List());
          image.dispose();
        });
      }
      await tester.pump(const Duration(seconds: 2));
      expect(bridge.answers.length, 1, reason: 'No silent retry');
      await tester.tap(find.text('Back'));
      await tester.pumpAndSettle();
      expect(chat.questionAnswers.first, 'Previous entry');
      await tester.tap(find.text('Next'));
      await tester.pumpAndSettle();
      expect(
        tester
            .widget<TextField>(find.byKey(const Key('question-custom-answer')))
            .controller!
            .text,
        '简体中文，保留引用',
      );
      bridge.fail = false;
      await tester.tap(find.text('Send answer'));
      await tester.pump(const Duration(milliseconds: 250));
      await tester.pumpAndSettle();
      expect(bridge.answers.length, 2);
      expect(bridge.answers[1], bridge.answers[0]);
      expect(find.byType(UserQuestionCard), findsNothing);
      expect(find.text('Report prepared.'), findsOneWidget);
      expect(chat.busy, isFalse);
      expect(
        bridge.commands.where((c) => c['command'] == 'approveTool'),
        isEmpty,
      );
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    });
  }
  for (final dark in [false, true]) {
    for (final width in [390.0, 1200.0]) {
      testWidgets(
        'Question card fits $width and ${dark ? 'dark' : 'light'} with long options',
        (tester) async {
          tester.view.physicalSize = Size(width, 820);
          tester.view.devicePixelRatio = 1;
          addTearDown(tester.view.resetPhysicalSize);
          addTearDown(tester.view.resetDevicePixelRatio);
          final chat = ready(QuestionBridge())
            ..busy = true
            ..userQuestion = QuestionBridge.request
            ..questionAnswers = ['', ''];
          final capture = GlobalKey();
          await tester.pumpWidget(
            DoloresApp(
              chat: chat,
              captureKey: capture,
              themeMode: dark ? ThemeMode.dark : ThemeMode.light,
            ),
          );
          await tester.pumpAndSettle();
          expect(find.text('Answer question'), findsOneWidget);
          expect(find.text('Previous entry'), findsOneWidget);
          expect(tester.takeException(), isNull);
          const directory = String.fromEnvironment('QUESTION_RENDER_DIR');
          if (directory.isNotEmpty) {
            await tester.runAsync(() async {
              final image =
                  await (capture.currentContext!.findRenderObject()
                          as RenderRepaintBoundary)
                      .toImage(pixelRatio: 1);
              final bytes = await image.toByteData(
                format: ui.ImageByteFormat.png,
              );
              await Directory(directory).create(recursive: true);
              await File(
                '$directory/question-${width.toInt()}-${dark ? 'dark' : 'light'}.png',
              ).writeAsBytes(bytes!.buffer.asUint8List());
              image.dispose();
            });
          }
          await tester.pumpWidget(const SizedBox());
          chat.dispose();
        },
      );
    }
  }
}
