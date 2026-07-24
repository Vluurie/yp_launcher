// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'thirdparty_state.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(ThirdPartyStateController)
const thirdPartyStateControllerProvider = ThirdPartyStateControllerProvider._();

final class ThirdPartyStateControllerProvider
    extends $NotifierProvider<ThirdPartyStateController, ThirdPartyData> {
  const ThirdPartyStateControllerProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'thirdPartyStateControllerProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$thirdPartyStateControllerHash();

  @$internal
  @override
  ThirdPartyStateController create() => ThirdPartyStateController();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(ThirdPartyData value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<ThirdPartyData>(value),
    );
  }
}

String _$thirdPartyStateControllerHash() =>
    r'8cb1842d050dbaf0c40d8fde5da6826594ecd173';

abstract class _$ThirdPartyStateController extends $Notifier<ThirdPartyData> {
  ThirdPartyData build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<ThirdPartyData, ThirdPartyData>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<ThirdPartyData, ThirdPartyData>,
              ThirdPartyData,
              Object?,
              Object?
            >;
    element.handleValue(ref, created);
  }
}
