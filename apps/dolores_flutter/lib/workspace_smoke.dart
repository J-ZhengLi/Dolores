/// Disposable native qualification using the production Home/Folders modules.
/// Programmatic edits do not qualify physical keyboard, IME or accessibility.
library;

import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:re_editor/re_editor.dart';
import 'package:window_manager/window_manager.dart';

import 'app_host.dart';
import 'bridge.dart';
import 'chat.dart';
import 'desktop_frame.dart';
import 'document_buffer.dart';
import 'file_host.dart';
import 'theme.dart';
import 'workspace_shell.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final value = Platform.environment['DOLORES_WORKSPACE_SMOKE_DIR'];
  if (value == null) {
    throw StateError('Disposable qualification directory required.');
  }
  final directory = Directory(value).absolute;
  if (!directory.path.replaceAll('\\', '/').contains('/output/')) {
    throw StateError('Qualification must stay under output/.');
  }
  await directory.create(recursive: true);
  final profile = Directory(Platform.environment['DOLORES_DATA_DIR'] ?? '')
      .absolute;
  if (!profile.path.startsWith(directory.path)) {
    throw StateError(
      'An isolated profile inside the qualification directory is required.',
    );
  }
  await initializeDesktopFrame();
  final bridge = NativeBridge();
  await bridge.open();
  final chat = ChatController(bridge);
  await chat.refresh();
  chat.loading = false;
  runApp(WorkspaceQualification(directory: directory, host: AppHost(chat)));
}

class WorkspaceQualification extends StatefulWidget {
  final Directory directory;
  final AppHost host;
  const WorkspaceQualification({
    super.key,
    required this.directory,
    required this.host,
  });
  @override
  State<WorkspaceQualification> createState() => _WorkspaceQualificationState();
}

class _WorkspaceQualificationState extends State<WorkspaceQualification> {
  final boundary = GlobalKey();
  bool folders = false, dark = false;
  final report = <String, dynamic>{
    'physicalInput': 'not exercised',
    'openMs': <double>[],
    'typingMs': <double>[],
    'mutationMs': <double>[],
    'frameBuildMs': <double>[],
    'frameRasterMs': <double>[],
  };
  FileHost get files => widget.host.files;
  Future<void> publish(String stage) async {
    report['stage'] = stage;
    final file = File('${widget.directory.path}/stage.json');
    final temp = File('${file.path}.pending');
    await temp.writeAsString(jsonEncode(report));
    await temp.rename(file.path);
  }

  Future<void> frame() async {
    WidgetsBinding.instance.scheduleFrame();
    await WidgetsBinding.instance.endOfFrame;
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

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => unawaited(exercise()));
  }

  Future<void> exercise() async {
    try {
      await windowManager.setSize(const Size(1400, 900));
      final root = Directory('${widget.directory.path}/project-A');
      await root.create();
      final ordinary = File('${root.path}/main.rs');
      await ordinary.writeAsString(
        'fn main() {\n    println!("Hello Dolores");\n}\n',
      );
      final line = '// ${'x' * 75}\n';
      final large = File('${root.path}/large.rs');
      await large.writeAsString(
        (line * ((1024 * 1024 - 64) ~/ line.length)).padRight(
          1024 * 1024 - 64,
          '\n',
        ),
      );
      for (var i = 0; i < 4; i++) {
        await File('${root.path}/part$i.rs')
            .writeAsString(line * (256 * 1024 ~/ line.length));
      }
      await widget.host.initial.openProject(root.path);
      widget.host.initial.draft =
          'A retained Home draft for the public workspace fixture.';
      await capture('home-light');
      await publish('home-idle');
      await Future<void>.delayed(const Duration(seconds: 10));
      await files.bind(widget.host.visible.session, widget.host.projectRoot);
      final w = files.selected!;
      setState(() => folders = true);
      await frame();
      Future<FileDocument> open(String path) async {
        final timer = Stopwatch()..start();
        final d = await files.open(w, path);
        if (d == null) throw StateError(files.error ?? 'Open failed.');
        await frame();
        (report['openMs'] as List<double>).add(
          timer.elapsedMicroseconds / 1000,
        );
        return d;
      }

      final largeDoc = await open('large.rs');
      w.layoutOwner.pin(w.layoutOwner.activeGroup, largeDoc.id);
      await publish('one-large');
      await Future<void>.delayed(const Duration(seconds: 3));
      w.layoutOwner.split(
        w.layoutOwner.activeGroup,
        largeDoc.id,
        Axis.horizontal,
      );
      await frame();
      await publish('two-large');
      await Future<void>.delayed(const Duration(seconds: 3));
      final ids = w.layoutOwner.groups.keys.toList();
      w.layoutOwner.split(ids[0], largeDoc.id, Axis.vertical);
      w.layoutOwner.split(ids[1], largeDoc.id, Axis.vertical);
      await frame();
      await publish('four-large');
      await Future<void>.delayed(const Duration(seconds: 3));
      final editors = <CodeEditor>[];
      void mountedEditors(Element element) {
        final child = element.widget;
        if (child is CodeEditor && child.controller is DocumentView) {
          editors.add(child);
        }
        element.visitChildElements(mountedEditors);
      }

      mountedEditors(boundary.currentContext! as Element);
      if (editors.length != 4) {
        throw StateError('Expected four mounted file views.');
      }
      report['mountedViews'] = editors.length;
      final view = editors.first.controller!;
      editors.first.focusNode!.requestFocus();
      await frame();
      var measuring = true;
      void timings(List<ui.FrameTiming> values) {
        if (!measuring) return;
        for (final value in values) {
          (report['frameBuildMs'] as List<double>).add(
            value.buildDuration.inMicroseconds / 1000,
          );
          (report['frameRasterMs'] as List<double>).add(
            value.rasterDuration.inMicroseconds / 1000,
          );
        }
      }

      WidgetsBinding.instance.addTimingsCallback(timings);
      for (var i = 0; i < 40; i++) {
        final timer = Stopwatch()..start();
        view.selection = const CodeLineSelection.collapsed(index: 0, offset: 3);
        view.replaceSelection('a');
        (report['mutationMs'] as List<double>).add(
          timer.elapsedMicroseconds / 1000,
        );
        if (largeDoc.error != null || !largeDoc.dirty) {
          throw StateError('Typing probe was refused: ${largeDoc.error}');
        }
        await frame();
        (report['typingMs'] as List<double>).add(
          timer.elapsedMicroseconds / 1000,
        );
        await files.flush(largeDoc);
        await Future<void>.delayed(const Duration(milliseconds: 80));
      }
      measuring = false;
      WidgetsBinding.instance.removeTimingsCallback(timings);
      view.undo();
      view.redo();
      await files.flush(largeDoc);
      await files.close(w, largeDoc, discard: true);
      await frame();
      final groups = w.layoutOwner.groups.keys.toList();
      for (var i = 0; i < 4; i++) {
        w.layoutOwner.focus(groups[i]);
        final d = await open('part$i.rs');
        w.layoutOwner.pin(groups[i], d.id);
      }
      await publish('four-distinct');
      await capture('folders-four-light');
      setState(() => dark = true);
      await capture('folders-four-dark');
      await Future<void>.delayed(const Duration(seconds: 3));
      await windowManager.setSize(const Size(420, 480));
      widget.host.togglePanel();
      await capture('folders-compact-dark');
      setState(() => dark = false);
      await capture('folders-compact-light');
      await windowManager.setSize(const Size(1400, 900));
      for (final d in files.documents.values.toList()) {
        await files.close(w, d);
      }
      final sample = await open('main.rs');
      w.layoutOwner.pin(w.layoutOwner.activeGroup, sample.id);
      await publish('folders-idle');
      await Future<void>.delayed(const Duration(seconds: 10));
      for (var i = 0; i < 8; i++) {
        for (final d in files.documents.values.toList()) {
          await files.close(w, d);
        }
        final next = await open('main.rs');
        if (i < 7) await files.close(w, next);
      }
      await files.checkpoint(w);
      await files.persistLayout(w);
      await capture('folders-code-light');
      setState(() => dark = true);
      await capture('folders-code-dark');
      await publish('complete');
    } catch (e, stack) {
      report['error'] = '$e';
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
      key: ValueKey(folders),
      host: widget.host,
      nativeTitleBar: true,
      initialPage: folders ? WorkspacePage.folders : WorkspacePage.home,
    ),
  );
}
