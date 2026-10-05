import 'mcp_import.dart';
import 'settings_frame.dart';

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
  bool editing = false;
  String? baseline;
  String get draft => jsonEncode([
    _name.text,
    _program.text,
    _args.map((a) => a.text).toList(),
    _credentials.map((r) => [r.name.text, r.value.text]).toList(),
  ]);
  bool get dirty => baseline != null && (draft != baseline || _review != null);
  Future<bool> saveDraft() async {
    if (_review == null || _selected.isEmpty) {
      setState(
        () => _error = 'Review the launch and choose tools before saving. Nothing was started.',
      );
      return false;
    }
    await _enable();
    return !dirty;
  }

  Future<void> switchConnection({bool fresh = false, String? id}) async {
    if (dirty && !await resolveSettingsDraft(context, save: saveDraft)) return;
    final token = _review?['token'];
    if (token != null) await _call('discardMcpReview', {'token': token});
    await _act(() => _load(newConnection: fresh, chooseId: id));
    if (mounted) setState(() => editing = true);
  }

  Future<void> importConnection() async {
    if (dirty && !await resolveSettingsDraft(context, save: saveDraft)) return;
    if (!mounted) return;
    final value = await showMcpImport(context);
    if (value == null || !mounted) return;
    final token = _review?['token'];
    if (token != null) await _call('discardMcpReview', {'token': token});
    await _act(() => _load(newConnection: true));
    if (!mounted) return;
    setState(() {
      editing = true;
      _name.text = value['label'];
      _program.text = value['executable'];
      for (final arg in value['args']) {
        _args.add(TextEditingController(text: arg));
      }
      for (final entry in (value['env'] as Map).entries) {
        final row = _CredentialRow(entry.key);
        row.value.text = entry.value;
        _credentials.add(row);
      }
    });
  }

  final _name = TextEditingController(), _program = TextEditingController();
  final _args = <TextEditingController>[];
  final _credentials = <_CredentialRow>[];
  final _scroll = ScrollController();
  Map<String, dynamic>? _connection, _review;
  List<Map<String, dynamic>> _connections = [];
  String? _editingId;
  int get _otherTools => _connections
      .where((c) => (c['id'] ?? 'legacy') != _editingId && c['enabled'] == true)
      .fold(0, (n, c) => n + (c['tools'] as List).length);
  final _selected = <String>{};
  bool _busy = false, _stopping = false;
  int? _run;
  String? _error, _notice;
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

  Future<void> _load({bool newConnection = false, String? chooseId}) async {
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
    final entries =
        (result['connections'] as List? ??
                [if (result['connection'] != null) result['connection']])
            .map((c) => (c as Map).cast<String, dynamic>())
            .toList();
    var id = newConnection
        ? ''
        : chooseId ??
              _editingId ??
              (entries.isEmpty
                  ? ''
                  : (entries.first['id'] as String? ?? 'legacy'));
    if (id.isNotEmpty && !entries.any((c) => (c['id'] ?? 'legacy') == id)) {
      id = '';
    }
    final saved = entries.where((c) => (c['id'] ?? 'legacy') == id).firstOrNull;
    setState(() {
      for (final binding in saved?['credentials'] as List? ?? []) {
        _credentials.add(_CredentialRow(binding['name'] as String));
      }
      _connections = entries;
      _editingId = id;
      _connection = saved;

      _review = null;
      _selected.clear();
      _name.text = saved?['launch']['label'] as String? ?? '';
      _program.text = saved?['launch']['executable'] as String? ?? '';
      for (final arg in saved?['launch']['args'] as List? ?? []) {
        _args.add(TextEditingController(text: arg as String));
      }
    });
    baseline = draft;
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
  Future<void> _inspect() async {
    final allowed = await showDialog<bool>(
      context: context,
      barrierDismissible: false,
      builder: (context) => AlertDialog(
        title: const Text('Review connection launch'),
        content: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text(
                'Starts this installed program with your OS permissions to discover its tools. It may access files or the network.',
              ),
              const SizedBox(height: 12),
              SelectableText(_program.text),
              for (var i = 0; i < _args.length; i++)
                SelectableText('${i + 1}. ${jsonEncode(_args[i].text)}'),
              if (_credentials.isNotEmpty)
                Text(
                  'Credentials sent to this program: ${_credentials.map((r) => r.name.text).join(', ')}',
                ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancel'),
          ),
          FilledButton(
            key: const Key('mcp-launch-confirm'),
            onPressed: () => Navigator.pop(context, true),
            child: const Text('Inspect tools'),
          ),
        ],
      ),
    );
    if (allowed != true || !mounted) return;
    await _inspectApproved();
  }

  Future<void> _inspectApproved() => _act(() async {
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
      'connectionId': _editingId ?? '',
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
    _editingId =
        (result as Map?)?['id'] as String? ??
        _review!['connectionId'] as String? ??
        _editingId;
    await _load();
    if (mounted) {
      setState(
        () => _notice = result?['warning'] as String? ?? 'Selected tools enabled for this folder. Chat access controls each request.',
      );
      _status();
    }
  });
  Future<void> _mutate(bool forget) => _act(() async {
    try {
      await _call(forget ? 'forgetMcp' : 'disableMcp', {
        'revision': _connection!['revision'],
        'connectionId': _editingId,
      });
    } catch (error) {
      await _load(); // Forget may have disabled tools before a vault failure.
      rethrow;
    }
    if (forget) _editingId = null;
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
  Future<void> _disableOther(Map<String, dynamic> server) => _act(() async {
    final result = await _call('disableMcp', {
      'connectionId': server['id'] ?? 'legacy',
      'revision': server['revision'],
    });
    if (!mounted) return;
    setState(() {
      if (result is Map && result['connections'] is List) {
        _connections = (result['connections'] as List)
            .map((c) => (c as Map).cast<String, dynamic>())
            .toList();
      } else {
        server['enabled'] = false;
        server['revision'] = (server['revision'] as int) + 1;
      }
      _notice = 'Server disabled. Your current fields and review are preserved; retry Enable or adjust the selected tools.';
    });
    _status();
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
    reportSettingsDraft(context, dirty: () => dirty, save: saveDraft);
    return PopScope(
      canPop: !_busy,
      child: InspectorFrame(
        title: 'Connections',
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
                  const Text('Connect external tools to this working folder.'),
                  const SizedBox(height: 12),
                  Text(
                    'Servers · ${_connections.length}/4 saved · ${_connections.where((c) => c['enabled'] == true).fold<int>(0, (n, c) => n + (c['tools'] as List).length)}/2 external tools enabled',
                  ),
                  for (final server in _connections)
                    ListTile(
                      key: Key('mcp-server-${server['id'] ?? 'legacy'}'),
                      contentPadding: EdgeInsets.zero,
                      selected: (server['id'] ?? 'legacy') == _editingId,
                      leading: Icon(
                        server['enabled'] == true
                            ? Icons.check_circle_outline
                            : Icons.pause_circle_outline,
                      ),
                      title: Text(server['launch']['label'] as String),
                      trailing:
                          server['enabled'] == true &&
                              (server['id'] ?? 'legacy') != _editingId
                          ? TextButton(
                              key: Key(
                                'mcp-disable-other-${server['id'] ?? 'legacy'}',
                              ),
                              onPressed: _busy
                                  ? null
                                  : () => _disableOther(server),
                              child: const Text('Disable'),
                            )
                          : null,
                      subtitle: Text(
                        '${server['enabled'] == true ? 'Enabled' : 'Disabled'} · ${(server['tools'] as List).map((t) => t['name']).join(', ')}',
                      ),
                      onTap: _busy
                          ? null
                          : () => switchConnection(
                              id: server['id'] as String? ?? 'legacy',
                            ),
                    ),
                  Align(
                    alignment: Alignment.centerLeft,
                    child: TextButton(
                      key: const Key('mcp-add-server'),
                      onPressed: _busy || _connections.length >= 4
                          ? null
                          : () => switchConnection(fresh: true),
                      child: const Text('Add connection'),
                    ),
                  ),
                  Align(
                    alignment: Alignment.centerLeft,
                    child: TextButton.icon(
                      key: const Key('mcp-import'),
                      onPressed: _busy || _connections.length >= 4
                          ? null
                          : importConnection,
                      icon: const Icon(Icons.file_download_outlined),
                      label: const Text('Import JSON'),
                    ),
                  ),
                  if (editing || _connection != null) ...[
                    if (_connection == null)
                      const Text('New connection · inspect before enabling'),
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
                      decoration: const InputDecoration(
                        labelText: 'Server name',
                      ),
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
                                    WidgetsBinding.instance
                                        .addPostFrameCallback(
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
                          'Up to four saved connections per folder, sharing two enabled MCP tools. Inspection and each approved call start a fresh server and stop it afterward; no idle server or automatic startup. 30 seconds per operation, up to 32 tools / four pages / 32 KiB for discovery, and 8 KiB text results. Up to eight explicit credential bindings (4 KiB each / 16 KiB total), stored in the native vault. No general environment editor, remote transport, images/resources, sampling or task execution in this first connection.',
                          style: TextStyle(color: p.muted, fontSize: 12),
                        ),
                      ],
                    ),
                    if (_review != null) ...[
                      Text(
                        '${2 - _otherTools} external tool slots available. If Enable exceeds the limit, select fewer tools or disable another server; the current review stays available.',
                      ),
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
                                              _error = 'Choose at most two MCP tools. This folder shares two external tool slots across servers.';
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
                    onPressed: _busy
                        ? null
                        : () => switchConnection(id: _editingId),
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
                    onPressed: _busy || (!editing && _connection == null)
                        ? null
                        : _inspect,
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
