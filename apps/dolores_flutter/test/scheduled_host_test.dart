import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/scheduled_host.dart';
import 'workspace_test.dart' show WorkspaceBridge;

class ScheduleBridge extends WorkspaceBridge {
  bool paused = false, fail = false, claimed = false;
  Map get listing => {'items': [
    {'task': {'id':'task', 'paused':paused, 'nextDue':1791385200}, 'occurrences': []}
  ]};
  @override Future<dynamic> call(Map<String,dynamic> command) async {
    final name=command['command'];
    if (name.toString().startsWith('scheduled')) commands.add(command);
    if(name=='scheduledTasks') { if(fail) throw StateError('Storage unavailable'); return listing; }
    if(name=='scheduledTick') {
      final fresh=!claimed; claimed=true;
      return {'tasks':listing, 'claimed': fresh ? [
        {'id':'occurrence', 'snapshot':{'prompt':'Write a report every day at 9pm'}}
      ] : []};
    }
    if(name=='scheduledStart') return {'session':'result', 'model':'pinned-model'};
    if(name=='scheduledAbandon') return null;
    if(name=='poll') return <Map>[];
    return super.call(command);
  }
}
void main() {
  test('background task keeps Home project model and draft', () async {
    final bridge=ScheduleBridge();
    final chat=ChatController(bridge)..loading=false..session='home'
      ..workspaceRoot='C:/home-project'..model='home-model'..draft='my draft';
    final host=AppHost(chat);
    await Future<void>.delayed(Duration.zero);
    await host.scheduled.tick();
    final owner=host.owners.last;
    expect(owner.session,'result'); expect(owner.busy,true);
    expect(host.visible,same(chat));expect(chat.draft,'my draft');expect(chat.model,'home-model');expect(host.projectRoot,'C:/home-project');
    expect(bridge.commands.where((c)=>c['command']=='start'),isEmpty);
    await owner.stop();expect(bridge.commands.lastWhere((c)=>c['command']=='cancel')['id'],bridge.commands.firstWhere((c)=>c['command']=='scheduledStart')['id']);
    host.dispose();
  });
  test('paused schedules have no idle clock; failed refresh keeps prior list',() async {
    final bridge=ScheduleBridge()..paused=true;
    final host=ScheduledHost(bridge,onClaim:(_)async{});
    await host.refresh();expect(host.needsClock,false);expect(host.items.length,1);
    bridge.fail=true;await host.refresh();expect(host.items.length,1);expect(host.error,contains('unavailable'));host.dispose();
  });
  test('launch failure records recovery rather than retrying the same claim',() async {
    final bridge=ScheduleBridge();var attempts=0;
    final host=ScheduledHost(bridge,onClaim:(_)async{attempts++;throw StateError('Owner limit');});
    await host.tick();await host.tick();expect(attempts,1);
    expect(bridge.commands.where((c)=>c['command']=='scheduledAbandon').length,1);host.dispose();
  });
}
