import 'dart:async';

import 'package:dolores_flutter/attention_host.dart';
import 'package:dolores_flutter/attention_panel.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'app_host_test.dart' show HostBridge;
import 'frost_design_test.dart' show capture, viewport;
import 'support/render_fonts.dart';

class AttentionBridge extends HostBridge {
  final attentionCalls = <Map<String, dynamic>>[];
  bool failFinish = false, failSave = false, live = true, failPoll = false;
  Map<String, dynamic> prefs = {
    'revision': 0,
    'enabled': true,
    'completed': true,
    'failed': true,
    'approval': true,
    'question': true,
  };
  final notices = <Map<String, dynamic>>[
    {
      'id': '11111111-1111-4111-8111-111111111111',
      'session': 'task',
      'run': '22222222-2222-4222-8222-222222222222',
      'sequence': 4,
      'kind': 'question',
      'createdAt': 1,
      'seen': false,
      'delivery': 'pending',
    },
  ];
  @override
  Future<dynamic> call(Map<String, dynamic> c) async {
    attentionCalls.add(c);
    if (c['command'] != 'attention') return super.call(c);
    final r = c['request'] as Map;
    final n = notices.first;
    switch (r['action']) {
      case 'poll':
        if (failPoll) throw StateError('Attention unavailable');
        return {
          'preferences': {...prefs},
          'items': notices.map((n) => {...n}).toList(),
          'nextCursor': null,
          'moreSources': false,
          'omitted': 0,
          'pendingDelivery': notices
              .where((n) => n['delivery'] == 'pending' && n['seen'] != true)
              .map((n) => {...n})
              .toList(),
        };
      case 'claim':
        if (n['delivery'] != 'pending') return null;
        n['delivery'] = 'claimed';
        return {...n};
      case 'finish':
        if (failFinish) throw StateError('Delivery receipt unavailable');
        n['delivery'] = r['delivery'];
        return {...n};
      case 'savePreferences':
        if (failSave) throw StateError('Preference storage unavailable');
        prefs = {
          ...r['preferences'] as Map<String, dynamic>,
          'revision': (prefs['revision'] as int) + 1,
        };
        return {...prefs};
      case 'resolve':
        return {
          'notice': {...n},
          'liveDecision': live,
          'currentState': live ? 'waitingForApproval' : 'completed',
          'navigationOnly': true,
          'note': 'Opening never approves or answers.',
        };
      case 'seen':
        n['seen'] = true;
        return {...n};
    }
    throw StateError('Unexpected attention action');
  }
}

class NoticeWindow implements AttentionWindow {
  final shown = <String>[];
  bool supported = true, throws = false;
  String delivery = 'accepted';
  Future<void> Function(String)? opened;
  Completer<bool>? availability;
  @override
  Future<bool> available() async => availability?.future ?? supported;
  @override
  Future<String> show(String id, String kind) async {
    shown.add('$id:$kind');
    if (throws) throw StateError('Native handoff unavailable');
    return delivery;
  }

  @override
  void listen(Future<void> Function(String) open) {
    opened = open;
  }

  @override
  void dispose() {
    opened = null;
  }
}

void main() {
  setUpAll(loadReviewFonts);
  testWidgets('disposing during native availability stops claims and timers', (
    t,
  ) async {
    final b = AttentionBridge();
    final w = NoticeWindow()..availability = Completer<bool>();
    final h = AttentionHost(b, window: w, onOpen: (_) async => true);
    final refreshing = h.refresh();
    await t.pump();
    h.dispose();
    w.availability!.complete(true);
    await refreshing;
    expect(w.shown, isEmpty);
    expect(
      b.attentionCalls.where((c) => c['request']?['action'] == 'claim'),
      isEmpty,
    );
    expect(h.timer?.isActive, isNot(true));
  });
  testWidgets(
    'notification handoff is claimed once and click resolves current state without answering',
    (t) async {
      final b = AttentionBridge();
      final w = NoticeWindow();
      final opened = <Map<String, dynamic>>[];
      final h = AttentionHost(
        b,
        window: w,
        onOpen: (target) async {
          opened.add(target);
          return true;
        },
      );
      await h.refresh();
      await h.refresh();
      expect(w.shown, hasLength(1));
      expect(h.items.single['delivery'], 'accepted');
      expect(h.unread, 1);
      b.live = false;
      await w.opened!(b.notices.single['id'] as String);
      expect(opened.single['liveDecision'], false);
      expect(opened.single['notice']['session'], 'task');
      expect(b.notices.single['seen'], true);
      expect(h.unread, 0);
      expect(
        b.attentionCalls.where(
          (c) => [
            'start',
            'approveTool',
            'answerQuestion',
            'cancel',
          ].contains(c['command']),
        ),
        isEmpty,
      );
      h.dispose();
      await t.pump(const Duration(seconds: 2));
    },
  );
  testWidgets(
    'denied native delivery and uncertain receipt keep in-app targets and never retry automatically',
    (t) async {
      for (final uncertain in [false, true]) {
        final b = AttentionBridge()..failFinish = uncertain;
        final w = NoticeWindow()..supported = uncertain;
        final h = AttentionHost(b, window: w, onOpen: (_) async => false);
        await h.refresh();
        expect(w.shown, hasLength(uncertain ? 1 : 0));
        expect(b.notices.single['delivery'], uncertain ? 'claimed' : 'failed');
        b.failFinish = false;
        await h.refresh();
        expect(w.shown, hasLength(uncertain ? 1 : 0));
        await h.open(b.notices.single['id'] as String);
        expect(h.deferred, isNotNull);
        expect(b.notices.single['seen'], false);
        b.failSave = true;
        await h.save({...h.preferences, 'enabled': false});
        expect(h.preferences['enabled'], true);
        h.dispose();
        await t.pump(const Duration(seconds: 2));
      }
    },
  );
  testWidgets(
    'attention opt-in failure receipts and exact target actions fit compact Light and Dark',
    (t) async {
      viewport(t, const Size(390, 740));
      for (final dark in [false, true]) {
        final b = AttentionBridge();
        final w = NoticeWindow()..delivery = 'suppressed';
        final h = AttentionHost(b, window: w, onOpen: (_) async => false);
        final key = GlobalKey();
        await t.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark),
            builder: (_, child) => RepaintBoundary(key: key, child: child!),
            home: Scaffold(body: AttentionPanel(host: h)),
          ),
        );
        await t.pumpAndSettle();
        await capture(t, key, 'attention-${dark ? 'dark' : 'light'}');
        expect(find.text('Windows notifications'), findsOneWidget);
        await t.scrollUntilVisible(
          find.byKey(ValueKey('attention-open-${b.notices.single['id']}')),
          200,
          scrollable: find.byType(Scrollable).first,
        );
        await t.pumpAndSettle();
        await capture(t, key, 'attention-target-${dark ? 'dark' : 'light'}');
        expect(t.takeException(), isNull);
        b.failPoll = true;
        await h.refresh();
        await t.pumpAndSettle();
        await capture(t, key, 'attention-recovery-${dark ? 'dark' : 'light'}');
        expect(find.textContaining('Attention unavailable'), findsOneWidget);
        await t.pumpWidget(const SizedBox());
        h.dispose();
        await t.pump(const Duration(seconds: 2));
      }
    },
  );
}
