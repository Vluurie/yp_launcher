// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'disabled_mods_state.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(DisabledModsStateController)
const disabledModsStateControllerProvider =
    DisabledModsStateControllerProvider._();

final class DisabledModsStateControllerProvider
    extends $NotifierProvider<DisabledModsStateController, DisabledModsData> {
  const DisabledModsStateControllerProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'disabledModsStateControllerProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$disabledModsStateControllerHash();

  @$internal
  @override
  DisabledModsStateController create() => DisabledModsStateController();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(DisabledModsData value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<DisabledModsData>(value),
    );
  }
}

String _$disabledModsStateControllerHash() =>
    r'07029e7c1056f6fe0a3f07e7491e455c708290e6';

abstract class _$DisabledModsStateController
    extends $Notifier<DisabledModsData> {
  DisabledModsData build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<DisabledModsData, DisabledModsData>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<DisabledModsData, DisabledModsData>,
              DisabledModsData,
              Object?,
              Object?
            >;
    element.handleValue(ref, created);
  }
}
