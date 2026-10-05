import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';

class CapabilityBridge implements ChatBridge {
  final commands = <Map<String, dynamic>>[];
  List<String> images = [];
  bool failSave = false;
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    commands.add(command);
    switch (command['command']) {
      case 'modelDetails':
        return {
          'contextWindowTokens': null,
          'imageInput': images.contains(command['preferences']['model']),
          'requestSettings': null,
        };
      case 'bootstrap':
        return {
          'sessions': <Map<String, dynamic>>[],
          'preferences': {
            'baseUrl': 'http://localhost:11434/v1',
            'model': 'text-model',
          },
          'configured': true,
          'enabledModels': ['text-model', 'vision-model'],
          'attachments': true,
          'imageModels': images,
        };
      case 'setImageModels':
        if (failSave) {
          throw StateError('Image settings could not be saved. Try again.');
        }
        images = (command['models'] as List).cast<String>();
      case 'start':
        throw StateError(
          'This model has image input disabled. Your draft remains.',
        );
    }
    return null;
  }
}

void main() {
  testWidgets(
    'Attachment action stays at the left with aligned right controls',
    (tester) async {
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      for (final width in [1120.0, 420.0]) {
        tester.view.physicalSize = Size(width, 780);
        for (final dark in [false, true]) {
          for (final name in [
            'short',
            'provider/a-very-long-model-name-with-a-large-context-window',
          ]) {
            final chat = ChatController(CapabilityBridge())
              ..loading = false
              ..configured = true
              ..attachmentsAvailable = true
              ..model = name
              ..enabledModels = [name];
            await tester.pumpWidget(
              DoloresApp(
                chat: chat,
                themeMode: dark ? ThemeMode.dark : ThemeMode.light,
              ),
            );
            await tester.pumpAndSettle();
            final attach = tester.getRect(find.byKey(const Key('attach-file')));
            final actions = tester.getRect(
              find.byKey(const Key('composer-actions')),
            );
            final picker = tester.getRect(
              find.byKey(const Key('chat-model-picker')),
            );
            final context = tester.getRect(
              find.byKey(const Key('context-preview')),
            );
            final send = tester.getRect(find.byKey(const Key('send')));
            expect(attach.left, closeTo(actions.left, 0.1));
            expect(attach.center.dy, closeTo(send.center.dy, 0.1));
            expect(picker.right, closeTo(context.left, 0.1));
            expect(context.center.dy, closeTo(send.center.dy, 0.1));
            expect(send.right, closeTo(actions.right, 0.1));
            expect(picker.left, greaterThanOrEqualTo(attach.right));
            expect(tester.takeException(), isNull);
            await tester.pumpWidget(const SizedBox());
            chat.dispose();
          }
        }
      }
    },
  );

  testWidgets(
    'Image refusal links visible per-model settings and keeps the draft on cancel',
    (tester) async {
      tester.view.physicalSize = const Size(420, 680);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      for (final dark in [false, true]) {
        final chat = ChatController(CapabilityBridge())
          ..session = 'fixture'
          ..loading = false
          ..configured = true
          ..attachmentsAvailable = true
          ..model = 'text-model'
          ..enabledModels = ['text-model', 'vision-model']
          ..attachments = [
            {
              'digest': 'a' * 64,
              'name': 'sample.png',
              'mime': 'image/png',
              'bytes': 100,
            },
          ]
          ..draft = 'Keep my image question';
        await chat.send();
        await tester.pumpWidget(
          DoloresApp(
            chat: chat,
            themeMode: dark ? ThemeMode.dark : ThemeMode.light,
          ),
        );
        await tester.tap(find.byKey(const Key('image-model-settings')));
        await tester.pumpAndSettle();
        await tester.ensureVisible(
          find.byKey(const Key('model-detail-images')),
        );
        await tester.pumpAndSettle();
        expect(
          find.byKey(const Key('model-detail-images')).hitTestable(),
          findsOneWidget,
        );
        expect(
          tester
              .widget<SwitchListTile>(
                find.byKey(const Key('model-detail-images')),
              )
              .value,
          isFalse,
        );
        await tester.tap(find.byKey(const Key('model-detail-images')));
        await tester.pumpAndSettle();
        await tester.tap(find.byTooltip('Close model details'));
        await tester.pumpAndSettle();
        await tester.tap(find.text('Discard'));
        await tester.pumpAndSettle();
        await tester.tap(find.byKey(const Key('close-settings')));
        await tester.pumpAndSettle();
        expect(chat.imageModels, isEmpty);
        expect(chat.attachments.single['name'], 'sample.png');
        expect(chat.draft, 'Keep my image question');
        // Switching to a text-only model can also reject images in saved history.
        chat.messages = [
          {
            'role': 'user',
            'content': 'Earlier image',
            'parts': chat.attachments,
          },
        ];
        chat.attachments = [];
        chat.reportLocalError('This model has image input disabled.');
        await tester.pumpAndSettle();
        expect(
          find.byKey(const Key('image-model-settings')).hitTestable(),
          findsOneWidget,
        );
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
        chat.dispose();
      }
    },
  );

  testWidgets(
    'Capability selection is per model; failed save retains corrections for retry',
    (tester) async {
      tester.view.physicalSize = const Size(420, 680);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final bridge = CapabilityBridge()..failSave = true;
      final chat = ChatController(bridge)
        ..loading = false
        ..draft = 'Preserved';
      await chat.refresh();
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: TextButton(
                child: const Text('Open'),
                onPressed: () => showDialog<void>(
                  context: context,
                  builder: (_) => ConnectionDialog(chat: chat),
                ),
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.text('Open'));
      await tester.pumpAndSettle();
      await tester.tap(find.byType(DropdownButtonFormField<String>));
      await tester.pumpAndSettle();
      await tester.tap(find.text('vision-model').last);
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('model-image-input')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('save-connection')));
      await tester.pumpAndSettle();
      await tester.ensureVisible(
        find.textContaining('Image settings could not be saved'),
      );
      expect(find.byType(ConnectionDialog), findsOneWidget);
      expect(chat.imageModels, isEmpty);
      expect(chat.draft, 'Preserved');
      expect(
        tester
            .widget<CheckboxListTile>(
              find.byKey(const Key('model-image-input')),
            )
            .value,
        isTrue,
      );
      bridge.failSave = false;
      await tester.tap(find.byKey(const Key('save-connection')));
      await tester.pumpAndSettle();
      expect(chat.model, 'text-model');
      expect(chat.imageModels, ['vision-model']);
      expect(chat.draft, 'Preserved');
      expect(find.byType(ConnectionDialog), findsNothing);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
}
