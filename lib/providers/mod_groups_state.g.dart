// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'mod_groups_state.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(ModGroupsStateController)
const modGroupsStateControllerProvider = ModGroupsStateControllerProvider._();

final class ModGroupsStateControllerProvider
    extends $NotifierProvider<ModGroupsStateController, ModGroupsData> {
  const ModGroupsStateControllerProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'modGroupsStateControllerProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$modGroupsStateControllerHash();

  @$internal
  @override
  ModGroupsStateController create() => ModGroupsStateController();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(ModGroupsData value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<ModGroupsData>(value),
    );
  }
}

String _$modGroupsStateControllerHash() =>
    r'3b15bec6667982d13b35c9f87db7632dd5be3081';

abstract class _$ModGroupsStateController extends $Notifier<ModGroupsData> {
  ModGroupsData build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<ModGroupsData, ModGroupsData>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<ModGroupsData, ModGroupsData>,
              ModGroupsData,
              Object?,
              Object?
            >;
    element.handleValue(ref, created);
  }
}
