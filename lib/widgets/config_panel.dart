import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/config_state.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/config_field_bool.dart';
import 'package:yp_launcher/widgets/config_field_dropdown.dart';
import 'package:yp_launcher/widgets/config_field_float.dart';
import 'package:yp_launcher/widgets/config_section.dart';
import 'package:yp_launcher/models/config_fields.dart';

class ConfigPanel extends ConsumerStatefulWidget {
  final VoidCallback onClose;

  const ConfigPanel({super.key, required this.onClose});

  @override
  ConfigPanelState createState() => ConfigPanelState();
}

class ConfigPanelState extends ConsumerState<ConfigPanel>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<Offset> _slideAnimation;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 250),
    );
    _slideAnimation = Tween<Offset>(
      begin: const Offset(1, 0),
      end: Offset.zero,
    ).animate(CurvedAnimation(parent: _controller, curve: Curves.easeInOut));
    _controller.forward();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  Future<void> animateClose() async {
    await _controller.reverse();
    widget.onClose();
  }

  @override
  Widget build(BuildContext context) {
    final config = ref.watch(configStateControllerProvider);
    final configNotifier = ref.read(configStateControllerProvider.notifier);
    final gameDir = ref.watch(appStateControllerProvider).selectedDirectory;

    return SlideTransition(
      position: _slideAnimation,
      child: Container(
        width: AppSizes.titleBarButtonWidth,
        decoration: const BoxDecoration(
          color: AppColors.backgroundCard,
          border: Border(
            left: BorderSide(color: AppColors.borderMedium, width: 1.5),
          ),
          boxShadow: [
            BoxShadow(
              color: AppColors.shadow,
              blurRadius: 8,
              offset: Offset(-2, 0),
            ),
          ],
        ),
        child: Column(
          children: [
            _buildHeader(context, config, configNotifier, gameDir),
            Expanded(
              child: config.isLoading
                  ? const Center(
                      child: CircularProgressIndicator(
                        color: AppColors.accentPrimary,
                      ),
                    )
                  : SingleChildScrollView(
                      padding: const EdgeInsets.all(16),
                      child: _buildForm(context, config, configNotifier),
                    ),
            ),
          ],
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
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: const BoxDecoration(
        color: AppColors.surfaceMedium,
        border: Border(bottom: BorderSide(color: AppColors.borderLight)),
      ),
      child: Row(
        children: [
          Text(
            AppLocalizations.of(context)!.configEditorTitle,
            style: TextStyle(
              fontSize: AppSizes.fontXL(context),
              fontWeight: FontWeight.bold,
              color: AppColors.textPrimary,
              letterSpacing: 1.0,
            ),
          ),
          if (config.hasUnsavedChanges)
            Padding(
              padding: const EdgeInsets.only(left: 6),
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
          if (config.hasUnsavedChanges) ...[
            _buildHeaderButton(
              context,
              AppLocalizations.of(context)!.buttonSave,
              AppColors.success,
              () => notifier.saveConfigs(gameDir),
            ),
            const SizedBox(width: 4),
            _buildHeaderButton(
              context,
              AppLocalizations.of(context)!.buttonDiscard,
              AppColors.textMuted,
              () => notifier.discardChanges(gameDir),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildHeaderButton(
    BuildContext context,
    String label,
    Color color,
    VoidCallback onPressed,
  ) {
    return InkWell(
      onTap: onPressed,
      borderRadius: BorderRadius.circular(4),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        decoration: BoxDecoration(
          border: Border.all(color: color),
          borderRadius: BorderRadius.circular(4),
        ),
        child: Text(
          label,
          style: TextStyle(
            fontSize: AppSizes.fontSM(context),
            color: color,
            fontWeight: FontWeight.bold,
          ),
        ),
      ),
    );
  }

  Widget _buildForm(
    BuildContext context,
    ConfigData config,
    ConfigStateController notifier,
  ) {
    final l10n = AppLocalizations.of(context)!;
    final nams = config.namsValues;
    final lod = config.lodmodValues;
    final tex = config.textureInjectionValues;

    final lodEnabled = lod[LodModFields.enabled.key] == true;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        ConfigSection(
          title: l10n.sectionNams,
          children: [
            ConfigFieldBool(
              label: NamsFields.validateModelData.label(l10n),
              value: nams[NamsFields.validateModelData.key] == true,
              onChanged: (v) =>
                  notifier.updateNams(NamsFields.validateModelData.key, v),
              tooltip: l10n.tooltipValidateModelData,
            ),
          ],
        ),

        ConfigSection(
          title: l10n.sectionTextureInjection,
          children: [
            ConfigFieldDropdown(
              label: l10n.labelVramBudget,
              value:
                  (tex[TextureInjectionFields.vramBudgetMb.key]
                      as int?) ??
                  TextureInjectionFields.vramBudgetMb.defaultValue,
              options: const [0, 1024, 2048, 4096, 8192],
              labels: const {0: 'Auto'},
              onChanged: (v) => notifier.updateTextureInjection(
                TextureInjectionFields.vramBudgetMb.key,
                v,
              ),
              tooltip: l10n.tooltipVramBudget,
            ),
            ConfigFieldBool(
              label: l10n.labelStreamingEnabled,
              value: tex[TextureInjectionFields.streamingEnabled.key] != false,
              onChanged: (v) => notifier.updateTextureInjection(
                TextureInjectionFields.streamingEnabled.key,
                v,
              ),
              tooltip: l10n.tooltipStreamingEnabled,
            ),
            ConfigFieldBool(
              label: l10n.labelLoadOnlyRelevant,
              value: tex[TextureInjectionFields.loadOnlyRelevant.key] == true,
              onChanged: (v) => notifier.updateTextureInjection(
                TextureInjectionFields.loadOnlyRelevant.key,
                v,
              ),
              tooltip: l10n.tooltipLoadOnlyRelevant,
            ),
          ],
        ),

        ConfigSection(
          title: l10n.sectionLodMod,
          children: [
            ConfigFieldBool(
              label: LodModFields.enabled.label(l10n),
              value: lodEnabled,
              onChanged: (v) =>
                  notifier.updateLodmod(LodModFields.enabled.key, v),
              tooltip: LodModFields.enabled.tooltip!(l10n),
            ),
          ],
        ),

        if (lodEnabled) ...[
          ConfigSection(
            title: l10n.sectionLevelOfDetail,
            children: [
              ConfigFieldFloat(
                label: LodModFields.lodMultiplier.label(l10n),
                value:
                    (lod[LodModFields.lodMultiplier.key] as num?)?.toDouble() ??
                    LodModFields.lodMultiplier.defaultValue,
                onChanged: (v) =>
                    notifier.updateLodmod(LodModFields.lodMultiplier.key, v),
                tooltip: LodModFields.lodMultiplier.tooltip!(l10n),
              ),
              ConfigFieldBool(
                label: LodModFields.disableManualCulling.label(l10n),
                value: lod[LodModFields.disableManualCulling.key] == true,
                onChanged: (v) => notifier.updateLodmod(
                  LodModFields.disableManualCulling.key,
                  v,
                ),
                tooltip: LodModFields.disableManualCulling.tooltip!(l10n),
              ),
            ],
          ),

          ConfigSection(
            title: l10n.sectionAmbientOcclusion,
            children: [
              ConfigFieldFloat(
                label: LodModFields.aoMultiplierWidth.label(l10n),
                value:
                    (lod[LodModFields.aoMultiplierWidth.key] as num?)
                        ?.toDouble() ??
                    LodModFields.aoMultiplierWidth.defaultValue,
                onChanged: (v) => notifier.updateLodmod(
                  LodModFields.aoMultiplierWidth.key,
                  v,
                ),
                tooltip: LodModFields.aoMultiplierWidth.tooltip!(l10n),
              ),
              ConfigFieldFloat(
                label: LodModFields.aoMultiplierHeight.label(l10n),
                value:
                    (lod[LodModFields.aoMultiplierHeight.key] as num?)
                        ?.toDouble() ??
                    LodModFields.aoMultiplierHeight.defaultValue,
                onChanged: (v) => notifier.updateLodmod(
                  LodModFields.aoMultiplierHeight.key,
                  v,
                ),
                tooltip: LodModFields.aoMultiplierHeight.tooltip!(l10n),
              ),
            ],
          ),

          ConfigSection(
            title: l10n.sectionShadows,
            children: [
              ConfigFieldDropdown(
                label: LodModFields.shadowResolution.label(l10n),
                value:
                    (lod[LodModFields.shadowResolution.key] as int?) ??
                    LodModFields.shadowResolution.defaultValue,
                options: const [2048, 4096, 8192],
                onChanged: (v) =>
                    notifier.updateLodmod(LodModFields.shadowResolution.key, v),
                tooltip: LodModFields.shadowResolution.tooltip!(l10n),
              ),
              ConfigFieldFloat(
                label: LodModFields.shadowDistanceMultiplier.label(l10n),
                value:
                    (lod[LodModFields.shadowDistanceMultiplier.key] as num?)
                        ?.toDouble() ??
                    LodModFields.shadowDistanceMultiplier.defaultValue,
                onChanged: (v) => notifier.updateLodmod(
                  LodModFields.shadowDistanceMultiplier.key,
                  v,
                ),
                tooltip: LodModFields.shadowDistanceMultiplier.tooltip!(l10n),
              ),
              ConfigFieldFloat(
                label: LodModFields.shadowDistanceMinimum.label(l10n),
                value:
                    (lod[LodModFields.shadowDistanceMinimum.key] as num?)
                        ?.toDouble() ??
                    LodModFields.shadowDistanceMinimum.defaultValue,
                onChanged: (v) => notifier.updateLodmod(
                  LodModFields.shadowDistanceMinimum.key,
                  v,
                ),
                tooltip: LodModFields.shadowDistanceMinimum.tooltip!(l10n),
              ),
              ConfigFieldFloat(
                label: LodModFields.shadowDistanceMaximum.label(l10n),
                value:
                    (lod[LodModFields.shadowDistanceMaximum.key] as num?)
                        ?.toDouble() ??
                    LodModFields.shadowDistanceMaximum.defaultValue,
                onChanged: (v) => notifier.updateLodmod(
                  LodModFields.shadowDistanceMaximum.key,
                  v,
                ),
                tooltip: LodModFields.shadowDistanceMaximum.tooltip!(l10n),
              ),
              ConfigFieldFloat(
                label: LodModFields.shadowDistancePss.label(l10n),
                value:
                    (lod[LodModFields.shadowDistancePss.key] as num?)
                        ?.toDouble() ??
                    LodModFields.shadowDistancePss.defaultValue,
                onChanged: (v) => notifier.updateLodmod(
                  LodModFields.shadowDistancePss.key,
                  v,
                ),
                tooltip: LodModFields.shadowDistancePss.tooltip!(l10n),
              ),
              ConfigFieldFloat(
                label: LodModFields.shadowFilterStrengthBias.label(l10n),
                value:
                    (lod[LodModFields.shadowFilterStrengthBias.key] as num?)
                        ?.toDouble() ??
                    LodModFields.shadowFilterStrengthBias.defaultValue,
                onChanged: (v) => notifier.updateLodmod(
                  LodModFields.shadowFilterStrengthBias.key,
                  v,
                ),
                tooltip: LodModFields.shadowFilterStrengthBias.tooltip!(l10n),
              ),
              ConfigFieldFloat(
                label: LodModFields.shadowFilterStrengthMinimum.label(l10n),
                value:
                    (lod[LodModFields.shadowFilterStrengthMinimum.key] as num?)
                        ?.toDouble() ??
                    LodModFields.shadowFilterStrengthMinimum.defaultValue,
                onChanged: (v) => notifier.updateLodmod(
                  LodModFields.shadowFilterStrengthMinimum.key,
                  v,
                ),
                tooltip: LodModFields.shadowFilterStrengthMinimum.tooltip!(
                  l10n,
                ),
              ),
              ConfigFieldFloat(
                label: LodModFields.shadowFilterStrengthMaximum.label(l10n),
                value:
                    (lod[LodModFields.shadowFilterStrengthMaximum.key] as num?)
                        ?.toDouble() ??
                    LodModFields.shadowFilterStrengthMaximum.defaultValue,
                onChanged: (v) => notifier.updateLodmod(
                  LodModFields.shadowFilterStrengthMaximum.key,
                  v,
                ),
                tooltip: LodModFields.shadowFilterStrengthMaximum.tooltip!(
                  l10n,
                ),
              ),
              ConfigFieldBool(
                label: LodModFields.shadowModelHq.label(l10n),
                value: lod[LodModFields.shadowModelHq.key] == true,
                onChanged: (v) =>
                    notifier.updateLodmod(LodModFields.shadowModelHq.key, v),
                tooltip: LodModFields.shadowModelHq.tooltip!(l10n),
              ),
              ConfigFieldBool(
                label: LodModFields.shadowModelForceAll.label(l10n),
                value: lod[LodModFields.shadowModelForceAll.key] == true,
                onChanged: (v) => notifier.updateLodmod(
                  LodModFields.shadowModelForceAll.key,
                  v,
                ),
                tooltip: LodModFields.shadowModelForceAll.tooltip!(l10n),
              ),
            ],
          ),

          ConfigSection(
            title: l10n.sectionPostProcessing,
            children: [
              ConfigFieldBool(
                label: LodModFields.disableVignette.label(l10n),
                value: lod[LodModFields.disableVignette.key] == true,
                onChanged: (v) =>
                    notifier.updateLodmod(LodModFields.disableVignette.key, v),
                tooltip: LodModFields.disableVignette.tooltip!(l10n),
              ),
            ],
          ),
        ],
      ],
    );
  }
}
