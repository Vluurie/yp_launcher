// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'default_mods_state.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(DefaultModsStateController)
const defaultModsStateControllerProvider =
    DefaultModsStateControllerProvider._();

final class DefaultModsStateControllerProvider
    extends $NotifierProvider<DefaultModsStateController, DefaultModsData> {
  const DefaultModsStateControllerProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'defaultModsStateControllerProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$defaultModsStateControllerHash();

  @$internal
  @override
  DefaultModsStateController create() => DefaultModsStateController();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(DefaultModsData value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<DefaultModsData>(value),
    );
  }
}

String _$defaultModsStateControllerHash() =>
    r'01a361463072302b8b7b2455835f64881269a02f';

abstract class _$DefaultModsStateController extends $Notifier<DefaultModsData> {
  DefaultModsData build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<DefaultModsData, DefaultModsData>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<DefaultModsData, DefaultModsData>,
              DefaultModsData,
              Object?,
              Object?
            >;
    element.handleValue(ref, created);
  }
}
