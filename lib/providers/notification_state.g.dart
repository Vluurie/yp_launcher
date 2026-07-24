// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'notification_state.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(NotificationStateController)
const notificationStateControllerProvider =
    NotificationStateControllerProvider._();

final class NotificationStateControllerProvider
    extends
        $NotifierProvider<NotificationStateController, List<NotificationItem>> {
  const NotificationStateControllerProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'notificationStateControllerProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$notificationStateControllerHash();

  @$internal
  @override
  NotificationStateController create() => NotificationStateController();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(List<NotificationItem> value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<List<NotificationItem>>(value),
    );
  }
}

String _$notificationStateControllerHash() =>
    r'ef2b09a57d7c3cb26576998ec2b2ca795df4adc6';

abstract class _$NotificationStateController
    extends $Notifier<List<NotificationItem>> {
  List<NotificationItem> build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref =
        this.ref as $Ref<List<NotificationItem>, List<NotificationItem>>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<List<NotificationItem>, List<NotificationItem>>,
              List<NotificationItem>,
              Object?,
              Object?
            >;
    element.handleValue(ref, created);
  }
}
