import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';

class DraftBridge implements ChatBridge {
  final drafts = <String, String>{};
  bool fail = false;
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> input) async {
    switch (input['command']) {
      case 'saveDraft':
        if (fail) throw StateError('Storage unavailable');
        drafts[input['session'] as String] = input['text'] as String;
        return null;
      case 'savedDraft': return drafts[input['session']] ?? '';
      case 'workspace': return {'kind': 'side', 'root': null};
      case 'messagesPage': return {'items': [], 'hasOlder': false, 'hasNewer': false};
      case 'checkpointDraft': return 'Resume the saved task with its evidence.';
      default: throw StateError('Unexpected command');
    }
  }
}

void main() {
  testWidgets('Saved drafts restore and failed persistence keeps usable text', (tester) async {
    final bridge = DraftBridge();
    final chat = ChatController(bridge)..loading = false..durableDrafts = true..session = 'one';
    chat.draft = '世界 draft';
    await tester.pump(const Duration(milliseconds: 300));
    expect(bridge.drafts['one'], '世界 draft');
    chat.dispose();
    final restarted = ChatController(bridge)..loading = false..durableDrafts = true;
    await restarted.select('one');
    expect(restarted.draft, '世界 draft');
    bridge.fail = true;
    restarted.draft = 'Keep this correction';
    await tester.pump(const Duration(milliseconds: 300));
    expect(restarted.draft, 'Keep this correction');
    expect(restarted.error, contains('copy it'));
    bridge.fail = false;
    restarted.draft = 'Retry this correction';
    await tester.pump(const Duration(milliseconds: 300));
    expect(bridge.drafts['one'], 'Retry this correction');
    restarted.dispose();
  });
  testWidgets('Recovery cannot replace an existing draft and remains explicit', (tester) async {
    final chat = ChatController(DraftBridge())..loading = false..durableDrafts = true..session = 'one';
    chat.draft = 'My unsent request';
    await expectLater(chat.prepareCheckpoint('source'), throwsStateError);
    expect(chat.draft, 'My unsent request');
    chat.draft = '';
    await chat.prepareCheckpoint('source');
    expect(chat.resumeRun, 'source');
    expect(chat.busy, isFalse);
    await tester.pump(const Duration(milliseconds: 300));
    chat.dispose();
  });
}
