import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/services/cheat_table_service.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/config_state.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/widgets/collapsible_card.dart';
import 'package:yp_launcher/widgets/config_change_status.dart';
import 'package:yp_launcher/widgets/config_error_banner.dart';
import 'package:yp_launcher/widgets/two_column_layout.dart';
import 'package:yp_launcher/widgets/config_field_bool.dart';
import 'package:yp_launcher/widgets/header_info_icon.dart';
import 'package:yp_launcher/widgets/hover_button.dart';
import 'package:yp_launcher/widgets/config_field_preview.dart'
    show ConfigPreviewImage;
import 'package:yp_launcher/models/config_fields.dart';

class SettingsView extends ConsumerStatefulWidget {
  const SettingsView({super.key});

  @override
  ConsumerState<SettingsView> createState() => _SettingsViewState();
}

class _SettingsViewState extends ConsumerState<SettingsView> {
  final _scrollController = ScrollController();
  bool _loaded = false;
  String? _ctResult;
  bool _ctSuccess = false;

  Future<void> _convertCheatTable() async {
    final l10n = AppLocalizations.of(context)!;
    final result = await FilePicker.pickFiles(
      type: FileType.custom,
      allowedExtensions: const ['ct', 'CT'],
    );
    final picked = result?.files.single.path;
    if (picked == null || !mounted) return;

    final converted = await CheatTableService.convert(picked);
    if (!mounted) return;
    setState(() {
      if (converted.success) {
        _ctSuccess = true;
        _ctResult = l10n.cheatTableConvertSuccess(
          p.basename(converted.outputPath!),
        );
      } else if (converted.error == null) {
        _ctSuccess = false;
        _ctResult = l10n.cheatTableConvertNone;
      } else {
        _ctSuccess = false;
        _ctResult = l10n.cheatTableConvertError;
      }
    });
  }

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
            l10n.headerNams,
            style: TextStyle(
              fontSize: AppSizes.fontXL(context),
              fontWeight: FontWeight.bold,
              color: AppColors.textPrimary,
              letterSpacing: 1.0,
            ),
          ),
          HeaderInfoIcon(
            tooltip: l10n.tooltipEditsNamsToml,
            revealPath: p.join(gameDir, 'nams', 'nams.toml'),
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

  Widget _buildContent(
    BuildContext context,
    ConfigData config,
    ConfigStateController notifier,
  ) {
    final l10n = AppLocalizations.of(context)!;
    final nams = config.namsValues;

    return Scrollbar(
      controller: _scrollController,
      thumbVisibility: true,
      child: SingleChildScrollView(
        controller: _scrollController,
        padding: EdgeInsets.all(AppSizes.contentPadding(context)),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (config.namsError != null)
              ConfigErrorBanner(
                fileName: 'nams.toml',
                error: config.namsError!,
              ),
            _descriptionBanner(context, l10n.namsDescription),
            TwoColumnLayout(
              left: Column(
                children: [
                  _card(context, l10n.cardGeneral, [
                    ConfigFieldBool(
                      label: NamsFields.validateModelData.label(l10n),
                      defaultValue: NamsFields.validateModelData.defaultValue,
                      value: nams[NamsFields.validateModelData.key] == true,
                      onChanged: (v) => notifier.updateNams(
                        NamsFields.validateModelData.key,
                        v,
                      ),
                      tooltip: NamsFields.validateModelData.tooltip!(l10n),
                    ),
                    ConfigPreviewImage(
                      image: 'assets/images/config/validate_model_data.jpg',
                      label: l10n.previewValidationDialog,
                    ),
                    ConfigFieldBool(
                      label: NamsFields.validateScripts.label(l10n),
                      defaultValue: NamsFields.validateScripts.defaultValue,
                      value: nams[NamsFields.validateScripts.key] == true,
                      onChanged: (v) => notifier.updateNams(
                        NamsFields.validateScripts.key,
                        v,
                      ),
                      tooltip: NamsFields.validateScripts.tooltip!(l10n),
                    ),
                    ConfigPreviewImage(
                      image: 'assets/images/config/script_error_validation.jpg',
                      label: l10n.previewScriptErrorDialog,
                    ),
                    ConfigFieldBool(
                      label: NamsFields.loadingStallHints.label(l10n),
                      defaultValue: NamsFields.loadingStallHints.defaultValue,
                      value: nams[NamsFields.loadingStallHints.key] != false,
                      onChanged: (v) => notifier.updateNams(
                        NamsFields.loadingStallHints.key,
                        v,
                      ),
                      tooltip: NamsFields.loadingStallHints.tooltip!(l10n),
                    ),
                    ConfigFieldBool(
                      label: NamsFields.fixWindTimerBug.label(l10n),
                      defaultValue: NamsFields.fixWindTimerBug.defaultValue,
                      value: nams[NamsFields.fixWindTimerBug.key] != false,
                      onChanged: (v) => notifier.updateNams(
                        NamsFields.fixWindTimerBug.key,
                        v,
                      ),
                      tooltip: NamsFields.fixWindTimerBug.tooltip!(l10n),
                    ),
                    ConfigFieldBool(
                      label: NamsFields.disableDebugHotkeys.label(l10n),
                      defaultValue: NamsFields.disableDebugHotkeys.defaultValue,
                      value: nams[NamsFields.disableDebugHotkeys.key] != false,
                      onChanged: (v) => notifier.updateNams(
                        NamsFields.disableDebugHotkeys.key,
                        v,
                      ),
                      tooltip: NamsFields.disableDebugHotkeys.tooltip!(l10n),
                    ),
                    Padding(
                      padding: EdgeInsets.symmetric(
                        vertical: AppSizes.paddingXS(context),
                      ),
                      child: Divider(height: 1, color: AppColors.borderLight),
                    ),
                    Text(
                      l10n.cheatTableConvertDesc,
                      style: TextStyle(
                        fontSize: AppSizes.fontXS(context),
                        color: AppColors.textMuted,
                        height: 1.4,
                      ),
                    ),
                    SizedBox(height: AppSizes.spacingMD(context)),
                    Row(
                      children: [
                        HoverButton(
                          label: l10n.cheatTableConvertButton,
                          color: AppColors.accentPrimary,
                          onTap: _convertCheatTable,
                        ),
                      ],
                    ),
                    if (_ctResult != null)
                      Padding(
                        padding: EdgeInsets.only(
                          top: AppSizes.paddingSM(context),
                        ),
                        child: Row(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Icon(
                              _ctSuccess
                                  ? Icons.check_circle_outline
                                  : Icons.info_outline,
                              size: 14,
                              color: _ctSuccess
                                  ? AppColors.success
                                  : AppColors.warning,
                            ),
                            const SizedBox(width: 6),
                            Expanded(
                              child: Text(
                                _ctResult!,
                                style: TextStyle(
                                  fontSize: AppSizes.fontXS(context),
                                  color: _ctSuccess
                                      ? AppColors.success
                                      : AppColors.warning,
                                  height: 1.35,
                                ),
                              ),
                            ),
                          ],
                        ),
                      ),
                  ]),
                ],
              ),
              right: Column(
                children: [
                  _card(context, l10n.cardLoading, [
                    ConfigFieldBool(
                      label: NamsFields.disablePluginLoading.label(l10n),
                      defaultValue:
                          NamsFields.disablePluginLoading.defaultValue,
                      value: nams[NamsFields.disablePluginLoading.key] == true,
                      onChanged: (v) => notifier.updateNams(
                        NamsFields.disablePluginLoading.key,
                        v,
                      ),
                      tooltip: NamsFields.disablePluginLoading.tooltip!(l10n),
                    ),
                    ConfigFieldBool(
                      label: NamsFields.disableMainOverlay.label(l10n),
                      defaultValue: NamsFields.disableMainOverlay.defaultValue,
                      value: nams[NamsFields.disableMainOverlay.key] == true,
                      onChanged: (v) => notifier.updateNams(
                        NamsFields.disableMainOverlay.key,
                        v,
                      ),
                      tooltip: NamsFields.disableMainOverlay.tooltip!(l10n),
                    ),
                    ConfigFieldBool(
                      label: NamsFields.disableContentFeatures.label(l10n),
                      defaultValue:
                          NamsFields.disableContentFeatures.defaultValue,
                      value:
                          nams[NamsFields.disableContentFeatures.key] == true,
                      onChanged: (v) => notifier.updateNams(
                        NamsFields.disableContentFeatures.key,
                        v,
                      ),
                      tooltip: NamsFields.disableContentFeatures.tooltip!(l10n),
                    ),
                    ConfigFieldBool(
                      label: NamsFields.disableReShadeLoading.label(l10n),
                      defaultValue:
                          NamsFields.disableReShadeLoading.defaultValue,
                      value: nams[NamsFields.disableReShadeLoading.key] == true,
                      onChanged: (v) => notifier.updateNams(
                        NamsFields.disableReShadeLoading.key,
                        v,
                      ),
                      tooltip: NamsFields.disableReShadeLoading.tooltip!(l10n),
                    ),
                    ConfigFieldBool(
                      label: NamsFields.disable3dmigotoLoading.label(l10n),
                      defaultValue:
                          NamsFields.disable3dmigotoLoading.defaultValue,
                      value:
                          nams[NamsFields.disable3dmigotoLoading.key] == true,
                      onChanged: (v) => notifier.updateNams(
                        NamsFields.disable3dmigotoLoading.key,
                        v,
                      ),
                      tooltip: NamsFields.disable3dmigotoLoading.tooltip!(l10n),
                    ),
                    ConfigFieldBool(
                      label: NamsFields.disableGraphicModsLoading.label(l10n),
                      defaultValue:
                          NamsFields.disableGraphicModsLoading.defaultValue,
                      value: NamsFields.disableGraphicModsLoading.valueIn(nams),
                      onChanged: (v) => notifier.updateNams(
                        NamsFields.disableGraphicModsLoading.key,
                        v,
                      ),
                      tooltip: NamsFields.disableGraphicModsLoading.tooltip!(
                        l10n,
                      ),
                    ),
                    ConfigFieldBool(
                      label: NamsFields.disableTextureInjection.label(l10n),
                      defaultValue:
                          NamsFields.disableTextureInjection.defaultValue,
                      value:
                          nams[NamsFields.disableTextureInjection.key] == true,
                      onChanged: (v) => notifier.updateNams(
                        NamsFields.disableTextureInjection.key,
                        v,
                      ),
                      tooltip: NamsFields.disableTextureInjection.tooltip!(
                        l10n,
                      ),
                    ),
                    ConfigFieldBool(
                      label: NamsFields.disableSplashScreen.label(l10n),
                      defaultValue: NamsFields.disableSplashScreen.defaultValue,
                      value: nams[NamsFields.disableSplashScreen.key] == true,
                      onChanged: (v) => notifier.updateNams(
                        NamsFields.disableSplashScreen.key,
                        v,
                      ),
                      tooltip: NamsFields.disableSplashScreen.tooltip!(l10n),
                    ),
                    ConfigFieldBool(
                      label: NamsFields.skipStartupLogos.label(l10n),
                      defaultValue: NamsFields.skipStartupLogos.defaultValue,
                      value: nams[NamsFields.skipStartupLogos.key] == true,
                      onChanged: (v) => notifier.updateNams(
                        NamsFields.skipStartupLogos.key,
                        v,
                      ),
                      tooltip: NamsFields.skipStartupLogos.tooltip!(l10n),
                    ),
                  ]),
                ],
              ),
            ),
            _contentCard(context, nams, notifier, l10n),
          ],
        ),
      ),
    );
  }

  Widget _contentCard(
    BuildContext context,
    Map<String, dynamic> nams,
    ConfigStateController notifier,
    AppLocalizations l10n,
  ) {
    return _card(context, l10n.cardContentFeatures, [
      Padding(
        padding: EdgeInsets.only(bottom: AppSizes.spacingMD(context)),
        child: Text(
          l10n.contentFeaturesDescription,
          style: TextStyle(
            fontSize: AppSizes.fontXS(context),
            color: AppColors.textMuted,
            fontStyle: FontStyle.italic,
          ),
        ),
      ),
      for (final group in NamsContentFields.groups) ...[
        _subheading(context, group.title(l10n)),
        _contentGrid(context, group.fields, nams, notifier, l10n),
      ],
      Padding(
        padding: EdgeInsets.symmetric(vertical: AppSizes.paddingXS(context)),
        child: Divider(height: 1, color: AppColors.borderLight),
      ),
      ConfigFieldBool(
        label: NamsFields.outfitSwapVisualEffects.label(l10n),
        defaultValue: NamsFields.outfitSwapVisualEffects.defaultValue,
        value: nams[NamsFields.outfitSwapVisualEffects.key] != false,
        onChanged: (v) =>
            notifier.updateNams(NamsFields.outfitSwapVisualEffects.key, v),
        tooltip: NamsFields.outfitSwapVisualEffects.tooltip!(l10n),
      ),
      ConfigFieldBool(
        label: NamsFields.experimentalDefaultOutfits.label(l10n),
        defaultValue: NamsFields.experimentalDefaultOutfits.defaultValue,
        value: nams[NamsFields.experimentalDefaultOutfits.key] == true,
        onChanged: (v) =>
            notifier.updateNams(NamsFields.experimentalDefaultOutfits.key, v),
        tooltip: NamsFields.experimentalDefaultOutfits.tooltip!(l10n),
      ),
    ]);
  }

  Widget _contentGrid(
    BuildContext context,
    List<ConfigField<bool>> fields,
    Map<String, dynamic> nams,
    ConfigStateController notifier,
    AppLocalizations l10n,
  ) {
    final gap = AppSizes.spacingLG(context);
    return ConfigChangeGroup(
      changedLabels: [
        for (final f in fields)
          ...changedLabelOf(f.label(l10n), f.valueIn(nams), f.defaultValue),
      ],
      child: Padding(
        padding: EdgeInsets.only(bottom: AppSizes.paddingMD(context)),
        child: LayoutBuilder(
          builder: (context, constraints) {
            final width = constraints.maxWidth;
            final columns = width >= 1100
                ? 4
                : width >= 820
                ? 3
                : width >= 520
                ? 2
                : 1;
            final cellWidth = ((width - gap * (columns - 1)) / columns)
                .floorToDouble();
            return Wrap(
              spacing: gap,
              children: [
                for (final f in fields)
                  SizedBox(
                    width: cellWidth,
                    child: ConfigFieldBool(
                      label: f.label(l10n),
                      defaultValue: f.defaultValue,
                      value: f.valueIn(nams),
                      onChanged: (v) =>
                          notifier.updateNams(f.key, v, section: f.section),
                      tooltip: f.tooltip!(l10n),
                    ),
                  ),
              ],
            );
          },
        ),
      ),
    );
  }

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

  Widget _card(BuildContext context, String title, List<Widget> children) {
    return CollapsibleCard(title: title, children: children);
  }

  Widget _descriptionBanner(BuildContext context, String text) {
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
            Icons.info_outline,
            size: AppSizes.iconLG(context),
            color: AppColors.textSecondary,
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
}
