import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/background_host.dart';
import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/settings.dart';
import 'package:dolores_flutter/theme.dart';

import 'background_host_test.dart' show BackgroundBridge;

class SettingsBridge extends BackgroundBridge {
  bool fail = false;
  SettingsBridge() {
    policy = {'revision': 1, 'enabled': false};
  }
  @override
  Future<dynamic> call(Map<String, dynamic> c) async {
    if (c['command'] == 'setBackgroundPolicy') {
      if (fail) throw StateError('Save failed. Refresh and try again.');
      expect(c['revision'], policy['revision']);
      policy = {'revision': policy['revision'] + 1, 'enabled': c['enabled']};
      return policy;
    }
    return super.call(c);
  }
}

void main() {
  testWidgets('Tray failure stays visible with a recovery message on Home', (
    t,
  ) async {
    debugDefaultTargetPlatformOverride = TargetPlatform.windows;
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(NativeAvailabilityWindow.channel, (
          call,
        ) async {
          if (call.method == 'hide') {
            throw PlatformException(
              code: 'tray_unavailable',
              message: 'Tray unavailable.',
            );
          }
          return true;
        });
    final bridge = BackgroundBridge();
    final chat = ChatController(bridge)..loading = false;
    final host = AppHost(chat);
    await t.pumpWidget(DoloresApp(chat: chat, host: host));
    await t.pumpAndSettle();
    expect(host.background.enabled, true);
    expect(await host.requestClose(), false);
    await t.pumpAndSettle();
    expect(host.background.hidden, false);
    expect(find.textContaining('Keep Dolores open'), findsOneWidget);
    await t.pumpWidget(const SizedBox());
    host.dispose();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(NativeAvailabilityWindow.channel, null);
    debugDefaultTargetPlatformOverride = null;
  });
  final output = Platform.environment['DOLORES_BACKGROUND_RENDER_DIRECTORY'];
  setUpAll(() async {
    if (output == null) return;
    final font = ByteData.sublistView(
      await File('C:/Windows/Fonts/segoeui.ttf').readAsBytes(),
    );
    for (final family in ['Segoe UI', 'Roboto']) {
      await (FontLoader(family)..addFont(Future.value(font))).load();
    }
    final icons = ByteData.sublistView(
      await File(
        '../../output/toolchains/flutter/bin/cache/artifacts/material_fonts/MaterialIcons-Regular.otf',
      ).readAsBytes(),
    );
    await (FontLoader('MaterialIcons')..addFont(Future.value(icons))).load();
  });
  for (final dark in [false, true]) {
    for (final width in [390.0, 1040.0]) {
      testWidgets(
        'Background settings are concise and failed saves are recoverable $width $dark',
        (t) async {
          t.view.physicalSize = Size(width, 820);
          t.view.devicePixelRatio = 1;
          debugDefaultTargetPlatformOverride = TargetPlatform.windows;
          addTearDown(() => debugDefaultTargetPlatformOverride = null);
          addTearDown(t.view.resetPhysicalSize);
          addTearDown(t.view.resetDevicePixelRatio);
          TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
              .setMockMethodCallHandler(
                NativeAvailabilityWindow.channel,
                (_) async => true,
              );
          addTearDown(
            () => TestDefaultBinaryMessengerBinding
                .instance
                .defaultBinaryMessenger
                .setMockMethodCallHandler(
                  NativeAvailabilityWindow.channel,
                  null,
                ),
          );
          final b = SettingsBridge();
          final tested = ChatController(b)..loading = false;
          final key = GlobalKey();
          await t.pumpWidget(
            MaterialApp(
              theme: doloresTheme(dark).copyWith(
                textButtonTheme: TextButtonThemeData(
                  style: TextButton.styleFrom(
                    textStyle: const TextStyle(
                      fontFamily: 'Segoe UI',
                      fontSize: 14,
                    ),
                  ),
                ),
              ),
              home: RepaintBoundary(
                key: key,
                child: SettingsWindow(
                  chat: tested,
                  initial: SettingsCategory.background,
                ),
              ),
            ),
          );
          await t.pumpAndSettle();
          expect(t.takeException(), isNull);
          expect(find.text('Run tasks in background'), findsOneWidget);
          expect(find.textContaining('computer must be awake'), findsNothing);
          if (output != null) {
            final boundary =
                key.currentContext!.findRenderObject() as RenderRepaintBoundary;
            await t.runAsync(() async {
              final img = await boundary.toImage();
              final bytes = await img.toByteData(
                format: ui.ImageByteFormat.png,
              );
              img.dispose();
              final file = File(
                '$output/background-${dark ? 'dark' : 'light'}-${width.toInt()}.png',
              );
              await file.parent.create(recursive: true);
              await file.writeAsBytes(bytes!.buffer.asUint8List());
            });
          }
          b.fail = true;
          await t.tap(find.byKey(const Key('background-enabled')));
          await t.pumpAndSettle();
          expect(b.policy['enabled'], false);
          expect(find.textContaining('Save failed'), findsOneWidget);
          b.fail = false;
          await t.tap(find.byKey(const Key('background-enabled')));
          await t.pumpAndSettle();
          expect(b.policy['enabled'], true);
          await t.tap(find.byKey(const Key('background-enabled')));
          await t.pumpAndSettle();
          expect(b.policy['enabled'], false);
          await t.pumpWidget(const SizedBox());
          tested.dispose();
          debugDefaultTargetPlatformOverride = null;
        },
      );
    }
  }
}
