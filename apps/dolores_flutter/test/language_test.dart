import 'dart:async';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/language_host.dart';
import 'package:dolores_flutter/file_host.dart';

class LanguageBridge implements ChatBridge {
  final finished = Completer<void>();
  int version = 0;
  @override Future<void> open() async {}
  @override Future<void> close() async {}
  @override Future<dynamic> call(Map<String,dynamic> v) async {
    final r = v['request'];
    if (r['action']=='feature') {version=r['version'];return {'id':'job'};}
    if (r['action']=='poll') {await finished.future;return {'state':'done','document':'doc','version':version,'result':{'value':'result'}};}
    return null;
  }
}
void main() {
  test('language ranges respect UTF-16 and reject overlapping edits', () {
    expect(LanguageHost.offset('a😀b\n世界',{'line':0,'character':3}),3);
    expect(()=>LanguageHost.offset('a😀b',{'line':0,'character':2}),throwsStateError);
    expect(LanguageHost.apply('a😀b',[{'range':{'start':{'line':0,'character':1},'end':{'line':0,'character':3}},'newText':'世界'}]),'a世界b');
    expect(()=>LanguageHost.apply('abc',[for (var i=0;i<2;i++) {'range':{'start':{'line':0,'character':0},'end':{'line':0,'character':1}},'newText':'x'}]),throwsStateError);
  });
  test('late result after typing or project change cannot enter the editor', () async {
    final b=LanguageBridge(), files=FileHost(LanguageBridge());
    final w=FileWorkspace('p','root','chat');files.workspaces['p']=w;files.selected=w;
    final d=FileDocument({'document':'doc','project':'p','version':0,'text':'before','snapshot':{'path':'a.ts','text':'before','readonly':false}});
    final h=LanguageHost(b);
    final result=h.feature(files,w,d,'hover',{'line':0,'character':0});
    await Future<void>.delayed(Duration.zero);d.text='typed later';b.finished.complete();
    await expectLater(result,throwsStateError);
    expect(d.text,'typed later');files.dispose();d.dispose();
  });
}
