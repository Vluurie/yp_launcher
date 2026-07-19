import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/config_state.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/widgets/collapsible_card.dart';
import 'package:yp_launcher/widgets/config_field_bool.dart';
import 'package:yp_launcher/widgets/config_field_dropdown.dart';
import 'package:yp_launcher/widgets/config_field_int.dart';
import 'package:yp_launcher/widgets/config_field_slider.dart';
import 'package:yp_launcher/widgets/header_info_icon.dart';
import 'package:yp_launcher/widgets/hover_button.dart';
import 'package:yp_launcher/widgets/config_field_preview.dart'
    show ConfigComparison;
import 'package:yp_launcher/models/config_fields.dart';

class LodmodView extends ConsumerStatefulWidget {
  const LodmodView({super.key});

  @override
  ConsumerState<LodmodView> createState() => _LodmodViewState();
}

class _LodmodViewState extends ConsumerState<LodmodView> {
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
                : _buildContent(context, config, notifier),
          ),
        ],
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
            l10n.headerLodMod,
            style: TextStyle(
              fontSize: AppSizes.fontXL(context),
              fontWeight: FontWeight.bold,
              color: AppColors.textPrimary,
              letterSpacing: 1.0,
            ),
          ),
          HeaderInfoIcon(
            tooltip: l10n.tooltipEditsLodmodToml,
            revealPath: p.join(gameDir, 'nams', 'lodmod.toml'),
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

  Widget _slider(
    ConfigData config,
    ConfigStateController notifier,
    String gameDir,
    AppLocalizations l10n,
    ConfigField<double> field, {
    int decimals = 2,
  }) {
    return ConfigFieldSlider(
      label: field.label(l10n),
      tooltip: field.tooltip?.call(l10n),
      restartRequired: field.restartRequired,
      showApplyMarker: true,
      min: field.min!.toDouble(),
      max: field.max!.toDouble(),
      step: field.step!.toDouble(),
      decimals: decimals,
      value: (config.lodmodValues[field.key] as num?)?.toDouble() ??
          field.defaultValue,
      onChanged: (v) => notifier.updateLodmodLive(gameDir, field.key, v),
    );
  }

  Widget _bool(
    ConfigData config,
    ConfigStateController notifier,
    String gameDir,
    AppLocalizations l10n,
    ConfigField<bool> field,
  ) {
    return ConfigFieldBool(
      label: field.label(l10n),
      tooltip: field.tooltip?.call(l10n),
      restartRequired: field.restartRequired,
      showApplyMarker: true,
      value: config.lodmodValues[field.key] == true,
      onChanged: (v) => notifier.updateLodmodLive(gameDir, field.key, v),
    );
  }

  Widget _int(
    ConfigData config,
    ConfigStateController notifier,
    String gameDir,
    AppLocalizations l10n,
    ConfigField<int> field,
  ) {
    return ConfigFieldInt(
      label: field.label(l10n),
      tooltip: field.tooltip?.call(l10n),
      restartRequired: field.restartRequired,
      showApplyMarker: true,
      min: field.min?.toInt(),
      max: field.max?.toInt(),
      value: (config.lodmodValues[field.key] as int?) ?? field.defaultValue,
      onChanged: (v) => notifier.updateLodmodLive(gameDir, field.key, v),
    );
  }

  Widget _lodMultiplierDropdown(
    ConfigData config,
    ConfigStateController notifier,
    String gameDir,
    AppLocalizations l10n,
  ) {
    final field = LodModFields.lodMultiplier;
    final current =
        (config.lodmodValues[field.key] as num?)?.toDouble() ??
            field.defaultValue;
    final options = field.allowedValues!.map((n) => n.toInt()).toList();
    return ConfigFieldDropdown(
      label: field.label(l10n),
      tooltip: field.tooltip?.call(l10n),
      restartRequired: field.restartRequired,
      showApplyMarker: true,
      value: current.round(),
      options: options,
      onChanged: (v) =>
          notifier.updateLodmodLive(gameDir, field.key, v.toDouble()),
    );
  }

  Widget _shadowResDropdown(
    ConfigData config,
    ConfigStateController notifier,
    String gameDir,
    AppLocalizations l10n,
  ) {
    final field = LodModFields.shadowResolution;
    return ConfigFieldDropdown(
      label: field.label(l10n),
      tooltip: field.tooltip?.call(l10n),
      restartRequired: field.restartRequired,
      showApplyMarker: true,
      value: (config.lodmodValues[field.key] as int?) ?? field.defaultValue,
      options: field.allowedValues!.cast<int>(),
      onChanged: (v) => notifier.updateLodmodLive(gameDir, field.key, v),
    );
  }

  Widget _resetButton(
    BuildContext context,
    ConfigStateController notifier,
    String gameDir,
    AppLocalizations l10n,
  ) {
    return HoverButton(
      label: l10n.lodModResetButton,
      color: AppColors.warning,
      onTap: () async {
        final confirmed = await showDialog<bool>(
          context: context,
          builder: (ctx) => AlertDialog(
            backgroundColor: AppColors.backgroundCard,
            title: Text(
              l10n.lodModResetConfirmTitle,
              style: TextStyle(
                fontSize: AppSizes.fontLG(ctx),
                color: AppColors.textPrimary,
              ),
            ),
            content: Text(
              l10n.lodModResetConfirmBody,
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
        if (confirmed != true) return;
        await notifier.resetLodmodToDefaults(gameDir);
      },
    );
  }

  Widget _experimentalWarning(BuildContext context, AppLocalizations l10n) {
    return Container(
      margin: EdgeInsets.only(bottom: AppSizes.paddingSM(context)),
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.paddingMD(context),
        vertical: AppSizes.paddingSM(context),
      ),
      decoration: BoxDecoration(
        color: AppColors.error.withValues(alpha: 0.08),
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(color: AppColors.error.withValues(alpha: 0.5)),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(
            Icons.warning_amber_rounded,
            size: AppSizes.iconMD(context),
            color: AppColors.error,
          ),
          SizedBox(width: AppSizes.spacingSM(context)),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  l10n.experimentalWarningTitle,
                  style: TextStyle(
                    fontSize: AppSizes.fontMD(context),
                    fontWeight: FontWeight.w700,
                    color: AppColors.error,
                  ),
                ),
                const SizedBox(height: 4),
                Text(
                  l10n.experimentalWarningBody,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.textSecondary,
                    height: 1.35,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _giWorkgroupDropdown(
    ConfigData config,
    ConfigStateController notifier,
    String gameDir,
    AppLocalizations l10n,
  ) {
    final field = LodModFields.giWorkgroupSize;
    return ConfigFieldDropdown(
      label: field.label(l10n),
      tooltip: field.tooltip?.call(l10n),
      restartRequired: field.restartRequired,
      showApplyMarker: true,
      value: (config.lodmodValues[field.key] as int?) ?? field.defaultValue,
      options: field.allowedValues!.cast<int>(),
      onChanged: (v) => notifier.updateLodmodLive(gameDir, field.key, v),
    );
  }

  Widget _buildContent(
    BuildContext context,
    ConfigData config,
    ConfigStateController notifier,
  ) {
    final l10n = AppLocalizations.of(context)!;
    final gameDir = ref.read(appStateControllerProvider).selectedDirectory;
    final lod = config.lodmodValues;
    final lodEnabled = lod[LodModFields.enabled.key] == true;

    return Scrollbar(
      controller: _scrollController,
      thumbVisibility: true,
      child: SingleChildScrollView(
        controller: _scrollController,
        padding: EdgeInsets.all(AppSizes.contentPadding(context)),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Container(
              width: double.infinity,
              margin: EdgeInsets.only(bottom: AppSizes.paddingMD(context)),
              padding: EdgeInsets.symmetric(
                horizontal: AppSizes.paddingMD(context),
                vertical: AppSizes.paddingSM(context),
              ),
              decoration: BoxDecoration(
                color: AppColors.surfaceLight,
                borderRadius: BorderRadius.circular(
                  AppSizes.borderRadius(context),
                ),
                border: Border.all(color: AppColors.borderLight),
              ),
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.center,
                children: [
                  Expanded(
                    child: Text(
                      l10n.lodModDescription,
                      style: TextStyle(
                        fontSize: AppSizes.fontXS(context),
                        color: AppColors.textMuted,
                        height: 1.4,
                      ),
                    ),
                  ),
                  SizedBox(width: AppSizes.spacingMD(context)),
                  _resetButton(context, notifier, gameDir, l10n),
                ],
              ),
            ),
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Expanded(
                  child: _bool(
                    config, notifier, gameDir, l10n, LodModFields.enabled,
                  ),
                ),
                SizedBox(width: AppSizes.spacingLG(context)),
                Expanded(
                  child: ConfigComparison(
                    beforeImage: 'assets/images/config/lod_mod_off.jpg',
                    afterImage: 'assets/images/config/lod_mod_on.jpg',
                    beforeLabel: l10n.comparisonVanilla,
                    afterLabel: l10n.comparisonDefaultEnabled,
                  ),
                ),
              ],
            ),
            if (lodEnabled) ...[
              SizedBox(height: AppSizes.spacingLG(context)),
              Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Expanded(
                    child: Column(
                      children: [
                        _card(context, l10n.cardLevelOfDetail, [
                          _lodMultiplierDropdown(
                              config, notifier, gameDir, l10n),
                          _bool(config, notifier, gameDir, l10n,
                              LodModFields.disableManualCulling),
                        ]),
                        _card(context, l10n.cardAmbientOcclusion, [
                          _slider(config, notifier, gameDir, l10n,
                              LodModFields.aoMultiplierWidth, decimals: 2),
                          _slider(config, notifier, gameDir, l10n,
                              LodModFields.aoMultiplierHeight, decimals: 2),
                          ConfigComparison(
                            beforeImage:
                                'assets/images/config/ao_multiplier_0.5.jpg',
                            afterImage:
                                'assets/images/config/ao_multiplier_2.0.jpg',
                            beforeLabel: l10n.comparisonAo05x,
                            afterLabel: l10n.comparisonAo20x,
                          ),
                        ]),
                        _card(context, l10n.cardPostProcessing, [
                          _bool(config, notifier, gameDir, l10n,
                              LodModFields.disableVignette),
                          ConfigComparison(
                            beforeImage: 'assets/images/config/vignett_on.jpg',
                            afterImage: 'assets/images/config/vignett_off.jpg',
                            beforeLabel: l10n.comparisonVignetteOn,
                            afterLabel: l10n.comparisonVignetteOff,
                          ),
                        ]),
                        _card(context, l10n.cardGlobalIllumination, [
                          _bool(config, notifier, gameDir, l10n,
                              LodModFields.giEnabled),
                          _giWorkgroupDropdown(
                              config, notifier, gameDir, l10n),
                          _slider(config, notifier, gameDir, l10n,
                              LodModFields.giMinLightExtent, decimals: 2),
                        ]),
                      ],
                    ),
                  ),
                  SizedBox(width: AppSizes.spacingLG(context)),
                  Expanded(
                    child: Column(
                      children: [
                        _card(context, l10n.cardShadows, [
                          _shadowResDropdown(config, notifier, gameDir, l10n),
                          ConfigComparison(
                            beforeImage:
                                'assets/images/config/shadow_resolution_2148.jpg',
                            afterImage:
                                'assets/images/config/shadow_resolution_8192.jpg',
                            beforeLabel: l10n.comparison2048,
                            afterLabel: l10n.comparison8192,
                          ),
                          _slider(config, notifier, gameDir, l10n,
                              LodModFields.shadowDistanceMultiplier,
                              decimals: 1),
                          ConfigComparison(
                            beforeImage:
                                'assets/images/config/shadow_distance_default.jpg',
                            afterImage:
                                'assets/images/config/shadow_distance_multiplier_2.0.jpg',
                            beforeLabel: l10n.comparisonDefault,
                            afterLabel: l10n.comparison20x,
                          ),
                          _slider(config, notifier, gameDir, l10n,
                              LodModFields.shadowDistanceMinimum, decimals: 0),
                          _slider(config, notifier, gameDir, l10n,
                              LodModFields.shadowDistanceMaximum, decimals: 0),
                          _slider(config, notifier, gameDir, l10n,
                              LodModFields.shadowDistancePss, decimals: 0),
                          ConfigComparison(
                            beforeImage:
                                'assets/images/config/shadow_dist_filter_off.jpg',
                            afterImage:
                                'assets/images/config/pss_distance_shadow_-5.0.jpg',
                            beforeLabel: l10n.comparisonDefault,
                            afterLabel: l10n.comparisonPssMinus5,
                          ),
                          _slider(config, notifier, gameDir, l10n,
                              LodModFields.shadowFilterStrengthBias,
                              decimals: 1),
                          ConfigComparison(
                            beforeImage:
                                'assets/images/config/shadow_dist_filter_off.jpg',
                            afterImage:
                                'assets/images/config/pss_distance_shadow_-5.0.jpg',
                            beforeLabel: l10n.comparisonDefault,
                            afterLabel: l10n.comparisonBiasMinus5,
                          ),
                          _slider(config, notifier, gameDir, l10n,
                              LodModFields.shadowFilterStrengthMinimum,
                              decimals: 1),
                          ConfigComparison(
                            beforeImage:
                                'assets/images/config/shadow_filter_min_off.jpg',
                            afterImage:
                                'assets/images/config/shadow_filter_min_3.0.jpg',
                            beforeLabel: l10n.comparisonOff,
                            afterLabel: l10n.comparison30,
                          ),
                          _slider(config, notifier, gameDir, l10n,
                              LodModFields.shadowFilterStrengthMaximum,
                              decimals: 1),
                          _bool(config, notifier, gameDir, l10n,
                              LodModFields.shadowModelHq),
                          _bool(config, notifier, gameDir, l10n,
                              LodModFields.shadowModelForceAll),
                          ConfigComparison(
                            beforeImage:
                                'assets/images/config/hd_shadow_models_off.jpg',
                            afterImage:
                                'assets/images/config/hq_shadow_models_on.jpg',
                            beforeLabel: l10n.comparisonOff,
                            afterLabel: l10n.comparisonHqForceAll,
                          ),
                        ]),
                        _card(context, l10n.cardExperimental, [
                          _experimentalWarning(context, l10n),
                          _bool(config, notifier, gameDir, l10n,
                              LodModFields.fpsUncapInMenus),
                          _bool(config, notifier, gameDir, l10n,
                              LodModFields.fpsUncapInGameplay),
                          _int(config, notifier, gameDir, l10n,
                              LodModFields.fpsLimit),
                        ]),
                      ],
                    ),
                  ),
                ],
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _card(BuildContext context, String title, List<Widget> children) {
    return CollapsibleCard(title: title, children: children);
  }
}
