import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:window_manager/window_manager.dart';

import 'theme.dart';

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
  const DesktopFrame({super.key, required this.child});

  @override
  State<DesktopFrame> createState() => _DesktopFrameState();
}

class _DesktopFrameState extends State<DesktopFrame> with WindowListener {
  bool maximized = false, pending = false;
  String? failure;
  Future<void> Function()? retry;

  @override
  void initState() {
    super.initState();
    windowManager.addListener(this);
    unawaited(readWindowState());
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

  Widget control(String label, IconData icon, VoidCallback action, Palette p) =>
      Tooltip(
        message: label,
        child: SizedBox(
          width: 46,
          height: 32,
          child: TextButton(
            key: Key('window-${label.toLowerCase()}'),
            onPressed: pending ? null : action,
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
        child: Column(
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
                      child: const SizedBox.expand(),
                    ),
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
                    control(
                      'Close',
                      Icons.close,
                      () => perform('close', windowManager.close),
                      p,
                    ),
                  ],
                ],
              ),
            ),
            if (failure != null)
              Container(
                color: p.errorSurface,
                padding: const EdgeInsets.symmetric(horizontal: 12),
                child: Row(
                  children: [
                    Expanded(
                      child: Text(
                        failure!,
                        style: TextStyle(color: p.errorText),
                      ),
                    ),
                    TextButton(
                      onPressed: pending ? null : retry,
                      child: const Text('Retry'),
                    ),
                    IconButton(
                      tooltip: 'Dismiss window error',
                      icon: const Icon(Icons.close, size: 16),
                      onPressed: () => setState(() {
                        failure = null;
                        retry = null;
                      }),
                    ),
                  ],
                ),
              ),
            Expanded(child: widget.child),
          ],
        ),
      ),
    );
  }
}
