/// Disposable native render qualification; this entry never opens the normal profile.
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
import 'theme.dart';
import 'workspace_shell.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final directory = Directory(
    Platform.environment['DOLORES_SOURCE_CONTROL_SMOKE_DIR'] ?? '',
  ).absolute;
  final profile = Directory(Platform.environment['DOLORES_DATA_DIR'] ?? '')
      .absolute;
  if (!directory.path.replaceAll('\\', '/').contains('/output/') ||
      !profile.path.startsWith('${directory.path}${Platform.pathSeparator}')) {
    throw StateError('Isolated output profile required.');
  }
  await directory.create(recursive: true);
  await initializeDesktopFrame();
  final root = Directory('${directory.path}/public-repository');
  await root.create();
  Future<void> git(List<String> args) async {
    final p = await Process.run('git', args, workingDirectory: root.path);
    if (p.exitCode != 0) throw StateError('Public Git fixture failed.');
  }

  await git(['init', '-b', 'main']);
  await git(['config', 'user.name', 'Public Fixture']);
  await git(['config', 'user.email', 'fixture@example.invalid']);
  await git(['config', 'core.autocrlf', 'false']);
  final tail = '// Unchanged public source context.\n' * (240 * 1024 ~/ 36);
  for (var i = 0; i < 8; i++) {
    await File('${root.path}/example$i.rs')
        .writeAsString('fn main() {\n    println!("Hello Dolores");\n}\n$tail');
  }
  await git(['add', '.']);
  await git(['commit', '-m', 'Public base']);
  for (var i = 0; i < 8; i++) {
    await File(
      '${root.path}/example$i.rs',
    ).writeAsString('fn main() {\n    println!("Hello workspace");\n}\n$tail');
  }
  final bridge = NativeBridge();
  await bridge.open();
  final chat = ChatController(bridge);
  await chat.refresh();
  chat.loading = false;
  await chat.openProject(root.path);
  final host = AppHost(chat);
  await host.git.bind(chat.session, chat.workspaceRoot);
  runApp(SourceControlQualification(directory: directory, host: host));
}

class SourceControlQualification extends StatefulWidget {
  final Directory directory;
  final AppHost host;
  const SourceControlQualification({
    super.key,
    required this.directory,
    required this.host,
  });
  @override
  State<SourceControlQualification> createState() =>
      _SourceControlQualificationState();
}

class _SourceControlQualificationState
    extends State<SourceControlQualification> {
  final boundary = GlobalKey();
  bool dark = false;
  final report = <String, dynamic>{
    'evidence': 'Diagnostic native release with normal workspace/Git components and public fixture only',
  };
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => unawaited(exercise()));
  }

  Future<void> frame() async {
    WidgetsBinding.instance.scheduleFrame();
    await WidgetsBinding.instance.endOfFrame;
  }

  Future<void> publish(String stage) async {
    report['stage'] = stage;
    await File('${widget.directory.path}/stage.json')
        .writeAsString(jsonEncode(report));
  }

  Future<void> capture(String name) async {
    await frame();
    await Future<void>.delayed(const Duration(milliseconds: 500));
    await frame();
    final image =
        await (boundary.currentContext!.findRenderObject()
                as RenderRepaintBoundary)
            .toImage();
    final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
    image.dispose();
    await File('${widget.directory.path}/$name.png')
        .writeAsBytes(bytes!.buffer.asUint8List());
  }

  Future<void> exercise() async {
    try {
      await windowManager.setSize(const Size(1400, 900));
      await frame();
      await Future<void>.delayed(const Duration(milliseconds: 500));
      await publish('baseline');
      await Future<void>.delayed(const Duration(seconds: 4));
      final git = widget.host.git;
      final w = git.selected!;
      await publish('rendering');
      await git.openDiff(w, 'example0.rs', 'working');
      await capture('git-inline-light');
      w.sideBySide = true;
      git.changed();
      await capture('git-side-light');
      setState(() => dark = true);
      await capture('git-side-dark');
      for (var i = 1; i < 8; i++) {
        await git.openDiff(w, 'example$i.rs', 'working');
      }
      await git.loadHistory(w);
      await git.loadLocal(w);
      await git.loadRemotes(w);
      await publish('eight-diffs');
      await Future<void>.delayed(const Duration(seconds: 4));
      await publish('rendering-compact');
      await capture('git-eight-dark');
      await windowManager.setSize(const Size(420, 480));
      widget.host.togglePanel();
      await capture('git-compact-dark');
      setState(() => dark = false);
      await capture('git-compact-light');
      await windowManager.setSize(const Size(1400, 900));
      widget.host.togglePanel();
      final preview = await git.review(w, {
        'kind': 'stage',
        'paths': ['example0.rs'],
      });
      if (preview == null) throw StateError(w.error ?? 'Review failed');
      await git.resolveReview(w, preview, apply: false);
      report['reviewCancel'] = 'passed';
      report['tabs'] = w.tabs.length;
      report['error'] = w.error;
      await publish('complete');
    } catch (e) {
      report['error'] = '$e';
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
      host: widget.host,
      nativeTitleBar: true,
      initialPage: WorkspacePage.sourceControl,
    ),
  );
}
