import 'package:flutter/material.dart';
import 'package:file_selector/file_selector.dart';

import 'chat.dart';
import 'inspector.dart';
import 'settings_frame.dart';

Future<void> showCapabilities(BuildContext context, ChatController chat) =>
    showDialog<void>(
      context: context,
      builder: (_) => CapabilitiesInspector(chat: chat),
    );

class CapabilitiesInspector extends StatefulWidget {
  final ChatController chat;
  const CapabilitiesInspector({super.key, required this.chat});
  @override
  State<CapabilitiesInspector> createState() => _CapabilitiesInspectorState();
}

class _CapabilitiesInspectorState extends State<CapabilitiesInspector> {
  Map<String, dynamic>? inventory, source;
  String? error, selected, checkout;
  bool pending = false;
  int line = 1;
  @override
  void initState() {
    super.initState();
    refresh();
  }

  Future<void> refresh() async {
    setState(() {
      pending = true;
      error = null;
    });
    try {
      final result = await widget.chat.bridge.call({
        'command': 'harnessInventory',
        'session': widget.chat.session,
      });
      if (mounted) {
        setState(() {
          inventory = Map<String, dynamic>.from(result as Map);
        });
      }
    } catch (failure) {
      if (mounted) {
        setState(() {
          error = failure.toString();
        });
      }
    } finally {
      if (mounted) {
        setState(() {
          pending = false;
        });
      }
    }
  }

  Future<void> readSource({bool next = false}) async {
    if (selected == null) return;
    setState(() {
      pending = true;
      error = null;
    });
    final start = next ? (source?['nextLine'] as int? ?? 1) : line;
    try {
      final result = await widget.chat.bridge.call({
        'command': 'harnessSource',
        'source': selected,
        'startLine': start,
        'lineCount': 60,
        if (checkout != null) 'checkout': checkout,
      });
      if (mounted) {
        setState(() {
          source = Map<String, dynamic>.from(result as Map);
          line = start;
        });
      }
    } catch (failure) {
      if (mounted) {
        setState(() {
          error = failure.toString();
        });
      }
    } finally {
      if (mounted) {
        setState(() {
          pending = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final data = inventory;
    final tools = (data?['tools'] as List?) ?? [];
    return InspectorFrame(
      title: 'Capabilities & source',
      subtitle: 'Inspect this app locally',
      child: Column(
        children: [
          Expanded(
            child: SingleChildScrollView(
              padding: const EdgeInsets.all(20),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  if (error != null) SelectableText(error!),
                  if (data != null) ...[
                    SelectableText(
                      'Dolores ${data['version']} · ${data['workspace'] ?? 'No saved session'}',
                    ),
                    const SizedBox(height: 12),
                    SettingsDetails(
                      title: 'Runtime details',
                      children: [
                        SelectableText(
                          'Model: ${data['model']}\nContext: ${data['contextWindowTokens']} tokens (${data['contextOrigin']})\nApproval: ${data['approval']}\n${data['containment']}',
                        ),
                        const SizedBox(height: 12),
                        SelectableText(
                          'Run: ${data['limits']['modelCalls'] ?? 'Automatic'} model calls / ${data['limits']['toolOperations'] ?? 'Automatic'} tool operations\nSelf-updates: ${data['selfUpdate']}',
                        ),
                      ],
                    ),
                    if (data['extensions'] is List)
                      ExpansionTile(
                        tilePadding: EdgeInsets.zero,
                        title: const Text('Extension registry'),
                        subtitle: Text(
                          'Host API ${data['extensionApi']} · versions are pinned for each run',
                        ),
                        children: [
                          for (final entry in data['extensions'] as List)
                            ListTile(
                              contentPadding: EdgeInsets.zero,
                              title: Text(
                                '${entry['descriptor']['id']} · ${entry['active'] == true ? 'Available' : 'Unavailable'}',
                              ),
                              subtitle: Text(
                                '${entry['descriptor']['kind']} · version ${entry['descriptor']['version']} · configuration ${entry['descriptor']['configRevision']}\n${entry['reason'].toString().isEmpty ? entry['descriptor']['health'] : entry['reason']}',
                              ),
                            ),
                        ],
                      ),
                    if (tools.isEmpty)
                      SelectableText(data['unavailableReason'] as String),
                    for (final tool in tools)
                      ListTile(
                        contentPadding: EdgeInsets.zero,
                        title: Text(tool['id'] as String),
                        subtitle: Text(
                          tool['available'] == true
                              ? 'Available after review'
                              : 'Needs model connection',
                        ),
                      ),
                    SettingsDetails(
                      title: 'Tool evidence',
                      children: [
                        for (final tool in tools)
                          SelectableText(
                            '${tool['id']} · ${tool['source']}\nModel reliability: ${tool['empiricallyTested']}',
                          ),
                      ],
                    ),
                    const Divider(),
                    SettingsDetails(
                      title: 'Browse matching source',
                      children: [
                        const Text(
                          'Bundled source matches this build. A checkout is only compared; it never replaces this source.',
                        ),
                        DropdownButtonFormField<String>(
                          initialValue: selected,
                          decoration: const InputDecoration(
                            labelText: 'Source component',
                          ),
                          items: [
                            for (final item in data['sources'] as List)
                              DropdownMenuItem(
                                value: item['id'] as String,
                                child: Text(item['id'] as String),
                              ),
                          ],
                          onChanged: pending
                              ? null
                              : (value) {
                                  setState(() {
                                    selected = value;
                                    source = null;
                                    line = 1;
                                  });
                                  readSource();
                                },
                        ),
                      ],
                    ),
                    if (source != null) ...[
                      SelectableText(
                        '${source!['path']}\n${source!['sourceId']} · lines ${source!['startLine']}–${(source!['nextLine'] as int) - 1} / ${source!['totalLines']}',
                      ),
                      if (source!['checkoutStatus'] != null)
                        SelectableText(
                          'Selected checkout: ${source!['checkoutStatus']}',
                        ),
                      const SizedBox(height: 8),
                      SelectableText(source!['text'] as String),
                    ],
                  ],
                  if (pending) const LinearProgressIndicator(),
                ],
              ),
            ),
          ),
          Padding(
            padding: const EdgeInsets.all(12),
            child: Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                TextButton(
                  onPressed: pending ? null : refresh,
                  child: const Text('Refresh'),
                ),
                TextButton(
                  onPressed: pending || selected == null
                      ? null
                      : () async {
                          final folder = await getDirectoryPath();
                          if (!mounted || folder == null) return;
                          setState(() {
                            checkout = folder;
                          });
                          await readSource();
                        },
                  child: const Text('Compare checkout'),
                ),
                TextButton(
                  onPressed:
                      pending ||
                          source == null ||
                          (source!['nextLine'] as int) >
                              (source!['totalLines'] as int)
                      ? null
                      : () => readSource(next: true),
                  child: const Text('Next source lines'),
                ),
                TextButton(
                  onPressed: pending || line == 1
                      ? null
                      : () {
                          setState(() {
                            line = 1;
                          });
                          readSource();
                        },
                  child: const Text('First lines'),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
