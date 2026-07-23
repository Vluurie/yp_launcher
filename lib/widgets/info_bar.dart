import 'dart:io';
import 'package:yp_launcher/services/platform_gate.dart';
import 'package:flutter/material.dart';
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

  const InfoBar({super.key, this.logPanelKey, required this.onOpenLogs});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final appState = ref.watch(appStateControllerProvider);
    if (!appState.isDirectorySelected) return const SizedBox.shrink();

    final gameDir = appState.selectedDirectory;

    return Positioned(
      bottom: AppSizes.infoBarBottom(context),
      left: AppSizes.infoBarPaddingH(context),
      right: AppSizes.infoBarPaddingH(context),
      child: Container(
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
            Expanded(
              child: Wrap(
                spacing: AppSizes.spacingSM(context),
                runSpacing: AppSizes.spacingSM(context),
                crossAxisAlignment: WrapCrossAlignment.center,
                alignment: WrapAlignment.start,
                children: [
                  DetectionChip(gameDir: gameDir, type: 'exe', iconOnly: true),
                  DetectionChip(gameDir: gameDir, type: 'dlc', iconOnly: true),
                  DetectionChip(gameDir: gameDir, type: 'lodmod', iconOnly: true),
                  DetectionChip(gameDir: gameDir, type: 'reshade', iconOnly: true),
                  DetectionChip(gameDir: gameDir, type: 'textures', iconOnly: true),
                  DetectionChip(gameDir: gameDir, type: 'mods', iconOnly: true),
                  DetectionChip(gameDir: gameDir, type: 'cutscene', iconOnly: true),
                ],
              ),
            ),
            SizedBox(width: AppSizes.spacingMD(context)),
            Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                _InfoBarButton(
                  label: l10n.infoBarLogs,
                  tooltip: l10n.tooltipOpenLogs,
                  onTap: () {
                    ref.read(logPanelOpenProvider.notifier).state = true;
                    ref.read(logStateControllerProvider.notifier).loadLogs();
                  },
                ),
                if (PlatformGate.isWindows) ...[
                  SizedBox(width: AppSizes.spacingMD(context)),
                  _InfoBarButton(
                    label: l10n.infoBarShortcut,
                    tooltip: l10n.tooltipCreateShortcut,
                    onTap: () async {
                      final success =
                          await ShortcutService.createDesktopShortcut(
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
                          icon: success
                              ? Icons.check_circle
                              : Icons.error_outline,
                          color: success ? AppColors.success : AppColors.error,
                          type: NotificationType.shortcut,
                        ),
                      );
                    },
                  ),
                ],
              ],
            ),
          ],
        ),
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

class _DetectionChipState extends ConsumerState<DetectionChip> {
  bool _detected = false;
  String _label = '';
  bool _checked = false;
  bool _warning = false;
  String? _tooltip;
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

  Future<void> _detect() async {
    final l10n = AppLocalizations.of(context)!;
    switch (widget.type) {
      case 'lodmod':
        final tomlPath = path.join(widget.gameDir, 'nams', 'lodmod.toml');
        final raw = await TomlService.readTomlFile(tomlPath);
        if (raw.isNotEmpty) {
          final values = TomlService.parse(raw);
          final enabled = values[LodModFields.enabled.key] == true;
          if (mounted)
            setState(() {
              _detected = enabled;
              _label = enabled ? l10n.chipLodModOn : l10n.chipLodModOff;
              _checked = true;
            });
        } else {
          if (mounted)
            setState(() {
              _detected = false;
              _label = l10n.chipLodModOff;
              _checked = true;
            });
        }
      case 'reshade':
        final status = await ReShadeDetection.detectReShade(widget.gameDir);
        if (mounted)
          setState(() {
            _detected = status == ReShadeStatus.detected;
            _label = l10n.chipReShade;
            _checked = true;
          });
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
        if (count > 0 || hasSkRes) {
          final parts = <String>[];
          if (count > 0) parts.add(l10n.chipInjectedCount(count));
          if (hasSkRes) parts.add(l10n.chipSkRes);
          if (mounted)
            setState(() {
              _detected = true;
              _label = l10n.chipTexturesCount(parts.join(', '));
              _checked = true;
            });
        } else {
          if (mounted)
            setState(() {
              _detected = false;
              _label = l10n.chipNoTextures;
              _checked = true;
            });
        }
      case 'mods':
        final modsDirPath = path.join(widget.gameDir, 'nams', 'mods');
        final count = await IsolateService.run(
          _countEntriesSync,
          _CountParams(dirPath: modsDirPath, dirsOnly: true),
        );
        if (count > 0) {
          if (mounted)
            setState(() {
              _detected = true;
              _label = l10n.chipModsCount(count);
              _checked = true;
            });
          return;
        }
        if (mounted)
          setState(() {
            _detected = false;
            _label = l10n.chipNoMods;
            _checked = true;
          });
      case 'cutscene':
        final result = await CutsceneDetectionService.scan(widget.gameDir);
        if (result.filesScanned > 0 && result.hasHdCutscenes) {
          final codec = result.needsH264 ? 'H264' : 'MPEG-2';
          if (mounted)
            setState(() {
              _detected = true;
              _label = l10n.chipCutsceneMod(
                result.largestWidth,
                result.largestHeight,
                codec,
              );
              _checked = true;
            });
        } else {
          if (mounted)
            setState(() {
              _detected = false;
              _label = l10n.chipNoCutsceneMod;
              _checked = true;
            });
        }
      case 'dlc':
        final has = await GameDetection.hasDlc(widget.gameDir);
        if (!mounted) return;
        setState(() {
          _detected = has;
          _warning = false;
          _label =
              has ? l10n.detectionDlcPresent : l10n.detectionDlcNotDetected;
          _tooltip = has
              ? l10n.detectionDlcPresentTooltip
              : l10n.detectionDlcNotDetectedTooltip;
          _checked = true;
        });
      case 'exe':
        final variant = await GameDetection.detectExeVariant(widget.gameDir);
        if (!mounted) return;
        switch (variant) {
          case ExeVariant.wolfLimitBreak:
            setState(() {
              _detected = true;
              _warning = true;
              _label = l10n.detectionExeWolfLimitBreak;
              _tooltip = l10n.detectionExeWolfLimitBreakTooltip;
              _checked = true;
            });
          case ExeVariant.original:
            setState(() {
              _detected = true;
              _warning = false;
              _label = l10n.detectionExeOriginal;
              _tooltip = null;
              _checked = true;
            });
          case ExeVariant.missing:
            setState(() {
              _detected = false;
              _warning = false;
              _label = l10n.detectionExeMissing;
              _tooltip = null;
              _checked = true;
            });
          case ExeVariant.unknown:
            setState(() {
              _detected = false;
              _warning = false;
              _label = l10n.detectionExeUnrecognised;
              _tooltip = l10n.detectionExeUnrecognisedTooltip;
              _checked = true;
            });
        }
    }
  }

  @override
  Widget build(BuildContext context) {
    final refresh = ref.watch(detectionRefreshProvider);
    if (refresh != _lastRefresh) {
      _lastRefresh = refresh;
      if (_checked) _detect();
    }
    if (!_checked) return const SizedBox.shrink();
    final color = _warning
        ? AppColors.warning
        : (_detected ? AppColors.success : AppColors.textMuted);
    final icon = _warning
        ? Icons.warning_amber_rounded
        : (_detected ? Icons.check_circle : Icons.remove_circle_outline);
    final typeIcon = _typeIcon(widget.type);

    if (widget.iconOnly) {
      final tooltipMsg = _tooltip != null && _tooltip!.isNotEmpty
          ? '$_label\n\n$_tooltip'
          : _label;
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
          _label,
          style: TextStyle(
            fontSize: AppSizes.fontXS(context),
            fontWeight: FontWeight.bold,
            color: color,
          ),
        ),
      ],
    );
    if (_tooltip == null) return row;
    return Tooltip(message: _tooltip!, child: row);
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
    ).animate(CurvedAnimation(parent: _controller, curve: Curves.easeOutCubic));
    _fade = Tween<double>(
      begin: 0.0,
      end: 1.0,
    ).animate(CurvedAnimation(parent: _controller, curve: Curves.easeIn));
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

