import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/nams_settings_state.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/config_field_bool.dart';
import 'package:yp_launcher/widgets/header_info_icon.dart';
import 'package:yp_launcher/widgets/hover_button.dart';
import 'package:yp_launcher/widgets/config_field_float.dart';
import 'package:yp_launcher/widgets/config_field_keybind.dart';
import 'package:yp_launcher/models/config_fields.dart';

class YorhaProtocolView extends ConsumerStatefulWidget {
  const YorhaProtocolView({super.key});

  @override
  ConsumerState<YorhaProtocolView> createState() => _YorhaProtocolViewState();
}

class _YorhaProtocolViewState extends ConsumerState<YorhaProtocolView> {
  final _scrollController = ScrollController();
  bool _loaded = false;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!_loaded) {
        _loaded = true;
        ref.read(namsSettingsStateControllerProvider.notifier).loadSettings();
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
    final data = ref.watch(namsSettingsStateControllerProvider);
    final notifier = ref.read(namsSettingsStateControllerProvider.notifier);

    return Container(
      color: AppColors.backgroundPrimary,
      child: Column(
        children: [
          _buildHeader(context, data, notifier),
          Expanded(
            child: data.isLoading
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
                        children: [_buildContent(context, data, notifier)],
                      ),
                    ),
                  ),
          ),
        ],
      ),
    );
  }

  Widget _buildHeader(
    BuildContext context,
    NamsSettingsData data,
    NamsSettingsStateController notifier,
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
            l10n.headerYorhaProtocol,
            style: TextStyle(
              fontSize: AppSizes.fontXL(context),
              fontWeight: FontWeight.bold,
              color: AppColors.textPrimary,
              letterSpacing: 1.0,
            ),
          ),
          if (data.settingsPath != null)
            HeaderInfoIcon(
              tooltip: l10n.tooltipEditsSettingsJson,
              revealPath: data.settingsPath!,
            ),
          if (data.hasUnsavedChanges)
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
          if (data.hasUnsavedChanges) ...[
            HoverButton(
              label: l10n.buttonSave,
              color: AppColors.success,
              onTap: () => notifier.saveSettings(),
            ),
            SizedBox(width: AppSizes.paddingXS(context)),
            HoverButton(
              label: l10n.buttonDiscard,
              color: AppColors.textMuted,
              onTap: () => notifier.discardChanges(),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildContent(
    BuildContext context,
    NamsSettingsData data,
    NamsSettingsStateController notifier,
  ) {
    final l10n = AppLocalizations.of(context)!;
    final mainBinds = data.mainKeybinds;
    final ypBinds = data.ypKeybinds;
    final randomizer = data.randomizerConfig;
    final cheats = data.cheatsConfig;

    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Expanded(
          flex: 3,
          child: Column(
            children: [
              _card(context, l10n.cardKeybinds, [
                _keybindRow(
                  context,
                  YpKeybindFields.openWorkspace.label(l10n),
                  (mainBinds[YpKeybindFields.openWorkspace.key] as String?) ??
                      YpKeybindFields.openWorkspace.defaultValue,
                  (v) => notifier.updateKeybind(
                    YpKeybindFields.openWorkspace.section!,
                    YpKeybindFields.openWorkspace.key,
                    v,
                  ),
                  tooltip: YpKeybindFields.openWorkspace.tooltip!(l10n),
                ),
                _keybindRow(
                  context,
                  YpKeybindFields.freezeGame.label(l10n),
                  (ypBinds[YpKeybindFields.freezeGame.key] as String?) ??
                      YpKeybindFields.freezeGame.defaultValue,
                  (v) => notifier.updateKeybind(
                    YpKeybindFields.freezeGame.section!,
                    YpKeybindFields.freezeGame.key,
                    v,
                  ),
                  tooltip: YpKeybindFields.freezeGame.tooltip!(l10n),
                ),
                _keybindRow(
                  context,
                  YpKeybindFields.maxSpeed.label(l10n),
                  (ypBinds[YpKeybindFields.maxSpeed.key] as String?) ??
                      YpKeybindFields.maxSpeed.defaultValue,
                  (v) => notifier.updateKeybind(
                    YpKeybindFields.maxSpeed.section!,
                    YpKeybindFields.maxSpeed.key,
                    v,
                  ),
                  tooltip: YpKeybindFields.maxSpeed.tooltip!(l10n),
                ),
                _keybindRow(
                  context,
                  YpKeybindFields.freeCam.label(l10n),
                  (ypBinds[YpKeybindFields.freeCam.key] as String?) ??
                      YpKeybindFields.freeCam.defaultValue,
                  (v) => notifier.updateKeybind(
                    YpKeybindFields.freeCam.section!,
                    YpKeybindFields.freeCam.key,
                    v,
                  ),
                  tooltip: YpKeybindFields.freeCam.tooltip!(l10n),
                ),
                _keybindRow(
                  context,
                  YpKeybindFields.phaseJump.label(l10n),
                  (ypBinds[YpKeybindFields.phaseJump.key] as String?) ??
                      YpKeybindFields.phaseJump.defaultValue,
                  (v) => notifier.updateKeybind(
                    YpKeybindFields.phaseJump.section!,
                    YpKeybindFields.phaseJump.key,
                    v,
                  ),
                  tooltip: YpKeybindFields.phaseJump.tooltip!(l10n),
                ),
                _keybindRow(
                  context,
                  YpKeybindFields.toggleGameInput.label(l10n),
                  (ypBinds[YpKeybindFields.toggleGameInput.key] as String?) ??
                      YpKeybindFields.toggleGameInput.defaultValue,
                  (v) => notifier.updateKeybind(
                    YpKeybindFields.toggleGameInput.section!,
                    YpKeybindFields.toggleGameInput.key,
                    v,
                  ),
                  tooltip: YpKeybindFields.toggleGameInput.tooltip!(l10n),
                ),
                _keybindRow(
                  context,
                  YpKeybindFields.advanceFrame.label(l10n),
                  (ypBinds[YpKeybindFields.advanceFrame.key] as String?) ??
                      YpKeybindFields.advanceFrame.defaultValue,
                  (v) => notifier.updateKeybind(
                    YpKeybindFields.advanceFrame.section!,
                    YpKeybindFields.advanceFrame.key,
                    v,
                  ),
                  tooltip: YpKeybindFields.advanceFrame.tooltip!(l10n),
                ),
                _keybindRow(
                  context,
                  YpKeybindFields.devMode.label(l10n),
                  (ypBinds[YpKeybindFields.devMode.key] as String?) ??
                      YpKeybindFields.devMode.defaultValue,
                  (v) => notifier.updateKeybind(
                    YpKeybindFields.devMode.section!,
                    YpKeybindFields.devMode.key,
                    v,
                  ),
                  tooltip: YpKeybindFields.devMode.tooltip!(l10n),
                ),
              ]),
              _card(context, l10n.cardWorkspace, [
                ConfigFieldBool(
                  label: YpWorkspaceFields.gameKeybindsGlobal.label(l10n),
                  value: data.gameKeybindsGlobal,
                  onChanged: (v) => notifier.updateToggle(
                    YpWorkspaceFields.gameKeybindsGlobal.key,
                    v,
                  ),
                  tooltip: YpWorkspaceFields.gameKeybindsGlobal.tooltip!(l10n),
                ),
                ConfigFieldBool(
                  label: YpWorkspaceFields.loadingSpeedupEnabled.label(l10n),
                  value: data.loadingSpeedupEnabled,
                  onChanged: (v) => notifier.updateToggle(
                    YpWorkspaceFields.loadingSpeedupEnabled.key,
                    v,
                  ),
                  tooltip: YpWorkspaceFields.loadingSpeedupEnabled.tooltip!(
                    l10n,
                  ),
                ),
                ConfigFieldBool(
                  label: YpWorkspaceFields.shadersEnabled.label(l10n),
                  value: data.shadersEnabled,
                  onChanged: (v) => notifier.updateToggle(
                    YpWorkspaceFields.shadersEnabled.key,
                    v,
                  ),
                  tooltip: YpWorkspaceFields.shadersEnabled.tooltip!(l10n),
                ),
                ConfigFieldBool(
                  label: YpWorkspaceFields.soundEnabled.label(l10n),
                  value: data.soundEnabled,
                  onChanged: (v) => notifier.updateToggle(
                    YpWorkspaceFields.soundEnabled.key,
                    v,
                  ),
                  tooltip: YpWorkspaceFields.soundEnabled.tooltip!(l10n),
                ),
              ]),
            ],
          ),
        ),
        SizedBox(width: AppSizes.paddingMD(context)),
        Expanded(
          flex: 3,
          child: Column(
            children: [
              _card(context, l10n.cardCheats, [
                ConfigFieldFloat(
                  label: YpCheatsFields.damageMultiplier.label(l10n),
                  value:
                      (cheats[YpCheatsFields.damageMultiplier.key] as num?)
                          ?.toDouble() ??
                      YpCheatsFields.damageMultiplier.defaultValue,
                  onChanged: (v) => notifier.updateCheat(
                    YpCheatsFields.damageMultiplier.key,
                    v,
                  ),
                  tooltip: YpCheatsFields.damageMultiplier.tooltip!(l10n),
                ),
                ConfigFieldBool(
                  label: YpCheatsFields.syncEnemyLevels.label(l10n),
                  value: cheats[YpCheatsFields.syncEnemyLevels.key] == true,
                  onChanged: (v) => notifier.updateCheat(
                    YpCheatsFields.syncEnemyLevels.key,
                    v,
                  ),
                  tooltip: YpCheatsFields.syncEnemyLevels.tooltip!(l10n),
                ),
                ConfigFieldBool(
                  label: YpCheatsFields.infiniteHealth.label(l10n),
                  value: cheats[YpCheatsFields.infiniteHealth.key] == true,
                  onChanged: (v) => notifier.updateCheat(
                    YpCheatsFields.infiniteHealth.key,
                    v,
                  ),
                  tooltip: YpCheatsFields.infiniteHealth.tooltip!(l10n),
                ),
                ConfigFieldBool(
                  label: YpCheatsFields.infiniteJump.label(l10n),
                  value: cheats[YpCheatsFields.infiniteJump.key] == true,
                  onChanged: (v) =>
                      notifier.updateCheat(YpCheatsFields.infiniteJump.key, v),
                  tooltip: YpCheatsFields.infiniteJump.tooltip!(l10n),
                ),
                ConfigFieldBool(
                  label: YpCheatsFields.noPodCooldown.label(l10n),
                  value: cheats[YpCheatsFields.noPodCooldown.key] == true,
                  onChanged: (v) =>
                      notifier.updateCheat(YpCheatsFields.noPodCooldown.key, v),
                  tooltip: YpCheatsFields.noPodCooldown.tooltip!(l10n),
                ),
                ConfigFieldBool(
                  label: YpCheatsFields.infiniteAirDash.label(l10n),
                  value: cheats[YpCheatsFields.infiniteAirDash.key] == true,
                  onChanged: (v) => notifier.updateCheat(
                    YpCheatsFields.infiniteAirDash.key,
                    v,
                  ),
                  tooltip: YpCheatsFields.infiniteAirDash.tooltip!(l10n),
                ),
                Padding(
                  padding: EdgeInsets.only(top: AppSizes.paddingXS(context)),
                  child: Text(
                    l10n.cheatsAppliedNote,
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      color: AppColors.textMuted,
                      fontStyle: FontStyle.italic,
                    ),
                  ),
                ),
              ]),
              _card(context, l10n.cardRandomizer, [
                ConfigFieldBool(
                  label: YpRandomizerFields.autoStartOnStartup.label(l10n),
                  value:
                      randomizer[YpRandomizerFields.autoStartOnStartup.key] ==
                      true,
                  onChanged: (v) => notifier.updateRandomizer(
                    YpRandomizerFields.autoStartOnStartup.key,
                    v,
                  ),
                  tooltip: YpRandomizerFields.autoStartOnStartup.tooltip!(l10n),
                ),
                ConfigFieldBool(
                  label: YpRandomizerFields.randomizeGroundEnemies.label(l10n),
                  value:
                      randomizer[YpRandomizerFields
                          .randomizeGroundEnemies
                          .key] ==
                      true,
                  onChanged: (v) => notifier.updateRandomizer(
                    YpRandomizerFields.randomizeGroundEnemies.key,
                    v,
                  ),
                  tooltip: YpRandomizerFields.randomizeGroundEnemies.tooltip!(
                    l10n,
                  ),
                ),
                ConfigFieldBool(
                  label: YpRandomizerFields.randomizeFlyingEnemies.label(l10n),
                  value:
                      randomizer[YpRandomizerFields
                          .randomizeFlyingEnemies
                          .key] ==
                      true,
                  onChanged: (v) => notifier.updateRandomizer(
                    YpRandomizerFields.randomizeFlyingEnemies.key,
                    v,
                  ),
                  tooltip: YpRandomizerFields.randomizeFlyingEnemies.tooltip!(
                    l10n,
                  ),
                ),
                ConfigFieldBool(
                  label: YpRandomizerFields.allowBigEnemies.label(l10n),
                  value:
                      randomizer[YpRandomizerFields.allowBigEnemies.key] ==
                      true,
                  onChanged: (v) => notifier.updateRandomizer(
                    YpRandomizerFields.allowBigEnemies.key,
                    v,
                  ),
                  tooltip: YpRandomizerFields.allowBigEnemies.tooltip!(l10n),
                ),
                ConfigFieldBool(
                  label: YpRandomizerFields.includeDlcEnemies.label(l10n),
                  value:
                      randomizer[YpRandomizerFields.includeDlcEnemies.key] ==
                      true,
                  onChanged: (v) => notifier.updateRandomizer(
                    YpRandomizerFields.includeDlcEnemies.key,
                    v,
                  ),
                  tooltip: YpRandomizerFields.includeDlcEnemies.tooltip!(l10n),
                ),
              ]),
            ],
          ),
        ),
      ],
    );
  }

  Widget _keybindRow(
    BuildContext context,
    String label,
    String value,
    ValueChanged<String> onChanged, {
    String? tooltip,
    GlobalKey? rowKey,
  }) {
    return Padding(
      key: rowKey,
      padding: EdgeInsets.symmetric(vertical: AppSizes.paddingXS(context)),
      child: Row(
        children: [
          Expanded(
            child: tooltip != null
                ? Tooltip(
                    message: tooltip,
                    child: Text(
                      label,
                      style: TextStyle(
                        fontSize: AppSizes.fontSM(context),
                        color: AppColors.textSecondary,
                      ),
                    ),
                  )
                : Text(
                    label,
                    style: TextStyle(
                      fontSize: AppSizes.fontSM(context),
                      color: AppColors.textSecondary,
                    ),
                  ),
          ),
          SizedBox(
            width: 100,
            child: ConfigFieldKeybind(
              label: '',
              value: value,
              onChanged: onChanged,
            ),
          ),
        ],
      ),
    );
  }

  Widget _card(BuildContext context, String title, List<Widget> children) {
    return Container(
      margin: EdgeInsets.only(bottom: AppSizes.paddingMD(context)),
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
