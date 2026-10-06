import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:re_editor/re_editor.dart';

class FileViewMemory {
  CodeLineSelection selection = const CodeLineSelection.collapsed(
    index: 0,
    offset: 0,
  );
  double vertical = 0, horizontal = 0;
}

class FileGroup {
  final String id;
  final tabs = <String>[];
  String? active, preview;
  FileGroup(this.id);
}

class FileLayoutNode {
  String? group;
  Axis axis;
  double ratio;
  FileLayoutNode? first, second;
  FileLayoutNode.leaf(this.group) : axis = Axis.horizontal, ratio = .5;
  FileLayoutNode.split(this.axis, this.first, this.second, {this.ratio = .5});
  Map<String, dynamic> json() => group != null
      ? {'group': group}
      : {
          'axis': axis.name,
          'ratio': ratio,
          'first': first!.json(),
          'second': second!.json(),
        };
}

class FileLayout {
  final groups = <String, FileGroup>{};
  final memories = <String, FileViewMemory>{};
  late FileLayoutNode tree;
  late String activeGroup;
  int _next = 0;
  void Function()? onChanged;
  FileLayout() {
    final g = _group();
    tree = FileLayoutNode.leaf(g.id);
    activeGroup = g.id;
  }
  FileGroup _group() {
    String id;
    do {
      id = 'group-${++_next}';
    } while (groups.containsKey(id));
    final g = FileGroup(id);
    groups[g.id] = g;
    return g;
  }

  FileGroup get active => groups[activeGroup] ?? groups.values.first;
  void changed() => onChanged?.call();
  FileViewMemory memory(String group, String document) =>
      memories.putIfAbsent('$group:$document', () => FileViewMemory());
  void focus(String group) {
    if (activeGroup == group) return;
    activeGroup = group;
    changed();
  }

  void select(String group, String document) {
    final g = groups[group]!;
    if (!g.tabs.contains(document)) return;
    g.active = document;
    activeGroup = group;
    changed();
  }

  void open(String document, {bool preview = true}) {
    final g = active;
    if (!g.tabs.contains(document)) g.tabs.add(document);
    g.active = document;
    if (preview) g.preview = document;
    changed();
  }

  void pin(String group, String document) {
    final g = groups[group]!;
    if (g.preview == document) g.preview = null;
    changed();
  }

  bool visibleElsewhere(String group, String document) =>
      groups.values.any((g) => g.id != group && g.tabs.contains(document));
  void remove(String group, String document) {
    final g = groups[group]!;
    g.tabs.remove(document);
    if (g.preview == document) g.preview = null;
    if (g.active == document) g.active = g.tabs.firstOrNull;
    memories.remove('$group:$document');
    changed();
  }

  void removeDocument(String document) {
    for (final g in groups.values) {
      g.tabs.remove(document);
      if (g.preview == document) g.preview = null;
      if (g.active == document) g.active = g.tabs.firstOrNull;
      memories.remove('${g.id}:$document');
    }
    changed();
  }

  void replaceIdentity(String before, String after) {
    for (final g in groups.values) {
      for (var i = 0; i < g.tabs.length; i++) {
        if (g.tabs[i] == before) g.tabs[i] = after;
      }
      if (g.active == before) g.active = after;
      if (g.preview == before) g.preview = after;
      final old = memories.remove('${g.id}:$before');
      if (old != null) memories['${g.id}:$after'] = old;
    }
  }

  void move(String document, String source, String target, {int? index}) {
    final a = groups[source], b = groups[target];
    if (a == null || b == null || !a.tabs.contains(document)) return;
    final old = a.tabs.indexOf(document);
    a.tabs.remove(document);
    if (a.active == document) a.active = a.tabs.firstOrNull;
    if (a.preview == document) a.preview = null;
    b.tabs.remove(document);
    var at = index ?? b.tabs.length;
    if (source == target && index != null && old < at) at--;
    at = at.clamp(0, b.tabs.length);
    b.tabs.insert(at, document);
    b.active = document;
    b.preview = null;
    activeGroup = target;
    changed();
  }

  bool split(
    String target,
    String document,
    Axis axis, {
    bool before = false,
    String? source,
  }) {
    if (groups.length >= 4 || !groups.containsKey(target)) return false;
    final next = _group()
      ..tabs.add(document)
      ..active = document;
    FileLayoutNode wrap(FileLayoutNode node) {
      if (node.group == target) {
        return FileLayoutNode.split(
          axis,
          before ? FileLayoutNode.leaf(next.id) : node,
          before ? node : FileLayoutNode.leaf(next.id),
        );
      }
      if (node.group == null) {
        node.first = wrap(node.first!);
        node.second = wrap(node.second!);
      }
      return node;
    }

    tree = wrap(tree);
    if (source != null) {
      final old = groups[source]!;
      old.tabs.remove(document);
      if (old.active == document) old.active = old.tabs.firstOrNull;
      if (old.preview == document) old.preview = null;
    }
    activeGroup = next.id;
    changed();
    return true;
  }

  bool closeGroup(String id) {
    if (groups.length <= 1 || !groups.containsKey(id)) return false;
    final removed = groups.remove(id)!;
    final destination = groups.values.first;
    memories.removeWhere((key, _) => key.startsWith('$id:'));
    for (final doc in removed.tabs) {
      if (!destination.tabs.contains(doc)) destination.tabs.add(doc);
    }
    destination.active ??= destination.tabs.firstOrNull;
    FileLayoutNode remove(FileLayoutNode node) {
      if (node.group != null) return node;
      if (node.first!.group == id) return node.second!;
      if (node.second!.group == id) return node.first!;
      node.first = remove(node.first!);
      node.second = remove(node.second!);
      return node;
    }

    tree = remove(tree);
    activeGroup = destination.id;
    changed();
    return true;
  }

  Map<String, dynamic> serialize(Map<String, String> paths) => {
    'version': 1,
    'tree': tree.json(),
    'activeGroup': activeGroup,
    'groups': [
      for (final g in groups.values)
        {
          'id': g.id,
          'paths': g.tabs.map((id) => paths[id]).whereType<String>().toList(),
          'active': paths[g.active],
          'preview': paths[g.preview],
        },
    ],
    'views': [
      for (final g in groups.values)
        for (final id in g.tabs)
          if (paths[id] != null && memories.containsKey('${g.id}:$id'))
            {
              'group': g.id,
              'path': paths[id],
              'baseIndex': memory(g.id, id).selection.baseIndex,
              'baseOffset': memory(g.id, id).selection.baseOffset,
              'extentIndex': memory(g.id, id).selection.extentIndex,
              'extentOffset': memory(g.id, id).selection.extentOffset,
              'vertical': memory(g.id, id).vertical,
              'horizontal': memory(g.id, id).horizontal,
            },
    ],
  };
  static List<String> paths(dynamic raw) {
    validate(raw);
    return (raw['groups'] as List)
        .expand((g) => g['paths'] as List)
        .cast<String>()
        .toSet()
        .toList();
  }

  static void validate(dynamic raw) {
    if (raw is! Map ||
        raw['version'] != 1 ||
        utf8.encode(jsonEncode(raw)).length > 8192) {
      throw const FormatException('Saved layout is invalid.');
    }
    final list = raw['groups'];
    if (list is! List || list.isEmpty || list.length > 4) {
      throw const FormatException('Saved groups are invalid.');
    }
    final ids = <String>{};
    final paths = <String>{};
    for (final g in list) {
      if (g is! Map ||
          g['id'] is! String ||
          (g['id'] as String).length > 64 ||
          !ids.add(g['id'] as String) ||
          g['paths'] is! List ||
          (g['paths'] as List).length > 4) {
        throw const FormatException('Saved tabs are invalid.');
      }
      final local = <String>{};
      for (final path in g['paths'] as List) {
        if (path is! String || path.length > 1024 || !local.add(path)) {
          throw const FormatException('Saved tab path is invalid.');
        }
        paths.add(path);
      }
      if (g['active'] != null && !local.contains(g['active']) ||
          g['preview'] != null && !local.contains(g['preview'])) {
        throw const FormatException('Saved active tab is invalid.');
      }
    }
    if (paths.length > 4 || !ids.contains(raw['activeGroup'])) {
      throw const FormatException('Saved document limit is invalid.');
    }
    final seen = <String>{};
    void node(dynamic n, int depth) {
      if (depth > 3 || n is! Map) {
        throw const FormatException('Saved split tree is invalid.');
      }
      if (n['group'] != null) {
        if (!ids.contains(n['group']) || !seen.add(n['group'] as String)) {
          throw const FormatException('Saved group identity is invalid.');
        }
        return;
      }
      if (!['horizontal', 'vertical'].contains(n['axis']) ||
          n['ratio'] is! num ||
          !(n['ratio'] as num).isFinite ||
          (n['ratio'] as num) < .15 ||
          (n['ratio'] as num) > .85) {
        throw const FormatException('Saved split ratio is invalid.');
      }
      node(n['first'], depth + 1);
      node(n['second'], depth + 1);
    }

    node(raw['tree'], 0);
    if (seen.length != ids.length) {
      throw const FormatException('Saved groups are unreachable.');
    }
    if (raw['views'] != null &&
        (raw['views'] is! List || (raw['views'] as List).length > 16)) {
      throw const FormatException('Saved view state is invalid.');
    }
  }

  void restore(dynamic raw, Map<String, String> documents) {
    validate(raw);
    groups.clear();
    memories.clear();
    _next = 0;
    for (final entry in raw['groups'] as List) {
      final g = FileGroup(entry['id'] as String);
      groups[g.id] = g;
      for (final path in entry['paths'] as List) {
        final id = documents[path];
        if (id != null) g.tabs.add(id);
      }
      g.active = documents[entry['active']] ?? g.tabs.firstOrNull;
      g.preview = documents[entry['preview']];
      _next++;
    }
    FileLayoutNode node(Map n) {
      if (n['group'] != null) return FileLayoutNode.leaf(n['group'] as String);
      return FileLayoutNode.split(
        n['axis'] == 'horizontal' ? Axis.horizontal : Axis.vertical,
        node(n['first'] as Map),
        node(n['second'] as Map),
        ratio: (n['ratio'] as num).toDouble(),
      );
    }

    tree = node(raw['tree'] as Map);
    activeGroup = raw['activeGroup'] as String;
    for (final entry in raw['views'] as List? ?? []) {
      if (entry is! Map ||
          !groups.containsKey(entry['group']) ||
          !documents.containsKey(entry['path'])) {
        continue;
      }
      final fields = ['baseIndex', 'baseOffset', 'extentIndex', 'extentOffset'];
      if (fields.any((f) => entry[f] is! int || (entry[f] as int) < 0)) {
        continue;
      }
      final v = entry['vertical'], h = entry['horizontal'];
      if (v is! num ||
          h is! num ||
          !v.isFinite ||
          !h.isFinite ||
          v < 0 ||
          h < 0 ||
          v > 1e8 ||
          h > 1e8) {
        continue;
      }
      final m = memory(entry['group'] as String, documents[entry['path']]!)
        ..vertical = v.toDouble()
        ..horizontal = h.toDouble();
      m.selection = CodeLineSelection(
        baseIndex: entry['baseIndex'] as int,
        baseOffset: entry['baseOffset'] as int,
        extentIndex: entry['extentIndex'] as int,
        extentOffset: entry['extentOffset'] as int,
      );
    }
    changed();
  }
}
