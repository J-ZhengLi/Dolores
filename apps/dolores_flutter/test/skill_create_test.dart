import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/skill_create.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

class CreateBridge implements ChatBridge {
  final commands = <Map<String, dynamic>>[];
  bool invalid = false, stale = false;
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    commands.add(command);
    if (command['command'] == 'saveSkillText') {
      if (invalid) throw 'Invalid skill document';
      if (stale) throw 'Skill could not be saved. Try again.';
      return {
        'token': 'reviewed',
        'document': {'text': command['text']},
      };
    }
    if (command['command'] == 'activateSkill' && stale) {
      throw 'Skill changed. Review again.';
    }
    return null;
  }
}

Future<void> open(WidgetTester t, CreateBridge b) async {
  await t.pumpWidget(
    MaterialApp(
      home: Builder(
        builder: (context) => Scaffold(
          body: TextButton(
            onPressed: () => showDialog<void>(
              context: context,
              builder: (_) =>
                  SkillCreateDialog(bridge: b, session: '', scope: 'global'),
            ),
            child: const Text('Open'),
          ),
        ),
      ),
    ),
  );
  await t.tap(find.text('Open'));
  await t.pumpAndSettle();
  await t.enterText(find.byKey(const Key('create-skill-name')), 'review');
  await t.enterText(
    find.byKey(const Key('create-skill-description')),
    'When reviewing code',
  );
  await t.enterText(
    find.byKey(const Key('create-skill-text')),
    'Check the changed behavior.',
  );
}

void main() {
  testWidgets(
    'global creation saves in one action without review or activation',
    (t) async {
      final b = CreateBridge();
      await open(t, b);
      await t.tap(find.text('Save skill'));
      await t.pumpAndSettle();
      expect(b.commands.where((c) => c['command'] == 'activateSkill'), isEmpty);
      expect(b.commands.first['session'], '');
      expect(
        b.commands.where((c) => c['command'] == 'saveSkillText').single['name'],
        'review',
      );
      expect(find.byType(SkillCreateDialog), findsNothing);
      expect(t.takeException(), isNull);
    },
  );
  testWidgets('malformed and failed saves preserve editable instructions', (
    t,
  ) async {
    final b = CreateBridge()..invalid = true;
    await open(t, b);
    await t.tap(find.text('Save skill'));
    await t.pumpAndSettle();
    expect(find.text('Invalid skill document'), findsOneWidget);
    expect(find.text('Activate skill'), findsNothing);
    b.invalid = false;
    b.stale = true;
    await t.tap(find.text('Save skill'));
    await t.pumpAndSettle();
    expect(find.text('Skill could not be saved. Try again.'), findsOneWidget);
    expect(
      t
          .widget<TextField>(find.byKey(const Key('create-skill-text')))
          .controller!
          .text,
      'Check the changed behavior.',
    );
    await t.tap(find.text('Cancel'));
    await t.pumpAndSettle();
    await t.tap(find.text('Keep editing'));
    await t.pumpAndSettle();
    expect(find.byType(SkillCreateDialog), findsOneWidget);
    expect(t.takeException(), isNull);
  });
}
