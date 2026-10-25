import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/file_workspace.dart';

import 'editor_test.dart' show EditingBridge;

class RecoveryBridge extends EditingBridge {
  bool failCheckpoint = false, failDraft = false;
  int draftAttempts = 0;
  int checkpoints = 0;
  @override
  Future<dynamic> call(Map<String, dynamic> cmd) async {
    if (cmd['command'] == 'saveDraft') {
      draftAttempts++;
      if (failDraft) {
        throw StateError('Could not read or save local conversation data.');
      }
    }
    if (cmd['command'] == 'editor' &&
        cmd['request']['action'] == 'checkpoint') {
      checkpoints++;
      if (failCheckpoint) throw StateError('Recovery storage unavailable.');
    }
    return super.call(cmd);
  }
}

void main() {
  testWidgets('conversation storage failure cannot veto Close', (t) async {
    final bridge = RecoveryBridge()..failDraft = true;
    final chat = ChatController(bridge)
      ..loading = false
      ..session = 'A';
    final host = AppHost(chat);
    await t.pumpWidget(DoloresApp(chat: chat, host: host));
    await t.pump(const Duration(milliseconds: 300));
    expect(await host.requestClose(), true);
    expect(bridge.draftAttempts, 1);
    expect(
      host.error,
      contains('Could not read or save local conversation data'),
    );
    expect(find.text('Finish the open review'), findsNothing);
    await t.pumpWidget(const SizedBox());
    host.dispose();
  });
  testWidgets('Close never checkpoints an unfinished conversation load', (
    t,
  ) async {
    final bridge = RecoveryBridge()..failDraft = true;
    final chat = ChatController(bridge)
      ..loading = true
      ..session = 'A';
    final host = AppHost(chat);
    expect(await host.requestClose(), true);
    expect(
      bridge.draftAttempts,
      0,
      reason: 'Keep the saved draft until the editor has loaded.',
    );
    host.dispose();
  });

  testWidgets(
    'one settled edit checkpoints once; failed recovery cannot veto quit or overwrite source',
    (t) async {
      final bridge = RecoveryBridge();
      final chat = ChatController(bridge)..loading = false;
      final host = AppHost(chat);
      await host.files.bind('A', 'C:/A');
      final w = host.files.selected!;
      final d = (await host.files.open(w, 'a.txt'))!;
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(body: FileWorkspaceView(host: host)),
        ),
      );
      await t.pumpAndSettle();
      final view = d.buffer.createView();
      view.replaceSelection('mine ');
      await host.files.flush(d);
      await t.pump(const Duration(milliseconds: 500));
      await t.pump();
      expect(bridge.checkpoints, 1);
      await t.pump(const Duration(seconds: 2));
      await t.pump();
      expect(bridge.checkpoints, 1);
      bridge.failCheckpoint = true;
      expect(await host.prepareQuit(saveFiles: false), true);
      expect(host.error, contains('Recovery storage unavailable'));
      expect(d.text, 'mine saved');
      expect(d.dirty, true);
      bridge.failCheckpoint = false;
      expect(await host.prepareQuit(saveFiles: false), true);
      expect(d.dirty, true);
      view.dispose();
      await t.pumpWidget(const SizedBox());
      host.dispose();
    },
  );
  testWidgets(
    'Close skips open dialogs and unsaved-file prompts; global theme follows cached conversations',
    (t) async {
      t.view.physicalSize = const Size(1000, 700);
      t.view.devicePixelRatio = 1;
      addTearDown(t.view.resetPhysicalSize);
      addTearDown(t.view.resetDevicePixelRatio);
      final bridge = RecoveryBridge();
      final chat = ChatController(bridge)..loading = false;
      final host = AppHost(chat);
      await host.files.bind('A', 'C:/A');
      final w = host.files.selected!;
      final d = (await host.files.open(w, 'a.txt'))!;
      final view = d.buffer.createView();
      view.replaceSelection('draft ');
      await host.files.flush(d);
      await t.pumpWidget(DoloresApp(chat: chat, host: host));
      await t.pumpAndSettle();
      bridge.failSave = true;
      // An open review and source-save failure cannot veto the close decision.
      final context = t.element(find.byType(Scaffold).first);
      showDialog<void>(
        context: context,
        builder: (_) => const AlertDialog(title: Text('Open review')),
      );
      await t.pumpAndSettle();
      expect(await host.requestClose(), true);
      expect(find.text('Close Dolores?'), findsNothing);
      expect(find.text('Finish the open review'), findsNothing);
      expect(d.closed, false);
      expect(d.text, 'draft saved');
      await host.select('B');
      await expectLater(host.visible.saveNativeDrafts(), throwsStateError);
      expect(d.text, 'draft saved');
      chat.appearance = 'dark';
      expect(host.visible.appearance, 'dark');
      await host.select('A');
      expect(host.visible.appearance, 'dark');
      view.dispose();
      await t.pumpWidget(const SizedBox());
      host.dispose();
    },
  );
}
