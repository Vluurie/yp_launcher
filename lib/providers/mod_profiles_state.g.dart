// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'mod_profiles_state.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(ModProfilesStateController)
const modProfilesStateControllerProvider =
    ModProfilesStateControllerProvider._();

final class ModProfilesStateControllerProvider
    extends $NotifierProvider<ModProfilesStateController, ModProfileState> {
  const ModProfilesStateControllerProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'modProfilesStateControllerProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$modProfilesStateControllerHash();

  @$internal
  @override
  ModProfilesStateController create() => ModProfilesStateController();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(ModProfileState value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<ModProfileState>(value),
    );
  }
}

String _$modProfilesStateControllerHash() =>
    r'd7c0cccfb72cb3985bd3e797438e89604fc005f0';

abstract class _$ModProfilesStateController extends $Notifier<ModProfileState> {
  ModProfileState build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<ModProfileState, ModProfileState>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<ModProfileState, ModProfileState>,
              ModProfileState,
              Object?,
              Object?
            >;
    element.handleValue(ref, created);
  }
}
