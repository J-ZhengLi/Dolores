import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:xterm/xterm.dart';

import 'file_layout.dart';
import 'terminal_host.dart';
import 'theme.dart';

class TerminalTabDrag {
  final String id, group;
  final TerminalHost owner;
  TerminalTabDrag(this.owner, this.id, this.group);
}

class TerminalPanel extends StatelessWidget {
  final TerminalHost host;
  final String? session;
  const TerminalPanel({super.key, required this.host, required this.session});
  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: host,
    builder: (context, _) => Column(
      children: [
        Padding(
          padding: const EdgeInsets.all(12),
          child: Row(
            children: [
              const Expanded(child: Text('TERMINALS')),
              IconButton(
                tooltip: 'New terminal',
                onPressed: host.opening ? null : () => host.create(session),
                icon: const Icon(Icons.add, size: 18),
              ),
            ],
          ),
        ),
        Expanded(
          child: ListView(
            children: [
              for (final s in host.sessions.values)
                ListTile(
                  dense: true,
                  selected: host.active == s.id,
                  onTap: () => host.select(s.id),
                  title: Text(
                    s.title,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                  subtitle: Text(
                    '${s.state} · ${s.displayCwd}',
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                  ),
                  leading: const Icon(Icons.terminal, size: 18),
                ),
            ],
          ),
        ),
      ],
    ),
  );
}

class TerminalPage extends StatelessWidget {
  final TerminalHost host;
  final String? session;
  final VoidCallback chooseFolder;
  const TerminalPage({
    super.key,
    required this.host,
    required this.session,
    required this.chooseFolder,
    this.shareSelection,
  });
  final Future<void> Function(TerminalSession)? shareSelection;
  Future<void> close(BuildContext context, TerminalSession s) async {
    if (s.live) {
      final yes = await showDialog<bool>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('Close terminal?'),
          content: Text(
            'Stop ${s.title} and its child processes? Output remains until this tab closes.',
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context, false),
              child: const Text('Keep open'),
            ),
            TextButton(
              onPressed: () => Navigator.pop(context, true),
              child: const Text('Stop and close'),
            ),
          ],
        ),
      );
      if (yes != true) return;
    }
    await host.close(s);
  }

  Future<void> copy(TerminalSession s) async {
    final text = s.selectedText();
    if (text.isNotEmpty) await Clipboard.setData(ClipboardData(text: text));
  }

  Widget recovery() => Center(
    child: Wrap(
      spacing: 8,
      children: [
        if (host.opening) const CircularProgressIndicator(),
        if (!host.opening)
          TextButton(
            onPressed: () => unawaited(host.create(session)),
            child: const Text('Retry'),
          ),
        if (!host.opening)
          TextButton(
            onPressed: chooseFolder,
            child: const Text('Choose folder'),
          ),
        if (!host.opening)
          TextButton(
            onPressed: () => unawaited(host.create(null, home: true)),
            child: const Text('Open at home'),
          ),
      ],
    ),
  );
  bool valid(TerminalTabDrag d) =>
      identical(d.owner, host) &&
      host.sessions.containsKey(d.id) &&
      host.layout.groups[d.group]?.tabs.contains(d.id) == true;
  void drop(
    TerminalTabDrag d,
    String group,
    Axis? axis, {
    bool before = false,
    int? index,
  }) {
    if (!valid(d)) return;
    if (axis == null) {
      host.layout.move(d.id, d.group, group, index: index);
    } else if (!host.layout.split(
      group,
      d.id,
      axis,
      source: d.group,
      before: before,
    )) {
      host.error =
          'Four terminal groups are open. Move to an existing group instead.';
      host.layout.changed();
    }
  }

  Widget tab(BuildContext context, FileGroup g, String id, int index) {
    final s = host.sessions[id]!;
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final body = Container(
      color: g.active == id ? p.soft : p.sidebar,
      padding: const EdgeInsets.only(left: 10),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          InkWell(
            onTap: () => host.layout.select(g.id, id),
            child: Padding(
              padding: const EdgeInsets.symmetric(vertical: 10),
              child: Tooltip(
                message: '${s.shell}\n${s.displayCwd}\n${s.state}',
                child: SizedBox(
                  width: 128,
                  child: Text(
                    s.title,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
              ),
            ),
          ),
          IconButton(
            tooltip: 'Close terminal',
            onPressed: () => close(context, s),
            icon: const Icon(Icons.close, size: 16),
            visualDensity: VisualDensity.compact,
          ),
        ],
      ),
    );
    return DragTarget<TerminalTabDrag>(
      onWillAcceptWithDetails: (d) => valid(d.data),
      onAcceptWithDetails: (d) => drop(d.data, g.id, null, index: index),
      builder: (context, _, _) => Draggable<TerminalTabDrag>(
        data: TerminalTabDrag(host, id, g.id),
        feedback: Material(
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: Text(s.title),
          ),
        ),
        childWhenDragging: Opacity(opacity: .4, child: body),
        child: body,
      ),
    );
  }

  Widget edge(
    BuildContext context,
    FileGroup g,
    String label,
    Axis axis,
    bool before,
  ) => DragTarget<TerminalTabDrag>(
    onWillAcceptWithDetails: (d) => valid(d.data),
    onAcceptWithDetails: (d) => drop(d.data, g.id, axis, before: before),
    builder: (context, candidates, _) => Container(
      color: Theme.of(context).colorScheme.primary
          .withValues(alpha: candidates.isEmpty ? 0.06 : 0.22),
      alignment: Alignment.center,
      child: Text(label),
    ),
  );
  Widget group(BuildContext context, FileGroup g) {
    final s = host.sessions[g.active];
    return DragTarget<TerminalTabDrag>(
      onWillAcceptWithDetails: (d) => valid(d.data),
      onAcceptWithDetails: (d) => drop(d.data, g.id, null),
      builder: (context, candidates, _) => Column(
        children: [
          Row(
            children: [
              Expanded(
                child: SingleChildScrollView(
                  scrollDirection: Axis.horizontal,
                  child: Row(
                    children: [
                      for (var i = 0; i < g.tabs.length; i++)
                        tab(context, g, g.tabs[i], i),
                    ],
                  ),
                ),
              ),
              IconButton(
                key: Key('terminal-plus-${g.id}'),
                tooltip: 'New terminal',
                onPressed: host.opening
                    ? null
                    : () => host.create(session, group: g.id),
                icon: const Icon(Icons.add, size: 18),
              ),
              PopupMenuButton<String>(
                tooltip: 'Terminal actions',
                icon: const Icon(Icons.more_horiz, size: 18),
                onSelected: (value) {
                  switch (value) {
                    case 'right':
                      unawaited(host.splitNew(session, g.id, Axis.horizontal));
                    case 'down':
                      unawaited(host.splitNew(session, g.id, Axis.vertical));
                    case 'copy':
                      if (s != null) unawaited(copy(s));
                    case 'share':
                      if (s != null && shareSelection != null) {
                        unawaited(shareSelection!(s));
                      }
                    case 'checkpoint':
                      unawaited(
                        host.checkpoint().catchError((Object e) {
                          host.error = '$e';
                          host.layout.changed();
                        }),
                      );
                    case 'paste':
                      if (s != null) {
                        unawaited(
                          Clipboard.getData(Clipboard.kTextPlain).then((v) {
                            if (v?.text != null) s.terminal.paste(v!.text!);
                          }),
                        );
                      }
                    case 'stop':
                      if (s != null) unawaited(host.stop(s));
                    case 'close-group':
                      host.layout.closeGroup(g.id);
                    default:
                      if (value.startsWith('move:') && s != null) {
                        host.layout.move(s.id, g.id, value.substring(5));
                      } else if (value.startsWith('focus:')) {
                        host.layout.focus(value.substring(6));
                      }
                  }
                },
                itemBuilder: (context) => [
                  PopupMenuItem(
                    value: 'right',
                    enabled: host.layout.groups.length < 4 && !host.opening,
                    child: const Text('Split terminal right'),
                  ),
                  PopupMenuItem(
                    value: 'down',
                    enabled: host.layout.groups.length < 4 && !host.opening,
                    child: const Text('Split terminal down'),
                  ),
                  const PopupMenuDivider(),
                  const PopupMenuItem(
                    value: 'copy',
                    child: Text('Copy selection'),
                  ),
                  PopupMenuItem(
                    value: 'share',
                    enabled: shareSelection != null,
                    child: const Text('Attach selection to conversation…'),
                  ),
                  const PopupMenuItem(
                    value: 'checkpoint',
                    child: Text('Save terminal recovery'),
                  ),
                  PopupMenuItem(
                    value: 'paste',
                    enabled: s?.live == true,
                    child: const Text('Paste'),
                  ),
                  PopupMenuItem(
                    value: 'stop',
                    enabled: s?.live == true,
                    child: const Text('Stop terminal'),
                  ),
                  for (final other in host.layout.groups.values.where(
                    (v) => v.id != g.id,
                  )) ...[
                    PopupMenuItem(
                      value: 'move:${other.id}',
                      child: Text('Move tab to ${other.id}'),
                    ),
                    PopupMenuItem(
                      value: 'focus:${other.id}',
                      child: Text('Focus ${other.id}'),
                    ),
                  ],
                  PopupMenuItem(
                    value: 'close-group',
                    enabled: host.layout.groups.length > 1,
                    child: const Text('Join this group'),
                  ),
                ],
              ),
            ],
          ),
          if (s != null)
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              child: Align(
                alignment: Alignment.centerLeft,
                child: Text(
                  '${s.state} · ${s.displayCwd}',
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              ),
            ),
          if (s?.notice.isNotEmpty == true)
            ConstrainedBox(
              constraints: const BoxConstraints(maxHeight: 90),
              child: SingleChildScrollView(
                child: Padding(
                  padding: const EdgeInsets.all(8),
                  child: Wrap(
                    children: [
                      Text(s!.notice),
                      if (s.suspended)
                        TextButton(
                          onPressed: () => host.retry(s),
                          child: const Text('Retry connection'),
                        ),
                    ],
                  ),
                ),
              ),
            ),
          Expanded(
            child: Stack(
              children: [
                Positioned.fill(
                  child: s == null ? recovery() : terminal(context, s, g),
                ),
                if (candidates.isNotEmpty) ...[
                  Positioned(
                    left: 0,
                    top: 0,
                    bottom: 0,
                    width: 60,
                    child: edge(context, g, 'Left', Axis.horizontal, true),
                  ),
                  Positioned(
                    right: 0,
                    top: 0,
                    bottom: 0,
                    width: 60,
                    child: edge(context, g, 'Right', Axis.horizontal, false),
                  ),
                  Positioned(
                    top: 0,
                    left: 60,
                    right: 60,
                    height: 50,
                    child: edge(context, g, 'Above', Axis.vertical, true),
                  ),
                  Positioned(
                    bottom: 0,
                    left: 60,
                    right: 60,
                    height: 50,
                    child: edge(context, g, 'Below', Axis.vertical, false),
                  ),
                ],
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget terminal(BuildContext context, TerminalSession s, FileGroup g) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return TerminalView(
      s.terminal,
      key: ValueKey(s.id),
      controller: s.controller,
      focusNode: s.focus,
      scrollController: s.scroll,
      autofocus: host.layout.activeGroup == g.id,
      readOnly: !s.live,
      textStyle: const TerminalStyle(fontFamily: 'Consolas', fontSize: 13),
      theme: TerminalTheme(
        cursor: p.text,
        selection: p.soft,
        foreground: p.text,
        background: p.bg,
        black: Colors.black,
        white: Colors.white,
        red: Colors.red,
        green: Colors.green,
        yellow: Colors.amber,
        blue: Colors.blue,
        magenta: Colors.purple,
        cyan: Colors.cyan,
        brightBlack: Colors.grey,
        brightWhite: Colors.white,
        brightRed: Colors.redAccent,
        brightGreen: Colors.lightGreen,
        brightYellow: Colors.yellow,
        brightBlue: Colors.lightBlue,
        brightMagenta: Colors.purpleAccent,
        brightCyan: Colors.cyanAccent,
        searchHitBackground: p.soft,
        searchHitBackgroundCurrent: p.accent,
        searchHitForeground: p.text,
      ),
      onTapUp: (_, _) => host.layout.focus(g.id),
      onKeyEvent: (node, event) {
        if (event is KeyDownEvent &&
            HardwareKeyboard.instance.isControlPressed &&
            event.logicalKey == LogicalKeyboardKey.keyC &&
            s.controller.selection != null) {
          unawaited(copy(s));
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
    );
  }

  Widget node(BuildContext context, FileLayoutNode n) {
    if (n.group != null) return group(context, host.layout.groups[n.group]!);
    return LayoutBuilder(
      builder: (context, size) {
        final length = n.axis == Axis.horizontal
            ? size.maxWidth
            : size.maxHeight;
        return Flex(
          direction: n.axis,
          children: [
            Expanded(
              flex: (n.ratio * 1000).round(),
              child: node(context, n.first!),
            ),
            GestureDetector(
              behavior: HitTestBehavior.opaque,
              onPanUpdate: (d) {
                n.ratio =
                    (n.ratio +
                            (n.axis == Axis.horizontal
                                    ? d.delta.dx
                                    : d.delta.dy) /
                                length)
                        .clamp(.15, .85);
                host.layout.changed();
              },
              child: MouseRegion(
                cursor: n.axis == Axis.horizontal
                    ? SystemMouseCursors.resizeLeftRight
                    : SystemMouseCursors.resizeUpDown,
                child: SizedBox(
                  width: n.axis == Axis.horizontal ? 7 : null,
                  height: n.axis == Axis.vertical ? 7 : null,
                  child: const ColoredBox(color: Colors.grey),
                ),
              ),
            ),
            Expanded(
              flex: ((1 - n.ratio) * 1000).round(),
              child: node(context, n.second!),
            ),
          ],
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: host,
    builder: (context, _) => Column(
      children: [
        if (host.error != null)
          ConstrainedBox(
            constraints: const BoxConstraints(maxHeight: 90),
            child: SingleChildScrollView(
              child: Padding(
                padding: const EdgeInsets.all(8),
                child: Text(host.error!),
              ),
            ),
          ),
        Expanded(
          child: LayoutBuilder(
            builder: (context, size) =>
                size.maxWidth < 640 || size.maxHeight < 350
                ? group(context, host.layout.active)
                : node(context, host.layout.tree),
          ),
        ),
      ],
    ),
  );
}
