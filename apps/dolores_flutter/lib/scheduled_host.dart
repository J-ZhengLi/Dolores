import 'dart:async';

import 'package:flutter/foundation.dart';

import 'bridge.dart';

/// One app-open clock; the existing chat owners execute and poll each run.
class ScheduledHost extends ChangeNotifier {
  final ChatBridge bridge;
  final Future<void> Function(Map occurrence) onClaim;
  List<Map> items = [];
  String availability = 'Runs while Dolores is open.';
  String filter = 'all';
  List<Map> get visibleItems => items.where((item) {
    final task = item['task'] as Map;
    return switch (filter) {
      'paused' => task['paused'] == true,
      'active' => task['paused'] != true && task['nextDue'] != null,
      'finished' => task['nextDue'] == null && task['paused'] != true,
      _ => true,
    };
  }).toList();
  void setFilter(String value) {
    filter = value;
    notifyListeners();
  }

  String? error;
  bool pending = false, _closed = false, _suspended = false;
  Timer? _timer;
  Completer<void>? _idle;
  ScheduledHost(this.bridge, {required this.onClaim});
  Future<dynamic> call(
    String command, [
    Map<String, dynamic> fields = const {},
  ]) => bridge.call({'command': command, ...fields});
  bool get needsClock => items.any((item) {
    final task = item['task'] as Map;
    final active = (item['occurrences'] as List).any(
      (o) => [
        'claimed',
        'queued',
        'running',
        'waitingForApproval',
      ].contains(o['state']),
    );
    return active || (task['paused'] != true && task['nextDue'] != null);
  });
  void suspend() {
    _suspended = true;
    _timer?.cancel();
  }

  Future<void> quiesce() async {
    suspend();
    if (_idle != null) {
      await _idle!.future.timeout(const Duration(seconds: 2));
    }
  }

  void _begin() {
    pending = true;
    _idle = Completer<void>();
  }

  void _end() {
    pending = false;
    _idle?.complete();
    _idle = null;
  }

  Future<void> resume() async {
    _suspended = false;
    await refresh();
  }

  void _arm() {
    _timer?.cancel();
    if (!_closed && !_suspended && needsClock) {
      _timer = Timer(const Duration(seconds: 15), tick);
    }
  }

  void _accept(dynamic data) {
    items = ((data as Map)['items'] as List).cast<Map>();
    availability = data['backgroundEnabled'] == true
        ? 'Runs while Dolores is open or in the tray.'
        : 'Runs while Dolores is open.';
  }

  Future<void> refresh() async {
    if (_closed || pending) return;
    _begin();
    try {
      _accept(await call('scheduledTasks'));
      error = null;
    } catch (e) {
      error = '$e';
    } finally {
      _end();
      if (!_closed) {
        notifyListeners();
        _arm();
      }
    }
  }

  Future<void> tick() async {
    if (_closed || _suspended || pending) return;
    _begin();
    try {
      final data = await call('scheduledTick') as Map;
      _accept(data['tasks']);
      for (final occurrence in (data['claimed'] as List).cast<Map>()) {
        try {
          await onClaim(occurrence);
        } catch (e) {
          await call('scheduledAbandon', {
            'occurrence': occurrence['id'],
            'error': '$e',
          });
        }
      }
      _accept(await call('scheduledTasks'));
      error = null;
    } catch (e) {
      error = '$e';
    } finally {
      _end();
      if (!_closed) {
        notifyListeners();
        _arm();
      }
    }
  }

  Future<void> manage(Map task, String action) async {
    if (_closed || pending) return;
    _begin();
    notifyListeners();
    try {
      final result = await call('scheduledManage', {
        'task': task['id'],
        'revision': task['revision'],
        'action': action,
      });
      if (action == 'runNow') {
        for (final occurrence
            in ((result as Map)['claimed'] as List).cast<Map>()) {
          try {
            await onClaim(occurrence);
          } catch (e) {
            await call('scheduledAbandon', {
              'occurrence': occurrence['id'],
              'error': '$e',
            });
          }
        }
      }
      _accept(await call('scheduledTasks'));
      error = null;
    } catch (e) {
      error = '$e';
    } finally {
      _end();
      if (!_closed) {
        notifyListeners();
        _arm();
      }
    }
  }

  @override
  void dispose() {
    _closed = true;
    _timer?.cancel();
    super.dispose();
  }
}
