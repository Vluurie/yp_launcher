import 'dart:io';
import 'package:flutter/material.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/services/detection/game_detection.dart';
import 'package:yp_launcher/services/detection/lodmod_detection.dart';
import 'package:yp_launcher/services/detection/naiom_detection.dart';
import 'package:yp_launcher/services/detection/reshade_detection.dart';
import 'package:yp_launcher/services/nams_config_service.dart';
import 'package:yp_launcher/services/launcher_setup_service.dart';
import 'package:yp_launcher/services/platform/platform_adapter.dart';
import 'package:yp_launcher/theme/app_colors.dart';

part 'notification_state.g.dart';

enum NotificationType { migration, reshade, textures, shortcut, general }

class NotificationItem {
  final String id;
  final String message;
  final IconData icon;
  final Color color;
  final NotificationType type;

  const NotificationItem({
    required this.id,
    required this.message,
    required this.icon,
    required this.color,
    required this.type,
  });
}

@Riverpod(keepAlive: true)
class NotificationStateController extends _$NotificationStateController {
  static const _prefKeyDetectedDir = 'detected_notifications_dir';

  @override
  List<NotificationItem> build() => [];

  Future<void> refreshDetections(String gameDir) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(_prefKeyDetectedDir);
    await runDetections(gameDir);
  }

  Future<void> runDetections(String gameDir) async {
    final prefs = await SharedPreferences.getInstance();
    final lastDir = prefs.getString(_prefKeyDetectedDir);
    if (lastDir == gameDir) return;

    await prefs.setString(_prefKeyDetectedDir, gameDir);

    final items = <NotificationItem>[];

    try {
      final missing = await LauncherSetupService.findMissingFiles();
      if (missing.isNotEmpty) {
        items.add(
          const NotificationItem(
            id: 'files_quarantined',
            message: AppStrings.notifyFilesQuarantined,
            icon: Icons.shield_outlined,
            color: AppColors.error,
            type: NotificationType.general,
          ),
        );
      }
    } catch (_) {}

    try {
      final iniValues = await LodModDetection.detectLegacyLodMod(gameDir);
      if (iniValues != null) {
        final migrated = await LodModDetection.migrateLodModIni(
          gameDir,
          iniValues,
        );
        if (migrated) {
          items.add(
            const NotificationItem(
              id: 'lodmod_migration',
              message: AppStrings.notifyLodModMigrated,
              icon: Icons.swap_horiz,
              color: AppColors.success,
              type: NotificationType.migration,
            ),
          );
        }
      }
    } catch (_) {}

    try {
      final iniValues = await NaiomDetection.detectLegacyNaiom(gameDir);
      if (iniValues != null) {
        await NamsConfigService.ensureConfigs(gameDir);
        final result = await NaiomDetection.migrateNaiomIni(
          gameDir,
          iniValues,
        );
        if (result.migrated) {
          items.add(
            const NotificationItem(
              id: 'naiom_migration',
              message: AppStrings.notifyNaiomMigrated,
              icon: Icons.swap_horiz,
              color: AppColors.success,
              type: NotificationType.migration,
            ),
          );
          if (result.skippedEntries.isNotEmpty) {
            items.add(
              NotificationItem(
                id: 'naiom_migration_skipped',
                message: AppStrings.notifyNaiomSkipped(
                  result.skippedEntries.join(', '),
                ),
                icon: Icons.warning_amber,
                color: AppColors.warning,
                type: NotificationType.migration,
              ),
            );
          }
        }
      }
    } catch (_) {}

    try {
      final reshadeStatus = await ReShadeDetection.detectReShade(gameDir);
      if (reshadeStatus == ReShadeStatus.detected) {
        await ReShadeDetection.autoDisableReShadeLoading(gameDir);
        items.add(
          const NotificationItem(
            id: 'reshade',
            message: AppStrings.notifyReShadeDetected,
            icon: Icons.auto_fix_high,
            color: AppColors.accentPrimary,
            type: NotificationType.reshade,
          ),
        );
      } else if (reshadeStatus == ReShadeStatus.incompatibleAddon) {
        items.add(
          const NotificationItem(
            id: 'reshade_incompatible',
            message: AppStrings.notifyReShadeIncompatible,
            icon: Icons.warning_amber,
            color: AppColors.warning,
            type: NotificationType.reshade,
          ),
        );
      }
    } catch (_) {}

    try {
      final textureFolders = await GameDetection.detectHDTextures(gameDir);
      for (final folder in textureFolders) {
        items.add(
          NotificationItem(
            id: 'textures_$folder',
            message: AppStrings.notifyTexturesDetected(folder),
            icon: Icons.image,
            color: AppColors.accentPrimary,
            type: NotificationType.textures,
          ),
        );
      }
    } catch (_) {}

    state = items;
  }

  void addNotification(NotificationItem item) {
    state = [...state, item];
  }

  void checkPlatformSupport() {
    const id = 'platform_unsupported';
    if (state.any((n) => n.id == id)) return;
    if (PlatformAdapter.current.canLaunchGame) return;
    final platformName = Platform.isLinux
        ? 'Linux'
        : Platform.isMacOS
            ? 'macOS'
            : Platform.operatingSystem;
    state = [
      ...state,
      NotificationItem(
        id: id,
        message: AppStrings.notifyPlatformUnsupported(platformName),
        icon: Icons.warning_amber,
        color: AppColors.warning,
        type: NotificationType.general,
      ),
    ];
  }

  void dismiss(String id) {
    state = state.where((n) => n.id != id).toList();
  }
}
