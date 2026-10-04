import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';

final part = <String, dynamic>{
  'digest': 'a' * 64,
  'name': 'notes.txt',
  'mime': 'text/plain',
  'bytes': 12,
};

class AttachmentBridge implements ChatBridge {
  final calls = <Map<String, dynamic>>[];
  bool fail = true;
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    calls.add(command);
    switch (command['command']) {
      case 'start':
        throw StateError(
          'Image input is disabled. Choose a capable model or remove the image; your draft remains.',
        );
      case 'removeAttachment':
        return <Map<String, dynamic>>[];
      case 'attachmentPreview':
        return {
          'reference': part,
          'text': 'Saved snapshot',
          'previewTruncated': false,
        };
      case 'cleanupAttachments':
        throw StateError('Synthetic database lock');
      case 'workspace':
        return {'kind': 'side', 'root': null};
      case 'messagesPage':
        return {
          'items': <Map<String, dynamic>>[],
          'hasOlder': false,
          'hasNewer': false,
        };
      case 'draftAttachments':
        if (fail) throw StateError('Snapshot unavailable');
        return <Map<String, dynamic>>[];
      default:
        return null;
    }
  }
}

void main() {
  testWidgets(
    'Preview and removal stay usable after rejected send in compact themes',
    (tester) async {
      tester.view.physicalSize = const Size(420, 680);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      for (final dark in [false, true]) {
        final bridge = AttachmentBridge();
        final chat = ChatController(bridge)
          ..session = 'a'
          ..loading = false
          ..configured = true
          ..attachmentsAvailable = true
          ..attachments = [part]
          ..draft = 'Keep my question';
        await chat.send();
        expect(chat.draft, 'Keep my question');
        expect(chat.attachments, [part]);
        expect(chat.busy, false);
        await tester.pumpWidget(
          DoloresApp(
            chat: chat,
            themeMode: dark ? ThemeMode.dark : ThemeMode.light,
          ),
        );
        await tester.tap(find.text('notes.txt'));
        await tester.pumpAndSettle();
        expect(find.text('Saved snapshot'), findsOneWidget);
        expect(find.textContaining('Sending shares'), findsOneWidget);
        await tester.tap(find.text('Close'));
        await tester.pumpAndSettle();
        final chip = tester.widget<InputChip>(find.byType(InputChip));
        expect(chip.onDeleted, isNotNull);
        chip.onDeleted!();
        await tester.pump();
        expect(chat.attachments, isEmpty);
        expect(chat.draft, 'Keep my question');
        await tester.tap(find.byTooltip('Chat actions'));
        await tester.pumpAndSettle();
        await tester.tap(find.text('Clean unused attachments'));
        await tester.pumpAndSettle();
        expect(chat.error, contains('could not be cleaned'));
        expect(chat.draft, 'Keep my question');
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
        chat.dispose();
      }
    },
  );
  test('Failed chat switch keeps prior draft and attachments bound together; attachment blocks resume', () async {
    final bridge = AttachmentBridge();
    final chat = ChatController(bridge)
      ..session = 'a'
      ..loading = false
      ..attachmentsAvailable = true
      ..attachments = [part]
      ..draft = 'Original draft';
    await chat.select('b');
    expect(chat.session, 'a');
    expect(chat.draft, 'Original draft');
    expect(chat.attachments, [part]);
    chat.draft = '';
    await expectLater(chat.prepareCheckpoint('run'), throwsStateError);
    expect(
      bridge.calls.where((c) => c['command'] == 'checkpointDraft'),
      isEmpty,
    );
    chat.dispose();
  });
}
