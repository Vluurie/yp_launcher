import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/config_state.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/widgets/config_field_bool.dart';
import 'package:yp_launcher/widgets/config_field_float.dart';
import 'package:yp_launcher/widgets/config_field_virtual_key.dart';
import 'package:yp_launcher/widgets/header_info_icon.dart';
import 'package:yp_launcher/widgets/hover_button.dart';
import 'package:yp_launcher/models/config_fields.dart';

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

  @override
  Widget build(BuildContext context) {
    final config = ref.watch(configStateControllerProvider);
    final notifier = ref.read(configStateControllerProvider.notifier);
    final gameDir = ref.watch(appStateControllerProvider).selectedDirectory;
    final nams = config.namsValues;
    final mouse = (nams['mouse'] as Map<String, dynamic>?) ?? {};

    return Container(
      color: AppColors.backgroundPrimary,
      child: Column(
        children: [
          _buildHeader(context, config, notifier, gameDir),
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
                          _wipBanner(context),
                          SizedBox(height: AppSizes.spacingLG(context)),
                          Row(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Expanded(
                                child: _buildLeftColumn(
                                  context,
                                  mouse,
                                  notifier,
                                ),
                              ),
                              SizedBox(width: AppSizes.spacingLG(context)),
                              Expanded(
                                child: _buildRightColumn(
                                  context,
                                  mouse,
                                  notifier,
                                ),
                              ),
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

  Widget _buildLeftColumn(
    BuildContext context,
    Map<String, dynamic> mouse,
    ConfigStateController notifier,
  ) {
    final l10n = AppLocalizations.of(context)!;
    return Column(
      children: [
        _card(context, l10n.cardThirdPersonCamera, [
          ConfigFieldBool(
            label: NamsFields.fixCameraAcceleration.label(l10n),
            value: mouse[NamsFields.fixCameraAcceleration.key] == true,
            onChanged: (v) => notifier.updateNams(
              NamsFields.fixCameraAcceleration.key,
              v,
              section: NamsFields.fixCameraAcceleration.section!,
            ),
            tooltip: NamsFields.fixCameraAcceleration.tooltip!(l10n),
          ),
          ConfigFieldFloat(
            label: NamsFields.sensitivity.label(l10n),
            value:
                (mouse[NamsFields.sensitivity.key] as num?)?.toDouble() ??
                NamsFields.sensitivity.defaultValue,
            onChanged: (v) => notifier.updateNams(
              NamsFields.sensitivity.key,
              v,
              section: NamsFields.sensitivity.section!,
            ),
            tooltip: NamsFields.sensitivity.tooltip!(l10n),
          ),
          _todo(
            context,
            l10n.todoThirdPersonMode,
            l10n.todoThirdPersonModeDesc,
          ),
          _todo(
            context,
            l10n.todoThirdPersonSensX,
            l10n.todoThirdPersonSensXDesc,
          ),
          _todo(
            context,
            l10n.todoThirdPersonSensY,
            l10n.todoThirdPersonSensYDesc,
          ),
          _todo(
            context,
            l10n.todoThirdPersonFixToggle,
            l10n.todoThirdPersonFixToggleDesc,
          ),
        ]),
        _card(context, l10n.cardPodAiming, [
          ConfigFieldBool(
            label: NamsFields.fixAimAcceleration.label(l10n),
            value: mouse[NamsFields.fixAimAcceleration.key] == true,
            onChanged: (v) => notifier.updateNams(
              NamsFields.fixAimAcceleration.key,
              v,
              section: NamsFields.fixAimAcceleration.section!,
            ),
            tooltip: NamsFields.fixAimAcceleration.tooltip!(l10n),
          ),
          ConfigFieldFloat(
            label: NamsFields.aimSensitivity.label(l10n),
            value:
                (mouse[NamsFields.aimSensitivity.key] as num?)?.toDouble() ??
                NamsFields.aimSensitivity.defaultValue,
            onChanged: (v) => notifier.updateNams(
              NamsFields.aimSensitivity.key,
              v,
              section: NamsFields.aimSensitivity.section!,
            ),
            tooltip: NamsFields.aimSensitivity.tooltip!(l10n),
          ),
          ConfigFieldFloat(
            label: NamsFields.aimOutputMultiplier.label(l10n),
            value:
                (mouse[NamsFields.aimOutputMultiplier.key] as num?)
                    ?.toDouble() ??
                NamsFields.aimOutputMultiplier.defaultValue,
            onChanged: (v) => notifier.updateNams(
              NamsFields.aimOutputMultiplier.key,
              v,
              section: NamsFields.aimOutputMultiplier.section!,
            ),
            tooltip: NamsFields.aimOutputMultiplier.tooltip!(l10n),
          ),
          ConfigFieldBool(
            label: NamsFields.disablePodPet.label(l10n),
            value: mouse[NamsFields.disablePodPet.key] == true,
            onChanged: (v) => notifier.updateNams(
              NamsFields.disablePodPet.key,
              v,
              section: NamsFields.disablePodPet.section!,
            ),
            tooltip: NamsFields.disablePodPet.tooltip!(l10n),
          ),
          _todo(context, l10n.todoAimMode, l10n.todoAimModeDesc),
          _todo(context, l10n.todoAimSensXY, l10n.todoAimSensXYDesc),
          _todo(context, l10n.todoAimFixToggle, l10n.todoAimFixToggleDesc),
          _todo(context, l10n.todoAutoFireToggle, l10n.todoAutoFireToggleDesc),
        ]),
        _card(context, l10n.cardMisc, [
          ConfigFieldVirtualKey(
            label: NamsFields.debugMenuKey.label(l10n),
            value:
                (mouse[NamsFields.debugMenuKey.key] as int?) ??
                NamsFields.debugMenuKey.defaultValue,
            onChanged: (v) => notifier.updateNams(
              NamsFields.debugMenuKey.key,
              v,
              section: NamsFields.debugMenuKey.section!,
            ),
            tooltip: NamsFields.debugMenuKey.tooltip!(l10n),
          ),
          _todo(context, l10n.todoCustomCursors, l10n.todoCustomCursorsDesc),
          _todo(context, l10n.todoStatusSounds, l10n.todoStatusSoundsDesc),
          _todo(
            context,
            l10n.todoReloadConfigBinding,
            l10n.todoReloadConfigBindingDesc,
          ),
        ]),
      ],
    );
  }

  Widget _buildRightColumn(
    BuildContext context,
    Map<String, dynamic> mouse,
    ConfigStateController notifier,
  ) {
    final l10n = AppLocalizations.of(context)!;
    return Column(
      children: [
        _card(context, l10n.cardMovementBindings, [
          _todo(context, l10n.todoMoveForward, l10n.todoMoveForwardDesc),
          _todo(context, l10n.todoJump, l10n.todoJumpDesc),
          _todo(context, l10n.todoWalk, l10n.todoWalkDesc),
          _todo(context, l10n.todoAutoRun, l10n.todoAutoRunDesc),
          _todo(
            context,
            l10n.todoDisableTapEvade,
            l10n.todoDisableTapEvadeDesc,
          ),
        ]),
        _card(context, l10n.cardCombatBindings, [
          _todo(context, l10n.todoLightAttack, l10n.todoLightAttackDesc),
          _todo(context, l10n.todoHeavyAttack, l10n.todoHeavyAttackDesc),
          _todo(context, l10n.todoFirePodDash, l10n.todoFirePodDashDesc),
          _todo(context, l10n.todoUseProgram, l10n.todoUseProgramDesc),
          _todo(context, l10n.todoLockOn, l10n.todoLockOnDesc),
          _todo(context, l10n.todoSwitchWeapon, l10n.todoSwitchWeaponDesc),
          _todo(
            context,
            l10n.todoNextPrevProgram,
            l10n.todoNextPrevProgramDesc,
          ),
          _todo(context, l10n.todoSelfDestruct, l10n.todoSelfDestructDesc),
        ]),
        _card(context, l10n.cardNonStandardBindings, [
          Opacity(
            opacity: 0.4,
            child: IgnorePointer(
              child: ConfigFieldVirtualKey(
                label: NamsFields.evadeKey.label(l10n),
                value:
                    (mouse[NamsFields.evadeKey.key] as int?) ??
                    NamsFields.evadeKey.defaultValue,
                onChanged: (_) {},
                tooltip: NamsFields.evadeKey.tooltip!(l10n),
              ),
            ),
          ),
          _todo(context, l10n.todoNextPrevItem, l10n.todoNextPrevItemDesc),
          _todo(context, l10n.todoUseItem, l10n.todoUseItemDesc),
        ]),
        _card(context, l10n.cardMenuNavigation, [
          _todo(context, l10n.todoMenuUpDown, l10n.todoMenuUpDownDesc),
          _todo(context, l10n.todoMenuOpen, l10n.todoMenuOpenDesc),
          _todo(context, l10n.todoMenuBackClose, l10n.todoMenuBackCloseDesc),
          _todo(context, l10n.todoMenuEnterSkip, l10n.todoMenuEnterSkipDesc),
          _todo(context, l10n.todoShortcutMenu, l10n.todoShortcutMenuDesc),
        ]),
      ],
    );
  }

  Widget _wipBanner(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return Container(
      width: double.infinity,
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.paddingMD(context),
        vertical: AppSizes.paddingSM(context),
      ),
      decoration: BoxDecoration(
        color: AppColors.warning.withValues(alpha: 0.08),
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(color: AppColors.warning.withValues(alpha: 0.3)),
      ),
      child: Row(
        children: [
          Icon(
            Icons.construction,
            size: AppSizes.iconLG(context),
            color: AppColors.warning,
          ),
          SizedBox(width: AppSizes.spacingMD(context)),
          Expanded(
            child: Text(
              l10n.naiomWipBanner,
              style: TextStyle(
                fontSize: AppSizes.fontSM(context),
                color: AppColors.warning,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _todo(BuildContext context, String label, String description) {
    final l10n = AppLocalizations.of(context)!;
    return Opacity(
      opacity: 0.35,
      child: IgnorePointer(
        child: Padding(
          padding: EdgeInsets.symmetric(vertical: AppSizes.paddingXS(context)),
          child: Row(
            children: [
              Icon(Icons.schedule, size: 14, color: AppColors.textMuted),
              SizedBox(width: AppSizes.spacingMD(context)),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      label,
                      style: TextStyle(
                        fontSize: AppSizes.fontSM(context),
                        color: AppColors.textSecondary,
                        fontWeight: FontWeight.w500,
                      ),
                    ),
                    Text(
                      description,
                      style: TextStyle(
                        fontSize: AppSizes.fontXS(context),
                        color: AppColors.textMuted,
                      ),
                    ),
                  ],
                ),
              ),
              Container(
                padding: EdgeInsets.symmetric(
                  horizontal: AppSizes.paddingXS(context),
                  vertical: 1,
                ),
                decoration: BoxDecoration(
                  borderRadius: BorderRadius.circular(3),
                  border: Border.all(
                    color: AppColors.textMuted.withValues(alpha: 0.3),
                  ),
                ),
                child: Text(
                  l10n.todoLabel,
                  style: TextStyle(
                    fontSize: 9,
                    color: AppColors.textMuted,
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildHeader(
    BuildContext context,
    ConfigData config,
    ConfigStateController notifier,
    String gameDir,
  ) {
    final l10n = AppLocalizations.of(context)!;
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
              color: AppColors.warning.withValues(alpha: 0.15),
              borderRadius: BorderRadius.circular(4),
            ),
            child: Text(
              l10n.wipLabel,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                fontWeight: FontWeight.bold,
                color: AppColors.warning,
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
    return Container(
      margin: EdgeInsets.only(bottom: AppSizes.paddingLG(context)),
      decoration: BoxDecoration(
        color: AppColors.backgroundCard,
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(color: AppColors.borderLight),
        boxShadow: const [
          BoxShadow(
            color: AppColors.shadow,
            blurRadius: 4,
            offset: Offset(0, 2),
          ),
        ],
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            width: double.infinity,
            padding: EdgeInsets.symmetric(
              horizontal: AppSizes.cardPaddingH(context),
              vertical: AppSizes.cardPaddingV(context),
            ),
            decoration: BoxDecoration(
              color: AppColors.surfaceMedium,
              borderRadius: BorderRadius.only(
                topLeft: Radius.circular(AppSizes.borderRadius(context)),
                topRight: Radius.circular(AppSizes.borderRadius(context)),
              ),
            ),
            child: Text(
              title,
              style: TextStyle(
                fontSize: AppSizes.fontSM(context),
                fontWeight: FontWeight.bold,
                color: AppColors.accentPrimary,
                letterSpacing: 1.0,
              ),
            ),
          ),
          Padding(
            padding: EdgeInsets.all(AppSizes.cardPaddingH(context)),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: children,
            ),
          ),
        ],
      ),
    );
  }
}
