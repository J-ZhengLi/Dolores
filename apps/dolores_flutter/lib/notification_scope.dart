import 'package:flutter/widgets.dart';

import 'attention_host.dart';

class NotificationScope extends InheritedWidget {
  final AttentionHost host;
  const NotificationScope({
    super.key,
    required this.host,
    required super.child,
  });
  static AttentionHost? of(BuildContext context) =>
      context.dependOnInheritedWidgetOfExactType<NotificationScope>()?.host;
  @override
  bool updateShouldNotify(NotificationScope old) => old.host != host;
}
