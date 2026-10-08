import 'package:dolores_flutter/git_host.dart';
import 'package:dolores_flutter/source_control.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'source_control_test.dart' show DiffBridge;

void main() {
  for (final dark in [false, true]) {
    testWidgets(
      'Source Control shows its enclosing root and follows selection ($dark)',
      (tester) async {
        final git = GitHost(DiffBridge());
        final nested =
            GitWorkspace('C:/public-repository/nested/project', 'nested')
              ..status = {
                'root': 'C:/public-repository',
                'branch': 'main',
                'entries': <Map>[],
              };
        git.selected = nested;
        await tester.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark),
            home: Scaffold(
              body: SizedBox(
                width: 252,
                child: SourceControlPanel(git: git, openFolder: () {}),
              ),
            ),
          ),
        );
        await tester.pumpAndSettle();
        expect(find.text('C:/public-repository'), findsOneWidget);
        expect(find.text('public-repository'), findsOneWidget);
        expect(find.text('Enclosing repository'), findsOneWidget);
        expect(find.text('main'), findsOneWidget);
        nested.error = 'Git unavailable. Refresh.';
        git.changed();
        await tester.pumpAndSettle();
        expect(find.text('C:/public-repository'), findsOneWidget);
        expect(find.text('Git unavailable. Refresh.'), findsOneWidget);

        git.selected = GitWorkspace('D:/Other', 'other')
          ..status = {
            'root': 'd:\\Other\\',
            'branch': 'topic',
            'entries': <Map>[],
          };
        git.changed();
        await tester.pumpAndSettle();
        expect(find.text('d:\\Other\\'), findsOneWidget);
        expect(find.text('Other'), findsOneWidget);
        expect(find.text('topic'), findsOneWidget);
        expect(find.text('Enclosing repository'), findsNothing);
        expect(find.text('C:/public-repository'), findsNothing);
        expect(find.text('public-repository'), findsNothing);
        const longRoot =
            r'\\?\D:\Workspace\a-long-parent-directory\another-long-folder\PublicProject';
        git.selected = GitWorkspace(longRoot, 'long')
          ..status = {'root': longRoot, 'branch': 'main', 'entries': <Map>[]};
        git.changed();
        await tester.pumpAndSettle();
        expect(find.text('PublicProject'), findsOneWidget);
        expect(find.text(longRoot), findsOneWidget);
        expect(find.byTooltip(longRoot), findsOneWidget);
        expect(find.text('Enclosing repository'), findsNothing);
        expect(find.text('Git unavailable. Refresh.'), findsNothing);
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
        git.dispose();
      },
    );
  }
}
