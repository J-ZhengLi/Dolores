import 'package:flutter/material.dart';

import 'chat.dart';
import 'inspector.dart';

class ToolsHome extends StatefulWidget {
  final ChatController chat;
  final void Function(String) open;
  const ToolsHome({super.key, required this.chat, required this.open});
  @override
  State<ToolsHome> createState() => _ToolsHomeState();
}

class _ToolsHomeState extends State<ToolsHome> {
  final status = <String, String>{};
  @override
  void initState() {
    super.initState();
    load();
  }

  Future<void> load() async {
    await Future.wait(
      ['web', 'browser', 'desktop'].map((tool) async {
        try {
          final report = await widget.chat.bridge.call({
            'command': switch (tool) {
              'web' => 'webSettings',
              'browser' => 'browserSettings',
              _ => 'desktopState',
            },
            if (tool == 'desktop' && widget.chat.session != null)
              'session': widget.chat.session,
          }) as Map;
          if (mounted) {
            setState(
              () => status[tool] = tool == 'web'
                  ? (report['enabled'] == false ? 'Off' : 'Ready')
                  : (report['available'] == true ? 'Ready' : 'Needs setup'),
            );
          }
        } catch (_) {
          if (mounted) setState(() => status[tool] = 'Check setup');
        }
      }),
    );
  }

  @override
  Widget build(BuildContext context) => InspectorFrame(
    title: 'Tools',
    subtitle: 'Built-in tools and connections',
    child: ListView(
      padding: const EdgeInsets.all(16),
      children: [
        const Text('Use tools from a project or temporary chat.'),
        for (final tool in [
          ('web', 'Web search', Icons.travel_explore),
          ('browser', 'Browser', Icons.language),
          ('desktop', 'Computer use', Icons.desktop_windows_outlined),
        ])
          ListTile(
            leading: Icon(tool.$3),
            title: Text(tool.$2),
            subtitle: Text(status[tool.$1] ?? 'Checking…'),
            trailing: const Icon(Icons.chevron_right),
            onTap: () => widget.open(tool.$1),
          ),
        ListTile(
          leading: const Icon(Icons.extension_outlined),
          title: const Text('Connections'),
          subtitle: const Text('Add external tools'),
          trailing: const Icon(Icons.chevron_right),
          onTap: () => widget.open('tools'),
        ),
        ListTile(
          leading: const Icon(Icons.shield_outlined),
          title: const Text('Chat access'),
          subtitle: const Text('Review, custom grants or full access'),
          trailing: const Icon(Icons.chevron_right),
          onTap: () => widget.open('permissions'),
        ),
      ],
    ),
  );
}
