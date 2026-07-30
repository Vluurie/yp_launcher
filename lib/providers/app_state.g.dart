// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'app_state.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(AppStateController)
final appStateControllerProvider = AppStateControllerProvider._();

final class AppStateControllerProvider
    extends $NotifierProvider<AppStateController, AppState> {
  AppStateControllerProvider._()
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
    r'cbaf0b3a28190290ecc88f42f45040267d8df0b5';

abstract class _$AppStateController extends $Notifier<AppState> {
  AppState build();
  @$mustCallSuper
  @override
  void runBuild() {
    final ref = this.ref as $Ref<AppState, AppState>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<AppState, AppState>,
              AppState,
              Object?,
              Object?
            >;
    element.handleCreate(ref, build);
  }
}
