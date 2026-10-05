import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/desktop_share.dart';

class ShareBridge implements ChatBridge {
  final requests = <Map<String, dynamic>>[];
  bool vision = true;
  bool failedCheck = false;
  String operation = '';
  static const target = {'title': 'Local form', 'handle': 17};
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> value) async {
    requests.add(Map.of(value));
    switch (value['command']) {
      case 'desktopState':
        return {
          'available': true,
          'models': vision ? ['fixture'] : [],
        };
      case 'desktopObserve':
        operation = value['target'] == null ? 'list' : 'capture';
        return {};
      case 'checkImageSupport':
        operation = 'check';
        return {};
      case 'poll':
        return [
          {
            'type': 'done',
            if (operation == 'check' && failedCheck) 'error': 'This model did not pass the image check. Choose another model here.',
            if (operation != 'check')
              'observation': operation == 'list'
                  ? {
                      'windows': [target],
                    }
                  : {
                      'id': 'capture',
                      'observation': {'target': target},
                    },
            if (operation == 'check' && !failedCheck) 'imageSupport': true,
          },
        ];
      case 'desktopGrant':
        return {'token': 'grant'};
      case 'desktopRevoke':
        return {};
      case 'start':
        throw StateError('Saved task changed. Return to the latest request.');
    }
    throw StateError('Unexpected ${value['command']}');
  }
}

void main() {
  ChatController controller(ShareBridge bridge) => ChatController(bridge)
    ..loading = false
    ..configured = true
    ..session = 'chat'
    ..workspaceRoot = 'fixture'
    ..model = 'fixture'
    ..enabledModels = ['fixture'];
  testWidgets(
    'local choice has one sharing action and closing does not upload or grant',
    (tester) async {
      final bridge = ShareBridge(), chat = controller(ShareBridge());
      addTearDown(chat.dispose);
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => TextButton(
              onPressed: () => showDialog<void>(
                context: context,
                builder: (_) => WindowSharing(
                  chat: ChatController(bridge)
                    ..session = 'chat'
                    ..model = 'fixture'
                    ..enabledModels = ['fixture'],
                ),
              ),
              child: const Text('Open'),
            ),
          ),
        ),
      );
      await tester.tap(find.text('Open'));
      await tester.pumpAndSettle();
      expect(find.text('Local form'), findsOneWidget);
      expect(find.byKey(const Key('confirm-share-window')), findsOneWidget);
      await tester.tap(find.text('Close'));
      await tester.pumpAndSettle();
      expect(
        bridge.requests.where(
          (r) => [
            'desktopGrant',
            'start',
            'checkImageSupport',
          ].contains(r['command']),
        ),
        isEmpty,
      );
      expect(
        bridge.requests.where(
          (r) => r['command'] == 'desktopObserve' && r['target'] != null,
        ),
        isEmpty,
      );
    },
  );
  testWidgets(
    'failed vision check preserves draft and prevents private capture and input',
    (tester) async {
      final bridge = ShareBridge()
        ..vision = false
        ..failedCheck = true;
      final chat = controller(bridge)..draft = 'Inspect this window';
      addTearDown(chat.dispose);
      tester.view.physicalSize = const Size(420, 480);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: WindowSharing(chat: chat)),
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Local form'));
      await tester.pump();
      await tester.tap(find.byKey(const Key('confirm-share-window')));
      await tester.pumpAndSettle();
      expect(chat.draft, 'Inspect this window');
      expect(find.textContaining('Choose another model here'), findsOneWidget);
      expect(
        bridge.requests.where(
          (r) => r['command'] == 'desktopObserve' && r['target'] != null,
        ),
        isEmpty,
      );
      expect(
        bridge.requests.where(
          (r) => ['desktopGrant', 'start'].contains(r['command']),
        ),
        isEmpty,
      );
      expect(tester.takeException(), isNull);
    },
  );
  testWidgets(
    'handoff sends the original goal and failed start retains an unrelated draft',
    (tester) async {
      final bridge = ShareBridge(), chat = controller(ShareBridge());
      final actual = controller(bridge)..draft = 'Keep this unrelated edit';
      addTearDown(chat.dispose);
      addTearDown(actual.dispose);
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: WindowSharing(
              chat: actual,
              goal: 'Original task',
              run: 'saved-run',
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Local form'));
      await tester.pump();
      await tester.tap(find.byKey(const Key('confirm-share-window')));
      await tester.pumpAndSettle();
      final start = bridge.requests.singleWhere((r) => r['command'] == 'start');
      expect(start['input'], 'Original task');
      expect(start['resumeRun'], 'saved-run');
      expect(start['desktopHandoff'], true);
      expect(actual.draft, 'Keep this unrelated edit');
      expect(find.textContaining('Saved task changed'), findsOneWidget);
      expect(tester.takeException(), isNull);
    },
  );
}
