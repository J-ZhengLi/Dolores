import 'package:flutter/material.dart';

import 'theme.dart';

class EditDiff extends StatelessWidget {
  final String source;
  final String identity;
  const EditDiff({super.key, required this.source, required this.identity});

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final lines = source.split('\n');
    final colored = lines.length <= 800;
    final spans = colored
        ? [
            for (var i = 0; i < lines.length; i++)
              TextSpan(
                text: '${lines[i]}${i < lines.length - 1 ? '\n' : ''}',
                style: TextStyle(
                  color: lines[i].startsWith('+') && !lines[i].startsWith('+++')
                      ? p.syntaxString
                      : lines[i].startsWith('-') && !lines[i].startsWith('---')
                      ? p.errorText
                      : p.text,
                ),
              ),
          ]
        : <TextSpan>[];
    return Container(
      key: Key('edit-diff-$identity'),
      width: double.infinity,
      padding: const EdgeInsets.all(10),
      decoration: BoxDecoration(
        color: p.sidebar,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: p.border),
      ),
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxHeight: 180),
        child: SingleChildScrollView(
          key: PageStorageKey('edit-diff-vertical-$identity'),
          child: SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            key: PageStorageKey('edit-diff-horizontal-$identity'),
            child: SelectableText.rich(
              TextSpan(
                text: colored ? null : source,
                children: colored ? spans : null,
              ),
              style: TextStyle(
                color: p.text,
                fontSize: 12,
                height: 1.5,
                fontFamily: UiTokens.codeFont,
                fontFamilyFallback: UiTokens.codeFontFallback,
              ),
            ),
          ),
        ),
      ),
    );
  }
}
