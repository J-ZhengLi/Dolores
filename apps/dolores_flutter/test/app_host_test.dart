import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/chat.dart';
import 'workspace_test.dart' show WorkspaceBridge;

class HostBridge extends WorkspaceBridge {
  int closed=0;
  @override Future<void> close() async { closed++; }
  @override Future<dynamic> call(Map<String,dynamic> command) async {
    if(command['command']=='workspace' && !roots.containsKey(command['session'])) {
      throw StateError('Conversation is unavailable.');
    }
    if(command['command']=='poll') return <Map<String,dynamic>>[];
    return super.call(command);
  }
}

void main() {
  test('switching conversations retains running owner draft and project without reopening the bridge', () async {
    final bridge = HostBridge();
    bridge.roots['A'] = {'kind':'project','root':'C:/project-A'};
    bridge.roots['B'] = {'kind':'project','root':'C:/project-B'};
    final first = ChatController(bridge)..loading=false..session='A'
      ..workspaceRoot='C:/project-A'..workspaceKind='project'..busy=true..draft='retained A';
    final host = AppHost(first);
    await host.select('B');
    expect(host.visible.session,'B');
    expect(host.projectRoot,'C:/project-B');
    expect(first.busy,true); expect(first.draft,'retained A');
    host.visible.draft='retained B';
    await host.select('A');
    expect(identical(host.visible, first),true);
    await host.select('B'); expect(host.visible.draft,'retained B');
    host.dispose();
    expect(bridge.closed,1);
  });
  test('failed selection retains the current project and an unbound chat clears it', () async {
    final bridge=HostBridge();
    bridge.roots['side']={'kind':'side','root':null};
    final first=ChatController(bridge)..loading=false..session='A'
      ..workspaceRoot='C:/project-A'..workspaceKind='project'..draft='usable work';
    final host=AppHost(first);
    await host.select('side');
    expect(host.projectRoot,isNull);
    final before=host.visible;
    await host.select('missing');
    expect(identical(host.visible,before),true);
    expect(host.error,contains('unavailable'));
    expect(first.draft,'usable work');
    expect(host.tasks,isEmpty);
    host.dispose();
  });
  test('two conversation owners use distinct run IDs and Stop targets only its owner', () async {
    final bridge=HostBridge();
    bridge.roots['A']={'kind':'project','root':'C:/A'};
    bridge.roots['B']={'kind':'project','root':'C:/B'};
    final first=ChatController(bridge)..loading=false..configured=true..session='A'
      ..workspaceRoot='C:/A'..workspaceKind='project'..draft='A request';
    final host=AppHost(first);
    await first.send();
    await host.select('B');
    host.visible.draft='B request';
    await host.visible.send();
    final starts=bridge.commands.where((c)=>c['command']=='start').toList();
    expect(starts.length,2); expect(starts[0]['id'],isNot(starts[1]['id']));
    await first.stop();
    expect(bridge.commands.lastWhere((c)=>c['command']=='cancel')['id'],starts.first['id']);
    expect(host.visible.busy,true);
    host.dispose();
  });
}
