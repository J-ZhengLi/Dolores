import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:re_editor/re_editor.dart';
import 'package:dolores_flutter/file_layout.dart';
import 'package:dolores_flutter/file_host.dart';

import 'file_host_test.dart' show FileBridge;
import 'editor_test.dart' show EditingBridge;

void main() {
  test('reorder, move, four-group bound and closing splits keep document references', () {
    final l = FileLayout();
    final first = l.activeGroup;
    l.open('a', preview: false);
    l.open('b', preview: false);
    l.open('c', preview: false);
    l.move('c', first, first, index: 0);
    expect(l.active.tabs, ['c', 'a', 'b']);
    expect(l.split(first, 'a', Axis.horizontal), true);
    final second = l.activeGroup;
    l.move('b', first, second);
    expect(l.groups[first]!.tabs, ['c', 'a']);
    expect(l.active.tabs, ['a', 'b']);
    l.split(first, 'c', Axis.vertical);
    l.split(second, 'b', Axis.vertical);
    final saved = jsonEncode(
      l.serialize({'a': 'a.txt', 'b': 'b.txt', 'c': 'c.txt'}),
    );
    expect(l.split(first, 'a', Axis.horizontal), false);
    expect(
      jsonEncode(l.serialize({'a': 'a.txt', 'b': 'b.txt', 'c': 'c.txt'})),
      saved,
    );
    for (final id in l.groups.keys.skip(1).toList()) {
      expect(l.closeGroup(id), true);
    }
    expect(l.groups.values.single.tabs.toSet(), {'a', 'b', 'c'});
    expect(l.closeGroup(l.activeGroup), false);
  });
  test('layout restores independent selection and rejects corrupt trees before paths are read', () {
    final l = FileLayout()..open('a');
    final first = l.activeGroup;
    l.split(first, 'a', Axis.horizontal);
    final second = l.activeGroup;
    l.memory(first, 'a').selection = const CodeLineSelection.collapsed(
      index: 2,
      offset: 3,
    );
    l.memory(second, 'a').vertical = 48;
    final data = l.serialize({'a': '世界.txt'});
    final restored = FileLayout()..restore(data, {'世界.txt': 'new-id'});
    expect(restored.memory(first, 'new-id').selection.baseIndex, 2);
    expect(restored.memory(second, 'new-id').vertical, 48);
    expect(restored.memory(first, 'new-id').vertical, 0);
    final corrupt = jsonDecode(jsonEncode(data)) as Map<String, dynamic>;
    corrupt['tree']['ratio'] = 3;
    expect(() => FileLayout.paths(corrupt), throwsFormatException);
    corrupt['tree']['ratio'] = .5;
    corrupt['tree']['second']['group'] = first;
    expect(() => FileLayout.paths(corrupt), throwsFormatException);
  });
  test('failed layout save retains draft, retry does not clear unrelated errors, restart restores groups', () async {
    final bridge = EditingBridge();
    final files = FileHost(bridge);
    await files.bind('A', 'C:/A');
    final w = files.selected!;
    final d = (await files.open(w, 'a.txt'))!;
    w.layoutOwner.pin(w.layoutOwner.activeGroup, d.id);
    w.layoutOwner.split(w.layoutOwner.activeGroup, d.id, Axis.horizontal);
    final view = d.buffer.createView();
    view.replaceSelection('mine ');
    await files.flush(d);
    bridge.failLayout = true;
    await files.persistLayout(w);
    expect(w.layoutError, contains('retained'));
    expect(d.text, 'mine saved');
    files.error = 'Unrelated recovery warning';
    bridge.failLayout = false;
    await files.persistLayout(w);
    expect(w.layoutError, isNull);
    expect(files.error, 'Unrelated recovery warning');
    view.dispose();
    files.dispose();
    final restarted = FileHost(bridge);
    await restarted.bind('A', 'C:/A');
    expect(restarted.selected!.layoutOwner.groups.length, 2);
    expect(restarted.active!.text, 'mine saved');
    restarted.dispose();
  });
  test('corrupt layout never opens stored paths and A B A restores an evicted clean document', () async {
    final bridge = FileBridge()..layouts['bad'] = {'version': 1, 'groups': []};
    final files = FileHost(bridge);
    await files.bind('bad', 'C:/bad');
    expect(files.error, contains('safe single group'));
    expect(bridge.editorCalls.where((r) => r['action'] == 'open'), isEmpty);
    await files.bind('A', 'C:/A');
    final a = files.selected!;
    final old = (await files.open(a, 'a.txt'))!;
    for (var n = 0; n < 4; n++) {
      final name = 'B$n';
      await files.bind(name, 'C:/$name');
      await files.open(files.selected!, 'b.txt');
    }
    expect(old.closed, true);
    expect(a.paths.values, contains('a.txt'));
    await files.bind('A', 'C:/A');
    expect(files.active!.path, 'a.txt');
    expect(files.documents.length, 4);
    files.dispose();
  });
}
