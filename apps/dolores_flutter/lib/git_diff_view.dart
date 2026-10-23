import 'package:flutter/material.dart';

/// Virtualized changed sections, with aligned old/new cells and file line numbers.
class GitDiffView extends StatelessWidget {
  final Map<String, dynamic> diff;
  final bool sideBySide;
  final void Function(String side, int line)? onReview;
  final List<Map<String, dynamic>> findings;
  const GitDiffView({
    super.key,
    required this.diff,
    required this.sideBySide,
    this.onReview,
    this.findings = const [],
  });

  Widget reviewLine(Map<String, dynamic>? row, bool old) {
    final line = row?[old ? 'oldLine' : 'newLine'];
    if (onReview == null ||
        line is! int ||
        row?['endLine'] == false ||
        row?['continued'] == true) {
      return const SizedBox.shrink();
    }
    return IconButton(
      key: ValueKey('review-${old ? 'left' : 'right'}-$line'),
      tooltip: 'Comment on ${old ? 'left' : 'right'} line $line',
      visualDensity: VisualDensity.compact,
      constraints: const BoxConstraints(minWidth: 28, minHeight: 28),
      padding: const EdgeInsets.all(4),
      icon: const Icon(Icons.add_comment_outlined, size: 15),
      onPressed: () => onReview!(old ? 'left' : 'right', line),
    );
  }

  Widget notes(Map<String, dynamic>? row, bool old) {
    final line = row?[old ? 'oldLine' : 'newLine'];
    final saved = findings.where(
      (f) =>
          f['anchor']['side'] == (old ? 'left' : 'right') &&
          f['anchor']['line'] == line,
    );
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        for (final f in saved)
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            child: Text(
              'Review: ${f['text']}',
              style: const TextStyle(fontSize: 12),
            ),
          ),
      ],
    );
  }

  List<Map<String, dynamic>> get rows {
    if (diff['rows'] is List) {
      return (diff['rows'] as List)
          .map((v) => Map<String, dynamic>.from(v))
          .toList();
    }
    // Older fixtures/receipts still carry a unified patch.
    var old = 0, next = 0;
    var inHunk = false;
    final result = <Map<String, dynamic>>[];
    for (final text in (diff['patch'] as String? ?? '').split('\n')) {
      if (text.isEmpty) continue;
      final header = RegExp(r'^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@')
          .firstMatch(text);
      if (header != null) {
        old = int.parse(header[1]!);
        next = int.parse(header[2]!);
        inHunk = true;
        result.add({'kind': 'meta', 'text': text});
        continue;
      }
      if (text.startsWith('diff --git ')) inHunk = false;
      final kind = inHunk && text.startsWith('-')
          ? 'remove'
          : inHunk && text.startsWith('+')
          ? 'add'
          : inHunk && text.startsWith(' ')
          ? 'context'
          : 'meta';
      result.add({
        'kind': kind,
        'text': kind == 'meta' ? text : text.substring(1),
        if (kind == 'remove' || kind == 'context') 'oldLine': old++,
        if (kind == 'add' || kind == 'context') 'newLine': next++,
      });
    }
    return result;
  }

  Color? shade(String? kind) => switch (kind) {
    'remove' => Colors.red.withValues(alpha: .18),
    'add' => Colors.green.withValues(alpha: .18),
    _ => null,
  };
  Widget number(dynamic value) => SizedBox(
    width: 48,
    child: Text(
      value?.toString() ?? '',
      textAlign: TextAlign.right,
      style: const TextStyle(
        color: Colors.grey,
        fontFamily: 'Consolas',
        fontSize: 12,
      ),
    ),
  );
  Widget text(String value) => SelectableText(
    value,
    style: const TextStyle(fontFamily: 'Consolas', fontSize: 13, height: 1.5),
  );
  Widget cell(Map<String, dynamic>? row, bool old, int index) => Container(
    key: ValueKey('git-${old ? 'left' : 'right'}-cell-$index'),
    color: shade(row?['kind'] as String?),
    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            reviewLine(row, old),
            number(row?[old ? 'oldLine' : 'newLine']),
            const SizedBox(width: 12),
            Expanded(child: text(row?['text'] as String? ?? '')),
          ],
        ),
        notes(row, old),
      ],
    ),
  );
  List<(Map<String, dynamic>?, Map<String, dynamic>?)> align(
    List<Map<String, dynamic>> rows,
  ) {
    final result = <(Map<String, dynamic>?, Map<String, dynamic>?)>[];
    final removed = <Map<String, dynamic>>[], added = <Map<String, dynamic>>[];
    void flush() {
      final count = removed.length > added.length
          ? removed.length
          : added.length;
      for (var i = 0; i < count; i++) {
        result.add((
          i < removed.length ? removed[i] : null,
          i < added.length ? added[i] : null,
        ));
      }
      removed.clear();
      added.clear();
    }

    for (final row in rows) {
      if (row['kind'] == 'remove') {
        if (added.isNotEmpty) flush();
        removed.add(row);
      } else if (row['kind'] == 'add') {
        added.add(row);
      } else {
        flush();
        result.add((row, row));
      }
    }
    flush();
    return result;
  }

  @override
  Widget build(BuildContext context) {
    if (diff['reason'] != null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: text(diff['reason'] as String),
        ),
      );
    }
    final source = rows;
    final streamed = diff['sideRows'] as List?;
    final pairs = !sideBySide
        ? null
        : streamed == null
        ? align(source)
        : streamed
              .map(
                (pair) => (
                  pair['left'] == null
                      ? null
                      : Map<String, dynamic>.from(pair['left']),
                  pair['right'] == null
                      ? null
                      : Map<String, dynamic>.from(pair['right']),
                ),
              )
              .toList();
    if (source.isEmpty && (pairs == null || pairs.isEmpty)) {
      return const Center(child: Text('No changes between these versions.'));
    }
    return LayoutBuilder(
      builder: (context, constraints) => Scrollbar(
        child: SingleChildScrollView(
          scrollDirection: Axis.horizontal,
          child: SizedBox(
            width: constraints.maxWidth.clamp(600, 1600),
            height: constraints.maxHeight,
            child: Column(
              children: [
                if (sideBySide)
                  Row(
                    children: [
                      Expanded(
                        child: Padding(
                          padding: const EdgeInsets.all(8),
                          child: Text(diff['leftLabel'] as String),
                        ),
                      ),
                      Expanded(
                        child: Padding(
                          padding: const EdgeInsets.all(8),
                          child: Text(diff['rightLabel'] as String),
                        ),
                      ),
                    ],
                  ),
                Expanded(
                  child: ListView.builder(
                    itemCount: pairs?.length ?? source.length,
                    itemBuilder: (context, index) {
                      if (pairs != null) {
                        final (old, next) = pairs[index];
                        if (old?['kind'] == 'meta') {
                          return Padding(
                            padding: const EdgeInsets.symmetric(
                              horizontal: 12,
                              vertical: 2,
                            ),
                            child: text(old!['text'] as String),
                          );
                        }
                        return IntrinsicHeight(
                          child: Row(
                            crossAxisAlignment: CrossAxisAlignment.stretch,
                            children: [
                              Expanded(child: cell(old, true, index)),
                              const VerticalDivider(width: 1),
                              Expanded(child: cell(next, false, index)),
                            ],
                          ),
                        );
                      }
                      final row = source[index];
                      return Container(
                        color: shade(row['kind'] as String?),
                        padding: const EdgeInsets.symmetric(
                          horizontal: 8,
                          vertical: 2,
                        ),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Row(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                reviewLine(row, row['newLine'] == null),
                                number(row['oldLine']),
                                number(row['newLine']),
                                const SizedBox(width: 12),
                                Text(
                                  row['kind'] == 'add'
                                      ? '+'
                                      : row['kind'] == 'remove'
                                      ? '−'
                                      : ' ',
                                  style: const TextStyle(
                                    fontFamily: 'Consolas',
                                    fontSize: 13,
                                  ),
                                ),
                                const SizedBox(width: 8),
                                Expanded(child: text(row['text'] as String)),
                              ],
                            ),
                            notes(row, true),
                            notes(row, false),
                          ],
                        ),
                      );
                    },
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
