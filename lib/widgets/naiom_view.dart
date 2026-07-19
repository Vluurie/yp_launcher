import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/config_state.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/widgets/collapsible_card.dart';
import 'package:yp_launcher/widgets/config_field_bool.dart';
import 'package:yp_launcher/widgets/config_field_cursor_file.dart';
import 'package:yp_launcher/widgets/config_field_slider.dart';
import 'package:yp_launcher/widgets/config_field_key_name.dart';
import 'package:yp_launcher/widgets/config_field_preview.dart'
    show ConfigPreviewImage;
import 'package:yp_launcher/widgets/config_field_virtual_key.dart';
import 'package:yp_launcher/widgets/header_info_icon.dart';
import 'package:yp_launcher/widgets/hover_button.dart';

class NaiomView extends ConsumerStatefulWidget {
  const NaiomView({super.key});

  @override
  ConsumerState<NaiomView> createState() => _NaiomViewState();
}

class _NaiomViewState extends ConsumerState<NaiomView> {
  final _scrollController = ScrollController();
  bool _loaded = false;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!_loaded) {
        _loaded = true;
        final gameDir = ref.read(appStateControllerProvider).selectedDirectory;
        ref.read(configStateControllerProvider.notifier).loadConfigs(gameDir);
      }
    });
  }

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  Map<String, dynamic> get _mouse =>
      (ref.watch(configStateControllerProvider).namsValues['mouse']
          as Map<String, dynamic>?) ??
      const {};

  Map<String, dynamic> get _bindings =>
      (_mouse['bindings'] as Map<String, dynamic>?) ?? const {};

  bool _boolOf(ConfigField<bool> f) => _mouse[f.key] == true;

  double _doubleOf(ConfigField<double> f) =>
      (_mouse[f.key] as num?)?.toDouble() ?? f.defaultValue;

  String _stringOf(ConfigField<String> f) =>
      (_mouse[f.key] as String?) ?? f.defaultValue;

  String _bindingOf(ConfigField<String> f) =>
      (_bindings[f.key] as String?) ?? '';

  void _setMouse(ConfigField f, dynamic value) {
    ref
        .read(configStateControllerProvider.notifier)
        .updateNams(f.key, value, section: f.section!);
  }

  static final List<ConfigField<bool>> _mouseBools = [
    NamsFields.fixCameraAcceleration,
    NamsFields.thirdPersonMode,
    NamsFields.thirdPersonCharFollow,
    NamsFields.aimMode,
    NamsFields.aimCrosshair,
    NamsFields.aimCrosshairAlways,
    NamsFields.movementDisableTapEvade,
    NamsFields.miscDisablePodPet,
    NamsFields.miscDisableDefaultCursor,
  ];

  static final List<ConfigField<double>> _mouseDoubles = [
    NamsFields.sensitivity,
    NamsFields.thirdPersonSensitivityX,
    NamsFields.thirdPersonSensitivityY,
    NamsFields.aimSensitivity,
    NamsFields.aimSensitivityX,
    NamsFields.aimSensitivityY,
    NamsFields.aimOutputMultiplier,
  ];

  static final List<ConfigField<String>> _mouseStrings = [
    NamsFields.miscCustomCursorMenu,
    NamsFields.miscCustomCursorHacking,
  ];

  bool _hasNonDefaults() {
    if (_mouseBools.any((f) => _boolOf(f) != f.defaultValue)) return true;
    if (_mouseDoubles.any((f) => _doubleOf(f) != f.defaultValue)) return true;
    if (_mouseStrings.any((f) => _stringOf(f) != f.defaultValue)) return true;
    final debugKey =
        (_mouse[NamsFields.miscOpenDebugMenu.key] as int?) ??
        NamsFields.miscOpenDebugMenu.defaultValue;
    if (debugKey != NamsFields.miscOpenDebugMenu.defaultValue) return true;
    return NamsBindingFields.all.any((f) => _bindingOf(f).isNotEmpty);
  }

  void _resetAll() {
    for (final f in _mouseBools) {
      _setMouse(f, f.defaultValue);
    }
    for (final f in _mouseDoubles) {
      _setMouse(f, f.defaultValue);
    }
    for (final f in _mouseStrings) {
      _setMouse(f, f.defaultValue);
    }
    _setMouse(
      NamsFields.miscOpenDebugMenu,
      NamsFields.miscOpenDebugMenu.defaultValue,
    );
    for (final f in NamsBindingFields.all) {
      _setMouse(f, '');
    }
  }

  Future<void> _confirmReset(BuildContext context) async {
    final l10n = AppLocalizations.of(context)!;
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: AppColors.backgroundCard,
        title: Text(
          l10n.naiomResetConfirmTitle,
          style: TextStyle(
            fontSize: AppSizes.fontLG(ctx),
            color: AppColors.textPrimary,
          ),
        ),
        content: Text(
          l10n.naiomResetConfirmBody,
          style: TextStyle(
            fontSize: AppSizes.fontSM(ctx),
            color: AppColors.textSecondary,
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(false),
            child: Text(
              MaterialLocalizations.of(ctx).cancelButtonLabel,
              style: const TextStyle(color: AppColors.textMuted),
            ),
          ),
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(true),
            child: Text(
              l10n.lodModResetConfirmAction,
              style: const TextStyle(
                color: AppColors.error,
                fontWeight: FontWeight.bold,
              ),
            ),
          ),
        ],
      ),
    );
    if (confirmed == true) _resetAll();
  }

  String? _conflictFor(ConfigField<String> field, AppLocalizations l10n) {
    final own = _bindingOf(field);
    if (own.isEmpty) return null;
    for (final other in NamsBindingFields.all) {
      if (other.key == field.key) continue;
      if (_bindingOf(other) == own) return other.label(l10n);
    }
    return null;
  }

  @override
  Widget build(BuildContext context) {
    final config = ref.watch(configStateControllerProvider);
    final gameDir = ref.watch(appStateControllerProvider).selectedDirectory;

    return Container(
      color: AppColors.backgroundPrimary,
      child: Column(
        children: [
          _buildHeader(context, config, gameDir),
          Expanded(
            child: config.isLoading
                ? const Center(
                    child: CircularProgressIndicator(
                      color: AppColors.accentPrimary,
                    ),
                  )
                : Scrollbar(
                    controller: _scrollController,
                    thumbVisibility: true,
                    child: SingleChildScrollView(
                      controller: _scrollController,
                      padding: EdgeInsets.all(AppSizes.contentPadding(context)),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          _controllerBanner(context),
                          Row(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Expanded(child: _buildLeftColumn(context)),
                              SizedBox(width: AppSizes.spacingLG(context)),
                              Expanded(child: _buildRightColumn(context)),
                            ],
                          ),
                        ],
                      ),
                    ),
                  ),
          ),
        ],
      ),
    );
  }

  Widget _buildLeftColumn(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final cameraFixOn = _boolOf(NamsFields.fixCameraAcceleration);
    final thirdPersonOn = _boolOf(NamsFields.thirdPersonMode);
    final aimOn = _boolOf(NamsFields.aimMode);
    final crosshairOn = _boolOf(NamsFields.aimCrosshair);

    return Column(
      children: [
        _card(context, l10n.cardThirdPersonCamera, [
          _bool(NamsFields.fixCameraAcceleration, l10n),
          _dependent(
            context,
            enabled: cameraFixOn,
            reason: l10n.naiomNeedsCameraFix,
            child: _float(NamsFields.sensitivity, l10n),
          ),
          _divider(context),
          _bool(NamsFields.thirdPersonMode, l10n),
          _note(context, l10n.naiomThirdPersonRestartNote, Icons.info_outline),
          _dependent(
            context,
            enabled: thirdPersonOn,
            reason: l10n.naiomNeedsThirdPerson,
            child: Column(
              children: [
                _bool(NamsFields.thirdPersonCharFollow, l10n),
                _float(NamsFields.thirdPersonSensitivityX, l10n),
                _float(NamsFields.thirdPersonSensitivityY, l10n),
              ],
            ),
          ),
        ]),
        _card(context, l10n.cardPodAiming, [
          _bool(NamsFields.aimMode, l10n),
          _dependent(
            context,
            enabled: aimOn,
            reason: l10n.naiomNeedsAimMode,
            child: Column(
              children: [
                _bool(NamsFields.aimCrosshair, l10n),
                ConfigPreviewImage(
                  image: 'assets/images/config/crosshair.jpg',
                  label: l10n.naiomCrosshairPreviewLabel,
                ),
                _dependent(
                  context,
                  enabled: crosshairOn,
                  reason: l10n.naiomNeedsCrosshair,
                  child: _bool(NamsFields.aimCrosshairAlways, l10n),
                ),
                if (crosshairOn)
                  _note(context, l10n.naiomCrosshairNote, Icons.info_outline),
                _dependent(
                  context,
                  enabled: !crosshairOn,
                  reason: l10n.naiomCrosshairOverrides,
                  child: Column(
                    children: [
                      _float(NamsFields.aimSensitivity, l10n, decimals: 4),
                      _float(NamsFields.aimOutputMultiplier, l10n, decimals: 1),
                    ],
                  ),
                ),
                _float(NamsFields.aimSensitivityX, l10n),
                _float(NamsFields.aimSensitivityY, l10n),
              ],
            ),
          ),
        ]),
        _card(context, l10n.cardMisc, [
          _bool(NamsFields.miscDisablePodPet, l10n),
          ConfigFieldVirtualKey(
            label: NamsFields.miscOpenDebugMenu.label(l10n),
            value:
                (_mouse[NamsFields.miscOpenDebugMenu.key] as int?) ??
                NamsFields.miscOpenDebugMenu.defaultValue,
            onChanged: (v) => _setMouse(NamsFields.miscOpenDebugMenu, v),
            tooltip: NamsFields.miscOpenDebugMenu.tooltip!(l10n),
          ),
          _divider(context),
          ConfigFieldCursorFile(
            label: NamsFields.miscCustomCursorMenu.label(l10n),
            value: _stringOf(NamsFields.miscCustomCursorMenu),
            onChanged: (v) => _setMouse(NamsFields.miscCustomCursorMenu, v),
            tooltip: NamsFields.miscCustomCursorMenu.tooltip!(l10n),
          ),
          ConfigFieldCursorFile(
            label: NamsFields.miscCustomCursorHacking.label(l10n),
            value: _stringOf(NamsFields.miscCustomCursorHacking),
            onChanged: (v) => _setMouse(NamsFields.miscCustomCursorHacking, v),
            tooltip: NamsFields.miscCustomCursorHacking.tooltip!(l10n),
          ),
          _bool(NamsFields.miscDisableDefaultCursor, l10n),
          _note(context, l10n.naiomRestartTooltip, Icons.refresh),
        ]),
      ],
    );
  }

  Widget _buildRightColumn(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final tapEvadeOn = _boolOf(NamsFields.movementDisableTapEvade);
    final evadeUnbound = _bindingOf(NamsBindingFields.nonStandardEvade).isEmpty;

    return Column(
      children: [
        _infoBanner(context, l10n.naiomBindingsIntro),
        _card(context, l10n.cardMovementBindings, [
          _bind(NamsBindingFields.standardMoveForward, l10n),
          _bind(NamsBindingFields.standardMoveBackward, l10n),
          _bind(NamsBindingFields.standardMoveLeft, l10n),
          _bind(NamsBindingFields.standardMoveRight, l10n),
          _bind(NamsBindingFields.standardJump, l10n),
          _bind(NamsBindingFields.standardWalk, l10n),
          _bind(NamsBindingFields.standardAutoRun, l10n),
          _divider(context),
          _bool(NamsFields.movementDisableTapEvade, l10n),
          if (tapEvadeOn && evadeUnbound)
            _warning(context, l10n.naiomTapEvadeWarning),
        ]),
        _card(context, l10n.cardCombatBindings, [
          _bind(NamsBindingFields.standardLightAttack, l10n),
          _bind(NamsBindingFields.standardHeavyAttack, l10n),
          _bind(NamsBindingFields.standardFire, l10n),
          _bind(NamsBindingFields.standardProgram, l10n),
          _bind(NamsBindingFields.standardLockOn, l10n),
          _bind(NamsBindingFields.standardUse, l10n),
          _bind(NamsBindingFields.standardSwitchWeapon, l10n),
          _bind(NamsBindingFields.standardNextProgram, l10n),
          _bind(NamsBindingFields.standardPreviousProgram, l10n),
          _bind(NamsBindingFields.standardSelfDestruct, l10n),
          _bind(NamsBindingFields.standardLight, l10n),
          _bind(NamsBindingFields.standardResetCamera, l10n),
        ]),
        _card(context, l10n.cardNonStandardBindings, [
          _bind(NamsBindingFields.nonStandardEvade, l10n),
          _bind(NamsBindingFields.nonStandardAutoFire, l10n),
          _bind(NamsBindingFields.nonStandardNextItem, l10n),
          _bind(NamsBindingFields.nonStandardPreviousItem, l10n),
          _bind(NamsBindingFields.nonStandardUseItem, l10n),
          _divider(context),
          _bind(NamsBindingFields.thirdPersonModeToggle, l10n),
          _bind(NamsBindingFields.aimModeToggle, l10n),
        ]),
        _card(context, l10n.cardMenuNavigation, [
          _bind(NamsBindingFields.standardMenuUp, l10n),
          _bind(NamsBindingFields.standardMenuDown, l10n),
          _bind(NamsBindingFields.standardMenuLeft, l10n),
          _bind(NamsBindingFields.standardMenuRight, l10n),
          _bind(NamsBindingFields.standardMenuOpen, l10n),
          _bind(NamsBindingFields.standardMenuBack, l10n),
          _bind(NamsBindingFields.standardMenuEnter, l10n),
          _bind(NamsBindingFields.standardShortcutMenu, l10n),
        ]),
      ],
    );
  }

  Widget _bool(ConfigField<bool> field, AppLocalizations l10n) {
    return ConfigFieldBool(
      label: field.label(l10n),
      value: _boolOf(field),
      onChanged: (v) => _setMouse(field, v),
      tooltip: field.tooltip!(l10n),
      restartRequired: field.restartRequired,
      showApplyMarker: field.restartRequired,
    );
  }

  Widget _float(
    ConfigField<double> field,
    AppLocalizations l10n, {
    int decimals = 2,
  }) {
    return ConfigFieldSlider(
      label: field.label(l10n),
      tooltip: field.tooltip!(l10n),
      restartRequired: field.restartRequired,
      showApplyMarker: field.restartRequired,
      min: field.min!.toDouble(),
      max: field.max!.toDouble(),
      step: field.step!.toDouble(),
      decimals: decimals,
      value: _doubleOf(field),
      onChanged: (v) => _setMouse(field, v),
    );
  }

  Widget _bind(ConfigField<String> field, AppLocalizations l10n) {
    return ConfigFieldKeyName(
      label: field.label(l10n),
      bindingKey: field.key,
      value: _bindingOf(field),
      onChanged: (v) => _setMouse(field, v),
      tooltip: field.tooltip!(l10n),
      conflictWith: _conflictFor(field, l10n),
    );
  }

  Widget _dependent(
    BuildContext context, {
    required bool enabled,
    required String reason,
    required Widget child,
  }) {
    if (enabled) return child;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Opacity(
          opacity: 0.4,
          child: IgnorePointer(child: child),
        ),
        Padding(
          padding: EdgeInsets.only(
            left: AppSizes.paddingSM(context),
            bottom: AppSizes.paddingXS(context),
          ),
          child: Row(
            children: [
              Icon(Icons.lock_outline, size: 12, color: AppColors.textMuted),
              const SizedBox(width: 4),
              Expanded(
                child: Text(
                  reason,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.textMuted,
                    fontStyle: FontStyle.italic,
                  ),
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _note(BuildContext context, String text, IconData icon) {
    return Padding(
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.paddingSM(context),
        vertical: 2,
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(icon, size: 13, color: AppColors.textMuted),
          const SizedBox(width: 6),
          Expanded(
            child: Text(
              text,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textMuted,
                height: 1.3,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _warning(BuildContext context, String text) {
    return Container(
      margin: const EdgeInsets.symmetric(vertical: 4),
      padding: EdgeInsets.all(AppSizes.paddingSM(context)),
      decoration: BoxDecoration(
        color: AppColors.warning.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: AppColors.warning.withValues(alpha: 0.4)),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(
            Icons.warning_amber_rounded,
            size: 16,
            color: AppColors.warning,
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              text,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.warning,
                height: 1.3,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _controllerBanner(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return Container(
      width: double.infinity,
      margin: EdgeInsets.only(bottom: AppSizes.paddingLG(context)),
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.paddingMD(context),
        vertical: AppSizes.paddingSM(context),
      ),
      decoration: BoxDecoration(
        color: AppColors.surfaceLight,
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(color: AppColors.borderLight),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          Icon(
            Icons.sports_esports_outlined,
            size: AppSizes.iconLG(context),
            color: AppColors.textSecondary,
          ),
          SizedBox(width: AppSizes.spacingMD(context)),
          Expanded(
            child: Text(
              l10n.naiomControllerNote,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textSecondary,
                height: 1.4,
              ),
            ),
          ),
          if (_hasNonDefaults()) ...[
            SizedBox(width: AppSizes.spacingMD(context)),
            HoverButton(
              label: l10n.lodModResetButton,
              color: AppColors.warning,
              onTap: () => _confirmReset(context),
            ),
          ],
        ],
      ),
    );
  }

  Widget _infoBanner(BuildContext context, String text) {
    return Container(
      width: double.infinity,
      margin: EdgeInsets.only(bottom: AppSizes.paddingLG(context)),
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.paddingMD(context),
        vertical: AppSizes.paddingSM(context),
      ),
      decoration: BoxDecoration(
        color: AppColors.accentPrimary.withValues(alpha: 0.06),
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(
          color: AppColors.accentPrimary.withValues(alpha: 0.25),
        ),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(
            Icons.keyboard_outlined,
            size: AppSizes.iconLG(context),
            color: AppColors.accentPrimary,
          ),
          SizedBox(width: AppSizes.spacingMD(context)),
          Expanded(
            child: Text(
              text,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textSecondary,
                height: 1.4,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _divider(BuildContext context) {
    return Padding(
      padding: EdgeInsets.symmetric(vertical: AppSizes.paddingXS(context)),
      child: const Divider(height: 1, color: AppColors.borderLight),
    );
  }

  Widget _buildHeader(
    BuildContext context,
    ConfigData config,
    String gameDir,
  ) {
    final l10n = AppLocalizations.of(context)!;
    final notifier = ref.read(configStateControllerProvider.notifier);
    return Container(
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.cardPaddingH(context),
        vertical: AppSizes.cardPaddingV(context),
      ),
      decoration: const BoxDecoration(
        color: AppColors.surfaceMedium,
        border: Border(bottom: BorderSide(color: AppColors.borderLight)),
      ),
      child: Row(
        children: [
          Text(
            l10n.headerNaiom,
            style: TextStyle(
              fontSize: AppSizes.fontXL(context),
              fontWeight: FontWeight.bold,
              color: AppColors.textPrimary,
              letterSpacing: 1.0,
            ),
          ),
          HeaderInfoIcon(
            tooltip: l10n.tooltipEditsNaiom,
            revealPath: p.join(gameDir, 'nams', 'nams.toml'),
          ),
          SizedBox(width: AppSizes.spacingMD(context)),
          Container(
            padding: EdgeInsets.symmetric(
              horizontal: AppSizes.paddingSM(context),
              vertical: AppSizes.paddingXS(context),
            ),
            decoration: BoxDecoration(
              color: AppColors.accentPrimary.withValues(alpha: 0.15),
              borderRadius: BorderRadius.circular(4),
            ),
            child: Text(
              l10n.naiomBetaBadge,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                fontWeight: FontWeight.bold,
                color: AppColors.accentPrimary,
              ),
            ),
          ),
          if (config.hasUnsavedChanges) ...[
            Padding(
              padding: EdgeInsets.only(left: AppSizes.spacingMD(context)),
              child: Text(
                '*',
                style: TextStyle(
                  fontSize: AppSizes.fontXL(context),
                  fontWeight: FontWeight.bold,
                  color: AppColors.warning,
                ),
              ),
            ),
            const Spacer(),
            HoverButton(
              label: l10n.buttonSave,
              color: AppColors.success,
              onTap: () => notifier.saveConfigs(gameDir),
            ),
            SizedBox(width: AppSizes.paddingXS(context)),
            HoverButton(
              label: l10n.buttonDiscard,
              color: AppColors.textMuted,
              onTap: () => notifier.discardChanges(gameDir),
            ),
          ] else
            const Spacer(),
        ],
      ),
    );
  }

  Widget _card(BuildContext context, String title, List<Widget> children) {
    return CollapsibleCard(title: title, children: children);
  }
}
