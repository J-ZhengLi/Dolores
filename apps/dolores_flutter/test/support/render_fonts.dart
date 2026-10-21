import 'dart:io';

import 'package:flutter/services.dart';

/// Use actual fonts for retained Windows review images; no product font change.
Future<void> loadReviewFonts() async {
  final fonts = {
    // Flutter tests use Ahem for style runs without an explicit family.
    // Alias it only in review renders so action labels remain inspectable.
    'Ahem':
        '${Platform.environment['SystemRoot'] ?? 'C:/Windows'}/Fonts/segoeui.ttf',
    'Segoe UI':
        '${Platform.environment['SystemRoot'] ?? 'C:/Windows'}/Fonts/segoeui.ttf',
    'MaterialIcons': '../../output/toolchains/flutter/bin/cache/artifacts/material_fonts/MaterialIcons-Regular.otf',
  };
  for (final font in fonts.entries) {
    final file = File(font.value);
    if (!file.existsSync()) continue;
    final bytes = await file.readAsBytes();
    final loader = FontLoader(font.key)
      ..addFont(Future.value(ByteData.sublistView(bytes)));
    await loader.load();
  }
}
