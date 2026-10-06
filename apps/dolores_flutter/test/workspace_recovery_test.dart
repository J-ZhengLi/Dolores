import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/file_workspace.dart';

import 'editor_test.dart' show EditingBridge;

class RecoveryBridge extends EditingBridge {
  bool failCheckpoint = false;
  int checkpoints = 0;
  @override
  Future<dynamic> call(Map<String, dynamic> cmd) async {
    if (cmd['command'] == 'editor' &&
        cmd['request']['action'] == 'checkpoint') {
      checkpoints++;
      if (failCheckpoint) throw StateError('Recovery storage unavailable.');
    }
    return super.call(cmd);
  }
}

void main() {
  testWidgets(
    'one settled edit checkpoints once; failed recovery blocks quit and explicit retry preserves source',
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
      expect(await host.prepareQuit(saveFiles: false), false);
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
    'quit review cancellation and failed Save leave the file open; global theme follows cached conversations',
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
      final cancelled = host.requestClose();
      await t.pumpAndSettle();
      await t.tap(find.text('Keep open'));
      await t.pumpAndSettle();
      expect(await cancelled, false);
      bridge.failSave = true;
      final failed = host.requestClose();
      await t.pumpAndSettle();
      await t.tap(find.text('Save files and close'));
      await t.pumpAndSettle();
      expect(await failed, false);
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
