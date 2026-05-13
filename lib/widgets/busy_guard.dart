import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:yp_launcher/providers/app_state.dart';

/// Wraps a Future with a global busy refcount bump so the window close
/// handler can warn the user about in-progress work.
Future<T> withBusyGuard<T>(WidgetRef ref, Future<T> Function() task) async {
  final notifier = ref.read(busyOperationsProvider.notifier);
  notifier.state = notifier.state + 1;
  try {
    return await task();
  } finally {
    notifier.state = notifier.state - 1;
  }
}
