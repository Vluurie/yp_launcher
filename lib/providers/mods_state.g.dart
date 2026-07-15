// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'mods_state.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(ModsStateController)
const modsStateControllerProvider = ModsStateControllerProvider._();

final class ModsStateControllerProvider
    extends $NotifierProvider<ModsStateController, ModsData> {
  const ModsStateControllerProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'modsStateControllerProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$modsStateControllerHash();

  @$internal
  @override
  ModsStateController create() => ModsStateController();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(ModsData value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<ModsData>(value),
    );
  }
}

String _$modsStateControllerHash() =>
    r'a923f6b624577a233b7b84e746e8e7ca25d33acb';

abstract class _$ModsStateController extends $Notifier<ModsData> {
  ModsData build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<ModsData, ModsData>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<ModsData, ModsData>,
              ModsData,
              Object?,
              Object?
            >;
    element.handleValue(ref, created);
  }
}
