import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';

import 'bridge.dart';
import 'chat.dart';
import 'inspector.dart';
import 'theme.dart';

Future<void> showMcp(BuildContext context, ChatController chat) =>
    chat.inspectLocalSettings(() async {
      if (chat.session == null || chat.workspaceRoot == null) return;
      await showDialog<void>(
        context: context,
        barrierDismissible: false,
        builder: (_) =>
            McpInspector(bridge: chat.bridge, session: chat.session!),
      );
      chat.invalidateContext();
    });

class McpInspector extends StatefulWidget {
  final ChatBridge bridge;
  final String session;
  final Future<String?> Function()? chooseExecutable;
  const McpInspector({
    super.key,
    required this.bridge,
    required this.session,
    this.chooseExecutable,
  });
  @override
  State<McpInspector> createState() => _McpInspectorState();
}

class _CredentialRow {
  final name = TextEditingController(), value = TextEditingController();
  String? savedName;
  _CredentialRow([this.savedName]) {
    name.text = savedName ?? '';
  }
  void dispose() {
    name.dispose();
    value.clear();
    value.dispose();
  }
}

class _McpInspectorState extends State<McpInspector> {
  final _name = TextEditingController(), _program = TextEditingController();
  final _args = <TextEditingController>[];
  final _credentials = <_CredentialRow>[];
  final _scroll = ScrollController();
  Map<String, dynamic>? _connection, _review;
  final _selected = <String>{};
  bool _busy = false, _stopping = false;
  int? _run;
  String? _error, _notice, _directory;
  Future<dynamic> _call(
    String command, [
    Map<String, dynamic> fields = const {},
  ]) => widget.bridge.call({
    'command': command,
    'session': widget.session,
    ...fields,
  });
  @override
  void initState() {
    super.initState();
    _act(_load);
  }

  void _status() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted && _scroll.hasClients) _scroll.jumpTo(0);
    });
  }

  Future<void> _act(Future<void> Function() action) async {
    if (_busy) return;
    setState(() {
      _busy = true;
      _error = null;
      _notice = null;
    });
    try {
      await action();
    } catch (error) {
      if (mounted) {
        setState(() => _error = error.toString());
        _status();
      }
    } finally {
      if (mounted) {
        setState(() {
          _busy = false;
          _run = null;
        });
      }
    }
  }

  Future<void> _load() async {
    final result = await _call('mcpSettings') as Map;
    if (!mounted) return;
    for (final arg in _args) {
      arg.dispose();
    }
    _args.clear();
    for (final row in _credentials) {
      row.dispose();
    }
    _credentials.clear();
    final saved = (result['connection'] as Map?)?.cast<String, dynamic>();
    setState(() {
      for (final binding in saved?['credentials'] as List? ?? []) {
        _credentials.add(_CredentialRow(binding['name'] as String));
      }
      _connection = saved;
      _directory = result['directory'] as String?;
      _review = null;
      _selected.clear();
      _name.text = saved?['launch']['label'] as String? ?? '';
      _program.text = saved?['launch']['executable'] as String? ?? '';
      for (final arg in saved?['launch']['args'] as List? ?? []) {
        _args.add(TextEditingController(text: arg as String));
      }
    });
    _status();
  }

  void _edited() {
    final token = _review?['token'];
    setState(() {
      _review = null;
      _selected.clear();
      _error = null;
      _notice = null;
    });
    if (token != null) {
      unawaited(
        _call('discardMcpReview', {'token': token}).catchError((_) => null),
      );
    }
  }

  Future<void> _choose() => _act(() async {
    final value = widget.chooseExecutable != null
        ? await widget.chooseExecutable!()
        : (await openFile(
            acceptedTypeGroups: Platform.isWindows
                ? [
                    const XTypeGroup(label: 'Program', extensions: ['exe']),
                  ]
                : [],
          ))?.path;
    if (value != null && mounted) {
      _program.text = value;
      _edited();
    }
  });
  Future<void> _inspect() => _act(() async {
    if (_name.text.trim().isEmpty || _program.text.trim().isEmpty) {
      throw 'Choose a server name and its direct executable first.';
    }
    final token = _review?['token'];
    _review = null;
    _selected.clear();
    if (token != null) await _call('discardMcpReview', {'token': token});
    final id = DateTime.now().microsecondsSinceEpoch;
    _stopping = false;
    setState(() => _run = id);
    await _call('inspectMcp', {
      'id': id,
      'credentials': _credentials
          .map(
            (row) => {
              'name': row.name.text.trim(),
              'value':
                  row.value.text.isEmpty &&
                      row.savedName == row.name.text.trim()
                  ? null
                  : row.value.text,
            },
          )
          .toList(),
      'launch': {
        'label': _name.text.trim(),
        'executable': _program.text,
        'args': _args.map((c) => c.text).toList(),
      },
    });
    final deadline = DateTime.now().add(const Duration(seconds: 35));
    while (DateTime.now().isBefore(deadline)) {
      final events = await _call('poll', {'id': id}) as List;
      for (final event in events) {
        if (event['type'] != 'done') continue;
        if (event['error'] != null) throw event['error'] as String;
        final review = (event['mcpInspection'] as Map).cast<String, dynamic>();
        if (_stopping) {
          await _call('discardMcpReview', {'token': review['token']});
          throw 'Inspection stopped. Nothing was saved.';
        }
        if (mounted) {
          setState(() {
            _review = review;
            _notice = 'Server inspected and stopped. Choose up to two tools to enable.';
          });
          _status();
        }
        return;
      }
      await Future<void>.delayed(const Duration(milliseconds: 40));
    }
    await _call('cancel', {'id': id});
    throw 'Inspection timed out. Nothing was enabled; inspect again when the server is ready.';
  });
  Future<void> _stop() async {
    if (_run == null || _stopping) return;
    setState(() => _stopping = true);
    try {
      await _call('cancel', {'id': _run});
    } catch (error) {
      if (mounted) {
        setState(() {
          _error = error.toString();
          _stopping = false;
        });
        _status();
      }
    }
  }

  Future<void> _enable() => _act(() async {
    if (_review == null || _selected.isEmpty) return;
    final result = await _call('enableMcp', {
      'token': _review!['token'],
      'names': _selected.toList(),
    });
    await _load();
    if (mounted) {
      setState(
        () => _notice = (result as Map?)?['warning'] as String? ?? 'Selected tools enabled for this folder. Each call still needs approval.',
      );
      _status();
    }
  });
  Future<void> _mutate(bool forget) => _act(() async {
    try {
      await _call(forget ? 'forgetMcp' : 'disableMcp', {
        'revision': _connection!['revision'],
      });
    } catch (error) {
      await _load(); // Forget may have disabled tools before a vault failure.
      rethrow;
    }
    await _load();
    if (mounted) {
      setState(
        () => _notice = forget
            ? 'Connection forgotten. External files are unchanged.'
            : 'MCP tools disabled for this folder.',
      );
      _status();
    }
  });
  @override
  void dispose() {
    final token = _review?['token'];
    if (token != null) {
      unawaited(
        _call('discardMcpReview', {'token': token}).catchError((_) => null),
      );
    }
    _name.dispose();
    _program.dispose();
    for (final arg in _args) {
      arg.dispose();
    }
    for (final row in _credentials) {
      row.dispose();
    }
    _scroll.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return PopScope(
      canPop: !_busy,
      child: InspectorFrame(
        title: 'External tools',
        subtitle: 'MCP · this working folder',
        canClose: !_busy,
        child: Column(
          children: [
            if (_busy) const LinearProgressIndicator(minHeight: 2),
            Expanded(
              child: ListView(
                key: const Key('mcp-scroll'),
                controller: _scroll,
                padding: const EdgeInsets.all(20),
                children: [
                  if (_error != null) ...[
                    SelectableText(
                      _error!,
                      style: TextStyle(color: p.errorText),
                    ),
                    const SizedBox(height: 12),
                  ],
                  if (_notice != null) ...[
                    Text(_notice!),
                    const SizedBox(height: 12),
                  ],
                  const Text(
                    'Inspect server starts the program below to list its tools. It runs with your OS permissions and may access files or the network. Connect only a program you trust. Tool approval controls what Dolores requests; it does not sandbox the program.',
                  ),
                  const SizedBox(height: 12),
                  if (_directory != null)
                    SelectableText(
                      _directory!,
                      style: TextStyle(color: p.muted, fontSize: 12),
                    ),
                  if (_connection != null) ...[
                    const SizedBox(height: 12),
                    Text(
                      '${_connection!['enabled'] == true ? 'Enabled' : 'Disabled'} · ${_connection!['launch']['label']}',
                    ),
                    Text(
                      'Saved tools: ${(_connection!['tools'] as List).map((t) => t['name']).join(', ')}',
                      style: TextStyle(color: p.muted),
                    ),
                  ],
                  const SizedBox(height: 16),
                  TextField(
                    key: const Key('mcp-name'),
                    controller: _name,
                    enabled: !_busy,
                    onChanged: (_) => _edited(),
                    maxLength: 128,
                    decoration: const InputDecoration(labelText: 'Server name'),
                  ),
                  const SizedBox(height: 12),
                  TextField(
                    key: const Key('mcp-program'),
                    controller: _program,
                    enabled: !_busy,
                    onChanged: (_) => _edited(),
                    decoration: const InputDecoration(
                      labelText: 'Executable',
                      hintText: 'Choose a direct installed program',
                    ),
                  ),
                  Align(
                    alignment: Alignment.centerLeft,
                    child: TextButton(
                      key: const Key('mcp-choose-program'),
                      onPressed: _busy ? null : _choose,
                      child: const Text('Choose executable'),
                    ),
                  ),
                  const Text(
                    'Arguments are passed literally, in order. No shell command, package installation or variable expansion. Keep secrets out of these saved fields.',
                  ),
                  for (var i = 0; i < _args.length; i++)
                    Padding(
                      padding: const EdgeInsets.only(top: 8),
                      child: Row(
                        children: [
                          Expanded(
                            child: TextField(
                              key: Key('mcp-arg-$i'),
                              controller: _args[i],
                              enabled: !_busy,
                              onChanged: (_) => _edited(),
                              decoration: InputDecoration(
                                labelText: 'Argument ${i + 1}',
                              ),
                            ),
                          ),
                          IconButton(
                            tooltip: 'Remove argument ${i + 1}',
                            onPressed: _busy
                                ? null
                                : () {
                                    final removed = _args.removeAt(i);
                                    _edited();
                                    WidgetsBinding.instance
                                        .addPostFrameCallback(
                                          (_) => removed.dispose(),
                                        );
                                  },
                            icon: const Icon(Icons.remove_circle_outline),
                          ),
                        ],
                      ),
                    ),
                  Align(
                    alignment: Alignment.centerLeft,
                    child: TextButton(
                      key: const Key('mcp-add-argument'),
                      onPressed: _busy || _args.length >= 32
                          ? null
                          : () {
                              _args.add(TextEditingController());
                              _edited();
                            },
                      child: const Text('Add argument'),
                    ),
                  ),
                  const SizedBox(height: 12),
                  const Text('Credentials'),
                  const Text(
                    'Optional keys for this server. Inspect sends them to the program; Enable saves them in secure storage. Saved keys stay masked. Leave a saved value blank to reuse it, or remove its row to revoke it when enabling. A changed program needs the key entered again.',
                  ),
                  for (var i = 0; i < _credentials.length; i++) ...[
                    const SizedBox(height: 12),
                    Row(
                      children: [
                        Expanded(
                          child: TextField(
                            key: Key('mcp-credential-name-$i'),
                            controller: _credentials[i].name,
                            enabled: !_busy,
                            maxLength: 64,
                            autocorrect: false,
                            enableSuggestions: false,
                            onChanged: (_) => _edited(),
                            decoration: const InputDecoration(
                              labelText: 'Environment name',
                              hintText: 'SERVICE_API_KEY',
                            ),
                          ),
                        ),
                        IconButton(
                          tooltip: 'Remove credential ${i + 1}',
                          onPressed: _busy
                              ? null
                              : () {
                                  final removed = _credentials.removeAt(i);
                                  _edited();
                                  WidgetsBinding.instance.addPostFrameCallback(
                                    (_) => removed.dispose(),
                                  );
                                },
                          icon: const Icon(Icons.remove_circle_outline),
                        ),
                      ],
                    ),
                    const SizedBox(height: 8),
                    TextField(
                      key: Key('mcp-credential-value-$i'),
                      controller: _credentials[i].value,
                      enabled: !_busy,
                      obscureText: true,
                      autocorrect: false,
                      enableSuggestions: false,
                      onChanged: (_) => _edited(),
                      decoration: InputDecoration(
                        labelText: 'Secret value',
                        hintText:
                            _credentials[i].savedName ==
                                _credentials[i].name.text.trim()
                            ? 'Saved securely · leave blank to reuse'
                            : 'Required for a new credential',
                      ),
                    ),
                  ],
                  Align(
                    alignment: Alignment.centerLeft,
                    child: TextButton(
                      key: const Key('mcp-add-credential'),
                      onPressed: _busy || _credentials.length >= 8
                          ? null
                          : () {
                              _credentials.add(_CredentialRow());
                              _edited();
                            },
                      child: const Text('Add credential'),
                    ),
                  ),
                  ExpansionTile(
                    tilePadding: EdgeInsets.zero,
                    title: const Text('Limits and supported features'),
                    children: [
                      Text(
                        'One connection per folder, up to two enabled MCP tools. Inspection and each approved call start a fresh server and stop it afterward; no idle server or automatic startup. 30 seconds per operation, up to 32 tools / four pages / 32 KiB for discovery, and 8 KiB text results. Up to eight explicit credential bindings (4 KiB each / 16 KiB total), stored in the native vault. No general environment editor, remote transport, images/resources, sampling or task execution in this first connection.',
                        style: TextStyle(color: p.muted, fontSize: 12),
                      ),
                    ],
                  ),
                  if (_review != null) ...[
                    Text(
                      '${_review!['serverName']} · ${_review!['serverVersion']} · MCP ${_review!['protocolVersion']}',
                    ),
                    const SizedBox(height: 8),
                    for (final tool in _review!['tools'] as List)
                      Card(
                        elevation: 0,
                        color: p.surface,
                        child: Column(
                          children: [
                            CheckboxListTile(
                              key: Key('mcp-tool-${tool['name']}'),
                              title: Text(tool['name'] as String),
                              subtitle: Text(tool['description'] as String),
                              value: _selected.contains(tool['name']),
                              onChanged: _busy
                                  ? null
                                  : (checked) {
                                      setState(() {
                                        if (checked == true) {
                                          if (_selected.length >= 2) {
                                            _error =
                                                'Choose at most two MCP tools.';
                                            _status();
                                          } else {
                                            _selected.add(
                                              tool['name'] as String,
                                            );
                                            _error = null;
                                          }
                                        } else {
                                          _selected.remove(tool['name']);
                                          _error = null;
                                        }
                                      });
                                    },
                            ),
                            ExpansionTile(
                              title: const Text('Input schema'),
                              children: [
                                Padding(
                                  padding: const EdgeInsets.all(12),
                                  child: SelectableText(
                                    const JsonEncoder.withIndent('  ')
                                        .convert(tool['inputSchema']),
                                  ),
                                ),
                              ],
                            ),
                          ],
                        ),
                      ),
                  ],
                ],
              ),
            ),
            Divider(height: 1, color: p.border),
            Padding(
              padding: const EdgeInsets.all(12),
              child: Wrap(
                spacing: 8,
                runSpacing: 8,
                children: [
                  TextButton(
                    key: const Key('mcp-refresh'),
                    onPressed: _busy ? null : () => _act(_load),
                    child: const Text('Refresh'),
                  ),
                  if (_connection != null)
                    TextButton(
                      key: const Key('mcp-forget'),
                      onPressed: _busy ? null : () => _mutate(true),
                      child: const Text('Forget'),
                    ),
                  if (_connection?['enabled'] == true)
                    TextButton(
                      key: const Key('mcp-disable'),
                      onPressed: _busy ? null : () => _mutate(false),
                      child: const Text('Disable'),
                    ),
                  if (_run != null)
                    TextButton(
                      key: const Key('mcp-stop'),
                      onPressed: _stopping ? null : _stop,
                      child: Text(_stopping ? 'Stopping…' : 'Stop'),
                    ),
                  FilledButton(
                    key: const Key('mcp-inspect'),
                    onPressed: _busy ? null : _inspect,
                    child: const Text('Inspect server'),
                  ),
                  if (_review != null)
                    FilledButton(
                      key: const Key('mcp-enable'),
                      onPressed: _busy || _selected.isEmpty ? null : _enable,
                      child: const Text('Enable selected tools'),
                    ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
