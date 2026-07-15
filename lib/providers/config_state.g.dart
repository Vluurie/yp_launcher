// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'config_state.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(ConfigStateController)
const configStateControllerProvider = ConfigStateControllerProvider._();

final class ConfigStateControllerProvider
    extends $NotifierProvider<ConfigStateController, ConfigData> {
  const ConfigStateControllerProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'configStateControllerProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$configStateControllerHash();

  @$internal
  @override
  ConfigStateController create() => ConfigStateController();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(ConfigData value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<ConfigData>(value),
    );
  }
}

String _$configStateControllerHash() =>
    r'b2ba3ac701539caeb863820c28309a92f15036b8';

abstract class _$ConfigStateController extends $Notifier<ConfigData> {
  ConfigData build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<ConfigData, ConfigData>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<ConfigData, ConfigData>,
              ConfigData,
              Object?,
              Object?
            >;
    element.handleValue(ref, created);
  }
}
