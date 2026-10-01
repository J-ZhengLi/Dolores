import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';

class HistoryBridge implements ChatBridge {
  final commands = <Map<String, dynamic>>[];
  bool fail = false;
  List<Map<String, dynamic>> sessionItems(int start) => List.generate(
    50,
    (i) => {
      'id': 'chat-${start + i}',
      'title': 'Chat ${start + i}',
      'updatedAt': 1000 - start - i,
    },
  );
  List<Map<String, dynamic>> messageItems(int end) => List.generate(
    80,
    (i) => {
      'id': end - 79 + i,
      'role': i.isEven ? 'user' : 'assistant',
      'content': 'Message ${end - 79 + i}',
    },
  );
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    commands.add(command);
    if (fail) throw StateError('Storage unavailable.');
    switch (command['command']) {
      case 'bootstrap':
        return {
          'sessions': sessionItems(0),
          'sessionPage': {'hasOlder': true, 'hasNewer': false},
          'preferences': {'baseUrl': 'http://localhost/v1', 'model': 'fixture'},
          'configured': true,
        };
      case 'sessionsPage':
        final newer = command['newer'] == true;
        return {
          'items': sessionItems(newer ? 0 : 50),
          'hasOlder': newer,
          'hasNewer': !newer,
        };
      case 'messagesPage':
        final cursor = command['cursor'] as int?;
        final end = cursor == null || command['newer'] == true
            ? 160
            : cursor - 1;
        return {
          'items': messageItems(end),
          'hasOlder': end > 80,
          'hasNewer': end < 160,
        };
      case 'export':
        return {'messageCount': 160};
      case 'start':
        return null;
      case 'poll':
        return [];
    }
    return null;
  }
}

void main() {
  test(
    'Pages replace memory, view state restores, failures keep the current page',
    () async {
      final bridge = HistoryBridge();
      final chat = ChatController(bridge);
      await chat.initialize();
      expect(chat.messages.length, 80);
      expect(chat.messages.first['id'], 81);
      await chat.browseSessions(newer: false);
      expect(chat.sessions.length, 50);
      expect(chat.sessions.first['id'], 'chat-50');
      await chat.browseSessions(newer: true);
      expect(chat.sessions.first['id'], 'chat-0');
      await chat.browseMessages(newer: false);
      expect(chat.messages.first['id'], 1);
      expect(chat.messagesNewer, true);
      chat.draft = 'Unsent first draft';
      chat.rememberScroll(120);
      await chat.select('chat-1');
      expect(chat.draft, '');
      chat.draft = 'Second draft';
      await chat.select('chat-0');
      expect(chat.draft, 'Unsent first draft');
      expect(chat.scrollOffset, 120);
      expect(chat.messages.first['id'], 1);
      bridge.fail = true;
      await chat.browseMessages(newer: true);
      expect(chat.messages.first['id'], 1);
      expect(chat.error, contains('Storage unavailable'));
      bridge.fail = false;
      await chat.browseMessages(newer: false, latest: true);
      expect(chat.messages.last['id'], 160);
      expect(chat.messagesNewer, false);
      expect(chat.draft, 'Unsent first draft');
      chat.newChat();
      expect(chat.messages, isEmpty);
      expect(chat.draft, '');
      chat.dispose();
    },
  );

  test('Export cancellation is inert and export/browsing are excluded while generating', () async {
    final bridge = HistoryBridge();
    final chat = ChatController(bridge);
    await chat.initialize();
    chat.draft = 'Keep this';
    expect(await chat.exportConversation('json', () async => null), null);
    expect(bridge.commands.where((c) => c['command'] == 'export'), isEmpty);
    expect(
      await chat.exportConversation('json', () async => 'D:/chosen/chat.json'),
      160,
    );
    expect(bridge.commands.last['path'], 'D:/chosen/chat.json');
    expect(chat.draft, 'Keep this');
    bridge.fail = true;
    expect(
      await chat.exportConversation(
        'markdown',
        () async => 'D:/chosen/chat.md',
      ),
      null,
    );
    expect(chat.error, contains('Storage unavailable'));
    bridge.fail = false;
    final count = bridge.commands.length;
    chat.busy = true;
    await chat.exportConversation(
      'json',
      () => throw StateError('Dialog must not open'),
    );
    await chat.browseSessions(newer: false);
    await chat.browseMessages(newer: false);
    expect(bridge.commands.length, count);
    chat.dispose();
  });

  testWidgets(
    'History controls use existing theme in wide and compact layouts',
    (tester) async {
      tester.view.physicalSize = const Size(1120, 780);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final chat = ChatController(HistoryBridge());
      await chat.initialize();
      await tester.pumpWidget(DoloresApp(chat: chat));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('older-chats')));
      await tester.pumpAndSettle();
      expect(chat.sessions.first['id'], 'chat-50');
      await tester.ensureVisible(find.byKey(const Key('older-messages')));
      await tester.tap(find.byKey(const Key('older-messages')));
      await tester.pumpAndSettle();
      expect(find.text('Earlier messages'), findsOneWidget);
      expect(chat.messages.first['id'], 1);
      await tester.tap(find.byKey(const Key('export-chat')));
      await tester.pumpAndSettle();
      expect(find.text('Export Markdown'), findsOneWidget);
      expect(find.text('Export JSON'), findsOneWidget);
      await tester.tapAt(const Offset(300, 400));
      await tester.pumpAndSettle();
      tester.view.physicalSize = const Size(620, 700);
      await tester.pumpWidget(
        DoloresApp(chat: chat, themeMode: ThemeMode.dark),
      );
      await tester.pumpAndSettle();
      expect(tester.takeException(), null);
      expect(find.byTooltip('Conversations'), findsOneWidget);
      chat.busy = true;
      await tester.pumpWidget(
        DoloresApp(chat: chat, themeMode: ThemeMode.dark),
      );
      expect(
        tester
            .widget<PopupMenuButton<String>>(
              find.byKey(const Key('export-chat')),
            )
            .enabled,
        false,
      );
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
}
