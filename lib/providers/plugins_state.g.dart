// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'plugins_state.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(PluginsStateController)
const pluginsStateControllerProvider = PluginsStateControllerProvider._();

final class PluginsStateControllerProvider
    extends $NotifierProvider<PluginsStateController, PluginsData> {
  const PluginsStateControllerProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'pluginsStateControllerProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$pluginsStateControllerHash();

  @$internal
  @override
  PluginsStateController create() => PluginsStateController();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(PluginsData value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<PluginsData>(value),
    );
  }
}

String _$pluginsStateControllerHash() =>
    r'86dec4a29b435c6cb86f28d16ba9819ee55ea355';

abstract class _$PluginsStateController extends $Notifier<PluginsData> {
  PluginsData build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<PluginsData, PluginsData>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<PluginsData, PluginsData>,
              PluginsData,
              Object?,
              Object?
            >;
    element.handleValue(ref, created);
  }
}
