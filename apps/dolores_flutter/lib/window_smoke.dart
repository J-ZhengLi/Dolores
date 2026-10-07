/// Explicit disposable same-engine window backend trial. Never a normal entry.
// ignore_for_file: invalid_use_of_internal_member, implementation_imports
library;

import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/src/foundation/_features.dart' as features;
import 'package:flutter/src/widgets/_window.dart';

import 'app_host.dart';
import 'bridge.dart';
import 'chat.dart';
import 'desktop_frame.dart';
import 'theme.dart';
import 'workspace_shell.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final directory = Directory(
    Platform.environment['DOLORES_WINDOW_SMOKE_DIR'] ?? '',
  ).absolute;
  final profile = Directory(Platform.environment['DOLORES_DATA_DIR'] ?? '')
      .absolute;
  if (!directory.path.replaceAll('\\', '/').contains('/output/') ||
      !profile.path.startsWith('${directory.path}${Platform.pathSeparator}')) {
    throw StateError(
      'A disposable output directory and nested profile are required.',
    );
  }
  await directory.create(recursive: true);
  await initializeDesktopFrame();
  final bridge = NativeBridge();
  await bridge.open();
  final chat = ChatController(bridge);
  await chat.refresh();
  chat.loading = false;
  runWidget(
    ViewCollection(
      views: [
        View(
          view: WidgetsBinding.instance.platformDispatcher.implicitView!,
          child: WindowQualification(directory: directory, host: AppHost(chat)),
        ),
      ],
    ),
  );
}

class WindowQualification extends StatefulWidget {
  final Directory directory;
  final AppHost host;
  const WindowQualification({
    super.key,
    required this.directory,
    required this.host,
  });
  @override
  State<WindowQualification> createState() => _WindowQualificationState();
}

class _WindowQualificationState extends State<WindowQualification> {
  final boundary = GlobalKey();
  final report = <String, dynamic>{
    'backend': 'Flutter 3.47.5 same-engine experimental Windows API',
    'hostOwners': 1,
    'physicalInputQualified': false,
  };
  RegularWindowController? secondary;
  Future<void> publish(String stage) async {
    report['stage'] = stage;
    final target = File('${widget.directory.path}/stage.json');
    await File('${target.path}.pending').writeAsString(jsonEncode(report));
    await File('${target.path}.pending').rename(target.path);
  }

  Future<void> capture(String name) async {
    WidgetsBinding.instance.scheduleFrame();
    await WidgetsBinding.instance.endOfFrame;
    final image =
        await (boundary.currentContext!.findRenderObject()
                as RenderRepaintBoundary)
            .toImage();
    final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
    image.dispose();
    await File('${widget.directory.path}/$name.png')
        .writeAsBytes(bytes!.buffer.asUint8List());
  }

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => unawaited(exercise()));
  }

  Future<void> exercise() async {
    try {
      widget.host.initial.draft = 'Retained primary draft 19';
      await publish('baseline');
      await Future<void>.delayed(const Duration(seconds: 10));
      // Diagnostic-local only. Never modify SDK files or production flags.
      features.debugEnabledFeatureFlags.add('windowing');
      features.isWindowingEnabled = true;
      // The binding cached the disabled owner before this deliberate trial.
      WidgetsBinding.instance.windowingOwner = createDefaultWindowingOwner();
      secondary = RegularWindowController(
        size: const Size(800, 600),
        title: 'Dolores experimental trial 19',
      );
      final primary = WidgetsBinding.instance.platformDispatcher.implicitView!;
      runWidget(
        ViewCollection(
          views: [
            View(view: primary, child: widget),
            RegularWindow(
              controller: secondary!,
              child: MaterialApp(
                theme: doloresTheme(true),
                home: const Scaffold(
                  body: Center(
                    child: Text('One shared host; empty experimental view 19'),
                  ),
                ),
              ),
            ),
          ],
        ),
      );
      secondary!.activate();
      report['secondViewCreated'] = true;
      report['views'] = WidgetsBinding.instance.platformDispatcher.views.length;
      await publish('two-window-idle');
      await Future<void>.delayed(const Duration(seconds: 10));
      report['activationObserved'] = secondary!.isActivated;
      secondary!.destroy();
      secondary = null;
      runWidget(
        ViewCollection(
          views: [View(view: primary, child: widget)],
        ),
      );
      await Future<void>.delayed(const Duration(seconds: 1));
      report['draftRetained'] =
          widget.host.initial.draft == 'Retained primary draft 19';
      report['primaryUsable'] = (await widget.host.initial.bridge.call({
        'command': 'experimentalPreferences',
      })) is Map;
      report['passed'] =
          report['draftRetained'] == true && report['primaryUsable'] == true;
      await capture('primary-after-close');
      await publish('complete');
    } catch (error, stack) {
      report['passed'] = false;
      report['error'] = '$error';
      report['stack'] = '$stack';
      report['draftRetained'] =
          widget.host.initial.draft == 'Retained primary draft 19';
      report['primaryUsable'] = (await widget.host.initial.bridge.call({
        'command': 'experimentalPreferences',
      })) is Map;
      await capture('primary-after-failure');
      await publish('held');
    }
  }

  @override
  Widget build(BuildContext context) => MaterialApp(
    theme: doloresTheme(false),
    debugShowCheckedModeBanner: false,
    builder: (context, child) => RepaintBoundary(
      key: boundary,
      child: DesktopFrame(
        onTogglePanel: widget.host.togglePanel,
        child: child!,
      ),
    ),
    home: WorkspaceShell(host: widget.host, nativeTitleBar: true),
  );
}
