import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:file_selector/file_selector.dart';

import 'app_host.dart';
import 'source_control.dart';
import 'git_host.dart';

import 'package:path/path.dart' as paths;

import 'chat_sidebar.dart';
import 'main.dart' show ChatPage;
import 'settings.dart';
import 'theme.dart';
import 'folders.dart';
import 'file_workspace.dart';
import 'terminal_page.dart';
import 'scheduled_page.dart';
import 'companion_note.dart';
import 'workspace_dock.dart';
import 'window_identity.dart';

enum WorkspacePage { home, scheduled, folders, sourceControl, terminal }

class WorkspaceShell extends StatefulWidget {
  final AppHost host;
  final bool nativeTitleBar;
  final WorkspacePage initialPage;
  const WorkspaceShell({
    super.key,
    required this.host,
    this.nativeTitleBar = false,
    this.initialPage = WorkspacePage.home,
  });
  @override
  State<WorkspaceShell> createState() => _WorkspaceShellState();
}

class _WorkspaceShellState extends State<WorkspaceShell> {
  WorkspacePage page = WorkspacePage.home;
  AppHost get host => widget.host;
  double dragWidth = 0;
  final panelFocus = FocusNode();
  final dockKey = GlobalKey<WorkspaceDockState>();
  String? selectedTask;
  static const labels = [
    'Home',
    'Scheduled',
    'Files',
    'Source Control',
    'Terminal',
  ];
  static const icons = [
    Icons.home_outlined,
    Icons.schedule,
    Icons.folder_outlined,
    Icons.account_tree_outlined,
    Icons.terminal,
  ];
  @override
  void initState() {
    super.initState();
    HardwareKeyboard.instance.addHandler(handleDockShortcut);
    page = widget.initialPage;
    if (page == WorkspacePage.scheduled) {
      unawaited(host.scheduled.refresh());
    }
    if (page == WorkspacePage.terminal) {
      unawaited(host.terminals.enter(host.visible.session));
    }
    host.addListener(changed);
    host.closeReview = requestClose;
    host.repositoryNavigation = (path) async {
      setState(() => page = WorkspacePage.sourceControl);
      await host.git.bind(host.visible.session, host.projectRoot);
      final w = host.git.selected;
      if (w?.status != null && host.projectRoot != null) {
        final relative = paths
            .relative(
              paths.join(host.projectRoot!, path),
              from: w!.status!['root'] as String,
            )
            .replaceAll('\\', '/');
        await host.git.openDiff(w, relative, 'working');
      }
    };
  }

  Future<bool> requestClose() async {
    if (ModalRoute.of(context)?.isCurrent == false) {
      await showDialog<void>(
        context: context,
        builder: (c) => AlertDialog(
          title: const Text('Finish the open review'),
          content: const Text(
            'Close settings or the current review before closing Dolores.',
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(c),
              child: const Text('Keep open'),
            ),
          ],
        ),
      );
      return false;
    }
    final tray = host.background.enabled && !host.quitting;
    final dirty = host.files.documents.values.where((d) => d.dirty).length;
    final tasks = host.tasks.length;
    final terminals = host.terminals.live.length;
    if (dirty == 0 && tasks == 0 && terminals == 0) {
      if (tray) {
        await host.closeToTray(saveFiles: false);
        return false;
      }
      return host.prepareQuit(saveFiles: false);
    }
    final choice = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(tray ? 'Close to tray?' : 'Close Dolores?'),
        content: Text(
          tray
              ? 'Scheduled tasks will continue. Other tasks and terminals stop; unsaved files keep private recovery.'
              : '$dirty unsaved files, $tasks running or queued tasks and $terminals live terminals. Closing stops tasks and terminal processes. Private recovery preserves unsaved edits without saving source files.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Keep open'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(context, 'recover'),
            child: const Text('Keep recovery and close'),
          ),
          if (dirty > 0)
            TextButton(
              onPressed: () => Navigator.pop(context, 'save'),
              child: const Text('Save files and close'),
            ),
        ],
      ),
    );
    if (choice == null || !mounted) return false;
    if (tray) {
      await host.closeToTray(saveFiles: choice == 'save');
      return false;
    }
    return host.prepareQuit(saveFiles: choice == 'save');
  }

  void changed() {
    if (page == WorkspacePage.sourceControl &&
        (GitHost.rootKey(host.git.selected?.root ?? '') !=
                GitHost.rootKey(host.projectRoot ?? '') ||
            host.git.selected?.session != host.visible.session)) {
      unawaited(host.git.bind(host.visible.session, host.projectRoot));
    }
    if (page == WorkspacePage.folders &&
        (host.files.selected?.root != host.projectRoot ||
            host.files.selected?.session != host.visible.session)) {
      unawaited(host.files.bind(host.visible.session, host.projectRoot));
    }
    if (mounted) setState(() {});
  }

  @override
  void dispose() {
    HardwareKeyboard.instance.removeHandler(handleDockShortcut);
    panelFocus.dispose();
    host.closeReview = null;
    host.repositoryNavigation = null;
    host.removeListener(changed);
    super.dispose();
  }

  Future<void> settings() => showSettings(context, host.visible);
  bool handleDockShortcut(KeyEvent event) {
    if (event is KeyDownEvent &&
        event.logicalKey == LogicalKeyboardKey.keyD &&
        HardwareKeyboard.instance.isControlPressed &&
        HardwareKeyboard.instance.isAltPressed &&
        ModalRoute.of(context)?.isCurrent == true) {
      dockKey.currentState?.reveal(fromKeyboard: true);
      return true;
    }
    return false;
  }

  void selectPage(WorkspacePage value) {
    setState(() => page = value);
    if (value == WorkspacePage.scheduled) unawaited(host.scheduled.refresh());
    if (value == WorkspacePage.terminal) {
      unawaited(host.terminals.enter(host.visible.session));
    }
    if (value == WorkspacePage.folders) {
      unawaited(host.files.bind(host.visible.session, host.projectRoot));
    }
    if (value == WorkspacePage.sourceControl) {
      unawaited(host.git.bind(host.visible.session, host.projectRoot));
    }
  }

  Widget dock() => WorkspaceDock(
    key: dockKey,
    selected: page.index,
    items: [
      for (final value in WorkspacePage.values)
        DockItem(
          key: Key('page-${value.name}'),
          label: labels[value.index],
          icon: icons[value.index],
          onPressed: () => selectPage(value),
        ),
      DockItem(
        key: const Key('rail-settings'),
        label: 'Settings',
        icon: Icons.settings_outlined,
        onPressed: settings,
      ),
    ],
  );

  Widget workspaceSidebar(Palette p) => page != WorkspacePage.home
      ? Container(decoration: p.sidebarDecoration, child: panel(p))
      : ChatSidebar(
          width: host.panelWidth,
          chat: host.visible,
          independentNavigation: true,
          showSettings: false,
          showBrand: false,
          settingsKey: const Key('rail-settings'),
          onNewTemporary: () async {
            await host.newConversation(kind: 'temporary');
            if (mounted) selectPage(WorkspacePage.home);
          },
          onNewSide: () async {
            await host.newConversation(kind: 'side');
            if (mounted) selectPage(WorkspacePage.home);
          },
          onOpenProject: openProject,
          onProject: (root) => openProject(root),
          onSelect: host.select,
          onSettings: settings,
        );
  Future<void> openProject([String? root]) async {
    root ??= await getDirectoryPath();
    if (root == null) return;
    await host.openProject(root);
  }

  Widget panel(Palette p) {
    if (page == WorkspacePage.scheduled) {
      return ScheduledPanel(
        host: host.scheduled,
        selected: selectedTask,
        onSelect: (id) => setState(() => selectedTask = id),
      );
    }
    if (page == WorkspacePage.terminal) {
      return TerminalPanel(host: host.terminals, session: host.visible.session);
    }
    if (page == WorkspacePage.sourceControl) {
      return SourceControlPanel(
        git: host.git,
        openFolder: () => unawaited(openProject()),
      );
    }
    if (page == WorkspacePage.folders) {
      return FolderTree(
        files: host.files,
        openFolder: () => unawaited(openProject()),
      );
    }
    if (page == WorkspacePage.home) {
      return ChatSidebar(
        width: host.panelWidth,
        chat: host.visible,
        independentNavigation: true,
        showSettings: false,
        onNewTemporary: () =>
            unawaited(host.newConversation(kind: 'temporary')),
        onNewSide: () => unawaited(host.newConversation(kind: 'side')),
        onOpenProject: openProject,
        onProject: (root) => openProject(root),
        onSelect: host.select,
        onSettings: settings,
      );
    }
    return Container(
      color: p.sidebar,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.all(16),
            child: Text(
              labels[page.index],
              style: const TextStyle(fontSize: 18),
            ),
          ),
          if (host.projectRoot != null)
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Text(
                host.projectRoot!,
                maxLines: 3,
                overflow: TextOverflow.ellipsis,
              ),
            ),
          const SizedBox(height: 12),
          if (page == WorkspacePage.folders && host.projectRoot == null)
            TextButton(
              onPressed: openProject,
              child: const Text('Open folder'),
            ),
        ],
      ),
    );
  }

  Widget content(WorkspacePage page) {
    if (page == WorkspacePage.scheduled) {
      return ScheduledPage(
        host: host.scheduled,
        selected: selectedTask,
        owners: host.owners,
        openResult: (session) async {
          await host.select(session);
          if (mounted) setState(() => page = WorkspacePage.home);
        },
      );
    }
    if (page == WorkspacePage.terminal) {
      return TerminalPage(
        host: host.terminals,
        session: host.visible.session,
        chooseFolder: () => unawaited(openProject()),
        shareSelection: (terminal) async {
          final text = terminal.selectedText();
          if (text.isEmpty) {
            terminal.notice = 'Select the output you want to attach first.';
            host.terminals.layout.changed();
            return;
          }
          final choice = await showDialog<String>(
            context: context,
            builder: (context) => AlertDialog(
              title: const Text('Attach selected terminal output'),
              content: SizedBox(
                width: 480,
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    const Text(
                      'Choose a conversation. Only this selection enters its draft; Send will share it with the model.',
                    ),
                    const SizedBox(height: 12),
                    SizedBox(
                      height: 120,
                      child: SingleChildScrollView(child: SelectableText(text)),
                    ),
                    const SizedBox(height: 12),
                    SizedBox(
                      height: 180,
                      child: ListView(
                        children: [
                          for (final chat in host.initial.sessions)
                            ListTile(
                              title: Text(
                                chat['title']?.toString() ?? 'Conversation',
                              ),
                              onTap: () =>
                                  Navigator.pop(context, chat['id'] as String),
                            ),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.pop(context),
                  child: const Text('Cancel'),
                ),
              ],
            ),
          );
          if (choice == null) return;
          try {
            await host.shareTerminal(choice, text);
            terminal.notice = 'Selected output attached to the chosen draft.';
          } catch (e) {
            terminal.notice = '$e Nothing was sent.';
          }
          host.terminals.layout.changed();
        },
      );
    }
    if (page == WorkspacePage.sourceControl) {
      return SourceControlView(git: host.git);
    }
    if (page == WorkspacePage.folders) {
      return FileWorkspaceView(host: host);
    }
    if (page == WorkspacePage.home) {
      return Column(
        children: [
          CompanionNote(host: host.companion, open: host.select),
          Expanded(
            child: ChatPage(
              key: ObjectKey(host.visible),
              chat: host.visible,
              embedded: true,
              onSelectChat: host.select,
              onSelectProject: (root) => openProject(root),
              onOpenProject: openProject,
            ),
          ),
        ],
      );
    }
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Text(switch (page) {
          WorkspacePage.scheduled => 'Scheduled tasks will appear here when chat-created scheduling is available.',
          WorkspacePage.sourceControl =>
            'Source Control is planned for milestone 17.',
          WorkspacePage.terminal => 'Terminal is planned for milestone 18.',
          _ =>
            host.projectRoot == null
                ? 'Open a folder or select a project conversation on Home.'
                : 'Files are being added in this milestone.',
        }, textAlign: TextAlign.center),
      ),
    );
  }

  Widget divider() => Semantics(
    label: 'Resize side panel',
    hint: 'Drag or use Left and Right arrows. Home resets the width.',
    onIncrease: () =>
        host.resizePanel((host.panelHidden ? 80 : host.panelWidth) + 16),
    onDecrease: () => host.resizePanel(host.panelWidth - 16),
    child: Focus(
      focusNode: panelFocus,
      onKeyEvent: (_, event) {
        if (event is! KeyDownEvent) return KeyEventResult.ignored;
        if (event.logicalKey == LogicalKeyboardKey.home) {
          host.resizePanel(252);
          return KeyEventResult.handled;
        }
        if (event.logicalKey == LogicalKeyboardKey.arrowLeft) {
          host.resizePanel(host.panelWidth - 16);
          return KeyEventResult.handled;
        }
        if (event.logicalKey == LogicalKeyboardKey.arrowRight) {
          host.resizePanel((host.panelHidden ? 80 : host.panelWidth) + 16);
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      child: MouseRegion(
        cursor: SystemMouseCursors.resizeLeftRight,
        child: GestureDetector(
          key: const Key('page-panel-divider'),
          behavior: HitTestBehavior.opaque,
          dragStartBehavior: DragStartBehavior.down,
          onTap: panelFocus.requestFocus,
          onHorizontalDragStart: (_) {
            panelFocus.requestFocus();
            dragWidth = host.panelHidden ? 0 : host.panelWidth;
          },
          onHorizontalDragUpdate: (e) {
            dragWidth += e.delta.dx;
            host.resizePanel(dragWidth);
          },
          child: const SizedBox(width: 8, height: double.infinity),
        ),
      ),
    ),
  );
  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return CallbackShortcuts(
      bindings: {
        if (page == WorkspacePage.folders)
          const SingleActivator(LogicalKeyboardKey.keyP, control: true): () =>
              unawaited(quickOpen(context, host.files)),
      },
      child: Scaffold(
        body: Stack(
          children: [
            Column(
              children: [
                if (!widget.nativeTitleBar)
                  SizedBox(
                    width: double.infinity,
                    height: 32,
                    child: WindowTitle(onTogglePanel: host.togglePanel),
                  ),
                if (host.error != null)
                  MaterialBanner(
                    content: Text(host.error!),
                    actions: [
                      TextButton(
                        onPressed: () => setState(() => host.error = null),
                        child: const Text('Dismiss'),
                      ),
                    ],
                  ),
                if (host.tasks.isNotEmpty)
                  SizedBox(
                    height: 40,
                    child: ListView(
                      scrollDirection: Axis.horizontal,
                      children: [
                        for (final task in host.tasks)
                          Padding(
                            padding: const EdgeInsets.symmetric(horizontal: 8),
                            child: TextButton(
                              onPressed: () => task.session == null
                                  ? null
                                  : host.select(task.session!),
                              child: Text(
                                '${task.workspaceLabel} · ${task.modelPhase == 'queued' ? 'Queued' : 'Running'}',
                              ),
                            ),
                          ),
                      ],
                    ),
                  ),
                Expanded(
                  child: LayoutBuilder(
                    builder: (context, c) {
                      final compact = c.maxWidth < 800 || c.maxHeight < 620;
                      return Stack(
                        children: [
                          Row(
                            children: [
                              if (!compact && !host.panelHidden)
                                SizedBox(
                                  width: host.panelWidth.clamp(
                                    180,
                                    (c.maxWidth - 300).clamp(180, 420),
                                  ),
                                  child: workspaceSidebar(p),
                                ),
                              if (!compact) divider(),
                              Expanded(
                                child: AnimatedSwitcher(
                                  duration:
                                      MediaQuery.disableAnimationsOf(context) ||
                                          MediaQuery.accessibleNavigationOf(
                                            context,
                                          )
                                      ? Duration.zero
                                      : const Duration(milliseconds: 240),
                                  switchInCurve: Curves.easeOutCubic,
                                  switchOutCurve: Curves.easeInCubic,
                                  layoutBuilder: (current, previous) => Stack(
                                    alignment: Alignment.center,
                                    children: [
                                      for (final child in previous)
                                        IgnorePointer(
                                          child: ExcludeFocus(
                                            child: ExcludeSemantics(
                                              child: child,
                                            ),
                                          ),
                                        ),
                                      ?current,
                                    ],
                                  ),
                                  transitionBuilder: (child, animation) =>
                                      FadeTransition(
                                        opacity: animation,
                                        child: SlideTransition(
                                          position: Tween(
                                            begin: const Offset(.012, 0),
                                            end: Offset.zero,
                                          ).animate(animation),
                                          child: child,
                                        ),
                                      ),
                                  child: KeyedSubtree(
                                    key: ValueKey(page),
                                    child: content(page),
                                  ),
                                ),
                              ),
                            ],
                          ),
                          if (compact && !host.panelHidden)
                            Positioned.fill(
                              child: ModalBarrier(
                                color: Colors.black.withValues(alpha: .12),
                                dismissible: true,
                                onDismiss: host.togglePanel,
                              ),
                            ),
                          if (compact && !host.panelHidden)
                            Positioned(
                              left: 0,
                              top: 0,
                              bottom: 0,
                              width: host.panelWidth.clamp(
                                180,
                                c.maxWidth - 48,
                              ),
                              child: Material(
                                elevation: 8,
                                child: Row(
                                  children: [
                                    Expanded(child: workspaceSidebar(p)),
                                    divider(),
                                  ],
                                ),
                              ),
                            ),
                        ],
                      );
                    },
                  ),
                ),
              ],
            ),
            Positioned(left: 0, right: 0, bottom: 0, child: dock()),
          ],
        ),
      ),
    );
  }
}
