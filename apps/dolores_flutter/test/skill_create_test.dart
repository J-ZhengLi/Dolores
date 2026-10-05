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
    if (command['command'] == 'reviewSkillText') {
      if (invalid) throw 'Invalid skill document';
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
  testWidgets('global creation needs exact review before explicit activation', (
    t,
  ) async {
    final b = CreateBridge();
    await open(t, b);
    await t.tap(find.text('Review'));
    await t.pumpAndSettle();
    expect(b.commands.where((c) => c['command'] == 'activateSkill'), isEmpty);
    expect(b.commands.first['session'], '');
    expect(find.text('Activate skill'), findsOneWidget);
    await t.tap(find.text('Activate skill'));
    await t.pumpAndSettle();
    expect(
      b.commands.where((c) => c['command'] == 'activateSkill').single['token'],
      'reviewed',
    );
    expect(find.byType(SkillCreateDialog), findsNothing);
    expect(t.takeException(), isNull);
  });
  testWidgets('malformed and stale reviews preserve editable instructions', (
    t,
  ) async {
    final b = CreateBridge()..invalid = true;
    await open(t, b);
    await t.tap(find.text('Review'));
    await t.pumpAndSettle();
    expect(find.text('Invalid skill document'), findsOneWidget);
    expect(find.text('Activate skill'), findsNothing);
    b.invalid = false;
    await t.tap(find.text('Review'));
    await t.pumpAndSettle();
    b.stale = true;
    await t.tap(find.text('Activate skill'));
    await t.pumpAndSettle();
    expect(find.text('Skill changed. Review again.'), findsOneWidget);
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
