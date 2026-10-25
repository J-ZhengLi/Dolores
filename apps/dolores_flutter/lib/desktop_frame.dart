import 'floating_notice.dart';

import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:window_manager/window_manager.dart';

import 'theme.dart';
import 'window_identity.dart';

/// Keep the native frame as a fallback if a desktop plugin cannot initialize.
Future<bool> initializeDesktopFrame() async {
  if (kIsWeb ||
      !{
        TargetPlatform.windows,
        TargetPlatform.linux,
        TargetPlatform.macOS,
      }.contains(defaultTargetPlatform)) {
    return false;
  }
  try {
    await windowManager.ensureInitialized();
    await windowManager.waitUntilReadyToShow(
      WindowOptions(
        minimumSize: const Size(420, 480),
        titleBarStyle: TitleBarStyle.hidden,
        windowButtonVisibility: defaultTargetPlatform == TargetPlatform.macOS,
      ),
    );
    return true;
  } catch (_) {
    try {
      await windowManager.setTitleBarStyle(TitleBarStyle.normal);
    } catch (_) {
      // A missing plugin leaves the original native frame in place.
    }
    debugPrint('Custom window controls unavailable; using the native frame.');
    return false;
  }
}

/// Lives above the Navigator, so window controls also work with dialogs open.
class DesktopFrame extends StatefulWidget {
  final Widget child;
  final Widget? attention, search;
  final VoidCallback? onTogglePanel;
  final Future<void> Function()? onOpenSettings;
  final Future<bool> Function()? onBeforeClose;
  final Future<void> Function()? onRequestClose;
  const DesktopFrame({
    super.key,
    required this.child,
    this.attention,
    this.search,
    this.onTogglePanel,
    this.onOpenSettings,
    this.onBeforeClose,
    this.onRequestClose,
  });

  @override
  State<DesktopFrame> createState() => _DesktopFrameState();
}

class _DesktopFrameState extends State<DesktopFrame> with WindowListener {
  bool maximized = false, pending = false, closing = false;
  bool settingsOpen = false;
  final settingsFocus = FocusNode();
  String? failure;
  Future<void> Function()? retry;

  @override
  void initState() {
    super.initState();
    windowManager.addListener(this);
    if (widget.onBeforeClose != null || widget.onRequestClose != null) {
      unawaited(
        perform(
          'protect Close for recovery',
          () => windowManager.setPreventClose(true),
        ),
      );
    }
    unawaited(readWindowState());
  }

  @override
  void didUpdateWidget(covariant DesktopFrame oldWidget) {
    super.didUpdateWidget(oldWidget);
    if ((oldWidget.onBeforeClose != null || oldWidget.onRequestClose != null) !=
        (widget.onBeforeClose != null || widget.onRequestClose != null)) {
      unawaited(
        perform(
          'protect Close for recovery',
          () => windowManager.setPreventClose(
            widget.onBeforeClose != null || widget.onRequestClose != null,
          ),
        ),
      );
    }
  }

  Future<void> readWindowState() async {
    try {
      final value = await windowManager.isMaximized();
      if (mounted) setState(() => maximized = value);
    } catch (_) {
      // The button queries the actual state again before toggling.
    }
  }

  @override
  void dispose() {
    settingsFocus.dispose();
    windowManager.removeListener(this);
    super.dispose();
  }

  @override
  void onWindowMaximize() {
    if (mounted) setState(() => maximized = true);
  }

  @override
  void onWindowUnmaximize() {
    if (mounted) setState(() => maximized = false);
  }

  Future<void> perform(String action, Future<void> Function() operation) async {
    if (pending) return;
    setState(() {
      pending = true;
      failure = null;
      retry = null;
    });
    try {
      await operation();
    } catch (_) {
      if (mounted) {
        setState(() {
          failure = 'Could not $action the window.';
          retry = () => perform(action, operation);
        });
      }
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  Future<void> toggleMaximize() async {
    // Read the OS state, including changes from keyboard/window-manager actions.
    if (await windowManager.isMaximized()) {
      await windowManager.unmaximize();
    } else {
      await windowManager.maximize();
    }
    await readWindowState();
  }

  Future<void> closeWindow() async {
    if (closing) return;
    closing = true;
    if (widget.onRequestClose != null) {
      try {
        await widget.onRequestClose!();
      } catch (_) {
        if (mounted) {
          setState(() => failure = 'Could not close Dolores. Try again.');
        }
      } finally {
        closing = false;
      }
      return;
    }
    // Closing the window is immediate. Persistence is best effort, never a veto.
    try {
      await windowManager.hide().timeout(const Duration(milliseconds: 250));
    } catch (_) {
      /* A native close below remains available if hiding fails. */
    }
    var quit = true;
    try {
      quit = await (widget.onBeforeClose?.call() ?? Future.value(true)).timeout(
        const Duration(seconds: 2),
      );
    } catch (_) {
      /* Failed or stalled recovery must not trap the window. */
    }
    if (quit) {
      try {
        if (widget.onBeforeClose == null) {
          await windowManager.close();
        } else {
          await windowManager.destroy();
        }
      } catch (_) {
        // Fall back to the native close path if the custom destroy call fails.
        await windowManager.setPreventClose(false);
        await windowManager.close();
      }
    }
    closing = false;
  }

  Future<void> openSettings() async {
    if (settingsOpen || widget.onOpenSettings == null) return;
    final restoreFocus = settingsFocus.hasFocus;
    setState(() {
      settingsOpen = true;
      failure = null;
      retry = null;
    });
    try {
      await widget.onOpenSettings!();
    } catch (_) {
      if (mounted) {
        setState(() {
          failure = 'Could not open Settings.';
          retry = openSettings;
        });
      }
    } finally {
      if (mounted) {
        setState(() => settingsOpen = false);
        if (restoreFocus) {
          WidgetsBinding.instance.addPostFrameCallback((_) {
            if (mounted) settingsFocus.requestFocus();
          });
        }
      }
    }
  }

  @override
  void onWindowClose() {
    if (widget.onBeforeClose != null || widget.onRequestClose != null) {
      unawaited(closeWindow());
    }
  }

  Widget control(String label, IconData icon, VoidCallback action, Palette p) =>
      Tooltip(
        message: label,
        child: SizedBox(
          width: 46,
          height: 32,
          child: TextButton(
            key: Key('window-${label.toLowerCase()}'),
            focusNode: label == 'Settings' ? settingsFocus : null,
            onPressed:
                (label != 'Close' && pending) ||
                    (label == 'Settings' && settingsOpen)
                ? null
                : action,
            style: ButtonStyle(
              padding: const WidgetStatePropertyAll(EdgeInsets.zero),
              shape: const WidgetStatePropertyAll(RoundedRectangleBorder()),
              foregroundColor: WidgetStateProperty.resolveWith(
                (states) =>
                    label == 'Close' && states.contains(WidgetState.hovered)
                    ? p.errorText
                    : p.text,
              ),
              backgroundColor: WidgetStateProperty.resolveWith(
                (states) => states.contains(WidgetState.hovered)
                    ? (label == 'Close' ? p.errorSurface : p.soft)
                    : Colors.transparent,
              ),
            ),
            child: Icon(
              icon,
              size: label == 'Close' ? 18 : 14,
              semanticLabel: label,
            ),
          ),
        ),
      );

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final mac = defaultTargetPlatform == TargetPlatform.macOS;
    return Overlay.wrap(
      child: Material(
        color: p.bg,
        child: Stack(
          children: [
            Column(
              children: [
                SizedBox(
                  key: const Key('desktop-title-bar'),
                  height: 32,
                  child: Row(
                    children: [
                      // macOS keeps its native traffic lights at the upper left.
                      if (mac) const SizedBox(width: 80),
                      Expanded(
                        child: GestureDetector(
                          key: const Key('window-drag-area'),
                          behavior: HitTestBehavior.opaque,
                          onPanStart: (_) =>
                              perform('move', windowManager.startDragging),
                          onDoubleTap: () => perform('resize', toggleMaximize),
                          onSecondaryTap:
                              defaultTargetPlatform == TargetPlatform.windows
                              ? () => perform(
                                  'show the menu for',
                                  windowManager.popUpWindowMenu,
                                )
                              : null,
                          child: WindowTitle(
                            onTogglePanel: widget.onTogglePanel,
                          ),
                        ),
                      ),
                      if (widget.search != null) widget.search!,
                      if (widget.attention != null) widget.attention!,
                      if (widget.onOpenSettings != null)
                        control(
                          'Settings',
                          Icons.settings_outlined,
                          openSettings,
                          p,
                        ),
                      if (!mac) ...[
                        control(
                          'Minimize',
                          Icons.remove,
                          () => perform('minimize', windowManager.minimize),
                          p,
                        ),
                        control(
                          maximized ? 'Restore' : 'Maximize',
                          maximized ? Icons.filter_none : Icons.crop_square,
                          () => perform('resize', toggleMaximize),
                          p,
                        ),
                        control('Close', Icons.close, closeWindow, p),
                      ],
                    ],
                  ),
                ),
                Expanded(child: widget.child),
              ],
            ),
            if (failure != null)
              Positioned(
                top: 40,
                right: 16,
                child: SizedBox(
                  width: (MediaQuery.sizeOf(context).width - 32).clamp(
                    200,
                    420,
                  ),
                  child: FloatingNotice(
                    title: 'Window needs attention',
                    severity: 'warning',
                    message: failure!,
                    onRetry: pending ? null : retry,
                    onDismiss: () => setState(() {
                      failure = null;
                      retry = null;
                    }),
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }
}
