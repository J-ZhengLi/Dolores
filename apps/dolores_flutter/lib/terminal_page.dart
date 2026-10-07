import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:xterm/xterm.dart';

import 'terminal_host.dart';
import 'theme.dart';

class TerminalPage extends StatelessWidget {
  final TerminalHost host;
  final String? session;
  final VoidCallback chooseFolder;
  const TerminalPage({
    super.key,
    required this.host,
    required this.session,
    required this.chooseFolder,
  });
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

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: host,
    builder: (context, _) {
      final s = host.sessions[host.active];
      final p = Palette(Theme.of(context).brightness == Brightness.dark);
      return Column(
        children: [
          if (host.error != null)
            Padding(padding: const EdgeInsets.all(8), child: Text(host.error!)),
          if (s == null)
            Expanded(
              child: Center(
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
                        onPressed: () =>
                            unawaited(host.create(null, home: true)),
                        child: const Text('Open at home'),
                      ),
                  ],
                ),
              ),
            )
          else ...[
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 8),
              child: Row(
                children: [
                  Expanded(
                    child: Text(
                      '${s.title} · ${s.state}\n${s.cwd}',
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                  IconButton(
                    tooltip: 'Copy selection',
                    onPressed: () => copy(s),
                    icon: const Icon(Icons.copy_outlined, size: 18),
                  ),
                  IconButton(
                    tooltip: 'Paste',
                    onPressed: !s.live
                        ? null
                        : () async {
                            final data = await Clipboard.getData(
                              Clipboard.kTextPlain,
                            );
                            if (data?.text != null) {
                              s.terminal.paste(data!.text!);
                            }
                          },
                    icon: const Icon(Icons.content_paste, size: 18),
                  ),
                  IconButton(
                    tooltip: 'Stop terminal',
                    onPressed: !s.live ? null : () => host.stop(s),
                    icon: const Icon(Icons.stop_circle_outlined, size: 18),
                  ),
                  IconButton(
                    tooltip: 'Close terminal',
                    onPressed: () => close(context, s),
                    icon: const Icon(Icons.close, size: 18),
                  ),
                ],
              ),
            ),
            if (s.notice.isNotEmpty)
              Padding(
                padding: const EdgeInsets.all(8),
                child: Wrap(
                  children: [
                    Text(s.notice),
                    if (s.suspended)
                      TextButton(
                        onPressed: () => host.retry(s),
                        child: const Text('Retry connection'),
                      ),
                  ],
                ),
              ),
            Expanded(
              child: TerminalView(
                s.terminal,
                key: ValueKey(s.id),
                controller: s.controller,
                focusNode: s.focus,
                scrollController: s.scroll,
                autofocus: true,
                readOnly: !s.live,
                textStyle: const TerminalStyle(
                  fontFamily: 'Consolas',
                  fontSize: 13,
                ),
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
              ),
            ),
          ],
        ],
      );
    },
  );
}
