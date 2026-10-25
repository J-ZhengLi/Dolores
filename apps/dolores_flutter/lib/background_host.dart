import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

import 'bridge.dart';

abstract interface class AvailabilityWindow {
  Future<bool> available();
  Future<void> hide();
  Future<void> restore();
  void listen(Future<void> Function(String) callback);
  void dispose();
}

class NativeAvailabilityWindow implements AvailabilityWindow {
  static const channel = MethodChannel('dolores/availability');
  @override
  Future<bool> available() async =>
      !kIsWeb &&
      defaultTargetPlatform == TargetPlatform.windows &&
      await channel.invokeMethod<bool>('status') == true;
  @override
  Future<void> hide() => channel.invokeMethod<void>('hide');
  @override
  Future<void> restore() => channel.invokeMethod<void>('restore');
  @override
  void listen(Future<void> Function(String) callback) {
    if (!kIsWeb && defaultTargetPlatform == TargetPlatform.windows) {
      channel.setMethodCallHandler((call) async {
        await callback(call.method);
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

class BackgroundHost extends ChangeNotifier {
  final ChatBridge bridge;
  final AvailabilityWindow window;
  final Future<void> Function() restored;
  final Future<void> Function() quit;
  Map policy = {'enabled': false, 'revision': 1};
  bool supported = false, hidden = false, pending = false, _closed = false;
  String? error;
  bool get enabled => supported && policy['enabled'] == true;
  BackgroundHost(
    this.bridge, {
    AvailabilityWindow? window,
    required this.restored,
    required this.quit,
  }) : window = window ?? NativeAvailabilityWindow() {
    this.window.listen((event) async {
      if (_closed) return;
      if (event == 'restored') {
        hidden = false;
        notifyListeners();
        await restored();
      } else if (event == 'quit') {
        await quit();
      }
    });
  }
  Future<void> refresh() async {
    if (_closed || pending) return;
    pending = true;
    try {
      policy = await bridge.call({'command': 'backgroundPolicy'}) as Map;
      supported = await window.available();
      error = null;
    } catch (e) {
      error = '$e';
      supported = false;
    } finally {
      pending = false;
      if (!_closed) notifyListeners();
    }
  }

  Future<bool> minimize() async {
    try {
      if (!await window.available()) return false;
      await window.hide().timeout(const Duration(milliseconds: 500));
      hidden = true;
      error = null;
      notifyListeners();
      return true;
    } catch (_) {
      error = 'Could not create the tray icon. Dolores remains open.';
      try {
        await window.restore().timeout(const Duration(milliseconds: 500));
      } catch (_) {}
      notifyListeners();
      return false;
    }
  }

  @override
  void dispose() {
    _closed = true;
    window.dispose();
    super.dispose();
  }
}
