import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

import 'bridge.dart';

abstract interface class AttentionWindow {
  Future<bool> available();
  Future<String> show(String id, String kind);
  void listen(Future<void> Function(String) open);
  void dispose();
}

class NativeAttentionWindow implements AttentionWindow {
  static const channel = MethodChannel('dolores/attention');
  @override
  Future<bool> available() async =>
      !kIsWeb &&
      defaultTargetPlatform == TargetPlatform.windows &&
      await channel.invokeMethod<bool>('status') == true;
  @override
  Future<String> show(String id, String kind) async =>
      await channel.invokeMethod<String>('show', {'id': id, 'kind': kind}) ??
      'uncertain';
  @override
  void listen(Future<void> Function(String) open) {
    if (!kIsWeb && defaultTargetPlatform == TargetPlatform.windows) {
      channel.setMethodCallHandler((call) async {
        if (call.method == 'opened' && call.arguments is String) {
          await open(call.arguments as String);
        }
      });
    }
  }

  @override
  void dispose() {
    if (!kIsWeb && defaultTargetPlatform == TargetPlatform.windows) {
      channel.setMethodCallHandler(null);
    }
  }
}

String attentionLabel(String kind) => switch (kind) {
  'completed' => 'Task completed',
  'failed' => 'Task needs attention',
  'approval' => 'Review requested',
  'question' => 'Question asked',
  _ => 'Task attention',
};

class AttentionHost extends ChangeNotifier {
  final ChatBridge bridge;
  final AttentionWindow window;
  final Future<bool> Function(Map<String, dynamic>) onOpen;
  bool pending = false, closed = false, supported = false;
  String? error, deferred;
  Map<String, dynamic> preferences = {
    'revision': 0,
    'enabled': false,
    'completed': true,
    'failed': true,
    'approval': true,
    'question': true,
  };
  List<Map<String, dynamic>> items = [];
  int? cursor;
  int omitted = 0;
  String signature = '';
  Timer? timer;
  int get unread => items.where((n) => n['seen'] != true).length;
  AttentionHost(this.bridge, {AttentionWindow? window, required this.onOpen})
    : window = window ?? NativeAttentionWindow() {
    this.window.listen(open);
  }
  Future<dynamic> call(Map<String, dynamic> request) =>
      bridge.call({'command': 'attention', 'request': request});
  void observe(String next) {
    if (closed || signature == next) return;
    signature = next;
    if (timer?.isActive == true) return;
    timer = Timer(
      const Duration(milliseconds: 250),
      () => unawaited(refresh()),
    );
  }

  Future<void> refresh({bool next = false}) async {
    if (closed || pending) return;
    pending = true;
    try {
      final page =
          await call({'action': 'poll', 'cursor': next ? cursor : null}) as Map;
      if (closed) return;
      preferences = Map<String, dynamic>.from(page['preferences'] as Map);
      items = (page['items'] as List)
          .map((n) => Map<String, dynamic>.from(n as Map))
          .toList();
      cursor = page['nextCursor'] as int?;
      omitted = page['omitted'] as int? ?? 0;
      error = null;
      try {
        supported = await window.available();
      } catch (_) {
        supported = false;
      }
      if (closed) return;
      if (preferences['enabled'] == true) {
        // Bounded delivery per refresh. Further pending entries remain visible;
        // immutable claims prevent duplicate notifications after lost replies.
        for (final notice
            in ((page['pendingDelivery'] as List?) ?? items)
                .cast<Map>()
                .where((n) => n['delivery'] == 'pending' && n['seen'] != true)
                .take(2)) {
          final id = notice['id'] as String;
          if (closed) break;
          final claim = await call({'action': 'claim', 'id': id});
          if (claim == null) continue;
          var delivery = 'failed';
          try {
            delivery = closed
                ? 'suppressed'
                : supported
                ? await window.show(id, notice['kind'] as String)
                : 'failed';
          } catch (_) {
            delivery = 'uncertain';
          }
          if (![
            'accepted',
            'failed',
            'suppressed',
            'uncertain',
          ].contains(delivery)) {
            delivery = 'uncertain';
          }
          try {
            await call({'action': 'finish', 'id': id, 'delivery': delivery});
            notice['delivery'] = delivery;
            for (final item in items.where((n) => n['id'] == id)) {
              item['delivery'] = delivery;
            }
          } catch (e) {
            notice['delivery'] = 'uncertain';
            error =
                '$e Notification handoff is uncertain. Use the in-app entry; it will not be resent automatically.';
          }
        }
      }
      if (!closed &&
          (page['moreSources'] == true ||
              (page['pendingDelivery'] as List? ?? []).isNotEmpty ||
              items.any(
                (n) => n['delivery'] == 'pending' && n['seen'] != true,
              ))) {
        timer?.cancel();
        timer = Timer(const Duration(seconds: 1), () => unawaited(refresh()));
      }
    } catch (e) {
      if (!closed) {
        error =
            '$e Task results and decisions remain in their conversations. Retry Attention.';
      }
    } finally {
      pending = false;
      if (!closed) notifyListeners();
    }
  }

  Future<void> save(Map<String, dynamic> value) async {
    if (closed || pending) return;
    pending = true;
    notifyListeners();
    var saved = false;
    try {
      preferences = Map<String, dynamic>.from(
        await call({'action': 'savePreferences', 'preferences': value}) as Map,
      );
      error = null;
      saved = true;
    } catch (e) {
      error = '$e The previous notification choice remains.';
    } finally {
      pending = false;
      if (!closed) notifyListeners();
    }
    if (!closed && saved) await refresh();
  }

  Future<void> open(String id) async {
    if (closed) return;
    try {
      final target = Map<String, dynamic>.from(
        await call({'action': 'resolve', 'id': id}) as Map,
      );
      if (closed) return;
      if (!await onOpen(target)) {
        deferred = id;
        error = 'Finish the open dialog, then open this Attention entry. Its target is retained.';
      } else {
        deferred = null;
        final seen = await call({'action': 'seen', 'id': id}) as Map;
        for (final item in items.where((n) => n['id'] == id)) {
          item['seen'] = seen['seen'];
          item['delivery'] = seen['delivery'];
        }
        error = null;
      }
    } catch (e) {
      if (!closed) {
        error = '$e Search the conversation or inspect its saved run history.';
      }
    }
    if (!closed) notifyListeners();
  }

  Future<void> seen(String id) async {
    try {
      await call({'action': 'seen', 'id': id});
      await refresh();
    } catch (e) {
      error = '$e';
      if (!closed) notifyListeners();
    }
  }

  @override
  void dispose() {
    closed = true;
    timer?.cancel();
    window.dispose();
    super.dispose();
  }
}
