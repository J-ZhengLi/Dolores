import 'dart:convert';

import 'package:flutter/material.dart';

import 'chat.dart';
import 'settings_frame.dart';

class ConnectionDialog extends StatefulWidget {
  final ChatController chat;
  const ConnectionDialog({super.key, required this.chat});
  @override
  State<ConnectionDialog> createState() => _ConnectionDialogState();
}

class _ConnectionDialogState extends State<ConnectionDialog> {
  final _scroll = ScrollController();
  late final url = TextEditingController(text: widget.chat.baseUrl);
  final manualModel = TextEditingController();
  final search = TextEditingController();
  late List<String> available = [...widget.chat.enabledModels];
  late final Set<String> selected = {
    ...widget.chat.enabledModels,
    if (widget.chat.model.isNotEmpty) widget.chat.model,
  };
  final keyInput = TextEditingController();
  late bool remember =
      widget.chat.rememberConnection || widget.chat.model.isEmpty;
  bool saving = false;
  String? error, notice;
  bool fetching = false, manual = false;
  bool clearKey = false;
  late String? contextModel = widget.chat.model.isEmpty
      ? null
      : widget.chat.model;
  bool contextEndpointChanged = false;
  late final Set<String> imageModels = widget.chat.imageModels.toSet();
  final contextInputs = <String, TextEditingController>{};
  TextEditingController contextInput(String id) => contextInputs.putIfAbsent(
    id,
    () => TextEditingController(
      text: contextEndpointChanged
          ? ''
          : widget.chat.modelContexts[id]?.toString() ?? '',
    ),
  );
  bool get working => saving || fetching;
  String? get requestKey =>
      !clearKey &&
          keyInput.text.isEmpty &&
          (widget.chat.hasSavedKey || widget.chat.configured)
      ? null
      : keyInput.text;
  void endpointChanged(String _) {
    setState(() {
      available = [];
      selected.clear();
      contextModel = null;
      contextEndpointChanged = true;
      imageModels.clear();
      for (final input in contextInputs.values) {
        input.dispose();
      }
      contextInputs.clear();
      search.clear();
      error = null;
    });
  }

  void addManual() {
    final id = manualModel.text.trim();
    if (id.isEmpty) return;
    setState(() {
      if (utf8.encode(id).length > 200 || selected.length >= 32) {
        error = 'Choose up to 32 models, with IDs up to 200 bytes.';
        return;
      }
      if (!available.contains(id)) available.add(id);
      selected.add(id);
      manualModel.clear();
      error = null;
    });
  }

  Future<void> fetch() async {
    setState(() {
      fetching = true;
      error = null;
    });
    try {
      final models = await widget.chat.listModels(url.text, requestKey);
      if (mounted) {
        setState(() {
          available = {...models, ...selected}.toList()..sort();
          if (selected.isEmpty) selected.add(models.first);
        });
      }
    } catch (failure) {
      if (mounted) {
        setState(() {
          error = failure.toString();
          manual = true;
        });
      }
    } finally {
      if (mounted) setState(() => fetching = false);
    }
  }

  @override
  void dispose() {
    _scroll.dispose();
    url.dispose();
    manualModel.dispose();
    search.dispose();
    keyInput.dispose();
    for (final input in contextInputs.values) {
      input.dispose();
    }
    super.dispose();
  }

  Future<void> save() async {
    setState(() {
      saving = true;
      error = null;
      notice = null;
    });
    try {
      final contexts = <String, int?>{};
      for (final id in selected) {
        final text = contextInput(id).text.trim();
        final tokens = int.tryParse(text);
        if (text.isNotEmpty &&
            (tokens == null || tokens < 1024 || tokens > 16777216)) {
          throw FormatException(
            'Context window for $id must be a whole number between 1024 and 16777216 tokens, or blank.',
          );
        }
        contexts[id] = text.isEmpty ? null : tokens;
      }
      await widget.chat.configure(
        url.text,
        selected.contains(widget.chat.model)
            ? widget.chat.model
            : selected.first,
        requestKey,
        remember: remember,
        models: selected.toList(),
        contexts: contexts,
      );
      if (widget.chat.attachmentsAvailable) {
        await widget.chat.bridge.call({
          'command': 'setImageModels',
          'models': imageModels.where(selected.contains).toList(),
        });
        await widget.chat.refresh();
      }
      if (mounted) {
        if (SettingsEmbedding.of(context) == null) {
          Navigator.pop(context);
        } else {
          setState(() {
            saving = false;
            keyInput.clear();
            clearKey = false;
            notice = 'Connection saved.';
          });
        }
      }
    } catch (failure) {
      if (mounted) {
        setState(() {
          error = failure is FormatException
              ? failure.message
              : failure.toString();
          saving = false;
        });
        if (_scroll.hasClients) _scroll.jumpTo(0);
      }
    }
  }

  Future<void> forget() async {
    setState(() {
      saving = true;
      error = null;
    });
    try {
      await widget.chat.forgetConnection();
      if (mounted) {
        if (SettingsEmbedding.of(context) == null) {
          Navigator.pop(context);
        } else {
          setState(() {
            saving = false;
            notice = 'Saved connection forgotten.';
          });
        }
      }
    } catch (failure) {
      if (mounted) {
        setState(() {
          saving = false;
          error = failure.toString();
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) => PopScope(
    canPop: !working,
    child: SettingsFormFrame(
      title: 'Connection & models',
      canClose: !working,
      content: SingleChildScrollView(
        controller: _scroll,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const Text(
              'Use an OpenAI-compatible local or hosted API.',
              style: TextStyle(fontSize: 13),
            ),
            if (notice != null)
              Padding(
                padding: const EdgeInsets.only(bottom: 12),
                child: Text(notice!),
              ),
            if (error != null)
              Padding(
                padding: const EdgeInsets.only(top: 16),
                child: Text(
                  error!,
                  style: TextStyle(
                    color: Theme.of(context).colorScheme.error,
                    fontSize: 13,
                  ),
                ),
              ),
            if (selected.isNotEmpty) ...[
              const SizedBox(height: 12),
              DropdownButtonFormField<String>(
                key: ValueKey(
                  'context-model-${selected.contains(contextModel) ? contextModel : selected.first}',
                ),
                initialValue: selected.contains(contextModel)
                    ? contextModel
                    : selected.first,
                isExpanded: true,
                decoration: const InputDecoration(labelText: 'Model settings'),
                items: [
                  for (final id in selected)
                    DropdownMenuItem(
                      value: id,
                      child: Text(id, overflow: TextOverflow.ellipsis),
                    ),
                ],
                onChanged: working
                    ? null
                    : (id) => setState(() => contextModel = id),
              ),
              if (widget.chat.attachmentsAvailable && selected.isNotEmpty)
                CheckboxListTile(
                  key: const Key('model-image-input'),
                  contentPadding: EdgeInsets.zero,
                  title: const Text('Supports image input'),
                  subtitle: const Text(
                    'Enable for a model that accepts images from your provider. Images are sent at low detail.',
                  ),
                  value: imageModels.contains(
                    selected.contains(contextModel)
                        ? contextModel
                        : selected.first,
                  ),
                  onChanged: working
                      ? null
                      : (enabled) => setState(() {
                          final id = selected.contains(contextModel)
                              ? contextModel!
                              : selected.first;
                          enabled == true
                              ? imageModels.add(id)
                              : imageModels.remove(id);
                        }),
                ),
              const SizedBox(height: 12),
              TextField(
                key: const Key('context-window'),
                controller: contextInput(
                  selected.contains(contextModel)
                      ? contextModel!
                      : selected.first,
                ),
                enabled: !working,
                keyboardType: TextInputType.number,
                decoration: const InputDecoration(
                  labelText: 'Context window (tokens)',
                  hintText: '131072 (128K default)',
                  helperText: 'Blank uses 128K tokens. Override with your provider’s limit.',
                  helperMaxLines: 2,
                ),
              ),
              const SizedBox(height: 12),
            ],
            const Divider(height: 24),
            TextField(
              key: const Key('base-url'),
              controller: url,
              enabled: !working,
              onChanged: endpointChanged,
              decoration: const InputDecoration(
                labelText: 'Base URL',
                hintText: 'http://localhost:11434/v1',
              ),
            ),
            const SizedBox(height: 18),
            TextField(
              key: const Key('api-key'),
              controller: keyInput,
              enabled: !working,
              obscureText: true,
              enableSuggestions: false,
              autocorrect: false,
              onChanged: (_) => setState(() => clearKey = false),
              decoration: InputDecoration(
                labelText: 'API key (optional for local servers)',
                helperText: clearKey
                    ? 'This connection will use no key.'
                    : widget.chat.hasSavedKey || widget.chat.configured
                    ? 'Leave blank to keep the current key.'
                    : null,
              ),
            ),
            if (widget.chat.hasSavedKey || widget.chat.configured)
              Align(
                alignment: Alignment.centerLeft,
                child: TextButton(
                  onPressed: working
                      ? null
                      : () => setState(() {
                          clearKey = !clearKey;
                          keyInput.clear();
                        }),
                  child: Text(
                    clearKey ? 'Keep existing key' : 'Use without a key',
                  ),
                ),
              ),
            const SizedBox(height: 18),
            OutlinedButton.icon(
              key: const Key('fetch-models'),
              onPressed: working ? null : fetch,
              icon: fetching
                  ? const SizedBox(
                      width: 14,
                      height: 14,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : const Icon(Icons.refresh, size: 16),
              label: Text(fetching ? 'Fetching models…' : 'Fetch models'),
            ),
            if (available.isNotEmpty) ...[
              const SizedBox(height: 12),
              Text(
                '${selected.length} selected · Available in the chat model picker',
                style: const TextStyle(fontSize: 12),
              ),
              const SizedBox(height: 8),
              TextField(
                key: const Key('model-search'),
                controller: search,
                onChanged: (_) => setState(() {}),
                enabled: !working,
                decoration: const InputDecoration(
                  hintText: 'Search models',
                  prefixIcon: Icon(Icons.search, size: 18),
                ),
              ),
              const SizedBox(height: 6),
              SizedBox(
                height: (available.length * 44.0).clamp(44, 176),
                child: Builder(
                  builder: (_) {
                    final filtered = available
                        .where(
                          (id) => id.toLowerCase().contains(
                            search.text.toLowerCase(),
                          ),
                        )
                        .toList();
                    return ListView.builder(
                      itemCount: filtered.length,
                      itemBuilder: (_, index) {
                        final id = filtered[index];
                        return CheckboxListTile(
                          key: ValueKey('enable-model-$id'),
                          dense: true,
                          contentPadding: EdgeInsets.zero,
                          title: Text(
                            id,
                            maxLines: 2,
                            overflow: TextOverflow.ellipsis,
                            style: const TextStyle(fontSize: 13),
                          ),
                          value: selected.contains(id),
                          onChanged: working
                              ? null
                              : (value) => setState(() {
                                  if (value! && selected.length >= 32) {
                                    error = 'Choose up to 32 models.';
                                  } else {
                                    value
                                        ? selected.add(id)
                                        : selected.remove(id);
                                    error = null;
                                  }
                                }),
                        );
                      },
                    );
                  },
                ),
              ),
            ],
            TextButton(
              key: const Key('manual-model-toggle'),
              onPressed: working
                  ? null
                  : () => setState(() => manual = !manual),
              child: const Text('Add a model manually'),
            ),
            if (manual)
              Row(
                children: [
                  Expanded(
                    child: TextField(
                      key: const Key('manual-model'),
                      controller: manualModel,
                      enabled: !working,
                      onSubmitted: (_) => addManual(),
                      decoration: const InputDecoration(labelText: 'Model ID'),
                    ),
                  ),
                  IconButton(
                    key: const Key('add-manual-model'),
                    tooltip: 'Add model',
                    onPressed: working ? null : addManual,
                    icon: const Icon(Icons.add),
                  ),
                ],
              ),
            CheckboxListTile(
              key: const Key('remember-connection'),
              contentPadding: EdgeInsets.zero,
              title: const Text(
                'Remember connection',
                style: TextStyle(fontSize: 14),
              ),
              subtitle: Text(
                remember
                    ? 'Store your key in the OS credential store.'
                    : 'Use this connection until Dolores closes.',
                style: const TextStyle(fontSize: 12),
              ),
              value: remember,
              onChanged: working
                  ? null
                  : (value) => setState(() => remember = value!),
            ),
            if (widget.chat.rememberConnection)
              TextButton(
                key: const Key('forget-connection'),
                onPressed: working ? null : forget,
                child: const Text('Forget saved connection'),
              ),
          ],
        ),
      ),
      actions: [
        if (SettingsEmbedding.of(context) == null)
          TextButton(
            onPressed: working ? null : () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
        FilledButton(
          key: const Key('save-connection'),
          onPressed: working || selected.isEmpty ? null : save,
          child: Text(saving ? 'Saving…' : 'Save connection'),
        ),
      ],
    ),
  );
}
