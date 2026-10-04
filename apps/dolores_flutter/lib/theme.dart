import 'package:flutter/material.dart';

// Shared visual contract: docs/UI.md. Update both when deliberately restyling.
abstract final class UiTokens {
  static const sidebarWidth = 252.0;
  static const sidebarMinWidth = 220.0;
  static const sidebarMaxWidth = 360.0;
  static const conversationMinWidth = 480.0;
  static const drawerBreakpoint = 760.0;
  static const contentWidth = 824.0;
  static const codeFont = 'Consolas';
  static const codeFontFallback = ['Menlo', 'DejaVu Sans Mono', 'monospace'];
}

class Palette {
  final bool dark;
  const Palette(this.dark);
  Color get bg => Color(dark ? 0xff191b20 : 0xfffaf9f7);
  Color get sidebar => Color(dark ? 0xff15171b : 0xfff1f0ed);
  Color get surface => Color(dark ? 0xff22252b : 0xffffffff);
  Color get text => Color(dark ? 0xffe4e6eb : 0xff292b30);
  Color get muted => Color(dark ? 0xffa0a6b2 : 0xff676d76);
  Color get border => Color(dark ? 0xff353941 : 0xffdedfdf);
  Color get accent => Color(dark ? 0xff9cb6ff : 0xff345fca);
  Color get soft => Color(dark ? 0xff2a3552 : 0xffe5ebf8);
  Color get errorSurface => Color(dark ? 0xff3b262b : 0xfffbecec);
  Color get errorText => Color(dark ? 0xffffb5bb : 0xff9c3030);
  Color get syntaxKeyword => Color(dark ? 0xffff9ccc : 0xff963464);
  Color get syntaxName => Color(dark ? 0xffc8a4ff : 0xff6940a5);
  Color get syntaxString => Color(dark ? 0xff93d69b : 0xff28743d);
  Color get syntaxValue => Color(dark ? 0xffffae78 : 0xff985221);
}

ThemeData doloresTheme(bool dark) {
  final p = Palette(dark);
  return ThemeData(
    useMaterial3: true,
    brightness: dark ? Brightness.dark : Brightness.light,
    scaffoldBackgroundColor: p.bg,
    fontFamily: 'Segoe UI',
    colorScheme: ColorScheme.fromSeed(
      seedColor: p.accent,
      brightness: dark ? Brightness.dark : Brightness.light,
      surface: p.surface,
    ),
    textTheme: TextTheme(
      bodyMedium: TextStyle(fontSize: 14, height: 1.55, color: p.text),
    ),
    dividerColor: p.border,
    tooltipTheme: const TooltipThemeData(
      waitDuration: Duration(milliseconds: 600),
    ),
    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: p.surface,
      contentPadding: const EdgeInsets.all(14),
      border: OutlineInputBorder(
        borderRadius: BorderRadius.circular(10),
        borderSide: BorderSide(color: p.border),
      ),
      enabledBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(10),
        borderSide: BorderSide(color: p.border),
      ),
    ),
    textButtonTheme: TextButtonThemeData(
      style: TextButton.styleFrom(
        foregroundColor: p.text,
        textStyle: const TextStyle(fontSize: 14),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
      ),
    ),
  );
}
