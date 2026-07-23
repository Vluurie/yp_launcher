import 'dart:io';
import 'package:yp_launcher/services/platform_gate.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:automato_theme/automato_theme.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:window_manager/window_manager.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/config_state.dart';
import 'package:yp_launcher/widgets/info_bar.dart'
    show showLaunchWarningsProvider;
import 'package:yp_launcher/services/process_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/widgets/launch_failure_dialog.dart';

class PlayButton extends ConsumerWidget {
  const PlayButton({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final appState = ref.watch(appStateControllerProvider);
    final controller = ref.read(appStateControllerProvider.notifier);

    final isRunning = appState.playButtonState == PlayButtonState.running;
    final isSearching = ref.watch(autoSearchingProvider);
    final canLaunch = PlatformGate.canLaunchGame;
    final buttonColor =
        !canLaunch ? AppColors.textMuted : _getButtonColor(appState);

    final button = SizedBox(
      width: AppSizes.playButtonWidth(context),
      height: AppSizes.playButtonHeight(context),
      child: TextButton(
        onPressed: appState.canPlay && !isSearching && canLaunch
            ? () => _handlePlayButton(context, ref, controller, appState, l10n)
            : null,
        style: TextButton.styleFrom(
          foregroundColor: buttonColor,
          backgroundColor: isRunning
              ? AppColors.error.withValues(alpha: 0.12)
              : AppColors.surfaceLight,
          disabledBackgroundColor: AppColors.surfaceLight.withValues(
            alpha: 0.5,
          ),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(6.0),
            side: BorderSide(color: buttonColor, width: isRunning ? 2.0 : 1.5),
          ),
          padding: const EdgeInsets.symmetric(vertical: 8.0, horizontal: 20.0),
        ),
        child: _buildButtonContent(context, ref, appState, l10n),
      ),
    );

    if (!canLaunch) {
      return Tooltip(
        message: l10n.playDisabledTooltip,
        child: button,
      );
    }
    return button;
  }

  Widget _buildButtonContent(
    BuildContext context,
    WidgetRef ref,
    AppState appState,
    AppLocalizations l10n,
  ) {
    if (appState.playButtonState == PlayButtonState.loading) {
      return SizedBox(
        width: AppSizes.playButtonLoadingSize(context),
        height: AppSizes.playButtonLoadingSize(context),
        child: AutomatoLoading(
          color: AppColors.accentPrimary,
          translateX: 0,
          svgString: AutomatoSvgStrings.automatoSvgStrHead,
        ),
      );
    }

    final isRunning = appState.playButtonState == PlayButtonState.running;
    final buttonColor = _getButtonColor(appState);
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Text(
          isRunning ? l10n.stopButton : l10n.playButton,
          style: TextStyle(
            fontSize: AppSizes.fontXXL(context),
            fontWeight: FontWeight.bold,
            color: buttonColor,
          ),
        ),
      ],
    );
  }

  Color _getButtonColor(AppState appState) {
    if (appState.playButtonState == PlayButtonState.running) {
      return AppColors.error;
    } else if (!appState.canPlay) {
      return AppColors.textMuted;
    } else {
      return AppColors.success;
    }
  }

  Future<void> _handlePlayButton(
    BuildContext context,
    WidgetRef ref,
    AppStateController controller,
    AppState appState,
    AppLocalizations l10n,
  ) async {
    if (appState.playButtonState == PlayButtonState.idle) {
      final nams = ref.read(configStateControllerProvider).namsValues;
      if (nams[NamsFields.disablePluginLoading.key] == true ||
          nams[NamsFields.disableReShadeLoading.key] == true ||
          nams[NamsFields.disableTextureInjection.key] == true) {
        ref.read(showLaunchWarningsProvider.notifier).state = true;
      }

      controller.setPlayButtonState(PlayButtonState.loading);
      controller.setStatus(l10n.statusLaunching);

      ProcessService.startNierAutomata(
            installDirectory: appState.selectedDirectory,
            l10n: l10n,
            onProcessStopped: () async {
              controller.setPlayButtonState(PlayButtonState.idle);
              controller.setStatus(l10n.statusStopped);
              if (Platform.isWindows) {
                try {
                  if (await windowManager.isMinimized()) {
                    await windowManager.restore();
                  }
                  await windowManager.focus();
                } catch (_) {}
              }
            },
          )
          .then((outcome) async {
            if (outcome.started) {
              controller.setPlayButtonState(PlayButtonState.running);
              controller.setStatus(l10n.statusRunning);
              if (Platform.isWindows) {
                try {
                  final prefs = await SharedPreferences.getInstance();
                  final shouldMinimize =
                      prefs.getBool(AppStrings.prefKeyMinimizeOnLaunch) ?? true;
                  if (shouldMinimize) {
                    await Future.delayed(const Duration(seconds: 3));
                    if (ref
                            .read(appStateControllerProvider)
                            .playButtonState ==
                        PlayButtonState.running) {
                      await windowManager.minimize();
                    }
                  }
                } catch (_) {}
              }
            } else {
              final failure = outcome.failure;
              controller.setError(
                failure?.friendlyTitle(l10n) ?? l10n.errorStartFailed,
              );
              controller.setPlayButtonState(PlayButtonState.idle);
              controller.setStatus(null);
              if (failure != null && context.mounted) {
                showLaunchFailureDialog(
                  context,
                  failure,
                  installDirectory: appState.selectedDirectory,
                );
              }
            }
          })
          .catchError((e) {
            controller.setError(e.toString());
            controller.setPlayButtonState(PlayButtonState.idle);
            controller.setStatus(null);
          });
    } else if (appState.playButtonState == PlayButtonState.running) {
      controller.setPlayButtonState(PlayButtonState.loading);
      controller.setStatus(l10n.statusStopping);

      final success = await ProcessService.terminateNierAutomata();
      if (success) {
        controller.setPlayButtonState(PlayButtonState.idle);
        controller.setStatus(l10n.statusStopped);
      } else {
        controller.setError(l10n.errorStopFailed);
        controller.setPlayButtonState(PlayButtonState.running);
        controller.setStatus(null);
      }
    }
  }
}

class PreferDedicatedGpuToggle extends StatefulWidget {
  const PreferDedicatedGpuToggle({super.key});

  @override
  State<PreferDedicatedGpuToggle> createState() =>
      _PreferDedicatedGpuToggleState();
}

class _PreferDedicatedGpuToggleState extends State<PreferDedicatedGpuToggle> {
  bool? _enabled;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final prefs = await SharedPreferences.getInstance();
    if (!mounted) return;
    setState(() {
      _enabled = prefs.getBool(AppStrings.prefKeyPreferDedicatedGpu) ?? true;
    });
  }

  Future<void> _set(bool value) async {
    setState(() => _enabled = value);
    final prefs = await SharedPreferences.getInstance();
    await prefs.setBool(AppStrings.prefKeyPreferDedicatedGpu, value);
  }

  @override
  Widget build(BuildContext context) {
    if (!Platform.isWindows && !Platform.isLinux) {
      return const SizedBox.shrink();
    }
    final value = _enabled ?? true;
    return Tooltip(
      message: AppLocalizations.of(
        context,
      )!.launchOptionPreferDedicatedGpuTooltip,
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          onTap: () => _set(!value),
          borderRadius: BorderRadius.circular(4),
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                SizedBox(
                  width: 16,
                  height: 16,
                  child: Checkbox(
                    value: value,
                    onChanged: (v) => _set(v ?? true),
                    visualDensity: VisualDensity.compact,
                    materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
                    activeColor: AppColors.accentPrimary,
                    checkColor: AppColors.backgroundPrimary,
                    side: const BorderSide(
                      color: AppColors.borderMedium,
                      width: 1.2,
                    ),
                  ),
                ),
                const SizedBox(width: 8),
                Flexible(
                  child: Text(
                    AppLocalizations.of(
                      context,
                    )!.launchOptionPreferDedicatedGpu,
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      color: AppColors.textMuted,
                      letterSpacing: 0.3,
                    ),
                    softWrap: true,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class MinimizeOnLaunchToggle extends StatefulWidget {
  const MinimizeOnLaunchToggle({super.key});

  @override
  State<MinimizeOnLaunchToggle> createState() => _MinimizeOnLaunchToggleState();
}

class _MinimizeOnLaunchToggleState extends State<MinimizeOnLaunchToggle> {
  bool? _enabled;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final prefs = await SharedPreferences.getInstance();
    if (!mounted) return;
    setState(() {
      _enabled = prefs.getBool(AppStrings.prefKeyMinimizeOnLaunch) ?? true;
    });
  }

  Future<void> _set(bool value) async {
    setState(() => _enabled = value);
    final prefs = await SharedPreferences.getInstance();
    await prefs.setBool(AppStrings.prefKeyMinimizeOnLaunch, value);
  }

  @override
  Widget build(BuildContext context) {
    if (!PlatformGate.isWindows) return const SizedBox.shrink();
    final value = _enabled ?? true;
    return Material(
      color: Colors.transparent,
      child: InkWell(
        onTap: () => _set(!value),
        borderRadius: BorderRadius.circular(4),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              SizedBox(
                width: 16,
                height: 16,
                child: Checkbox(
                  value: value,
                  onChanged: (v) => _set(v ?? true),
                  visualDensity: VisualDensity.compact,
                  materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
                  activeColor: AppColors.accentPrimary,
                  checkColor: AppColors.backgroundPrimary,
                  side: const BorderSide(
                    color: AppColors.borderMedium,
                    width: 1.2,
                  ),
                ),
              ),
              const SizedBox(width: 8),
              Flexible(
                child: Text(
                  AppLocalizations.of(context)!.launchOptionMinimizeOnLaunch,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.textMuted,
                    letterSpacing: 0.3,
                  ),
                  softWrap: true,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
