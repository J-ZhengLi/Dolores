import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:flutter_test/flutter_test.dart';

import 'workspace_test.dart' show WorkspaceBridge;

class CatalogBridge extends WorkspaceBridge {
  bool failCatalog = false;
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    if (command['command'] == 'bootstrap' && failCatalog) {
      throw StateError('Conversation storage unavailable');
    }
    return super.call(command);
  }
}

void main() {
  test(
    'Refreshing a retained running chat preserves its model and approval',
    () async {
      final bridge = CatalogBridge();
      final project = ChatController(bridge)..loading = false;
      await project.openProject('C:/public-project');
      final host = AppHost(project);
      await host.newConversation(kind: 'side');
      host.visible.draft = 'New side chat';
      await host.visible.checkpointDraft();
      final sideId = host.visible.session;
      project.model = 'retained-model';
      project.requestSettings = {'maxOutputTokens': 333};
      project.contextSummary = {'estimatedTokens': 42};
      project.draft = 'Next message';
      project.busy = true;
      project.partial = 'In progress';
      project.toolApproval = {'callId': 'retained-review'};

      await host.select(project.session!);
      expect(host.visible, same(project));
      expect(project.sessions.map((row) => row['id']), contains(sideId));
      expect(project.model, 'retained-model');
      expect(project.requestSettings['maxOutputTokens'], 333);
      expect(project.contextSummary?['estimatedTokens'], 42);
      expect(project.draft, 'Next message');
      expect(project.busy, isTrue);
      expect(project.partial, 'In progress');
      expect(project.toolApproval?['callId'], 'retained-review');
      expect(bridge.commands.where((c) => c['command'] == 'start'), isEmpty);
      expect(
        bridge.commands.where((c) => c['command'] == 'approveTool'),
        isEmpty,
      );
      project.busy = false;
      host.dispose();
    },
  );

  test(
    'Returning to a retained chat shows conversations created elsewhere',
    () async {
      final bridge = CatalogBridge();
      final project = ChatController(bridge)..loading = false;
      await project.openProject('C:/public-project');
      project.draft = 'Retain the project draft';
      final host = AppHost(project);
      await host.newConversation(kind: 'side');
      final side = host.visible;
      side.draft = 'Retain the side draft';
      await side.checkpointDraft();
      final sideId = side.session!;
      expect(side.sessions.map((row) => row['id']), contains(sideId));

      await host.select(project.session!);
      expect(host.visible, same(project));
      expect(project.sessions.map((row) => row['id']), contains(sideId));
      expect(project.draft, 'Retain the project draft');
      expect(project.workspaceRoot, 'C:/public-project');
      await host.select(sideId);
      expect(host.visible, same(side));
      expect(side.draft, 'Retain the side draft');
      expect(side.workspaceRoot, isNull);
      expect(bridge.commands.where((c) => c['command'] == 'start'), isEmpty);
      host.dispose();
    },
  );

  test(
    'Failed catalog refresh retains both chats and selection; retry recovers',
    () async {
      final bridge = CatalogBridge();
      final project = ChatController(bridge)..loading = false;
      await project.openProject('C:/public-project');
      project.draft = 'Project text';
      final host = AppHost(project);
      await host.newConversation(kind: 'side');
      final side = host.visible;
      side.draft = 'Side text';
      await side.checkpointDraft();
      bridge.failCatalog = true;
      await host.select(project.session!);
      expect(host.visible, same(side));
      expect(host.error, contains('storage unavailable'));
      expect(project.draft, 'Project text');
      expect(side.draft, 'Side text');
      expect(side.sessions.map((row) => row['id']), contains(side.session));
      bridge.failCatalog = false;
      await host.select(project.session!);
      expect(host.visible, same(project));
      expect(project.sessions.map((row) => row['id']), contains(side.session));
      expect(host.error, isNull);
      host.dispose();
    },
  );
}
