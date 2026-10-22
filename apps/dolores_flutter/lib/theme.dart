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

AnimationStyle surfaceAnimation(BuildContext context) =>
    MediaQuery.disableAnimationsOf(context) ||
        MediaQuery.accessibleNavigationOf(context)
    ? AnimationStyle.noAnimation
    : const AnimationStyle(
        duration: Duration(milliseconds: 200),
        reverseDuration: Duration(milliseconds: 140),
        curve: Curves.easeOutCubic,
      );

class Palette {
  final bool dark;
  const Palette(this.dark);
  Color get bg => Color(dark ? 0xff1b1b1b : 0xfff5f8fc);
  Color get sidebar => Color(dark ? 0xff222222 : 0xffe5edf8);
  Color get surface => Color(dark ? 0xff262626 : 0xffffffff);
  Color get text => Color(dark ? 0xffefefef : 0xff253044);
  Color get muted => Color(dark ? 0xffa6a6a6 : 0xff68778b);
  Color get border => Color(dark ? 0xff3b3b3b : 0xffdce4ef);
  Color get accent => Color(dark ? 0xffe2e2e2 : 0xff35475f);
  Color get soft => Color(dark ? 0xff353535 : 0xffe4edf8);
  BoxDecoration get sidebarDecoration => BoxDecoration(
    gradient: LinearGradient(
      begin: Alignment.topLeft,
      end: Alignment.bottomRight,
      colors: dark
          ? [sidebar, sidebar]
          : const [Color(0xffedf4fd), Color(0xffe0eafa), Color(0xffe8f0fb)],
    ),
  );
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
    colorScheme:
        ColorScheme.fromSeed(
          seedColor: p.accent,
          brightness: dark ? Brightness.dark : Brightness.light,
          surface: p.surface,
        ).copyWith(
          primary: p.accent,
          onPrimary: p.bg,
          secondary: p.accent,
          onSecondary: p.bg,
          primaryContainer: p.soft,
          secondaryContainer: p.soft,
          onPrimaryContainer: p.text,
          onSecondaryContainer: p.text,
          onSurface: p.text,
          onSurfaceVariant: p.muted,
          outline: p.muted,
          outlineVariant: p.border,
          surfaceContainerHighest: p.soft,
        ),
    textTheme: TextTheme(
      bodyMedium: TextStyle(fontSize: 14, height: 1.55, color: p.text),
    ),
    dividerColor: p.border,
    dialogTheme: DialogThemeData(
      backgroundColor: p.surface,
      surfaceTintColor: Colors.transparent,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(22)),
    ),
    popupMenuTheme: PopupMenuThemeData(
      color: p.surface,
      surfaceTintColor: Colors.transparent,
      elevation: 8,
      position: PopupMenuPosition.under,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(14),
        side: BorderSide(color: p.border),
      ),
      textStyle: TextStyle(fontSize: 13, color: p.text),
    ),
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
        textStyle: const TextStyle(fontFamily: 'Segoe UI', fontSize: 14),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
      ),
    ),
  );
}
