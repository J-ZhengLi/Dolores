import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/codex_compatibility.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'stop_recovery_test.dart' show InterruptedBridge;
import 'frost_design_test.dart' show viewport, capture;
import 'support/render_fonts.dart';

class CodexFixture extends InterruptedBridge {
  CodexFixture() : super(true);
  bool fail = false;
  Map<String, dynamic> profile = {
    'enabled': false,
    'home': 'C:/public/codex',
    'project': false,
    'referencedInstructions': false,
  };
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    if (command['command'] == 'codexSettings' ||
        command['command'] == 'saveCodexSettings') {
      commands.add(command);
      if (command['command'] == 'saveCodexSettings') {
        if (fail) throw Exception('Save failed. Keep your draft and retry.');
        profile = Map<String, dynamic>.from(command['profile'] as Map);
      }
      return {
        'profile': profile,
        'detectedHome': 'C:/public/codex',
        'ownedHome': 'C:/public/dolores/compat/codex',
        'report': {
          'servers': [
            {'name': 'generic-fixture', 'enabled': true},
          ],
          'hookCount': 2,
          'sources': ['C:/public/codex/config.toml'],
          'diagnostics': [],
        },
      };
    }
    return super.call(command);
  }
}

void main() {
  for (final dark in [false, true]) {
    testWidgets(
      'Codex profile saves explicitly and preserves failed draft ($dark)',
      (t) async {
        viewport(t, const Size(420, 800));
        await t.runAsync(loadReviewFonts);
        final b = CodexFixture();
        var opened=0;
        final c = ChatController(b)..loading = false;
        c.onOpenCodexSetupTerminal=()async{opened++;};
        final key = GlobalKey();
        await t.pumpWidget(
          RepaintBoundary(
            key: key,
            child: MaterialApp(
              debugShowCheckedModeBanner: false,
              theme: doloresTheme(dark),
              home: Scaffold(body: CodexCompatibility(chat: c)),
            ),
          ),
        );
        await t.pumpAndSettle();
        expect(
          b.commands.where((v) => v['command'] == 'saveCodexSettings'),
          isEmpty,
        );
        await t.tap(find.text('Use this Codex setup'));
        await t.pump();
        await t.enterText(
          find.byKey(const Key('codex-profile-home')),
          'C:/public/another-profile',
        );
        b.fail = true;
        await t.ensureVisible(find.text('Save'));
        await t.tap(find.text('Save'));
        await t.pumpAndSettle();
        expect(find.textContaining('Save failed'), findsOneWidget);
        expect(find.text('C:/public/another-profile'), findsOneWidget);
        b.fail = false;
        await t.tap(find.text('Save'));
        await t.pumpAndSettle();
        expect(b.profile['enabled'], true);
        expect(b.profile['home'], 'C:/public/another-profile');
        await t.ensureVisible(find.text('Open setup terminal'));
        await t.tap(find.text('Open setup terminal'));await t.pumpAndSettle();
        expect(opened,1);expect(find.textContaining('Setup terminal opened'),findsOneWidget);
        expect(t.takeException(), isNull);
        final scroll = t.state<ScrollableState>(find.byType(Scrollable).first);
        scroll.position.jumpTo(0);
        await t.pump();
        await capture(
          t,
          key,
          'codex-compatibility-${dark ? 'dark' : 'light'}-420',
        );
        await t.pumpWidget(const SizedBox.shrink());
        c.dispose();
      },
    );
  }
}
