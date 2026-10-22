import 'package:flutter/material.dart';

import 'theme.dart';
import 'spring_press.dart';

class HomeLanding extends StatelessWidget {
  final Widget composer;
  final String title, subtitle;
  final List<Map<String, dynamic>> projects, sessions;
  final VoidCallback? openProject, connectModel;
  final ValueChanged<String> selectProject, selectChat, suggest;
  final bool canSuggest;
  const HomeLanding({
    super.key,
    required this.composer,
    required this.title,
    required this.subtitle,
    required this.projects,
    required this.sessions,
    required this.openProject,
    required this.connectModel,
    required this.selectProject,
    required this.selectChat,
    required this.suggest,
    required this.canSuggest,
  });

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return SingleChildScrollView(
      key: const Key('home-landing'),
      padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 24),
      child: Center(
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 760),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                title,
                style: TextStyle(
                  fontSize: 32,
                  fontWeight: FontWeight.w600,
                  letterSpacing: -1,
                  color: p.text,
                ),
              ),
              const SizedBox(height: 8),
              Text(subtitle, style: TextStyle(color: p.muted)),
              const SizedBox(height: 24),
              composer,
              const SizedBox(height: 12),
              Wrap(
                spacing: 8,
                runSpacing: 8,
                children: [
                  for (final entry in const [
                    (
                      'Make a plan',
                      Icons.checklist_rounded,
                      'Help me make a plan for ',
                    ),
                    ('Build something', Icons.code_rounded, 'Help me build '),
                    (
                      'Explore an idea',
                      Icons.lightbulb_outline,
                      'Help me explore ',
                    ),
                  ])
                    OutlinedButton.icon(
                      onPressed: canSuggest ? () => suggest(entry.$3) : null,
                      icon: Icon(entry.$2, size: 16),
                      label: Text(entry.$1),
                    ),
                  if (connectModel != null)
                    TextButton.icon(
                      onPressed: connectModel,
                      icon: const Icon(Icons.tune, size: 17),
                      label: const Text('Connect a model'),
                    ),
                ],
              ),
              const SizedBox(height: 32),
              Row(
                children: [
                  const Expanded(
                    child: Text(
                      'Your projects',
                      style: TextStyle(fontWeight: FontWeight.w600),
                    ),
                  ),
                  TextButton.icon(
                    key: const Key('welcome-open-project'),
                    onPressed: openProject,
                    icon: const Icon(Icons.add, size: 18),
                    label: const Text('Open project…'),
                  ),
                ],
              ),
              if (projects.isNotEmpty)
                LayoutBuilder(
                  builder: (context, constraints) {
                    final columns = constraints.maxWidth < 440 ? 1 : 3;
                    return Wrap(
                      spacing: 12,
                      runSpacing: 12,
                      children: [
                        for (final project in projects.take(3))
                          SizedBox(
                            width:
                                (constraints.maxWidth - (columns - 1) * 12) /
                                columns,
                            child: SpringPress(
                              child: Card(
                                elevation: 0,
                                margin: EdgeInsets.zero,
                                color: p.surface,
                                shape: RoundedRectangleBorder(
                                  borderRadius: BorderRadius.circular(14),
                                  side: BorderSide(color: p.border),
                                ),
                                clipBehavior: Clip.antiAlias,
                                child: InkWell(
                                  onTap: () =>
                                      selectProject(project['root'] as String),
                                  child: Column(
                                    crossAxisAlignment:
                                        CrossAxisAlignment.start,
                                    children: [
                                      Container(
                                        height: 76,
                                        width: double.infinity,
                                        decoration: p.sidebarDecoration,
                                        child: Icon(
                                          Icons.folder_open_outlined,
                                          size: 32,
                                          color: p.muted,
                                        ),
                                      ),
                                      Padding(
                                        padding: const EdgeInsets.all(14),
                                        child: Text(
                                          project['name'] as String,
                                          maxLines: 1,
                                          overflow: TextOverflow.ellipsis,
                                        ),
                                      ),
                                    ],
                                  ),
                                ),
                              ),
                            ),
                          ),
                      ],
                    );
                  },
                )
              else
                Padding(
                  padding: const EdgeInsets.symmetric(vertical: 12),
                  child: Text(
                    'Bring your files into the conversation.',
                    style: TextStyle(color: p.muted),
                  ),
                ),
              if (sessions.isNotEmpty) ...[
                const SizedBox(height: 32),
                const Text(
                  'Pick up where you left off',
                  style: TextStyle(fontWeight: FontWeight.w600),
                ),
                const SizedBox(height: 12),
                for (final session in sessions.take(3))
                  ListTile(
                    contentPadding: EdgeInsets.zero,
                    leading: Icon(
                      Icons.chat_bubble_outline,
                      size: 20,
                      color: p.muted,
                    ),
                    title: Text(
                      session['title'] as String,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                    trailing: const Icon(Icons.chevron_right, size: 18),
                    onTap: () => selectChat(session['id'] as String),
                  ),
              ],
            ],
          ),
        ),
      ),
    );
  }
}
