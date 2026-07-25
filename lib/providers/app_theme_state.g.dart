// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'app_theme_state.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(AppThemeController)
const appThemeControllerProvider = AppThemeControllerProvider._();

final class AppThemeControllerProvider
    extends $NotifierProvider<AppThemeController, AppThemeId> {
  const AppThemeControllerProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'appThemeControllerProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$appThemeControllerHash();

  @$internal
  @override
  AppThemeController create() => AppThemeController();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(AppThemeId value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<AppThemeId>(value),
    );
  }
}

String _$appThemeControllerHash() =>
    r'1b56489e59606ba4d9faa2829d91e8eae6ce30fa';

abstract class _$AppThemeController extends $Notifier<AppThemeId> {
  AppThemeId build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<AppThemeId, AppThemeId>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<AppThemeId, AppThemeId>,
              AppThemeId,
              Object?,
              Object?
            >;
    element.handleValue(ref, created);
  }
}
