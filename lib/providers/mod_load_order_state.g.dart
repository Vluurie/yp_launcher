// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'mod_load_order_state.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(ModLoadOrderStateController)
const modLoadOrderStateControllerProvider =
    ModLoadOrderStateControllerProvider._();

final class ModLoadOrderStateControllerProvider
    extends $NotifierProvider<ModLoadOrderStateController, ModLoadOrderData> {
  const ModLoadOrderStateControllerProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'modLoadOrderStateControllerProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$modLoadOrderStateControllerHash();

  @$internal
  @override
  ModLoadOrderStateController create() => ModLoadOrderStateController();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(ModLoadOrderData value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<ModLoadOrderData>(value),
    );
  }
}

String _$modLoadOrderStateControllerHash() =>
    r'fa523a60e88db54df2b6636a56566a25e471844e';

abstract class _$ModLoadOrderStateController
    extends $Notifier<ModLoadOrderData> {
  ModLoadOrderData build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<ModLoadOrderData, ModLoadOrderData>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<ModLoadOrderData, ModLoadOrderData>,
              ModLoadOrderData,
              Object?,
              Object?
            >;
    element.handleValue(ref, created);
  }
}
