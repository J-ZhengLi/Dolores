import 'package:flutter/material.dart';
import 'package:highlight/highlight_core.dart' as syntax;
import 'package:highlight/languages/rust.dart';
import 'package:highlight/languages/dart.dart';
import 'package:highlight/languages/javascript.dart';
import 'package:highlight/languages/typescript.dart';
import 'package:highlight/languages/python.dart';
import 'package:highlight/languages/bash.dart';
import 'package:highlight/languages/json.dart';
import 'package:highlight/languages/go.dart';
import 'package:highlight/languages/cpp.dart';

import 'theme.dart';

const codeLanguages = {
  '': 'Plain text',
  'rust': 'Rust',
  'dart': 'Dart',
  'javascript': 'JavaScript',
  'typescript': 'TypeScript',
  'python': 'Python',
  'bash': 'Shell',
  'json': 'JSON',
  'go': 'Go',
  'cpp': 'C++',
};
String codeLanguage(String id) => switch (id.toLowerCase()) {
  'rs' => 'rust',
  'js' => 'javascript',
  'ts' => 'typescript',
  'py' => 'python',
  'sh' || 'shell' => 'bash',
  'c++' || 'c' => 'cpp',
  final v => v,
};
bool _registered = false;

class CodeSyntaxController extends TextEditingController {
  CodeSyntaxController({super.text, String language = ''})
    : _selectedLanguage = language;
  String _selectedLanguage;
  String get language => _selectedLanguage;
  set language(String next) {
    if (_selectedLanguage == next) return;
    _selectedLanguage = next;
    notifyListeners();
  }

  TextSpan? _cached;
  String? _text, _language;
  TextStyle? _style;
  bool? _dark;
  @override
  TextSpan buildTextSpan({
    required BuildContext context,
    TextStyle? style,
    required bool withComposing,
  }) {
    if (withComposing &&
        value.isComposingRangeValid &&
        !value.composing.isCollapsed) {
      return super.buildTextSpan(
        context: context,
        style: style,
        withComposing: withComposing,
      );
    }
    if (!_registered) {
      final languages = {
        'rust': rust,
        'dart': dart,
        'javascript': javascript,
        'typescript': typescript,
        'python': python,
        'bash': bash,
        'json': json,
        'go': go,
        'cpp': cpp,
      };
      languages.forEach(syntax.highlight.registerLanguage);
      _registered = true;
    }
    final dark = Theme.of(context).brightness == Brightness.dark;
    final lang = codeLanguage(language);
    if (_cached != null &&
        _text == text &&
        _language == lang &&
        _style == style &&
        _dark == dark) {
      return _cached!;
    }
    TextSpan result;
    if (text.length > 8192 ||
        '\n'.allMatches(text).length > 200 ||
        !codeLanguages.containsKey(lang) ||
        lang.isEmpty) {
      result = TextSpan(text: text, style: style);
    } else {
      final p = Palette(dark);
      var count = 0;
      TextSpan convert(syntax.Node node) {
        if (++count > 2048) throw StateError('Syntax span limit');
        final color = switch (node.className) {
          'keyword' || 'literal' => p.syntaxKeyword,
          'title' || 'type' => p.syntaxName,
          'string' || 'regexp' => p.syntaxString,
          'number' || 'built_in' || 'attr' => p.syntaxValue,
          'comment' || 'meta' => p.muted,
          _ => null,
        };
        return TextSpan(
          text: node.value,
          style: color == null ? null : TextStyle(color: color),
          children: node.children?.map(convert).toList(),
        );
      }

      try {
        result = TextSpan(
          style: style,
          children: syntax.highlight
              .parse(text, language: lang)
              .nodes
              ?.map(convert)
              .toList(),
        );
        if (result.toPlainText() != text) {
          result = TextSpan(text: text, style: style);
        }
      } catch (_) {
        result = TextSpan(text: text, style: style);
      }
    }
    _text = text;
    _language = lang;
    _style = style;
    _dark = dark;
    return _cached = result;
  }
}
