import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/config_state.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/config_field_bool.dart';
import 'package:yp_launcher/widgets/config_field_dropdown.dart';
import 'package:yp_launcher/widgets/config_field_float.dart';
import 'package:yp_launcher/widgets/config_section.dart';

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
        width: AppSizes.configPanelWidth,
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
            _buildHeader(config, configNotifier, gameDir),
            Expanded(
              child: config.isLoading
                  ? const Center(
                      child: CircularProgressIndicator(
                        color: AppColors.accentPrimary,
                      ),
                    )
                  : SingleChildScrollView(
                      padding: const EdgeInsets.all(16),
                      child: _buildForm(config, configNotifier),
                    ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildHeader(
    ConfigData config,
    ConfigStateController notifier,
    String gameDir,
  ) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: const BoxDecoration(
        color: AppColors.surfaceMedium,
        border: Border(
          bottom: BorderSide(color: AppColors.borderLight),
        ),
      ),
      child: Row(
        children: [
          Text(
            'CONFIG EDITOR',
            style: TextStyle(
              fontSize: AppSizes.fontXL,
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
                  fontSize: AppSizes.fontXL,
                  fontWeight: FontWeight.bold,
                  color: AppColors.warning,
                ),
              ),
            ),
          const Spacer(),
          if (config.hasUnsavedChanges) ...[
            _buildHeaderButton(
              'SAVE',
              AppColors.success,
              () => notifier.saveConfigs(gameDir),
            ),
            const SizedBox(width: 4),
            _buildHeaderButton(
              'DISCARD',
              AppColors.textMuted,
              () => notifier.discardChanges(gameDir),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildHeaderButton(String label, Color color, VoidCallback onPressed) {
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
          style: TextStyle(fontSize: AppSizes.fontSM, color: color, fontWeight: FontWeight.bold),
        ),
      ),
    );
  }

  Widget _buildForm(ConfigData config, ConfigStateController notifier) {
    final lod = config.lodmodValues;
    final tex = config.textureInjectionValues;

    final lodEnabled = lod['enabled'] == true;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Texture Injection
        ConfigSection(title: 'TEXTURE INJECTION', children: [
          ConfigFieldDropdown(
            label: 'Preload Max Dimension',
            value: (tex['preload_max_dimension'] as int?) ?? 2048,
            options: const [2048, 4096, 16384],
            onChanged: (v) => notifier.updateTextureInjection('preload_max_dimension', v),
            tooltip: 'Max texture size to preload into RAM at startup. '
                '2048 = default, 4096 = preload up to 4K textures, '
                '16384 = preload everything. Higher = longer loading but less stutter in-game.',
          ),
          ConfigFieldBool(
            label: 'Preload All Textures',
            value: tex['preload_all'] == true,
            onChanged: (v) => notifier.updateTextureInjection('preload_all', v),
            tooltip: 'Preload ALL textures into RAM regardless of size. '
                'Eliminates all texture pop-in stutter but needs 32GB+ RAM '
                'and makes startup significantly slower.',
          ),
        ]),

        // LodMod toggle
        ConfigSection(title: 'LOD MOD', children: [
          ConfigFieldBool(
            label: 'Enable LodMod',
            value: lodEnabled,
            onChanged: (v) => notifier.updateLodmod('enabled', v),
            tooltip: 'Master toggle for all LodMod visual patches below. '
                'YP only uses the faster loading screen feature from LodMod. '
                'SkipIntroMovies/SkipBootingScreens are not included — they can cause softlocks and are not recommended. '
                'Other original LodMod.ini features not available: '
                'DLL chainloading/Wrapper (YP uses its own injection), '
                'CommunicationScreenResolution, HQMapSlots, Buffers, Movies.',
          ),
        ]),

        // Only show LodMod settings when enabled
        if (lodEnabled) ...[
          // Level of Detail
          ConfigSection(title: 'LEVEL OF DETAIL', children: [
            ConfigFieldFloat(
              label: 'LOD Multiplier',
              value: (lod['lod_multiplier'] as num?)?.toDouble() ?? 0.0,
              onChanged: (v) => notifier.updateLodmod('lod_multiplier', v),
              tooltip: 'Controls LOD (Level of Detail) draw distances. '
                  '0 = LODs disabled (best quality, no pop-in). '
                  '1 = vanilla. 10+ helps reduce AO bleed without fully disabling LODs. '
                  'Lower values = better visuals but may cost performance.',
            ),
            ConfigFieldBool(
              label: 'Disable Manual Culling',
              value: lod['disable_manual_culling'] == true,
              onChanged: (v) => notifier.updateLodmod('disable_manual_culling', v),
              tooltip: 'Prevents models/geometry from randomly disappearing at certain '
                  'distances or camera angles. Fixes things like the mall interior vanishing '
                  'after crossing the bridge, buildings outside camp disappearing, etc. '
                  'Rare ugly LOD models that would show up are filtered out.',
            ),
          ]),

          // Ambient Occlusion
          ConfigSection(title: 'AMBIENT OCCLUSION', children: [
            ConfigFieldFloat(
              label: 'AO Width',
              value: (lod['ao_multiplier_width'] as num?)?.toDouble() ?? 1.0,
              onChanged: (v) => notifier.updateLodmod('ao_multiplier_width', v),
              tooltip: 'Multiplier for AO horizontal resolution. Vanilla AO runs at 1/4 '
                  'screen res. 2.0 = half screen res (crisper AO but heavy). '
                  '1.5 is a good balance. Range: 0.1 - 2.0. '
                  'Setting only one axis to 2 can be a lighter alternative.',
            ),
            ConfigFieldFloat(
              label: 'AO Height',
              value: (lod['ao_multiplier_height'] as num?)?.toDouble() ?? 1.0,
              onChanged: (v) => notifier.updateLodmod('ao_multiplier_height', v),
              tooltip: 'Multiplier for AO vertical resolution. Vanilla AO runs at 1/4 '
                  'screen res. 2.0 = half screen res (crisper AO but heavy). '
                  '1.5 is a good balance. Range: 0.1 - 2.0. '
                  'Both at 2.0 can cost ~10 FPS in worst case.',
            ),
          ]),

          // Shadows
          ConfigSection(title: 'SHADOWS', children: [
            ConfigFieldDropdown(
              label: 'Shadow Resolution',
              value: (lod['shadow_resolution'] as int?) ?? 2048,
              options: const [2048, 4096, 8192],
              onChanged: (v) => notifier.updateLodmod('shadow_resolution', v),
              tooltip: 'Shadow map texture size. Higher = sharper shadows but heavier on GPU. '
                  '2048 = vanilla, 4096 = good upgrade, 8192 = very sharp. '
                  'Must be power of 2. Sharpness depends on both resolution and distance '
                  '(larger distance = more area to fit, so quality decreases).',
            ),
            ConfigFieldFloat(
              label: 'Distance Multiplier',
              value: (lod['shadow_distance_multiplier'] as num?)?.toDouble() ?? 1.0,
              onChanged: (v) => notifier.updateLodmod('shadow_distance_multiplier', v),
              tooltip: 'Multiplies the per-scene shadow draw distance. '
                  '2.0 = shadows visible twice as far. Vanilla: 1.0. '
                  'Disable Min/Max below for this to work properly, '
                  'or use them to restrict the range this multiplier sets.',
            ),
            ConfigFieldFloat(
              label: 'Distance Minimum',
              value: (lod['shadow_distance_minimum'] as num?)?.toDouble() ?? 0.0,
              onChanged: (v) => notifier.updateLodmod('shadow_distance_minimum', v),
              tooltip: 'Minimum shadow draw distance clamp. 0 = off (no minimum). '
                  'Setting to ~70 with 8192 resolution matches vanilla quality '
                  'while greatly increasing shadow distance.',
            ),
            ConfigFieldFloat(
              label: 'Distance Maximum',
              value: (lod['shadow_distance_maximum'] as num?)?.toDouble() ?? 0.0,
              onChanged: (v) => notifier.updateLodmod('shadow_distance_maximum', v),
              tooltip: 'Maximum shadow draw distance clamp. 0 = off (no maximum). '
                  'Only worth setting if the default game distances cause performance issues.',
            ),
            ConfigFieldFloat(
              label: 'Distance PSS',
              value: (lod['shadow_distance_pss'] as num?)?.toDouble() ?? 0.0,
              onChanged: (v) => notifier.updateLodmod('shadow_distance_pss', v),
              tooltip: 'Enables PSS shadow distribution for more even shadow quality. '
                  '0 = off. Good values: 0.5 - 0.9. Looks great in some areas '
                  'but may appear blurry in others. Should be set much larger than '
                  'other distance values (~1500 for large open areas).',
            ),
            ConfigFieldFloat(
              label: 'Filter Strength Bias',
              value: (lod['shadow_filter_strength_bias'] as num?)?.toDouble() ?? 0.0,
              onChanged: (v) => notifier.updateLodmod('shadow_filter_strength_bias', v),
              tooltip: 'Adjusts shadow blur filter strength per-scene. 0 = off. '
                  '-1 = sharper shadows. Positive = softer. '
                  'Different areas use different strengths (forest = softer). '
                  'Can be combined with Min/Max to restrict the range.',
            ),
            ConfigFieldFloat(
              label: 'Filter Strength Min',
              value: (lod['shadow_filter_strength_minimum'] as num?)?.toDouble() ?? 0.0,
              onChanged: (v) => notifier.updateLodmod('shadow_filter_strength_minimum', v),
              tooltip: 'Forces a minimum shadow filter strength across all areas. '
                  '0 = off. Game default varies per scene (usually ~4). '
                  'Use to prevent shadows from being too sharp in any area.',
            ),
            ConfigFieldFloat(
              label: 'Filter Strength Max',
              value: (lod['shadow_filter_strength_maximum'] as num?)?.toDouble() ?? 0.0,
              onChanged: (v) => notifier.updateLodmod('shadow_filter_strength_maximum', v),
              tooltip: 'Forces a maximum shadow filter strength across all areas. '
                  '0 = off. Game default varies per scene (usually ~4). '
                  'Use to prevent shadows from being too blurry in any area.',
            ),
            ConfigFieldBool(
              label: 'HQ Shadow Models',
              value: lod['shadow_model_hq'] == true,
              onChanged: (v) => notifier.updateLodmod('shadow_model_hq', v),
              tooltip: 'Uses real-time HQ models for shadows instead of static LQ models. '
                  'Tree shadows will sway with the wind instead of being frozen. '
                  'Experimental \u2014 works well in city ruins, could cause issues in rare areas.',
            ),
            ConfigFieldBool(
              label: 'Force All Shadow Models',
              value: lod['shadow_model_force_all'] == true,
              onChanged: (v) => notifier.updateLodmod('shadow_model_force_all', v),
              tooltip: 'Forces all models to cast shadows, including small objects like rocks '
                  'and grass. Experimental \u2014 may rarely cause invisible models to cast '
                  'shadows. No issues noticed so far.',
            ),
          ]),

          // Post-Processing
          ConfigSection(title: 'POST-PROCESSING', children: [
            ConfigFieldBool(
              label: 'Disable Vignette',
              value: lod['disable_vignette'] == true,
              onChanged: (v) => notifier.updateLodmod('disable_vignette', v),
              tooltip: 'Removes the dark vignette effect on screen edges. '
                  'Some loading screens may still have it baked into textures.',
            ),
            ConfigFieldBool(
              label: 'Disable Fake HDR',
              value: lod['disable_fake_hdr'] == true,
              onChanged: (v) => notifier.updateLodmod('disable_fake_hdr', v),
              tooltip: 'Disables the pseudo-HDR mode added in the 2021 update. '
                  'The 2021 HDR is basically SDR inside an HDR container. '
                  'Disabling gives a cleaner image for proper color grading. '
                  'Note: YP does not support loading Special K.',
            ),
          ]),
        ],
      ],
    );
  }
}
