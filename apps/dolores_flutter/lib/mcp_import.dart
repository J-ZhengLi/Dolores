import 'dart:convert';

import 'package:flutter/material.dart';

/// Imports configuration only. Inspection remains a separate launch decision.
List<Map<String, dynamic>> parseMcpImport(String text) {
  if (utf8.encode(text).length > 65536) {
    throw const FormatException('Choose a configuration smaller than 64 KiB.');
  }
  final dynamic decoded;
  try {
    decoded = jsonDecode(text);
  } catch (_) {
    throw const FormatException('Paste a valid MCP JSON configuration.');
  }
  if (decoded is! Map || decoded.length != 1 || decoded['mcpServers'] is! Map) {
    throw const FormatException(
      'Expected an mcpServers object with local connections.',
    );
  }
  final servers = decoded['mcpServers'] as Map;
  if (servers.isEmpty || servers.length > 4) {
    throw const FormatException('Import one to four local connections.');
  }
  return servers.entries.map((entry) {
    final value = entry.value;
    if (entry.key is! String ||
        (entry.key as String).trim().isEmpty ||
        value is! Map ||
        value.keys.any(
          (k) => !['command', 'args', 'env', 'type'].contains(k),
        ) ||
        (value['type'] != null && value['type'] != 'stdio') ||
        value['command'] is! String ||
        (value['command'] as String).trim().isEmpty ||
        (value['args'] != null &&
            (value['args'] is! List ||
                (value['args'] as List).length > 32 ||
                (value['args'] as List).any((a) => a is! String))) ||
        (value['env'] != null &&
            (value['env'] is! Map ||
                (value['env'] as Map).length > 8 ||
                (value['env'] as Map).entries.any(
                  (e) =>
                      e.key is! String ||
                      !RegExp(r'^[A-Za-z_][A-Za-z0-9_]{0,63}$')
                          .hasMatch(e.key) ||
                      e.value is! String,
                )))) {
      throw const FormatException(
        'Use local command, string arguments and named credentials. Remote URLs are unsupported.',
      );
    }
    return <String, dynamic>{
      'label': entry.key,
      'executable': value['command'],
      'args': value['args'] ?? <String>[],
      'env': value['env'] ?? <String, String>{},
    };
  }).toList();
}

Future<Map<String, dynamic>?> showMcpImport(BuildContext context) async {
  final controller = TextEditingController();
  String? error;
  List<Map<String, dynamic>>? entries;
  Map<String, dynamic>? selected;
  ModalRoute<dynamic>? route;
  try {
    return await showDialog<Map<String, dynamic>>(
      context: context,
      barrierDismissible: false,
      builder: (context) {
        route = ModalRoute.of(context);
        return StatefulBuilder(
          builder: (context, setState) => AlertDialog(
            title: const Text('Import connection'),
            content: SizedBox(
              width: 520,
              child: SingleChildScrollView(
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text(
                      'Paste local MCP JSON. Nothing runs until you review the launch.',
                    ),
                    const SizedBox(height: 12),
                    if (entries == null)
                      TextField(
                        key: const Key('mcp-import-json'),
                        controller: controller,
                        minLines: 4,
                        maxLines: 8,
                        autocorrect: false,
                        enableSuggestions: false,
                        decoration: const InputDecoration(
                          labelText: 'Configuration JSON',
                        ),
                      ),
                    if (error != null) Text(error!),
                    if (entries != null) ...[
                      RadioGroup<Map<String, dynamic>>(
                        groupValue: selected,
                        onChanged: (v) => setState(() => selected = v),
                        child: Column(
                          children: [
                            for (final entry in entries!)
                              RadioListTile<Map<String, dynamic>>(
                                title: Text(entry['label'] as String),
                                subtitle: Text(entry['executable'] as String),
                                value: entry,
                              ),
                          ],
                        ),
                      ),
                      const Text(
                        'Imported credential values stay masked in the editor. Review them before inspection.',
                      ),
                    ],
                  ],
                ),
              ),
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(context),
                child: const Text('Cancel'),
              ),
              FilledButton(
                key: const Key('mcp-import-review'),
                onPressed: () {
                  if (entries != null) {
                    if (selected != null) Navigator.pop(context, selected);
                    return;
                  }
                  try {
                    final parsed = parseMcpImport(controller.text);
                    controller.clear();
                    setState(() {
                      entries = parsed;
                      selected = parsed.first;
                      error = null;
                    });
                  } catch (_) {
                    setState(
                      () => error = 'Invalid local MCP configuration. Check command, arguments and credential names. Nothing was imported.',
                    );
                  }
                },
                child: Text(
                  entries == null ? 'Review configuration' : 'Use connection',
                ),
              ),
            ],
          ),
        );
      },
    );
  } finally {
    await route?.completed;
    controller.dispose();
  }
}
