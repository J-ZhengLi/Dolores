import 'dart:async';

import 'package:flutter/foundation.dart';

import 'bridge.dart';

class CompanionHost extends ChangeNotifier {
  final ChatBridge bridge;
  final String? Function() session;
  final bool Function() busy;
  Map? state;
  String? error;
  bool pending = false, _closed = false;
  Timer? _timer;
  CompanionHost(this.bridge, {required this.session, required this.busy});
  Map? get unread => (state?['activity'] as List? ?? [])
      .cast<Map>()
      .where((a) => a['status'] == 'delivered' && a['seen'] != true)
      .lastOrNull;
  void _arm() {
    _timer?.cancel();
    if (!_closed && state?['policy']['enabled'] == true) {
      _timer = Timer(const Duration(seconds: 60), tick);
    }
  }

  Future<void> refresh() async {
    if (_closed || pending) return;
    pending = true;
    try {
      state =
          (await bridge.call({'command': 'companionState'}) as Map)['state']
              as Map;
      error = null;
    } catch (e) {
      error = '$e';
    } finally {
      pending = false;
      if (!_closed) {
        notifyListeners();
        _arm();
      }
    }
  }

  Future<void> tick() async {
    if (_closed || pending) return;
    pending = true;
    try {
      state =
          (await bridge.call({
                'command': 'companionTick',
                'session': session(),
                'busy': busy(),
              }) as Map)['state']
              as Map;
      error = null;
    } catch (e) {
      error = '$e';
    } finally {
      pending = false;
      if (!_closed) {
        notifyListeners();
        _arm();
      }
    }
  }

  Future<void> feedback(String id, String action) async {
    if (_closed || pending) return;
    pending = true;
    notifyListeners();
    try {
      state =
          (await bridge.call({
                'command': 'companionFeedback',
                'id': id,
                'action': action,
              }) as Map)['state']
              as Map;
      error = null;
    } catch (e) {
      error = '$e';
    } finally {
      pending = false;
      if (!_closed) {
        notifyListeners();
        _arm();
      }
    }
  }

  void suspend() {
    _timer?.cancel();
  }

  @override
  void dispose() {
    _closed = true;
    _timer?.cancel();
    super.dispose();
  }
}
