import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/config_state.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/widgets/app_dialog.dart';
import 'package:yp_launcher/widgets/collapsible_card.dart';
import 'package:yp_launcher/widgets/config_field_bool.dart';
import 'package:yp_launcher/widgets/config_field_dropdown.dart';
import 'package:yp_launcher/widgets/config_field_grid_rules.dart';
import 'package:yp_launcher/widgets/config_field_int.dart';
import 'package:yp_launcher/widgets/config_error_banner.dart';
import 'package:yp_launcher/widgets/config_field_slider.dart';
import 'package:yp_launcher/widgets/header_info_icon.dart';
import 'package:yp_launcher/widgets/hover_button.dart';
import 'package:yp_launcher/widgets/two_column_layout.dart';
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
                ? Center(
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
      decoration: BoxDecoration(
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
      value: field.valueIn(config.lodmodValues),
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
      value: field.valueIn(config.lodmodValues),
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
      value: field.valueIn(config.lodmodValues),
      onChanged: (v) => notifier.updateLodmodLive(
        gameDir,
        field.key,
        v,
        section: field.section,
      ),
    );
  }

  Widget _sectionBool(
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
      value: field.valueIn(config.lodmodValues),
      onChanged: (v) => notifier.updateLodmodLive(
        gameDir,
        field.key,
        v,
        section: field.section,
      ),
    );
  }

  Widget _bloomReferenceSlider(
    ConfigData config,
    ConfigStateController notifier,
    String gameDir,
    AppLocalizations l10n,
    ConfigField<int> field,
  ) {
    const floor = LodModFields.bloomReferenceMinimum;
    final current = field.valueIn(config.lodmodValues);
    return ConfigFieldSlider(
      label: field.label(l10n),
      tooltip: field.tooltip?.call(l10n),
      restartRequired: field.restartRequired,
      showApplyMarker: true,
      min: field.min!.toDouble(),
      max: field.max!.toDouble(),
      step: field.step!.toDouble(),
      decimals: 0,
      valueLabel: current == 0 ? l10n.bloomReferenceVanilla : '$current',
      value: current.toDouble(),
      onChanged: (v) {
        final rounded = v.round();
        final snapped = rounded == 0
            ? 0
            : (rounded < floor ? (current == 0 ? floor : 0) : rounded);
        notifier.updateLodmodLive(gameDir, field.key, snapped);
      },
    );
  }

  Widget _bloom2017Button(
    BuildContext context,
    ConfigData config,
    ConfigStateController notifier,
    String gameDir,
    AppLocalizations l10n,
  ) {
    final active =
        LodModFields.bloomReferenceHeight.valueIn(config.lodmodValues) ==
        LodModFields.bloom2017ReferenceHeight;
    return Padding(
      padding: EdgeInsets.only(top: AppSizes.spacingSM(context)),
      child: Align(
        alignment: Alignment.centerLeft,
        child: Tooltip(
          message: l10n.bloom2017Tooltip,
          decoration: BoxDecoration(
            color: const Color(0xE6303030),
            borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
          ),
          child: HoverButton(
            label: l10n.bloom2017Button,
            color: active ? AppColors.textMuted : AppColors.accentPrimary,
            onTap: () => notifier.applyLodmodBloom2017Preset(gameDir),
          ),
        ),
      ),
    );
  }

  Widget _gridRules(
    ConfigData config,
    ConfigStateController notifier,
    String gameDir,
    AppLocalizations l10n,
    ConfigField<List<dynamic>> field,
    List<GridRuleColumn> columns,
  ) {
    return ConfigFieldGridRules(
      label: field.label(l10n),
      tooltip: field.tooltip?.call(l10n),
      restartRequired: field.restartRequired,
      rules: field.valueIn(config.lodmodValues),
      columns: columns,
      addLabel: l10n.gridRuleAdd,
      emptyLabel: l10n.gridRuleEmpty,
      onChanged: (rules) => notifier.updateLodmodLive(
        gameDir,
        field.key,
        rules,
        section: field.section,
      ),
    );
  }

  Widget _ringsDropdown(
    ConfigData config,
    ConfigStateController notifier,
    String gameDir,
    AppLocalizations l10n,
  ) {
    final field = LodModFields.highGridsRings;
    final current = field.valueIn(config.lodmodValues);
    return ConfigFieldDropdown(
      label: field.label(l10n),
      tooltip: field.tooltip?.call(l10n),
      restartRequired: field.restartRequired,
      showApplyMarker: true,
      value: current,
      options: field.allowedValues!.cast<int>(),
      labels: {
        for (final rings in field.allowedValues!.cast<int>())
          rings: l10n.highGridsCellCount(
            rings,
            LodModFields.highGridsCellCounts[rings] ?? 7,
          ),
      },
      onChanged: (v) => notifier.updateLodmodLive(
        gameDir,
        field.key,
        v,
        section: field.section,
      ),
    );
  }

  Widget _lodMultiplierDropdown(
    ConfigData config,
    ConfigStateController notifier,
    String gameDir,
    AppLocalizations l10n,
  ) {
    final field = LodModFields.lodMultiplier;
    final current = field.valueIn(config.lodmodValues);
    final options = field.allowedValues!
        .map((n) => (n.toDouble() * 100).round())
        .toList();
    final labels = <int, String>{
      0: l10n.lodMultiplierOptionQuality,
      75: l10n.lodMultiplierOptionPerformance,
      100: l10n.lodMultiplierOptionVanilla,
      1000: l10n.lodMultiplierOptionFar,
    };
    return ConfigFieldDropdown(
      label: field.label(l10n),
      tooltip: field.tooltip?.call(l10n),
      restartRequired: field.restartRequired,
      showApplyMarker: true,
      value: (current * 100).round(),
      options: options,
      labels: labels,
      onChanged: (v) => notifier.updateLodmodLive(gameDir, field.key, v / 100),
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
      value: field.valueIn(config.lodmodValues),
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
          builder: (ctx) => AppDialog(
            title: Text(l10n.lodModResetConfirmTitle),
            content: Text(l10n.lodModResetConfirmBody),
            actions: [
              TextButton(
                onPressed: () => Navigator.of(ctx).pop(false),
                child: Text(
                  MaterialLocalizations.of(ctx).cancelButtonLabel,
                  style: TextStyle(color: AppColors.textMuted),
                ),
              ),
              TextButton(
                onPressed: () => Navigator.of(ctx).pop(true),
                child: Text(
                  l10n.lodModResetConfirmAction,
                  style: TextStyle(
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
    return Padding(
      padding: EdgeInsets.only(bottom: AppSizes.paddingSM(context)),
      child: Text(
        l10n.experimentalWarningBody,
        style: TextStyle(
          fontSize: AppSizes.fontXS(context),
          color: AppColors.textMuted,
          height: 1.35,
        ),
      ),
    );
  }

  Widget _highGridsWipNote(BuildContext context, AppLocalizations l10n) {
    return Container(
      margin: EdgeInsets.only(bottom: AppSizes.paddingSM(context)),
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.paddingMD(context),
        vertical: AppSizes.paddingSM(context),
      ),
      decoration: BoxDecoration(
        color: AppColors.warning.withValues(alpha: 0.08),
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(color: AppColors.warning.withValues(alpha: 0.5)),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(
            Icons.construction_rounded,
            size: AppSizes.iconMD(context),
            color: AppColors.warning,
          ),
          SizedBox(width: AppSizes.spacingSM(context)),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  l10n.highGridsWipTitle,
                  style: TextStyle(
                    fontSize: AppSizes.fontMD(context),
                    fontWeight: FontWeight.w700,
                    color: AppColors.warning,
                  ),
                ),
                const SizedBox(height: 4),
                Text(
                  l10n.highGridsWipBody,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.textSecondary,
                    height: 1.35,
                  ),
                ),
                SizedBox(height: AppSizes.spacingSM(context)),
                Text(
                  l10n.highGridsWipContribute,
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

  Widget _bloomDropLevelsDropdown(
    ConfigData config,
    ConfigStateController notifier,
    String gameDir,
    AppLocalizations l10n,
  ) {
    final field = LodModFields.bloomDropCoarseLevels;
    return ConfigFieldDropdown(
      label: field.label(l10n),
      tooltip: field.tooltip?.call(l10n),
      restartRequired: field.restartRequired,
      showApplyMarker: true,
      value: field.valueIn(config.lodmodValues),
      options: field.allowedValues!.cast<int>(),
      labels: {
        0: l10n.bloomDropLevelsOff,
        1: l10n.bloomDropLevelsWidest,
        2: l10n.bloomDropLevelsTwoWidest,
      },
      onChanged: (v) => notifier.updateLodmodLive(gameDir, field.key, v),
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
      value: field.valueIn(config.lodmodValues),
      options: field.allowedValues!.cast<int>(),
      onChanged: (v) => notifier.updateLodmodLive(gameDir, field.key, v),
    );
  }

  Widget _cascadesDropdown(
    ConfigData config,
    ConfigStateController notifier,
    String gameDir,
    AppLocalizations l10n,
  ) {
    final field = LodModFields.shadowCascades;
    return ConfigFieldDropdown(
      label: field.label(l10n),
      tooltip: field.tooltip?.call(l10n),
      restartRequired: field.restartRequired,
      showApplyMarker: true,
      value: field.valueIn(config.lodmodValues),
      options: field.allowedValues!.cast<int>(),
      onChanged: (v) => notifier.updateLodmodLive(gameDir, field.key, v),
    );
  }

  List<Widget> _blurScaleSliders(
    ConfigData config,
    ConfigStateController notifier,
    String gameDir,
    AppLocalizations l10n,
  ) {
    final field = LodModFields.shadowBlurScale;
    final raw = field.valueIn(config.lodmodValues);
    double entry(int i) {
      if (i >= raw.length) return 1.0;
      final v = raw[i];
      if (v is num) return v.toDouble();
      return double.tryParse('$v') ?? 1.0;
    }

    final values = [for (var i = 0; i < 4; i++) entry(i)];

    return [
      for (var i = 0; i < 4; i++)
        ConfigFieldSlider(
          label: '${field.label(l10n)} - ${l10n.shadowBlurCascade(i + 1)}',
          tooltip: i == 0 ? field.tooltip?.call(l10n) : null,
          restartRequired: field.restartRequired,
          showApplyMarker: true,
          min: field.min!.toDouble(),
          max: field.max!.toDouble(),
          step: field.step!.toDouble(),
          decimals: 2,
          value: values[i],
          onChanged: (v) {
            final updated = List<double>.from(values);
            updated[i] = v;
            notifier.updateLodmodLive(gameDir, field.key, updated);
          },
        ),
    ];
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
            if (config.lodmodError != null)
              ConfigErrorBanner(
                fileName: 'lodmod.toml',
                error: config.lodmodError!,
              ),
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
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    l10n.lodModDescription,
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      color: AppColors.textMuted,
                      height: 1.4,
                    ),
                  ),
                  SizedBox(height: AppSizes.spacingXS(context)),
                  Text(
                    l10n.comparisonResolutionNote,
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      color: AppColors.textMuted,
                      height: 1.4,
                      fontStyle: FontStyle.italic,
                    ),
                  ),
                  SizedBox(height: AppSizes.spacingMD(context)),
                  Wrap(
                    spacing: AppSizes.paddingXS(context),
                    runSpacing: AppSizes.paddingXS(context),
                    children: [_resetButton(context, notifier, gameDir, l10n)],
                  ),
                ],
              ),
            ),
            _twoColumn(
              context,
              _bool(config, notifier, gameDir, l10n, LodModFields.enabled),
              ConfigComparison(
                beforeImage: 'assets/images/config/lod_mod_off.jpg',
                afterImage: 'assets/images/config/lod_mod_on.jpg',
                beforeLabel: l10n.comparisonVanilla,
                afterLabel: l10n.comparisonDefaultEnabled,
              ),
            ),
            if (lodEnabled) ...[
              SizedBox(height: AppSizes.spacingLG(context)),
              _twoColumn(
                context,
                Column(
                  children: [
                    _card(context, l10n.cardPerformance, [
                      Padding(
                        padding: EdgeInsets.only(
                          bottom: AppSizes.paddingSM(context),
                        ),
                        child: Text(
                          l10n.performanceCardNote,
                          style: TextStyle(
                            fontSize: AppSizes.fontXS(context),
                            color: AppColors.textMuted,
                            height: 1.35,
                          ),
                        ),
                      ),
                      _bool(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.constantBufferDedup,
                      ),
                      _bool(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.constantBufferUploadOnBind,
                      ),
                      _bool(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.decreaseStutterDuringGridLoading,
                      ),
                    ]),
                    _card(context, l10n.cardLevelOfDetail, [
                      _lodMultiplierDropdown(config, notifier, gameDir, l10n),
                      _bool(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.disableManualCulling,
                      ),
                      _bool(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.disableCameraCulling,
                      ),
                      if (!LodModFields.layoutSpawnEverything.valueIn(
                        config.lodmodValues,
                      ))
                        _slider(
                          config,
                          notifier,
                          gameDir,
                          l10n,
                          LodModFields.layoutSpawnRange,
                          decimals: 0,
                        ),
                      _bool(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.layoutSpawnEverything,
                      ),
                    ]),
                    _card(context, l10n.cardAmbientOcclusion, [
                      _slider(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.aoMultiplierWidth,
                        decimals: 2,
                      ),
                      _slider(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.aoMultiplierHeight,
                        decimals: 2,
                      ),
                      ConfigComparison(
                        beforeImage:
                            'assets/images/config/ao_multiplier_0.5.jpg',
                        afterImage:
                            'assets/images/config/ao_multiplier_2.0.jpg',
                        beforeLabel: l10n.comparisonAo05x,
                        afterLabel: l10n.comparisonAo20x,
                      ),
                      _bool(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.aoFadeFix,
                      ),
                    ]),
                    _card(context, l10n.cardPostProcessing, [
                      _bool(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.disableVignette,
                      ),
                      ConfigComparison(
                        beforeImage: 'assets/images/config/vignett_on.jpg',
                        afterImage: 'assets/images/config/vignett_off.jpg',
                        beforeLabel: l10n.comparisonVignetteOn,
                        afterLabel: l10n.comparisonVignetteOff,
                      ),
                      _bool(config, notifier, gameDir, l10n, LodModFields.fxaa),
                      ConfigComparison(
                        beforeImage: 'assets/images/config/fxaa_off.jpg',
                        afterImage: 'assets/images/config/fxaa_on.jpg',
                        beforeLabel: l10n.comparisonOff,
                        afterLabel: l10n.comparisonOn,
                      ),
                      _bool(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.msaaPrepassFix,
                      ),
                      _bool(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.msaaShadowMaskFix,
                      ),
                      ConfigComparison(
                        beforeImage: 'assets/images/config/msaa8_vanilla.jpg',
                        afterImage:
                            'assets/images/config/msaa8_shadow_mask_fix.jpg',
                        beforeLabel: l10n.comparisonOff,
                        afterLabel: l10n.comparisonOn,
                      ),
                      _bool(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.disableHdr,
                      ),
                    ]),
                  ],
                ),
                Column(
                  children: [
                    _card(context, l10n.cardBloom, [
                      _bloomReferenceSlider(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.bloomReferenceHeight,
                      ),
                      ConfigComparison(
                        beforeImage: 'assets/images/config/bloom_2021.jpg',
                        afterImage: 'assets/images/config/bloom_2017.jpg',
                        beforeLabel: l10n.comparisonBloom2021,
                        afterLabel: l10n.comparisonBloom2017,
                      ),
                      _bloomReferenceSlider(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.bloomKernelReferenceHeight,
                      ),
                      _slider(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.bloomExtraBlur,
                        decimals: 1,
                      ),
                      _bloomDropLevelsDropdown(config, notifier, gameDir, l10n),
                      _bloom2017Button(
                        context,
                        config,
                        notifier,
                        gameDir,
                        l10n,
                      ),
                    ]),
                    _card(context, l10n.cardGlobalIllumination, [
                      _bool(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.giEnabled,
                      ),
                      _giWorkgroupDropdown(config, notifier, gameDir, l10n),
                      ConfigComparison(
                        beforeImage: 'assets/images/config/16_cube.jpg',
                        afterImage: 'assets/images/config/128_cube.jpg',
                        beforeLabel: l10n.comparisonCubemaps16,
                        afterLabel: l10n.comparisonCubemaps128,
                      ),
                    ]),
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
                      _slider(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.shadowDistanceMultiplier,
                        decimals: 1,
                      ),
                      ConfigComparison(
                        beforeImage:
                            'assets/images/config/shadow_distance_default.jpg',
                        afterImage:
                            'assets/images/config/shadow_distance_multiplier_2.0.jpg',
                        beforeLabel: l10n.comparisonDefault,
                        afterLabel: l10n.comparison20x,
                      ),
                      _slider(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.shadowDistanceMinimum,
                        decimals: 0,
                      ),
                      _slider(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.shadowDistanceMaximum,
                        decimals: 0,
                      ),
                      _slider(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.shadowDistancePss,
                        decimals: 0,
                      ),
                      ConfigComparison(
                        beforeImage:
                            'assets/images/config/shadow_dist_filter_off.jpg',
                        afterImage:
                            'assets/images/config/pss_distance_shadow_-5.0.jpg',
                        beforeLabel: l10n.comparisonDefault,
                        afterLabel: l10n.comparisonPssMinus5,
                      ),
                      _slider(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.shadowFilterStrengthBias,
                        decimals: 1,
                      ),
                      ConfigComparison(
                        beforeImage:
                            'assets/images/config/shadow_dist_filter_off.jpg',
                        afterImage:
                            'assets/images/config/pss_distance_shadow_-5.0.jpg',
                        beforeLabel: l10n.comparisonDefault,
                        afterLabel: l10n.comparisonBiasMinus5,
                      ),
                      _slider(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.shadowFilterStrengthMinimum,
                        decimals: 1,
                      ),
                      ConfigComparison(
                        beforeImage:
                            'assets/images/config/shadow_filter_min_off.jpg',
                        afterImage:
                            'assets/images/config/shadow_filter_min_3.0.jpg',
                        beforeLabel: l10n.comparisonOff,
                        afterLabel: l10n.comparison30,
                      ),
                      _slider(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.shadowFilterStrengthMaximum,
                        decimals: 1,
                      ),
                      _bool(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.shadowModelHq,
                      ),
                      _bool(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.shadowModelForceAll,
                      ),
                      ConfigComparison(
                        beforeImage:
                            'assets/images/config/hd_shadow_models_off.jpg',
                        afterImage:
                            'assets/images/config/hq_shadow_models_on.jpg',
                        beforeLabel: l10n.comparisonOff,
                        afterLabel: l10n.comparisonHqForceAll,
                      ),
                      ..._blurScaleSliders(config, notifier, gameDir, l10n),
                    ]),
                  ],
                ),
              ),
              _card(context, l10n.cardExperimental, [
                _experimentalWarning(context, l10n),
                _twoColumn(
                  context,
                  Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      _subheading(context, l10n.subheadingMsaaPerformance),
                      _sectionBool(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.msaaPerPixel,
                      ),
                      ConfigComparison(
                        beforeImage:
                            'assets/images/config/msaa8_both_fixes.jpg',
                        afterImage: 'assets/images/config/msaa8_per_pixel.jpg',
                        beforeLabel: l10n.comparisonOff,
                        afterLabel: l10n.comparisonOn,
                      ),
                      if (LodModFields.msaaPerPixel.valueIn(
                        config.lodmodValues,
                      ))
                        _sectionBool(
                          config,
                          notifier,
                          gameDir,
                          l10n,
                          LodModFields.msaaPerPixelTerrain,
                        ),
                      _subheading(context, l10n.subheadingHighGrids),
                      _highGridsWipNote(context, l10n),
                      _sectionBool(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.highGridsEnabled,
                      ),
                      if (LodModFields.highGridsEnabled.valueIn(
                        config.lodmodValues,
                      )) ...[
                        _ringsDropdown(config, notifier, gameDir, l10n),
                        ConfigComparison(
                          beforeImage:
                              'assets/images/config/Vanilla_High_grids_7.jpg',
                          afterImage:
                              'assets/images/config/YP_High_grids_61.jpg',
                          beforeLabel: l10n.comparisonHighGrids7,
                          afterLabel: l10n.comparisonHighGrids61,
                        ),
                        _gridRules(
                          config,
                          notifier,
                          gameDir,
                          l10n,
                          LodModFields.highGridsRoomRings,
                          [
                            GridRuleColumn(
                              key: 'room',
                              label: l10n.gridRuleRoom,
                              hex: true,
                              fallback: 0x100,
                              options: LodModFields.highGridsRooms.keys
                                  .toList(),
                            ),
                            GridRuleColumn(
                              key: 'rings',
                              label: l10n.gridRuleRings,
                              fallback: 1,
                              options: const [1, 2, 3, 4, 5],
                            ),
                          ],
                        ),
                        _gridRules(
                          config,
                          notifier,
                          gameDir,
                          l10n,
                          LodModFields.highGridsBlockedInRoom,
                          [
                            GridRuleColumn(
                              key: 'room',
                              label: l10n.gridRuleRoom,
                              hex: true,
                              fallback: 0x100,
                              options: LodModFields.highGridsRooms.keys
                                  .toList(),
                            ),
                            GridRuleColumn(
                              key: 'grid',
                              label: l10n.gridRuleGrid,
                              hex: true,
                              fallback: 0x1121,
                            ),
                          ],
                        ),
                        _gridRules(
                          config,
                          notifier,
                          gameDir,
                          l10n,
                          LodModFields.highGridsBlockedFromGrid,
                          [
                            GridRuleColumn(
                              key: 'from',
                              label: l10n.gridRuleFromGrid,
                              hex: true,
                              fallback: 0x1517,
                            ),
                            GridRuleColumn(
                              key: 'grid',
                              label: l10n.gridRuleGrid,
                              hex: true,
                              fallback: 0x1121,
                            ),
                          ],
                        ),
                        _gridRules(
                          config,
                          notifier,
                          gameDir,
                          l10n,
                          LodModFields.highGridsHiddenMeshes,
                          [
                            GridRuleColumn(
                              key: 'mesh',
                              label: l10n.gridRuleMesh,
                              text: true,
                              fallback: '',
                            ),
                            GridRuleColumn(
                              key: 'from_rings',
                              label: l10n.gridRuleFromRings,
                              fallback: 2,
                              options: const [2, 3, 4, 5],
                            ),
                            GridRuleColumn(
                              key: 'grid',
                              label: l10n.gridRuleGrid,
                              hex: true,
                              fallback: 0,
                            ),
                          ],
                        ),
                      ],
                    ],
                  ),
                  Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      _subheading(context, l10n.subheadingFrameRate),
                      _bool(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.fpsUncapInMenus,
                      ),
                      _bool(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.fpsUncapInGameplay,
                      ),
                      _bool(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.fpsCapInHacking,
                      ),
                      _bool(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.fpsCapInEvents,
                      ),
                      _int(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.fpsLimit,
                      ),
                      _subheading(context, l10n.subheadingRendering),
                      _slider(
                        config,
                        notifier,
                        gameDir,
                        l10n,
                        LodModFields.renderScale,
                        decimals: 2,
                      ),
                      _cascadesDropdown(config, notifier, gameDir, l10n),
                      ConfigComparison(
                        beforeImage: 'assets/images/config/4_cascades_8k.jpg',
                        afterImage: 'assets/images/config/8_cascades_8k.jpg',
                        beforeLabel: l10n.comparisonCascades4,
                        afterLabel: l10n.comparisonCascades8,
                      ),
                      if (LodModFields.shadowCascades.valueIn(
                            config.lodmodValues,
                          ) ==
                          8)
                        _slider(
                          config,
                          notifier,
                          gameDir,
                          l10n,
                          LodModFields.shadowCascadeRange,
                          decimals: 2,
                        ),
                    ],
                  ),
                ),
              ]),
            ],
          ],
        ),
      ),
    );
  }

  Widget _card(BuildContext context, String title, List<Widget> children) {
    return CollapsibleCard(title: title, children: children);
  }

  Widget _twoColumn(BuildContext context, Widget left, Widget right) =>
      TwoColumnLayout(left: left, right: right);

  Widget _subheading(BuildContext context, String title) {
    return Padding(
      padding: EdgeInsets.only(bottom: AppSizes.spacingSM(context)),
      child: Text(
        title,
        style: TextStyle(
          fontSize: AppSizes.fontSM(context),
          fontWeight: FontWeight.w700,
          color: AppColors.textSecondary,
          letterSpacing: 0.8,
        ),
      ),
    );
  }
}
