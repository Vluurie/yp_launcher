import 'dart:io';
import 'package:yp_launcher/services/platform_gate.dart';
import 'package:flutter/material.dart';
import 'package:yp_launcher/theme/nier_curves.dart';
import 'package:path/path.dart' as path;
import 'package:toml/toml.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_riverpod/legacy.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/config_state.dart';
import 'package:yp_launcher/providers/log_state.dart';
import 'package:yp_launcher/providers/notification_state.dart';
import 'package:yp_launcher/services/cutscene_detection_service.dart';
import 'package:yp_launcher/services/detection/game_detection.dart';
import 'package:yp_launcher/services/detection/reshade_detection.dart';
import 'package:yp_launcher/services/isolate_service.dart';
import 'package:yp_launcher/services/shortcut_service.dart';
import 'package:yp_launcher/services/toml_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/models/config_fields.dart';

final showLaunchWarningsProvider = StateProvider<bool>((ref) => false);

class InfoBar extends ConsumerWidget {
  final GlobalKey? logPanelKey;
  final VoidCallback onOpenLogs;
  final Widget? leading;

  const InfoBar({
    super.key,
    this.logPanelKey,
    required this.onOpenLogs,
    this.leading,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final appState = ref.watch(appStateControllerProvider);
    if (!appState.isDirectorySelected) return const SizedBox.shrink();

    final gameDir = appState.selectedDirectory;

    final buttons = Wrap(
      spacing: AppSizes.spacingMD(context),
      runSpacing: AppSizes.spacingSM(context),
      alignment: WrapAlignment.end,
      crossAxisAlignment: WrapCrossAlignment.center,
      children: [
          _InfoBarButton(
            label: l10n.infoBarLogs,
            tooltip: l10n.tooltipOpenLogs,
            onTap: () {
              ref.read(logPanelOpenProvider.notifier).state = true;
              ref.read(logStateControllerProvider.notifier).loadLogs();
            },
          ),
          if (PlatformGate.isWindows)
            _InfoBarButton(
              label: l10n.infoBarShortcut,
              tooltip: l10n.tooltipCreateShortcut,
              onTap: () async {
                final success = await ShortcutService.createDesktopShortcut(
                  gameDirectory: gameDir,
                );
                final notifier = ref.read(
                  notificationStateControllerProvider.notifier,
                );
                notifier.addNotification(
                  NotificationItem(
                    id: 'shortcut_${DateTime.now().millisecondsSinceEpoch}',
                    message: (l10n) => success
                        ? l10n.notifyShortcutCreated
                        : l10n.notifyShortcutFailed,
                    icon: success ? Icons.check_circle : Icons.error_outline,
                    color: success ? AppColors.success : AppColors.error,
                    type: NotificationType.shortcut,
                  ),
                );
              },
            ),
        ],
    );

    return Container(
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.cardPaddingH(context),
        vertical: AppSizes.cardPaddingV(context),
      ),
      decoration: BoxDecoration(
        color: AppColors.backgroundCard,
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(color: AppColors.borderLight),
      ),
      child: Row(
        children: [
          if (leading != null) leading!,
          Expanded(child: buttons),
        ],
      ),
    );
  }
}

/// Vertical strip of detection status icons, meant to be pinned to the right
/// edge of the launcher tab where nothing else competes for space.
class DetectionStatusStrip extends ConsumerWidget {
  const DetectionStatusStrip({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final appState = ref.watch(appStateControllerProvider);
    if (!appState.isDirectorySelected) return const SizedBox.shrink();
    final gameDir = appState.selectedDirectory;

    return Container(
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.chipPaddingH(context),
        vertical: AppSizes.chipPaddingV(context),
      ),
      decoration: BoxDecoration(
        color: AppColors.backgroundCard.withValues(alpha: 0.7),
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(color: AppColors.borderLight),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          for (final type in const [
            'exe',
            'dlc',
            'lodmod',
            'reshade',
            'textures',
            'mods',
            'cutscene',
          ])
            Padding(
              key: ValueKey('detection_$type'),
              padding: EdgeInsets.symmetric(
                vertical: AppSizes.spacingSM(context) / 2,
              ),
              child: DetectionChip(
                key: ValueKey('detection_chip_$type'),
                gameDir: gameDir,
                type: type,
                iconOnly: true,
              ),
            ),
        ],
      ),
    );
  }
}

class _InfoBarButton extends StatefulWidget {
  final String label;
  final String tooltip;
  final VoidCallback onTap;

  const _InfoBarButton({
    required this.label,
    required this.tooltip,
    required this.onTap,
  });

  @override
  State<_InfoBarButton> createState() => _InfoBarButtonState();
}

class _InfoBarButtonState extends State<_InfoBarButton> {
  bool _hovering = false;

  @override
  Widget build(BuildContext context) {
    return Tooltip(
      message: widget.tooltip,
      child: MouseRegion(
        cursor: SystemMouseCursors.click,
        onEnter: (_) => setState(() => _hovering = true),
        onExit: (_) => setState(() => _hovering = false),
        child: GestureDetector(
          onTap: widget.onTap,
          child: Container(
            padding: EdgeInsets.symmetric(
              horizontal: AppSizes.paddingMD(context),
              vertical: AppSizes.paddingXS(context),
            ),
            decoration: BoxDecoration(
              color: _hovering ? AppColors.accentPrimary : Colors.transparent,
              border: Border.all(color: AppColors.accentPrimary),
              borderRadius: BorderRadius.circular(
                AppSizes.borderRadius(context),
              ),
            ),
            child: Text(
              widget.label,
              style: TextStyle(
                fontSize: AppSizes.fontMD(context),
                color: _hovering
                    ? AppColors.buttonText
                    : AppColors.accentPrimary,
                fontWeight: FontWeight.bold,
              ),
            ),
          ),
        ),
      ),
    );
  }
}

class DetectionChip extends ConsumerStatefulWidget {
  final String gameDir;
  final String type;
  final bool iconOnly;

  const DetectionChip({
    super.key,
    required this.gameDir,
    required this.type,
    this.iconOnly = false,
  });

  @override
  ConsumerState<DetectionChip> createState() => _DetectionChipState();
}

class _DetectionResult {
  final bool detected;
  final bool warning;
  final String kind;
  final int count;
  final int count2;
  final bool flag;
  final ExeVariant? variant;

  const _DetectionResult({
    required this.detected,
    required this.kind,
    this.warning = false,
    this.count = 0,
    this.count2 = 0,
    this.flag = false,
    this.variant,
  });
}

class _DetectionChipState extends ConsumerState<DetectionChip> {
  _DetectionResult? _result;
  bool _checked = false;
  int _lastRefresh = -1;

  static int _spawnOrderCounter = 0;

  @override
  void initState() {
    super.initState();
    final order = _spawnOrderCounter++;
    WidgetsBinding.instance.addPostFrameCallback((_) async {
      if (!mounted) return;
      // Stagger isolate spawns so we don't freeze the first frame
      // when all detection chips boot up at once.
      await Future<void>.delayed(Duration(milliseconds: 60 * order));
      if (!mounted) return;
      _detect();
    });
  }

  @override
  void didUpdateWidget(DetectionChip old) {
    super.didUpdateWidget(old);
    if (old.gameDir != widget.gameDir) _detect();
  }

  void _apply(_DetectionResult result) {
    if (!mounted) return;
    setState(() {
      _result = result;
      _checked = true;
    });
  }

  Future<void> _detect() async {
    switch (widget.type) {
      case 'lodmod':
        final tomlPath = path.join(widget.gameDir, 'nams', 'lodmod.toml');
        final raw = await TomlService.readTomlFile(tomlPath);
        var enabled = false;
        if (raw.isNotEmpty) {
          enabled = TomlService.parse(raw)[LodModFields.enabled.key] == true;
        }
        _apply(_DetectionResult(detected: enabled, kind: 'lodmod'));
      case 'reshade':
        final status = await ReShadeDetection.detectReShade(widget.gameDir);
        _apply(_DetectionResult(
          detected: status == ReShadeStatus.detected,
          kind: 'reshade',
        ));
      case 'textures':
        final injectDirPath =
            path.join(widget.gameDir, 'nams', 'inject', 'textures');
        final textureTomlPath =
            path.join(widget.gameDir, 'nams', 'texture_injection.toml');
        final skRes = Directory(path.join(widget.gameDir, 'SK_Res'));
        final count = await IsolateService.run(
          _countActiveTexturesSync,
          _ActiveTextureCountParams(
            dirPath: injectDirPath,
            textureTomlPath: textureTomlPath,
          ),
        );
        final hasSkRes = await skRes.exists();
        _apply(_DetectionResult(
          detected: count > 0 || hasSkRes,
          kind: 'textures',
          count: count,
          flag: hasSkRes,
        ));
      case 'mods':
        final modsDirPath = path.join(widget.gameDir, 'nams', 'mods');
        final count = await IsolateService.run(
          _countEntriesSync,
          _CountParams(dirPath: modsDirPath, dirsOnly: true),
        );
        _apply(_DetectionResult(
          detected: count > 0,
          kind: 'mods',
          count: count,
        ));
      case 'cutscene':
        final result = await CutsceneDetectionService.scan(widget.gameDir);
        final has = result.filesScanned > 0 && result.hasHdCutscenes;
        _apply(_DetectionResult(
          detected: has,
          kind: 'cutscene',
          count: result.largestWidth,
          count2: result.largestHeight,
          flag: result.needsH264,
        ));
      case 'dlc':
        final has = await GameDetection.hasDlc(widget.gameDir);
        _apply(_DetectionResult(detected: has, kind: 'dlc'));
      case 'exe':
        final variant = await GameDetection.detectExeVariant(widget.gameDir);
        _apply(_DetectionResult(
          detected: variant != ExeVariant.missing &&
              variant != ExeVariant.unknown,
          warning: variant == ExeVariant.wolfLimitBreak ||
              variant == ExeVariant.legacyWindows7,
          kind: 'exe',
          variant: variant,
        ));
    }
  }

  ({String label, String? tooltip}) _localize(
    _DetectionResult r,
    AppLocalizations l10n,
  ) {
    switch (r.kind) {
      case 'lodmod':
        return (
          label: r.detected ? l10n.chipLodModOn : l10n.chipLodModOff,
          tooltip: null,
        );
      case 'reshade':
        return (label: l10n.chipReShade, tooltip: null);
      case 'textures':
        if (!r.detected) return (label: l10n.chipNoTextures, tooltip: null);
        final parts = <String>[];
        if (r.count > 0) parts.add(l10n.chipInjectedCount(r.count));
        if (r.flag) parts.add(l10n.chipSkRes);
        return (label: l10n.chipTexturesCount(parts.join(', ')), tooltip: null);
      case 'mods':
        return (
          label: r.detected ? l10n.chipModsCount(r.count) : l10n.chipNoMods,
          tooltip: null,
        );
      case 'cutscene':
        if (!r.detected) return (label: l10n.chipNoCutsceneMod, tooltip: null);
        return (
          label: l10n.chipCutsceneMod(
            r.count,
            r.count2,
            r.flag ? 'H264' : 'MPEG-2',
          ),
          tooltip: null,
        );
      case 'dlc':
        return (
          label: r.detected
              ? l10n.detectionDlcPresent
              : l10n.detectionDlcNotDetected,
          tooltip: r.detected
              ? l10n.detectionDlcPresentTooltip
              : l10n.detectionDlcNotDetectedTooltip,
        );
      case 'exe':
        switch (r.variant!) {
          case ExeVariant.wolfLimitBreak:
            return (
              label: l10n.detectionExeWolfLimitBreak,
              tooltip: l10n.detectionExeWolfLimitBreakTooltip,
            );
          case ExeVariant.legacyWindows7:
            return (
              label: l10n.detectionExeLegacyWin7,
              tooltip: l10n.detectionExeLegacyWin7Tooltip,
            );
          case ExeVariant.original:
            return (label: l10n.detectionExeOriginal, tooltip: null);
          case ExeVariant.missing:
            return (label: l10n.detectionExeMissing, tooltip: null);
          case ExeVariant.unknown:
            return (
              label: l10n.detectionExeUnrecognised,
              tooltip: l10n.detectionExeUnrecognisedTooltip,
            );
        }
      default:
        return (label: '', tooltip: null);
    }
  }

  @override
  Widget build(BuildContext context) {
    final refresh = ref.watch(detectionRefreshProvider);
    if (refresh != _lastRefresh) {
      _lastRefresh = refresh;
      if (_checked) _detect();
    }
    final result = _result;
    if (!_checked || result == null) return const SizedBox.shrink();

    final l10n = AppLocalizations.of(context)!;
    final text = _localize(result, l10n);
    final label = text.label;
    final tooltip = text.tooltip;

    final color = result.warning
        ? AppColors.warning
        : (result.detected ? AppColors.success : AppColors.textMuted);
    final icon = result.warning
        ? Icons.warning_amber_rounded
        : (result.detected
            ? Icons.check_circle
            : Icons.remove_circle_outline);
    final typeIcon = _typeIcon(widget.type);

    if (widget.iconOnly) {
      final tooltipMsg = tooltip != null && tooltip.isNotEmpty
          ? '$label\n\n$tooltip'
          : label;
      return Tooltip(
        message: tooltipMsg,
        child: Container(
          width: 36,
          height: 36,
          margin: const EdgeInsets.symmetric(vertical: 3),
          decoration: BoxDecoration(
            color: color.withValues(alpha: 0.10),
            border: Border.all(color: color.withValues(alpha: 0.45)),
            borderRadius: BorderRadius.circular(6),
          ),
          child: Stack(
            children: [
              Center(
                child: Icon(
                  typeIcon,
                  size: 18,
                  color: color,
                ),
              ),
              Positioned(
                right: 2,
                bottom: 2,
                child: Container(
                  width: 10,
                  height: 10,
                  decoration: BoxDecoration(
                    color: AppColors.backgroundCard,
                    shape: BoxShape.circle,
                  ),
                  child: Icon(
                    icon,
                    size: 10,
                    color: color,
                  ),
                ),
              ),
            ],
          ),
        ),
      );
    }

    final row = Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Icon(
          icon,
          size: AppSizes.iconSM(context),
          color: color,
        ),
        SizedBox(width: AppSizes.spacingSM(context)),
        Text(
          label,
          style: TextStyle(
            fontSize: AppSizes.fontXS(context),
            fontWeight: FontWeight.bold,
            color: color,
          ),
        ),
      ],
    );
    if (tooltip == null) return row;
    return Tooltip(message: tooltip, child: row);
  }

  IconData _typeIcon(String type) {
    switch (type) {
      case 'exe':
        return Icons.memory;
      case 'dlc':
        return Icons.videogame_asset_outlined;
      case 'lodmod':
        return Icons.image_outlined;
      case 'reshade':
        return Icons.auto_fix_high;
      case 'textures':
        return Icons.texture;
      case 'mods':
        return Icons.extension_outlined;
      case 'cutscene':
        return Icons.movie_creation_outlined;
      default:
        return Icons.info_outline;
    }
  }
}

class PersistentWarningBanner extends ConsumerWidget {
  const PersistentWarningBanner({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final show = ref.watch(showLaunchWarningsProvider);
    if (!show) return const SizedBox.shrink();

    final config = ref.watch(configStateControllerProvider);
    final nams = config.namsValues;
    final warnings = <String>[];
    if (nams[NamsFields.disablePluginLoading.key] == true)
      warnings.add(l10n.warningPluginLoadingDisabled);
    if (nams[NamsFields.disableReShadeLoading.key] == true)
      warnings.add(l10n.warningReShadeDisabled);
    if (nams[NamsFields.disableTextureInjection.key] == true)
      warnings.add(l10n.warningTextureInjectionDisabled);
    if (warnings.isEmpty) {
      Future.microtask(
        () => ref.read(showLaunchWarningsProvider.notifier).state = false,
      );
      return const SizedBox.shrink();
    }

    return Positioned(
      bottom: 80,
      left: AppSizes.infoBarPaddingH(context),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: warnings
            .map((msg) => AnimatedWarning(message: msg, key: ValueKey(msg)))
            .toList(),
      ),
    );
  }
}

class AnimatedWarning extends StatefulWidget {
  final String message;
  const AnimatedWarning({super.key, required this.message});

  @override
  State<AnimatedWarning> createState() => _AnimatedWarningState();
}

class _AnimatedWarningState extends State<AnimatedWarning>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<Offset> _slide;
  late Animation<double> _fade;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 300),
    );
    _slide = Tween<Offset>(
      begin: const Offset(-1.0, 0),
      end: Offset.zero,
    ).animate(CurvedAnimation(parent: _controller, curve: NierCurves.fill));
    _fade = Tween<double>(
      begin: 0.0,
      end: 1.0,
    ).animate(CurvedAnimation(parent: _controller, curve: NierCurves.rise));
    _controller.forward();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return SlideTransition(
      position: _slide,
      child: FadeTransition(
        opacity: _fade,
        child: Container(
          margin: EdgeInsets.only(bottom: AppSizes.spacingMD(context)),
          padding: EdgeInsets.symmetric(
            horizontal: AppSizes.paddingMD(context),
            vertical: AppSizes.paddingSM(context),
          ),
          decoration: BoxDecoration(
            color: AppColors.warning.withValues(alpha: 0.12),
            borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
            border: Border.all(color: AppColors.warning.withValues(alpha: 0.3)),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(
                Icons.warning_amber,
                size: AppSizes.iconSM(context),
                color: AppColors.warning,
              ),
              SizedBox(width: AppSizes.spacingMD(context)),
              Text(
                widget.message,
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  color: AppColors.warning,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _CountParams {
  final String dirPath;
  final bool dirsOnly;
  const _CountParams({required this.dirPath, required this.dirsOnly});
}

int _countEntriesSync(_CountParams params) {
  final dir = Directory(params.dirPath);
  if (!dir.existsSync()) return 0;
  var count = 0;
  for (final entity in dir.listSync(followLinks: false)) {
    if (params.dirsOnly) {
      if (entity is! Directory) continue;
      final name = path.basename(entity.path);
      if (name.startsWith('.') || name.startsWith('_')) continue;
    }
    count++;
  }
  return count;
}

class _ActiveTextureCountParams {
  final String dirPath;
  final String textureTomlPath;
  const _ActiveTextureCountParams({
    required this.dirPath,
    required this.textureTomlPath,
  });
}

int _countActiveTexturesSync(_ActiveTextureCountParams params) {
  final dir = Directory(params.dirPath);
  if (!dir.existsSync()) return 0;

  final disabled = <String>{};
  try {
    final tomlFile = File(params.textureTomlPath);
    if (tomlFile.existsSync()) {
      final parsed = TomlDocument.parse(tomlFile.readAsStringSync()).toMap();
      final raw = parsed['disabled_packs'];
      if (raw is List) {
        for (final entry in raw) {
          if (entry is String) disabled.add(entry);
        }
      }
    }
  } catch (_) {}

  var count = 0;
  for (final entity in dir.listSync(followLinks: false)) {
    final name = path.basename(entity.path);
    if (disabled.contains(name)) continue;
    count++;
  }
  return count;
}

