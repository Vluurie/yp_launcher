// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'nams_settings_state.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(NamsSettingsStateController)
const namsSettingsStateControllerProvider =
    NamsSettingsStateControllerProvider._();

final class NamsSettingsStateControllerProvider
    extends $NotifierProvider<NamsSettingsStateController, NamsSettingsData> {
  const NamsSettingsStateControllerProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'namsSettingsStateControllerProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$namsSettingsStateControllerHash();

  @$internal
  @override
  NamsSettingsStateController create() => NamsSettingsStateController();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(NamsSettingsData value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<NamsSettingsData>(value),
    );
  }
}

String _$namsSettingsStateControllerHash() =>
    r'0469e39617992f3e552d68b252d397afeead079a';

abstract class _$NamsSettingsStateController
    extends $Notifier<NamsSettingsData> {
  NamsSettingsData build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<NamsSettingsData, NamsSettingsData>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<NamsSettingsData, NamsSettingsData>,
              NamsSettingsData,
              Object?,
              Object?
            >;
    element.handleValue(ref, created);
  }
}
