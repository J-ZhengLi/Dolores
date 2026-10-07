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
  await File('${root.path}/history-note.md')
      .writeAsString('Public history note\n');
  await git(['add', 'history-note.md']);
  await git(['commit', '-m', 'Document the public workspace']);
  await File('${root.path}/history-note.md')
      .writeAsString('Updated public history note\n');
  await File('${root.path}/history-guide.md').writeAsString('Public guide\n');
  await git(['add', 'history-note.md', 'history-guide.md']);
  await git(['commit', '-m', 'Refine workspace guidance']);
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
      await git.loadHistory(w);
      w.changesExpanded = false;
      git.changed();
      await frame();
      Future<void> press(Key key) async {
        VoidCallback? callback;
        void visit(Element element) {
          if (element.widget.key == key) {
            final target = element.widget;
            if (target is ListTile) callback = target.onTap;
            if (target is IconButton) callback = target.onPressed;
          }
          element.visitChildren(visit);
        }

        boundary.currentContext!.visitChildElements(visit);
        if (callback == null) throw StateError('Missing rendered action $key');
        callback!();
        await frame();
        final deadline = DateTime.now().add(const Duration(seconds: 5));
        while (w.busy) {
          if (DateTime.now().isAfter(deadline)) {
            throw StateError('UI action timed out');
          }
          await Future<void>.delayed(const Duration(milliseconds: 50));
        }
        await frame();
      }

      final first = w.history[0]['id'] as String;
      final second = w.history[1]['id'] as String;
      await press(ValueKey('git-commit-$first'));
      await press(ValueKey('git-commit-$second'));
      await press(ValueKey('git-file-$first:history-note.md'));
      if (w.tabs[w.activeTab]?['commit'] != first ||
          !w.expandedCommits.containsAll([first, second])) {
        throw StateError('Rendered commit/file ownership failed');
      }
      await capture('git-history-light');
      setState(() => dark = true);
      await capture('git-history-dark');
      report['expandedHistoryFileClick'] = 'passed';
      w.tabs.removeWhere((key, _) => key.startsWith('commit:'));
      w.activeTab = w.tabs.keys.first;
      w.changesExpanded = true;
      git.changed();
      await frame();
      await press(const Key('git-changes-menu'));
      await capture('git-actions-dark');
      await press(const Key('git-changes-menu'));
      setState(() => dark = false);
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
