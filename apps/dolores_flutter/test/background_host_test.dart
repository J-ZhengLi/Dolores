import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/background_host.dart';

import 'workspace_test.dart' show WorkspaceBridge;

class BackgroundBridge extends WorkspaceBridge {
  Map policy = {'enabled': true, 'revision': 2};
  @override
  Future<dynamic> call(Map<String, dynamic> c) async {
    if (c['command'] == 'backgroundPolicy') return policy;
    return super.call(c);
  }
}

class TestWindow implements AvailabilityWindow {
  int hides = 0, opens = 0;
  bool fail = false;
  Future<void> Function(String)? event;
  @override
  Future<bool> available() async => true;
  @override
  Future<void> hide() async {
    if (fail) throw StateError('Tray unavailable');
    hides++;
  }

  @override
  Future<void> restore() async {
    opens++;
    await event?.call('restored');
  }

  @override
  void listen(Future<void> Function(String) callback) {
    event = callback;
  }

  @override
  void dispose() {
    event = null;
  }
}

void main() {
  test(
    'Close retains owner, deliberate restore and Quit stay distinct',
    () async {
      final b = BackgroundBridge(), w = TestWindow();
      var restored = 0, quit = 0;
      final h = BackgroundHost(
        b,
        window: w,
        restored: () async {
          restored++;
        },
        quit: () async {
          quit++;
        },
      );
      await h.refresh();
      expect(await h.closeToTray(() async => true), true);
      expect(h.hidden, true);
      expect(w.hides, 1);
      await w.event!('restored');
      expect(h.hidden, false);
      expect(restored, 1);
      expect(quit, 0);
      await w.event!('quit');
      expect(quit, 1);
      h.dispose();
    },
  );
  test('Blocked recovery and failed tray preserve visible ownership', () async {
    final b = BackgroundBridge(), w = TestWindow();
    var resumed = 0;
    final h = BackgroundHost(
      b,
      window: w,
      restored: () async {
        resumed++;
      },
      quit: () async {},
    );
    await h.refresh();
    expect(await h.closeToTray(() async => false), false);
    expect(w.hides, 0);
    w.fail = true;
    expect(await h.closeToTray(() async => true), false);
    expect(h.hidden, false);
    expect(h.error, contains('Keep Dolores open'));
    expect(resumed, 1);
    h.dispose();
  });
  test(
    'Turning off a hidden worker restores its UI and preserves policy',
    () async {
      final b = BackgroundBridge(), w = TestWindow();
      final h = BackgroundHost(
        b,
        window: w,
        restored: () async {},
        quit: () async {},
      );
      await h.refresh();
      await h.closeToTray(() async => true);
      b.policy = {'enabled': false, 'revision': 3};
      await h.refresh();
      expect(h.enabled, false);
      expect(h.hidden, false);
      expect(w.opens, 1);
      h.dispose();
    },
  );
}
