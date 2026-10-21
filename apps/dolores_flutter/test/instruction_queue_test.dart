import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/rich_composer.dart';

import 'stop_recovery_test.dart' show InterruptedBridge;
import 'support/render_fonts.dart';

class QueueBridge extends InterruptedBridge {
  QueueBridge() : super(false);
  bool failSave = false;
  Map<String, dynamic> state = {
    'revision': 0,
    'suspended': false,
    'entries': <Map<String, dynamic>>[],
  };
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    if (command['command'] == 'startTaskInput') {
      final entry = (state['entries'] as List).firstWhere(
        (v) => v['state'] == 'pending',
      ) as Map;
      entry['state'] = 'applied';
      await super.call({...command, 'command': 'start'});
      return {'input': entry['text'], 'instructionId': entry['id']};
    }
    if (command['command'] == 'taskInputs') return Map.of(state);
    if (command['command'] == 'editTaskInputs') {
      if (failSave) throw StateError('Storage is locked. Try again.');
      final edit = command['edit'] as Map;
      if (edit['action'] == 'enqueue') {
        (state['entries'] as List).add({
          'id': edit['id'],
          'text': edit['text'],
          'mode': edit['mode'],
          'state': 'pending',
          'revision': 1,
          'position': (state['entries'] as List).length + 1,
        });
      } else if (edit['action'] == 'resume') {
        state['suspended'] = false;
      } else if (edit['action'] == 'edit') {
        final item = (state['entries'] as List).firstWhere(
          (e) => e['id'] == edit['id'],
        ) as Map;
        if (item['revision'] != edit['revision']) {
          throw StateError('Task changed. Compare latest.');
        }
        item['text'] = edit['text'];
        item['revision'] = (item['revision'] as int) + 1;
      }
      state['revision'] = (state['revision'] as int) + 1;
      return Map.of(state);
    }
    if (command['command'] == 'cancel') state['suspended'] = true;
    final result = await super.call(command);
    if (command['command'] == 'bootstrap') {
      return {...result as Map, 'instructionQueue': true};
    }
    return result;
  }
}

ChatController controller(QueueBridge bridge) => ChatController(bridge)
  ..loading = false
  ..configured = true
  ..instructionQueueAvailable = true
  ..session = 'audit';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(loadReviewFonts);
  testWidgets(
    'failed enqueue keeps text; Stop retains newer draft and pauses pending input',
    (tester) async {
      final bridge = QueueBridge();
      final chat = controller(bridge)..draft = 'Original task';
      await chat.send();
      chat.draft = 'Follow-up 世界';
      bridge.failSave = true;
      await chat.queueDraft();
      expect(chat.draft, 'Follow-up 世界');
      expect(chat.inputEntries, isEmpty);
      bridge.failSave = false;
      await chat.queueDraft();
      expect(chat.draft, isEmpty);
      expect(chat.inputEntries.single['state'], 'pending');
      chat.draft = 'Newer unsubmitted draft';
      await chat.stop();
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pump();
      expect(chat.busy, isFalse);
      expect(chat.draft, 'Newer unsubmitted draft');
      expect(chat.inputsPaused, isTrue);
      expect(bridge.starts, 1);
      expect(chat.inputEntries.single['state'], 'pending');
      chat.dispose();
    },
  );

  testWidgets('reopened pending queue waits for explicit resume', (
    tester,
  ) async {
    final bridge = QueueBridge();
    bridge.state = {
      'revision': 2,
      'suspended': true,
      'recoveryRequired': true,
      'entries': [
        {
          'id': 'kept',
          'revision': 1,
          'position': 1,
          'text': 'Retained task',
          'mode': 'queue',
          'state': 'pending',
        },
      ],
    };
    final chat = controller(bridge);
    await chat.refreshInputs();
    await chat.dispatchNextInput();
    expect(bridge.starts, 0);
    expect(chat.inputsPaused, isTrue);
    chat.dispose();
  });

  testWidgets(
    'queued completion keeps newer unsent text and attachment references',
    (tester) async {
      final bridge = QueueBridge()..starts = 1;
      final chat = controller(bridge)
        ..draft = 'Unsubmitted draft'
        ..attachments = [
          {
            'digest': 'public-fixture',
            'name': 'not-shared.txt',
            'mime': 'text/plain',
            'bytes': 4,
          },
        ];
      await chat.changeInput({
        'action': 'enqueue',
        'id': 'kept',
        'text': 'Queued follow-up',
        'mode': 'queue',
      });
      await chat.dispatchNextInput();
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pump();
      expect(chat.busy, isFalse);
      expect(chat.draft, 'Unsubmitted draft');
      expect(chat.attachments.single['name'], 'not-shared.txt');
      expect(bridge.starts, 2);
      chat.dispose();
    },
  );

  for (final dark in [false, true]) {
    testWidgets(
      'busy composer exposes Queue Steer and Stop in ${dark ? 'dark' : 'light'} compact layout',
      (tester) async {
        final bridge = QueueBridge();
        final chat = controller(bridge)
          ..busy = true
          ..draft = 'Keep column order'
          ..appearance = dark ? 'dark' : 'light';
        await chat.changeInput({
          'action': 'enqueue',
          'id': 'first',
          'text': 'Add UTF-8 example',
          'mode': 'queue',
        });
        tester.view.physicalSize = const Size(440, 900);
        tester.view.devicePixelRatio = 1;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);
        final key = GlobalKey();
        await tester.pumpWidget(
          RepaintBoundary(
            key: key,
            child: DoloresApp(chat: chat),
          ),
        );
        await tester.pump();
        expect(
          tester
              .widget<RichComposer>(find.byKey(const Key('composer')))
              .readOnly,
          isFalse,
        );
        expect(find.byKey(const Key('queue-instruction')), findsOneWidget);
        expect(find.byKey(const Key('steer-instruction')), findsOneWidget);
        expect(find.byTooltip('Stop response'), findsOneWidget);
        final boundary =
            key.currentContext!.findRenderObject()! as RenderRepaintBoundary;
        await tester.runAsync(() async {
          final image = await boundary.toImage(pixelRatio: 1);
          final data = await image.toByteData(format: ui.ImageByteFormat.png);
          image.dispose();
          final file = File(
            '../../output/harness/eh01-${dark ? 'dark' : 'light'}-compact.png',
          );
          file.parent.createSync(recursive: true);
          file.writeAsBytesSync(data!.buffer.asUint8List());
        });
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
        chat.busy = false;
        chat.dispose();
      },
    );
  }
  testWidgets(
    'pending editor keeps unsaved text through failed save and explicit stale comparison',
    (tester) async {
      final bridge = QueueBridge();
      final chat = controller(bridge)..busy = true;
      await chat.changeInput({
        'action': 'enqueue',
        'id': 'edit',
        'text': 'Original instruction',
        'mode': 'queue',
      });
      await tester.pumpWidget(DoloresApp(chat: chat));
      await tester.pump();
      await tester.tap(find.byTooltip('Instruction actions'));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 250));
      await tester.tap(find.text('Edit'));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 250));
      final field = find.descendant(
        of: find.byType(AlertDialog),
        matching: find.byType(TextField),
      );
      await tester.enterText(field, 'My retained edits');
      bridge.failSave = true;
      await tester.tap(find.text('Save'));
      await tester.pump();
      await tester.pump();
      expect(
        tester.widget<TextField>(field).controller!.text,
        'My retained edits',
      );
      expect(find.text('Compare latest'), findsOneWidget);
      expect(find.text('Stop task'), findsOneWidget);
      final item = (bridge.state['entries'] as List).single as Map;
      item['text'] = 'Changed elsewhere';
      item['revision'] = 2;
      bridge.failSave = false;
      await tester.tap(find.text('Compare latest'));
      await tester.pump();
      await tester.pump();
      expect(
        find.textContaining('Latest saved instruction:\nChanged elsewhere'),
        findsOneWidget,
      );
      expect(
        tester.widget<TextField>(field).controller!.text,
        'My retained edits',
      );
      await tester.tap(find.text('Save'));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 250));
      expect(find.byType(AlertDialog), findsNothing);
      expect(item['text'], 'My retained edits');
      await tester.pumpWidget(const SizedBox());
      chat.busy = false;
      chat.dispose();
    },
  );
}
