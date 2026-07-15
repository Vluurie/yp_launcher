// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'mod_names_state.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(ModNamesStateController)
const modNamesStateControllerProvider = ModNamesStateControllerProvider._();

final class ModNamesStateControllerProvider
    extends $NotifierProvider<ModNamesStateController, ModNamesData> {
  const ModNamesStateControllerProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'modNamesStateControllerProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$modNamesStateControllerHash();

  @$internal
  @override
  ModNamesStateController create() => ModNamesStateController();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(ModNamesData value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<ModNamesData>(value),
    );
  }
}

String _$modNamesStateControllerHash() =>
    r'67c5d5119ac9f8b3cf9d0eb57ba3936bc40fca3f';

abstract class _$ModNamesStateController extends $Notifier<ModNamesData> {
  ModNamesData build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<ModNamesData, ModNamesData>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<ModNamesData, ModNamesData>,
              ModNamesData,
              Object?,
              Object?
            >;
    element.handleValue(ref, created);
  }
}
