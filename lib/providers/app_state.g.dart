// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'app_state.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(AppStateController)
const appStateControllerProvider = AppStateControllerProvider._();

final class AppStateControllerProvider
    extends $NotifierProvider<AppStateController, AppState> {
  const AppStateControllerProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'appStateControllerProvider',
        isAutoDispose: true,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$appStateControllerHash();

  @$internal
  @override
  AppStateController create() => AppStateController();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(AppState value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<AppState>(value),
    );
  }
}

String _$appStateControllerHash() =>
    r'bc7a3f4a667a5268bfed855e3a684cae7dfcdfab';

abstract class _$AppStateController extends $Notifier<AppState> {
  AppState build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<AppState, AppState>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<AppState, AppState>,
              AppState,
              Object?,
              Object?
            >;
    element.handleValue(ref, created);
  }
}
