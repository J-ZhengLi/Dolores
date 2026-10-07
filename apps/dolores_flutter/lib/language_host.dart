import 'dart:async';

import 'bridge.dart';
import 'file_host.dart';

/// No process starts until the user requests a language feature.
class LanguageHost {
  final ChatBridge bridge;
  int generation = 0;
  bool used = false;
  LanguageHost(this.bridge);
  Future<dynamic> call(Map<String, dynamic> request) =>
      bridge.call({'command': 'language', 'request': request});
  Future<void> stop() async {
    generation++;
    if (!used) return;
    await call({'action': 'stop'});
    used = false;
  }

  Future<dynamic> feature(
    FileHost files,
    FileWorkspace w,
    FileDocument d,
    String feature,
    Map<String, int> position, {
    String name = '',
  }) async {
    await files.flush(d);
    if (d.closed || d.blocked || d.text != d.acknowledged) {
      throw StateError('Retry editor sync before using language features.');
    }
    final token = generation, version = d.version, text = d.text;
    used = true;
    final start = await call({
      'action': 'feature',
      'session': w.session,
      'document': d.id,
      'version': version,
      'feature': feature,
      'position': position,
      'name': name,
    });
    final until = DateTime.now().add(const Duration(seconds: 20));
    while (DateTime.now().isBefore(until)) {
      if (generation != token) throw StateError('Language request canceled.');
      final v = await call({'action': 'poll', 'id': start['id']});
      if (v['state'] == 'done') {
        if (d.closed ||
            d.version != version ||
            d.text != text ||
            files.selected != w ||
            v['document'] != d.id ||
            v['version'] != version) {
          throw StateError(
            'Document or project changed. Request the feature again; edits remain.',
          );
        }
        return v['result'];
      }
      await Future<void>.delayed(const Duration(milliseconds: 100));
    }
    await stop();
    throw StateError(
      'Language service timed out. Services stopped; request the feature again.',
    );
  }

  static int offset(String text, Map position) {
    final line = position['line'] as int, column = position['character'] as int;
    final lines = text.split('\n');
    if (line < 0 ||
        line >= lines.length ||
        column < 0 ||
        column > lines[line].length) {
      throw StateError('Language range is outside the file.');
    }
    if (column > 0 &&
        column < lines[line].length &&
        (lines[line].codeUnitAt(column) & 0xfc00) == 0xdc00) {
      throw StateError('Language range splits Unicode.');
    }
    return lines.take(line).fold<int>(0, (n, l) => n + l.length + 1) + column;
  }

  static String apply(String text, List edits) {
    if (edits.length > 1000) {
      throw StateError(
        'Language result exceeds 1,000 edits. Narrow the change.',
      );
    }
    final ranges = [
      for (final e in edits)
        (
          offset(text, e['range']['start']),
          offset(text, e['range']['end']),
          e['newText'] as String,
        ),
    ];
    ranges.sort((a, b) => a.$1.compareTo(b.$1));
    var last = 0;
    final out = StringBuffer();
    for (final e in ranges) {
      if (e.$1 < last || e.$2 < e.$1) {
        throw StateError('Language edits overlap.');
      }
      out.write(text.substring(last, e.$1));
      out.write(e.$3);
      last = e.$2;
    }
    out.write(text.substring(last));
    return out.toString();
  }
}
