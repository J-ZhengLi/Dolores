import 'package:dolores_flutter/capabilities.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:dolores_flutter/tool_activity.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'history_test.dart' show HistoryBridge;

class CapabilityBridge extends HistoryBridge {
  bool inspectionFailed = false;
  final calls = <String>[];
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    calls.add(command['command'] as String);
    if (command['command'] == 'harnessInventory') {
      if (inspectionFailed) {
        throw 'Inspection unavailable. Refresh to try again.';
      }
      return {
        'version': 'test',
        'workspace': 'side',
        'model': 'test-model',
        'contextWindowTokens': 131072,
        'contextOrigin': '128K default',
        'approval': 'review each tool call',
        'containment': 'Commands are not sandboxed',
        'selfUpdate': 'not available',
        'limits': {'modelCalls': 4, 'toolOperations': 4},
        'tools': [],
        'unavailableReason': 'No working folder',
        'sources': [
          {'id': 'core'},
        ],
      };
    }
    if (command['command'] == 'harnessSource') {
      return {
        'path': 'core.rs',
        'sourceId': 'matching-build',
        'startLine': 1,
        'nextLine': 4,
        'totalLines': 3,
        'text': '1\tpub trait Tool {}',
      };
    }
    return super.call(command);
  }
}

void main() {
  testWidgets('harness inspection discloses sharing and read-only authority', (
    tester,
  ) async {
    final chat = ChatController(CapabilityBridge())
      ..model = 'test-model'
      ..toolApproval = {
        'name': 'inspect_harness',
        'target': 'running Dolores harness',
        'callId': 'one',
        'query': '{}',
      };
    addTearDown(chat.dispose);
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(body: ToolApprovalCard(chat: chat)),
      ),
    );
    expect(find.text('Inspect the running harness?'), findsOneWidget);
    expect(
      find.textContaining('cannot update Dolores or grant permissions'),
      findsOneWidget,
    );
    expect(find.text('Allow a file read?'), findsNothing);
    expect(find.text('Find this exact text:'), findsNothing);
    expect(toolLabel('inspect_harness'), 'Harness inspection');
  });
  testWidgets(
    'compact local inspection explains side-chat limits and reads matching source',
    (tester) async {
      tester.view.physicalSize = const Size(380, 640);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final bridge = CapabilityBridge();
      final chat = ChatController(bridge)..session = 'side';
      addTearDown(chat.dispose);
      await tester.pumpWidget(
        MaterialApp(
          theme: doloresTheme(true),
          home: Scaffold(body: CapabilitiesInspector(chat: chat)),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('No working folder'), findsOneWidget);
      await tester.ensureVisible(find.byType(DropdownButtonFormField<String>));
      await tester.tap(find.byType(DropdownButtonFormField<String>));
      await tester.pumpAndSettle();
      await tester.tap(find.text('core').last);
      await tester.pumpAndSettle();
      expect(find.text('1\tpub trait Tool {}'), findsOneWidget);
      expect(bridge.calls, ['harnessInventory', 'harnessSource']);
      expect(tester.takeException(), isNull);
    },
  );
  testWidgets(
    'failed local inspection can refresh without a provider request',
    (tester) async {
      final bridge = CapabilityBridge()..inspectionFailed = true;
      final chat = ChatController(bridge);
      addTearDown(chat.dispose);
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: CapabilitiesInspector(chat: chat)),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.textContaining('Refresh to try again'), findsOneWidget);
      bridge.inspectionFailed = false;
      await tester.tap(find.text('Refresh'));
      await tester.pumpAndSettle();
      expect(find.text('No working folder'), findsOneWidget);
      expect(bridge.calls.every((c) => c == 'harnessInventory'), isTrue);
      expect(tester.takeException(), isNull);
    },
  );
}
