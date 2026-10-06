import 'package:flutter/material.dart';

/// A bounded read-only comparison. Saved/index/commit bases stay explicit.
class GitDiffView extends StatelessWidget {
  final Map<String, dynamic> diff;
  final bool sideBySide;
  const GitDiffView({super.key, required this.diff, required this.sideBySide});
  Widget lines(String value, {bool patch = false}) {
    final rows = value.split('\n');
    return LayoutBuilder(
      builder: (context, c) => Scrollbar(
        child: SingleChildScrollView(
          scrollDirection: Axis.horizontal,
          child: SizedBox(
            width: c.maxWidth.clamp(600, 1400),
            child: ListView.builder(
              itemCount: rows.length,
              itemBuilder: (context, i) {
                final row = rows[i];
                final added =
                    patch && row.startsWith('+') && !row.startsWith('+++');
                final removed =
                    patch && row.startsWith('-') && !row.startsWith('---');
                return Container(
                  color: added
                      ? Colors.green.withValues(alpha: .12)
                      : removed
                      ? Colors.red.withValues(alpha: .12)
                      : null,
                  padding: const EdgeInsets.symmetric(
                    horizontal: 12,
                    vertical: 2,
                  ),
                  child: Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      SizedBox(
                        width: 44,
                        child: Text(
                          '${i + 1}',
                          style: const TextStyle(
                            color: Colors.grey,
                            fontFamily: 'Consolas',
                            fontSize: 12,
                          ),
                        ),
                      ),
                      Expanded(
                        child: SelectableText(
                          row,
                          style: const TextStyle(
                            fontFamily: 'Consolas',
                            fontSize: 13,
                          ),
                        ),
                      ),
                    ],
                  ),
                );
              },
            ),
          ),
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    if (diff['reason'] != null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: SelectableText(diff['reason'] as String),
        ),
      );
    }
    if (!sideBySide) {
      final patch = diff['patch'] as String;
      return lines(
        patch.isNotEmpty ? patch : diff['right'] as String,
        patch: patch.isNotEmpty,
      );
    }
    return Row(
      children: [
        Expanded(
          child: Column(
            children: [
              Padding(
                padding: const EdgeInsets.all(8),
                child: Text(diff['leftLabel'] as String),
              ),
              Expanded(child: lines(diff['left'] as String)),
            ],
          ),
        ),
        const VerticalDivider(width: 1),
        Expanded(
          child: Column(
            children: [
              Padding(
                padding: const EdgeInsets.all(8),
                child: Text(diff['rightLabel'] as String),
              ),
              Expanded(child: lines(diff['right'] as String)),
            ],
          ),
        ),
      ],
    );
  }
}
