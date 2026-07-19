import 'dart:io';
import 'package:path/path.dart' as path;
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:automato_theme/automato_theme.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/services/detection/game_detection.dart';
import 'package:yp_launcher/services/detection/lodmod_detection.dart';
import 'package:yp_launcher/services/detection/naiom_detection.dart';
import 'package:yp_launcher/services/detection/reshade_detection.dart';
import 'package:yp_launcher/services/nams_settings_service.dart';
import 'package:yp_launcher/services/platform_gate.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/widgets/onboarding/first_playthrough_step.dart';
import 'package:yp_launcher/widgets/onboarding/migration_step.dart';
import 'package:yp_launcher/widgets/onboarding/mods_step.dart';
import 'package:yp_launcher/widgets/onboarding/ready_step.dart';
import 'package:yp_launcher/widgets/onboarding/select_game_step.dart';
import 'package:yp_launcher/widgets/onboarding/shared.dart';
import 'package:yp_launcher/widgets/onboarding/welcome_step.dart';

class OnboardingWizard extends ConsumerStatefulWidget {
  final VoidCallback onComplete;

  const OnboardingWizard({super.key, required this.onComplete});

  @override
  ConsumerState<OnboardingWizard> createState() => _OnboardingWizardState();
}

class _OnboardingWizardState extends ConsumerState<OnboardingWizard> {
  int _currentStep = 0;
  String? _selectedPath;
  bool _firstPlaythrough = true;

  ReShadeStatus _reshadeStatus = ReShadeStatus.notFound;
  bool _hasLodMod = false;
  bool _hasSkRes = false;
  bool _hasNaiom = false;
  bool _hasCutsceneMods = false;
  int _existingNamsMods = 0;
  List<String> _textureResults = [];
  bool _detectionsLoaded = false;

  static const _totalSteps = 6;

  @override
  void initState() {
    super.initState();
    _loadFirstPlaythrough();
  }

  Future<void> _loadFirstPlaythrough() async {
    final settings = await NamsSettingsService.loadSettings(_selectedPath);
    if (mounted) {
      setState(() {
        _firstPlaythrough = settings['firstPlaythrough'] ?? true;
      });
    }
  }

  Future<void> _completeOnboarding() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setBool('onboarding_complete', true);
    widget.onComplete();
  }

  Future<void> _runDetections() async {
    if (_detectionsLoaded || _selectedPath == null) return;
    _detectionsLoaded = true;

    try {
      _reshadeStatus = await ReShadeDetection.detectReShade(_selectedPath!);
    } catch (_) {}

    try {
      _textureResults = await GameDetection.detectHDTextures(_selectedPath!);
    } catch (_) {}

    try {
      final iniValues =
          await LodModDetection.detectLegacyLodMod(_selectedPath!);
      _hasLodMod = iniValues != null;
    } catch (_) {}

    try {
      final skResDir = Directory(path.join(_selectedPath!, 'SK_Res'));
      _hasSkRes = await skResDir.exists();
    } catch (_) {}

    try {
      _hasNaiom =
          await NaiomDetection.detectLegacyNaiom(_selectedPath!) != null;
    } catch (_) {}

    try {
      final cutscenesDir =
          Directory(path.join(_selectedPath!, 'nams', 'cutscenes'));
      if (await cutscenesDir.exists()) {
        final entries =
            await cutscenesDir.list().where((e) => e is Directory).toList();
        _hasCutsceneMods = entries.isNotEmpty;
      }
    } catch (_) {}

    try {
      final modsDir = Directory(path.join(_selectedPath!, 'nams', 'mods'));
      if (await modsDir.exists()) {
        var count = 0;
        await for (final e in modsDir.list()) {
          if (e is! Directory) continue;
          final name = path.basename(e.path);
          if (name.startsWith('.') || name.startsWith('_')) continue;
          count++;
        }
        _existingNamsMods = count;
      }
    } catch (_) {}

    if (mounted) setState(() {});
  }

  void _next() {
    if (_currentStep < _totalSteps - 1) {
      setState(() => _currentStep++);
      if (_currentStep == 3 || _currentStep == 5) _runDetections();
    }
  }

  void _back() {
    if (_currentStep > 0) {
      setState(() => _currentStep--);
    }
  }

  @override
  Widget build(BuildContext context) {
    if (!PlatformGate.canLaunchGame) {
      return _buildUnsupportedPlatform(context);
    }
    return Stack(
      children: [
        AutomatoBackground(
          ref: ref,
          showBackgroundSVG: true,
          showMenuLines: true,
          backgroundColor: AppColors.backgroundPrimary,
          gradientColor: AppColors.backgroundSecondary,
          backgroundSvgConfig: const BackgroundSvgConfig(
            animateInner: true,
            animateOuter: true,
            showDual: true,
          ),
          linesConfig: LinesConfig(
            lineColor: AppColors.borderLight.withValues(alpha: 0.2),
            strokeWidth: 1.0,
            spacing: 10.0,
            drawVerticalLines: true,
            drawHorizontalLines: true,
            enableFlicker: false,
            flickerDuration: const Duration(milliseconds: 4000),
          ),
        ),
        Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 820),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Expanded(
                  child: AnimatedSwitcher(
                    duration: const Duration(milliseconds: 300),
                    switchInCurve: Curves.easeOut,
                    switchOutCurve: Curves.easeIn,
                    transitionBuilder: (child, animation) {
                      return FadeTransition(
                        opacity: animation,
                        child: SlideTransition(
                          position: Tween<Offset>(
                            begin: const Offset(0.15, 0),
                            end: Offset.zero,
                          ).animate(animation),
                          child: child,
                        ),
                      );
                    },
                    child: KeyedSubtree(
                      key: ValueKey(_currentStep),
                      child: _buildCurrentStep(),
                    ),
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.only(bottom: 32),
                  child: OnboardingDots(
                    currentStep: _currentStep,
                    totalSteps: _totalSteps,
                  ),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildCurrentStep() {
    switch (_currentStep) {
      case 0:
        return WelcomeStep(onNext: _next);
      case 1:
        return SelectGameStep(
          selectedPath: _selectedPath,
          onPathSelected: (dir) {
            setState(() {
              _selectedPath = dir;
              _detectionsLoaded = false;
            });
          },
          onNext: _next,
          onBack: _back,
        );
      case 2:
        return FirstPlaythroughStep(
          gameDir: _selectedPath,
          onNext: () {
            NamsSettingsService.loadSettings(_selectedPath).then((settings) {
              _firstPlaythrough = settings['firstPlaythrough'] ?? true;
              if (mounted) setState(() {});
            });
            _next();
          },
          onBack: _back,
        );
      case 3:
        return MigrationStep(
          reshadeStatus: _reshadeStatus,
          textureResults: _textureResults,
          hasLodMod: _hasLodMod,
          hasSkRes: _hasSkRes,
          hasNaiom: _hasNaiom,
          hasCutsceneMods: _hasCutsceneMods,
          existingNamsMods: _existingNamsMods,
          onNext: _next,
          onBack: _back,
        );
      case 4:
        return ModsStep(
          gameDir: _selectedPath,
          onNext: _next,
          onBack: _back,
        );
      case 5:
        return ReadyStep(
          selectedPath: _selectedPath,
          detections: ReadyDetections(
            reshadeStatus: _reshadeStatus,
            textureResults: _textureResults,
            hasLodMod: _hasLodMod,
            hasSkRes: _hasSkRes,
            hasNaiom: _hasNaiom,
            hasCutsceneMods: _hasCutsceneMods,
            firstPlaythrough: _firstPlaythrough,
          ),
          onComplete: _completeOnboarding,
        );
      default:
        return const SizedBox.shrink();
    }
  }

  Widget _buildUnsupportedPlatform(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final message = PlatformGate.isMacOS
        ? l10n.platformUnsupportedMacos
        : l10n.platformUnsupportedLinux;

    return Stack(
      children: [
        AutomatoBackground(
          ref: ref,
          showBackgroundSVG: true,
          showMenuLines: true,
          backgroundColor: AppColors.backgroundPrimary,
          gradientColor: AppColors.backgroundSecondary,
          backgroundSvgConfig: const BackgroundSvgConfig(
            animateInner: true,
            animateOuter: true,
            showDual: true,
          ),
          linesConfig: LinesConfig(
            lineColor: AppColors.borderLight.withValues(alpha: 0.2),
            strokeWidth: 1.0,
            spacing: 10.0,
            drawVerticalLines: true,
            drawHorizontalLines: true,
            enableFlicker: false,
            flickerDuration: const Duration(milliseconds: 4000),
          ),
        ),
        Positioned.fill(
          child: IgnorePointer(
            child: DecoratedBox(
              decoration: BoxDecoration(
                color: AppColors.error.withValues(alpha: 0.10),
              ),
            ),
          ),
        ),
        Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 560),
            child: Padding(
              padding: const EdgeInsets.all(32),
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(
                    Icons.error_outline,
                    color: AppColors.error,
                    size: 56,
                  ),
                  const SizedBox(height: 20),
                  Text(
                    l10n.platformUnsupportedTitle,
                    style: TextStyle(
                      fontSize: 22,
                      fontWeight: FontWeight.w600,
                      color: AppColors.error,
                    ),
                    textAlign: TextAlign.center,
                  ),
                  const SizedBox(height: 16),
                  Text(
                    message,
                    style: TextStyle(
                      fontSize: 14,
                      color: AppColors.textSecondary,
                      height: 1.5,
                    ),
                    textAlign: TextAlign.center,
                  ),
                ],
              ),
            ),
          ),
        ),
      ],
    );
  }
}
