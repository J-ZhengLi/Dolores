import 'package:flutter/material.dart';

// Semantic tokens implement docs/UI.md guidance. Feature workflows live in docs/design/.
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
  Color get bg => Color(dark ? 0xff1b1b1b : 0xfffafafa);
  Color get sidebar => Color(dark ? 0xff222222 : 0xfff3f3f2);
  Color get surface => Color(dark ? 0xff262626 : 0xffffffff);
  Color get text => Color(dark ? 0xffefefef : 0xff292929);
  Color get muted => Color(dark ? 0xffa6a6a6 : 0xff70706e);
  Color get border => Color(dark ? 0xff3b3b3b : 0xffe3e3e0);
  Color get accent => Color(dark ? 0xffe2e2e2 : 0xff454744);
  Color get soft => Color(dark ? 0xff353535 : 0xffeeefec);
  BoxDecoration get sidebarDecoration => BoxDecoration(
    gradient: LinearGradient(
      begin: Alignment.topLeft,
      end: Alignment.bottomRight,
      colors: dark
          ? [sidebar, sidebar]
          : const [Color(0xfff7f7f6), Color(0xfff1f1ef), Color(0xfff5f5f3)],
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
          surfaceContainer: p.bg,
          surfaceContainerLow: p.surface,
          surfaceContainerHigh: p.soft,
          surfaceTint: Colors.transparent,
        ),
    textTheme: TextTheme(
      bodyMedium: TextStyle(fontSize: 14, height: 1.55, color: p.text),
    ),
    dividerColor: p.border,
    listTileTheme: const ListTileThemeData(
      contentPadding: EdgeInsets.symmetric(horizontal: 16),
      minVerticalPadding: 12,
    ),
    expansionTileTheme: const ExpansionTileThemeData(
      tilePadding: EdgeInsets.symmetric(horizontal: 16),
      childrenPadding: EdgeInsets.fromLTRB(16, 8, 16, 16),
      expandedAlignment: Alignment.centerLeft,
      shape: Border(),
      collapsedShape: Border(),
    ),
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
    menuTheme: MenuThemeData(
      style: MenuStyle(
        backgroundColor: WidgetStatePropertyAll(p.surface),
        surfaceTintColor: const WidgetStatePropertyAll(Colors.transparent),
        elevation: const WidgetStatePropertyAll(6),
        padding: const WidgetStatePropertyAll(EdgeInsets.all(6)),
        shape: WidgetStatePropertyAll(
          RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(14),
            side: BorderSide(color: p.border),
          ),
        ),
      ),
    ),
    menuButtonTheme: MenuButtonThemeData(
      style: ButtonStyle(
        textStyle: const WidgetStatePropertyAll(
          TextStyle(fontSize: 13, fontFamily: 'Segoe UI'),
        ),
        minimumSize: const WidgetStatePropertyAll(Size(0, 40)),
        shape: WidgetStatePropertyAll(
          RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
        ),
      ),
    ),
    cardTheme: CardThemeData(
      color: p.surface,
      surfaceTintColor: Colors.transparent,
      elevation: 0,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(14),
        side: BorderSide(color: p.border),
      ),
    ),
    outlinedButtonTheme: OutlinedButtonThemeData(
      style: OutlinedButton.styleFrom(
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
        side: BorderSide(color: p.border),
      ),
    ),
    tooltipTheme: const TooltipThemeData(
      waitDuration: Duration(milliseconds: 600),
    ),
    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: p.surface,
      contentPadding: const EdgeInsets.all(14),
      floatingLabelBehavior: FloatingLabelBehavior.never,
      helperMaxLines: 3,
      errorMaxLines: 3,
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
