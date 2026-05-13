import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/config_state.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/widgets/app_dropdown.dart';
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

    final heap = (nams['heap'] as Map<String, dynamic>?) ?? {};

    return Scrollbar(
      controller: _scrollController,
      thumbVisibility: true,
      child: SingleChildScrollView(
        controller: _scrollController,
        padding: EdgeInsets.all(AppSizes.contentPadding(context)),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Expanded(
                  child: Column(
                    children: [
                      _card(context, l10n.cardGeneral, [
                        ConfigFieldBool(
                          label: NamsFields.validateModelData.label(l10n),
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
                          value: nams[NamsFields.validateScripts.key] == true,
                          onChanged: (v) => notifier.updateNams(
                            NamsFields.validateScripts.key,
                            v,
                          ),
                          tooltip: NamsFields.validateScripts.tooltip!(l10n),
                        ),
                        ConfigPreviewImage(
                          image:
                              'assets/images/config/script_error_validation.jpg',
                          label: l10n.previewScriptErrorDialog,
                        ),
                        ConfigFieldBool(
                          label: NamsFields.loadingStallHints.label(l10n),
                          value:
                              nams[NamsFields.loadingStallHints.key] != false,
                          onChanged: (v) => notifier.updateNams(
                            NamsFields.loadingStallHints.key,
                            v,
                          ),
                          tooltip: NamsFields.loadingStallHints.tooltip!(l10n),
                        ),
                        ConfigFieldBool(
                          label: NamsFields.fixWindTimerBug.label(l10n),
                          value: nams[NamsFields.fixWindTimerBug.key] != false,
                          onChanged: (v) => notifier.updateNams(
                            NamsFields.fixWindTimerBug.key,
                            v,
                          ),
                          tooltip: NamsFields.fixWindTimerBug.tooltip!(l10n),
                        ),
                      ]),
                      _card(context, l10n.cardContentFeatures, [
                        Padding(
                          padding: EdgeInsets.only(
                            bottom: AppSizes.spacingMD(context),
                          ),
                          child: Text(
                            l10n.contentFeaturesDescription,
                            style: TextStyle(
                              fontSize: AppSizes.fontXS(context),
                              color: AppColors.textMuted,
                              fontStyle: FontStyle.italic,
                            ),
                          ),
                        ),
                        ..._contentToggles(context, nams, notifier, l10n),
                      ]),
                    ],
                  ),
                ),
                SizedBox(width: AppSizes.spacingLG(context)),
                Expanded(
                  child: Column(
                    children: [
                      _card(context, l10n.cardLoading, [
                        ConfigFieldBool(
                          label: NamsFields.disablePluginLoading.label(l10n),
                          value:
                              nams[NamsFields.disablePluginLoading.key] == true,
                          onChanged: (v) => notifier.updateNams(
                            NamsFields.disablePluginLoading.key,
                            v,
                          ),
                          tooltip: NamsFields.disablePluginLoading.tooltip!(
                            l10n,
                          ),
                        ),
                        ConfigFieldBool(
                          label: NamsFields.disableContentFeatures.label(l10n),
                          value:
                              nams[NamsFields.disableContentFeatures.key] ==
                              true,
                          onChanged: (v) => notifier.updateNams(
                            NamsFields.disableContentFeatures.key,
                            v,
                          ),
                          tooltip: NamsFields.disableContentFeatures.tooltip!(
                            l10n,
                          ),
                        ),
                        ConfigFieldBool(
                          label: NamsFields.disableReShadeLoading.label(l10n),
                          value:
                              nams[NamsFields.disableReShadeLoading.key] ==
                              true,
                          onChanged: (v) => notifier.updateNams(
                            NamsFields.disableReShadeLoading.key,
                            v,
                          ),
                          tooltip: NamsFields.disableReShadeLoading.tooltip!(
                            l10n,
                          ),
                        ),
                        ConfigFieldBool(
                          label: NamsFields.disableTextureInjection.label(l10n),
                          value:
                              nams[NamsFields.disableTextureInjection.key] ==
                              true,
                          onChanged: (v) => notifier.updateNams(
                            NamsFields.disableTextureInjection.key,
                            v,
                          ),
                          tooltip: NamsFields.disableTextureInjection.tooltip!(
                            l10n,
                          ),
                        ),
                      ]),
                      _card(context, l10n.cardHeapOverrides, [
                        Padding(
                          padding: EdgeInsets.only(
                            bottom: AppSizes.spacingMD(context),
                          ),
                          child: Text(
                            l10n.heapOverridesDescription,
                            style: TextStyle(
                              fontSize: AppSizes.fontXS(context),
                              color: AppColors.textMuted,
                              fontStyle: FontStyle.italic,
                            ),
                          ),
                        ),
                        _heapField(
                          context,
                          NamsFields.globalHeapExtra.label(l10n),
                          NamsFields.globalHeapExtra.key,
                          heap,
                          notifier,
                          l10n.heapScriptEngineDesc,
                        ),
                        _heapField(
                          context,
                          NamsFields.plFileHeapExtra.label(l10n),
                          NamsFields.plFileHeapExtra.key,
                          heap,
                          notifier,
                          l10n.heapPlayerModelsDesc,
                        ),
                        _heapField(
                          context,
                          NamsFields.plVramHeapExtra.label(l10n),
                          NamsFields.plVramHeapExtra.key,
                          heap,
                          notifier,
                          l10n.heapPlayerTexturesDesc,
                        ),
                        _heapField(
                          context,
                          NamsFields.emBgFileHeapExtra.label(l10n),
                          NamsFields.emBgFileHeapExtra.key,
                          heap,
                          notifier,
                          l10n.heapEnemyBgModelsDesc,
                        ),
                        _heapField(
                          context,
                          NamsFields.emBgVramHeapExtra.label(l10n),
                          NamsFields.emBgVramHeapExtra.key,
                          heap,
                          notifier,
                          l10n.heapEnemyBgTexturesDesc,
                        ),
                      ]),
                    ],
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  static const _heapPresetBytes = <int>[
    0,
    16 * 1024 * 1024,
    32 * 1024 * 1024,
    64 * 1024 * 1024,
    128 * 1024 * 1024,
    256 * 1024 * 1024,
  ];

  Map<int, String> _heapPresets(AppLocalizations l10n) => {
    0: l10n.heapDefault,
    16 * 1024 * 1024: l10n.heapPlus16MB,
    32 * 1024 * 1024: l10n.heapPlus32MB,
    64 * 1024 * 1024: l10n.heapPlus64MB,
    128 * 1024 * 1024: l10n.heapPlus128MB,
    256 * 1024 * 1024: l10n.heapPlus256MB,
  };

  String _heapLabel(AppLocalizations l10n, int bytes) {
    if (bytes == 0) return l10n.heapDefault;
    final presets = _heapPresets(l10n);
    if (presets.containsKey(bytes)) return presets[bytes]!;
    final mb = bytes / (1024 * 1024);
    return l10n.heapCustomMB(
      mb.toStringAsFixed(mb == mb.roundToDouble() ? 0 : 1),
    );
  }

  List<Widget> _contentToggles(
    BuildContext context,
    Map<String, dynamic> nams,
    ConfigStateController notifier,
    AppLocalizations l10n,
  ) {
    final fields = <ConfigField<bool>>[
      NamsFields.contentItems,
      NamsFields.contentAccessories,
      NamsFields.contentAssembleMeshes,
      NamsFields.contentQuestIntegration,
      NamsFields.contentEffectsApplier,
      NamsFields.contentEquipTracker,
      NamsFields.contentMcd,
    ];
    return fields.map((f) {
      return ConfigFieldBool(
        label: f.label(l10n),
        value: nams[f.key] != false,
        onChanged: (v) => notifier.updateNams(f.key, v),
        tooltip: f.tooltip!(l10n),
      );
    }).toList();
  }

  Widget _heapField(
    BuildContext context,
    String label,
    String key,
    Map<String, dynamic> heap,
    ConfigStateController notifier,
    String description,
  ) {
    final l10n = AppLocalizations.of(context)!;
    final value = (heap[key] as int?) ?? 0;
    final isCustom = value != 0 && !_heapPresetBytes.contains(value);
    final items = [
      ..._heapPresetBytes,
      if (isCustom) value,
    ];

    return Padding(
      padding: EdgeInsets.symmetric(vertical: AppSizes.paddingXS(context)),
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  label,
                  style: TextStyle(
                    fontSize: AppSizes.fontSM(context),
                    color: AppColors.textPrimary,
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
          SizedBox(width: AppSizes.spacingMD(context)),
          AppDropdown<int>(
            value: value,
            items: items,
            highlight: value != 0,
            itemLabel: (v) => _heapLabel(l10n, v),
            onChanged: (v) =>
                notifier.updateNams(key, v, section: 'heap'),
          ),
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
