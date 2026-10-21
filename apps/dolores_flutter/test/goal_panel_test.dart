import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/goal_panel.dart';

import 'instruction_queue_test.dart' show QueueBridge;
import 'support/render_fonts.dart';

Map<String, dynamic> savedGoal({String state = 'active'}) => {
  'id': 'public-goal',
  'revision': 3,
  'progressRevision': 1,
  'objective': 'Create a report and verify its exact content without changing existing work',
  'constraints': 'Preserve unrelated files and record verification',
  'state': state,
  'steps': [
    {
      'criterion': 'Report exists with verified content',
      'done': false,
      'evidence': [],
    },
  ],
  'used': {'modelCalls': 7, 'toolCalls': 4, 'segments': 2, 'activeSeconds': 28},
  'limits': {'modelCalls': 12, 'toolCalls': 16, 'segments': 4},
  'currentRun': null,
  'note': state == 'paused'
      ? 'Reopened. Inspect retained work before Resume.'
      : 'Working toward saved criteria.',
};

class GoalBridge extends QueueBridge {
  Map<String, dynamic>? goal;
  bool failGoal = false;
  int goalStarts = 0;
  @override
  Future<dynamic> call(Map<String, dynamic> c) async {
    if (c['command'] == 'goal') return goal;
    if (c['command'] == 'editGoal') {
      if (failGoal) throw StateError('Storage unavailable. Your edits remain.');
      final e = c['edit'] as Map;
      goal = {
        ...savedGoal(),
        'objective': e['objective'] ?? goal?['objective'],
        'steps': e['criteria'] == null
            ? goal!['steps']
            : [
                for (final criterion in e['criteria'])
                  {'criterion': criterion, 'done': false, 'evidence': []},
              ],
        'state': e['action'] == 'pause'
            ? 'paused'
            : e['action'] == 'clear'
            ? 'cleared'
            : 'active',
      };
      return goal;
    }
    if (c['command'] == 'startGoal') {
      goalStarts++;
      return null;
    }
    return super.call(c);
  }
}

ChatController controller(GoalBridge b) => ChatController(b)
  ..loading = false
  ..configured = true
  ..goalsAvailable = true
  ..instructionQueueAvailable = true
  ..session = 'audit';
void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(loadReviewFonts);
  testWidgets(
    'failed goal save preserves form and chat draft; saving never starts execution',
    (tester) async {
      final bridge = GoalBridge()..failGoal = true;
      final chat = controller(bridge)..draft = 'Unsubmitted user work';
      await tester.pumpWidget(DoloresApp(chat: chat));
      await tester.pumpAndSettle();
      final dialog = showGoal(tester.element(find.byType(ChatPage)), chat);
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byKey(const Key('goal-objective')),
        'Create the public report',
      );
      await tester.enterText(
        find.byKey(const Key('goal-criteria')),
        'Read back exact content',
      );
      await tester.tap(find.byKey(const Key('save-goal')));
      await tester.pumpAndSettle();
      expect(find.text('Create the public report'), findsOneWidget);
      expect(chat.goal, isNull);
      expect(chat.draft, 'Unsubmitted user work');
      expect(bridge.goalStarts, 0);
      bridge.failGoal = false;
      await tester.tap(find.byKey(const Key('save-goal')));
      await tester.pumpAndSettle();
      expect(chat.goal?['state'], 'active');
      expect(bridge.goalStarts, 0);
      chat.draft = '';
      await chat.dispatchGoal();
      expect(
        bridge.goalStarts,
        0,
        reason: 'Saving cannot authorize later automatic execution',
      );
      await tester.tap(find.text('Close'));
      await tester.pumpAndSettle();
      await dialog;
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'goal dispatch yields to drafts, pending input and paused state',
    (tester) async {
      final b = GoalBridge()..goal = savedGoal();
      final chat = controller(b);
      await chat.refreshGoal();
      await chat.dispatchGoal();
      expect(b.goalStarts, 0);
      await chat.startGoal();
      expect(b.goalStarts, 1);
      chat.busy = false;
      await tester.pump(const Duration(milliseconds: 30));
      b.goalStarts = 0;
      chat.draft = 'New instruction';
      await chat.dispatchGoal();
      expect(b.goalStarts, 0);
      chat.draft = '';
      chat.taskInputs = {
        'entries': [
          {'state': 'pending'},
        ],
      };
      await chat.dispatchGoal();
      expect(b.goalStarts, 0);
      chat.taskInputs = {};
      chat.goal!['state'] = 'paused';
      await chat.dispatchGoal();
      expect(b.goalStarts, 0);
      chat.goal!['state'] = 'active';
      await chat.dispatchGoal();
      expect(b.goalStarts, 1);
      expect(chat.busy, isTrue);
      chat.busy = false;
      chat.dispose();
    },
  );
  testWidgets(
    'reopened goal stays paused and shows cumulative usage until explicit Resume',
    (tester) async {
      final b = GoalBridge()..goal = savedGoal(state: 'paused');
      final chat = controller(b);
      await chat.refreshGoal();
      await chat.dispatchGoal();
      expect(b.goalStarts, 0);
      await tester.pumpWidget(DoloresApp(chat: chat));
      await tester.pumpAndSettle();
      await tester.tap(find.byTooltip('Inspect goal and progress'));
      await tester.pumpAndSettle();
      expect(find.textContaining('7/12 model calls'), findsOneWidget);
      expect(find.byKey(const Key('resume-goal')), findsOneWidget);
      await tester.tap(find.byKey(const Key('resume-goal')));
      await tester.pumpAndSettle();
      expect(chat.goal?['state'], 'active');
      expect(b.goalStarts, 0);
      expect(chat.goal!['used']['modelCalls'], 7);
      await tester.tap(find.text('Close'));
      await tester.pumpAndSettle();
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  for (final dark in [false, true]) {
    testWidgets('goal progress compact ${dark ? 'dark' : 'light'}', (
      tester,
    ) async {
      tester.view.physicalSize = const Size(540, 900);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final b = GoalBridge()..goal = savedGoal(state: 'paused');
      final chat = controller(b);
      await chat.refreshGoal();
      final key = GlobalKey();
      await tester.pumpWidget(
        RepaintBoundary(
          key: key,
          child: DoloresApp(
            chat: chat,
            themeMode: dark ? ThemeMode.dark : ThemeMode.light,
          ),
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byTooltip('Inspect goal and progress'));
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
      final boundary =
          key.currentContext!.findRenderObject()! as RenderRepaintBoundary;
      await tester.runAsync(() async {
        final image = await boundary.toImage(pixelRatio: 1);
        final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
        image.dispose();
        final file = File(
          '../../output/harness/eh03-${dark ? 'dark' : 'light'}-compact.png',
        );
        file.parent.createSync(recursive: true);
        file.writeAsBytesSync(bytes!.buffer.asUint8List());
      });
      await tester.tap(find.text('Close'));
      await tester.pumpAndSettle();
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    });
  }
}
