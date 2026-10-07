/// Disposable production terminal render and ownership corpus.
/// Programmatic input does not qualify physical keyboard or IME behavior.
library;

import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:window_manager/window_manager.dart';

import 'app_host.dart';
import 'bridge.dart';
import 'chat.dart';
import 'desktop_frame.dart';
import 'terminal_host.dart';
import 'theme.dart';
import 'workspace_shell.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final directory = Directory(
    Platform.environment['DOLORES_TERMINAL_SMOKE_DIR'] ?? '',
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
  runApp(TerminalQualification(directory: directory, host: AppHost(chat)));
}

class TerminalQualification extends StatefulWidget {
  final Directory directory;
  final AppHost host;
  const TerminalQualification({
    super.key,
    required this.directory,
    required this.host,
  });
  @override
  State<TerminalQualification> createState() => _TerminalQualificationState();
}

class _TerminalQualificationState extends State<TerminalQualification> {
  final boundary = GlobalKey();
  final report = <String, dynamic>{
    'evidence': 'Production Flutter/native modules with programmatic input; physical input unqualified',
  };
  bool terminals = false, dark = false;
  TerminalHost get owner => widget.host.terminals;
  Future<void> publish(String stage) async {
    report['stage'] = stage;
    final file = File('${widget.directory.path}/stage.json');
    final pending = File('${file.path}.pending');
    await pending.writeAsString(jsonEncode(report));
    await pending.rename(file.path);
  }

  Future<void> frame() async {
    WidgetsBinding.instance.scheduleFrame();
    await WidgetsBinding.instance.endOfFrame;
  }

  Future<void> capture(String name) async {
    await frame();
    await Future<void>.delayed(const Duration(milliseconds: 300));
    await frame();
    final image =
        await (boundary.currentContext!.findRenderObject()
                as RenderRepaintBoundary)
            .toImage();
    final data = await image.toByteData(format: ui.ImageByteFormat.png);
    image.dispose();
    await File('${widget.directory.path}/$name.png')
        .writeAsBytes(data!.buffer.asUint8List());
  }

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => unawaited(exercise()));
  }

  Future<void> waitFor(TerminalSession terminal, String marker) async {
    final timer = Stopwatch()..start();
    while (timer.elapsed < const Duration(seconds: 25)) {
      if (terminal.terminal.buffer.getText().contains(marker)) return;
      await Future<void>.delayed(const Duration(milliseconds: 100));
    }
    throw StateError(
      'Executed terminal output was absent: $marker ${terminal.notice}',
    );
  }

  Future<void> exercise() async {
    try {
      await windowManager.setSize(const Size(1400, 900));
      final root = Directory('${widget.directory.path}/project 世界');
      await root.create();
      await File('${root.path}/render.cjs').writeAsString(
        'for(let i=0;i<6000;i++) console.log("\\x1b[32mRetained line "+i+" 世界\\x1b[0m");console.log("RENDER_READY_18");',
      );
      await widget.host.initial.openProject(root.path);
      await publish('baseline');
      await Future<void>.delayed(const Duration(seconds: 10));
      setState(() => terminals = true);
      await owner.enter(widget.host.visible.session);
      final first = owner.sessions.values.single;
      await waitFor(first, 'PS ');
      await Future<void>.delayed(const Duration(milliseconds: 500));
      await owner.input(first, 'node render.cjs\r');
      await waitFor(first, 'RENDER_READY_18');
      await owner.splitNew(
        widget.host.visible.session,
        owner.layout.activeGroup,
        Axis.horizontal,
      );
      final second = owner.sessions.values.last;
      await waitFor(second, 'PS ');
      await Future<void>.delayed(const Duration(milliseconds: 500));
      await owner.input(second, 'node render.cjs\r');
      await waitFor(second, 'RENDER_READY_18');
      report['twoOwnedShells'] =
          owner.sessions.length == 2 && owner.layout.groups.length == 2;
      report['retainedLines'] = [
        first.terminal.buffer.lines.length,
        second.terminal.buffer.lines.length,
      ];
      await capture('terminal-split-light');
      setState(() => dark = true);
      await capture('terminal-split-dark');
      await publish('two-full-scrollback-idle');
      await Future<void>.delayed(const Duration(seconds: 10));
      final groups = owner.layout.groups.keys.toList();
      owner.layout.move(first.id, groups.first, groups.last);
      owner.layout.move(first.id, groups.last, groups.first);
      if (owner.sessions.length != 2) {
        throw StateError('Move duplicated a terminal owner.');
      }
      await windowManager.setSize(const Size(480, 600));
      widget.host.togglePanel();
      await capture('terminal-compact-dark');
      await windowManager.setSize(const Size(1400, 900));
      for (final shell in owner.live.toList()) {
        if (!await owner.stop(shell)) throw StateError('PTY cleanup failed.');
      }
      await owner.checkpoint();
      await capture('terminal-stopped-dark');
      report['stoppedOutputRetained'] = first.terminal.buffer
          .getText()
          .contains('RENDER_READY_18');
      report['remainingLiveShells'] = owner.live.length;
      report['passed'] =
          report['twoOwnedShells'] == true &&
          report['stoppedOutputRetained'] == true &&
          owner.live.isEmpty;
      await publish('complete');
    } catch (error, stack) {
      report['passed'] = false;
      report['error'] = '$error';
      report['stack'] = '$stack';
      await publish('failed');
    }
  }

  @override
  Widget build(BuildContext context) => MaterialApp(
    debugShowCheckedModeBanner: false,
    theme: doloresTheme(false),
    darkTheme: doloresTheme(true),
    themeMode: dark ? ThemeMode.dark : ThemeMode.light,
    builder: (context, child) => RepaintBoundary(
      key: boundary,
      child: DesktopFrame(
        onTogglePanel: widget.host.togglePanel,
        child: child!,
      ),
    ),
    home: WorkspaceShell(
      key: ValueKey(terminals),
      host: widget.host,
      nativeTitleBar: true,
      initialPage: terminals ? WorkspacePage.terminal : WorkspacePage.home,
    ),
  );
}
