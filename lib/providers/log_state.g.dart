// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'log_state.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(LogStateController)
const logStateControllerProvider = LogStateControllerProvider._();

final class LogStateControllerProvider
    extends $NotifierProvider<LogStateController, LogData> {
  const LogStateControllerProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'logStateControllerProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$logStateControllerHash();

  @$internal
  @override
  LogStateController create() => LogStateController();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(LogData value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<LogData>(value),
    );
  }
}

String _$logStateControllerHash() =>
    r'80b238f94da6b13bcc588789f3bffd8dcd6b1803';

abstract class _$LogStateController extends $Notifier<LogData> {
  LogData build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<LogData, LogData>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<LogData, LogData>,
              LogData,
              Object?,
              Object?
            >;
    element.handleValue(ref, created);
  }
}
