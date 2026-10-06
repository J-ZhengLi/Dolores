import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:dolores_flutter/attachments.dart';
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
  Completer<void>? start;
  bool failAttach = false;
  final imagePart = {
    ...part,
    'name': 'Pasted image.png',
    'mime': 'image/png',
    'bytes': 70,
  };
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    calls.add(command);
    switch (command['command']) {
      case 'start':
        if (start != null) return start!.future;
        throw StateError(
          'Image input is disabled. Choose a capable model or remove the image; your draft remains.',
        );
      case 'removeAttachment':
        return <Map<String, dynamic>>[];
      case 'attachmentPreview':
        if (command['digest'] == imagePart['digest']) {
          // Test rendering separately from the text chip's preview.
          if (imagePreview) return {'imageBase64': png};
        }
        return {
          'reference': part,
          'text': 'Saved snapshot',
          'previewTruncated': false,
        };
      case 'attachFile':
        if (failAttach) throw StateError('Synthetic storage failure');
        expect(
          await File(command['path'] as String).readAsBytes(),
          base64Decode(png),
        );
        return [imagePart];
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

  bool imagePreview = false;
}

const png =
    'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aX1cAAAAASUVORK5CYII=';

void main() {
  testWidgets(
    'Images appear in the pending user bubble before the response, and in saved bubbles',
    (tester) async {
      final bridge = AttachmentBridge()
        ..imagePreview = true
        ..start = Completer<void>();
      final chat = ChatController(bridge)
        ..session = 'a'
        ..loading = false
        ..configured = true
        ..attachmentsAvailable = true
        ..attachments = [bridge.imagePart]
        ..draft = 'Inspect this image';
      await tester.pumpWidget(DoloresApp(chat: chat));
      final sending = chat.send();
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 50));
      expect(
        find.descendant(
          of: find.byKey(const ValueKey('pending-user')),
          matching: find.byType(AttachmentThumbnail),
        ),
        findsOneWidget,
      );
      expect(find.byType(AttachmentThumbnail), findsOneWidget);
      bridge.start!.completeError(StateError('Synthetic interrupted stream'));
      await sending;
      await tester.pump();
      expect(chat.draft, 'Inspect this image');
      expect(chat.attachments, [bridge.imagePart]);
      expect(chat.pendingParts, isEmpty);
      chat.attachments = [];
      chat.messages = [
        {
          'id': 1,
          'role': 'user',
          'content': 'Inspect this image',
          'parts': [bridge.imagePart],
        },
      ];
      await tester.pumpWidget(
        DoloresApp(chat: chat, themeMode: ThemeMode.light),
      );
      await tester.pump(const Duration(milliseconds: 50));
      expect(find.byType(AttachmentThumbnail), findsOneWidget);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );

  test('Image paste reserves its session, cleans its temporary copy and preserves text on failure', () async {
    final bridge = AttachmentBridge();
    final chat = ChatController(bridge)
      ..session = 'a'
      ..loading = false
      ..draft = 'Keep this';
    final read = Completer<Uint8List?>();
    final paste = chat.pasteImage(readImage: () => read.future);
    expect(chat.changing, true);
    await chat.select('b');
    expect(chat.session, 'a');
    read.complete(base64Decode(png));
    expect(await paste, true);
    final path =
        bridge.calls.singleWhere((c) => c['command'] == 'attachFile')['path']
            as String;
    expect(File(path).existsSync(), false);
    expect(chat.draft, 'Keep this');
    expect(chat.attachments, [bridge.imagePart]);
    bridge.failAttach = true;
    expect(
      await chat.pasteImage(readImage: () async => base64Decode(png)),
      true,
    );
    expect(chat.draft, 'Keep this');
    expect(chat.attachments, [bridge.imagePart]);
    expect(chat.error, contains('draft remains'));
    final count = bridge.calls.length;
    expect(
      await chat.pasteImage(
        readImage: () async => Uint8List(2 * 1024 * 1024 + 1),
      ),
      true,
    );
    expect(bridge.calls.length, count);
    expect(await chat.pasteImage(readImage: () async => null), false);
    expect(chat.changing, false);
    chat.dispose();
  });
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
        if (find.text('Settings').evaluate().isEmpty) {
          await tester.tap(find.byTooltip('Conversations'));
          await tester.pumpAndSettle();
        }
        await tester.tap(find.text('Settings'));
        await tester.pumpAndSettle();
        await tester.enterText(
          find.byKey(const Key('settings-search')),
          'Attachment storage',
        );
        await tester.pumpAndSettle();
        await tester.tap(
          find.byKey(const Key('setting-result-storage-connection')),
        );
        await tester.pumpAndSettle();
        await tester.tap(find.text('Clean unused attachments'));
        await tester.pumpAndSettle();
        await tester.tap(find.text('Clean unused'));
        await tester.pumpAndSettle();
        expect(find.textContaining('Cleanup failed'), findsOneWidget);
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
