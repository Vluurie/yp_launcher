import 'package:flutter/material.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/services/detection_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';

part 'notification_state.g.dart';

enum NotificationType { migration, reshade, textures, shortcut }

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

  Future<void> runDetections(String gameDir) async {
    final prefs = await SharedPreferences.getInstance();
    final lastDir = prefs.getString(_prefKeyDetectedDir);
    if (lastDir == gameDir) return;

    await prefs.setString(_prefKeyDetectedDir, gameDir);

    final items = <NotificationItem>[];

    try {
      final iniValues = await DetectionService.detectLegacyLodMod(gameDir);
      if (iniValues != null) {
        final migrated =
            await DetectionService.migrateLodModIni(gameDir, iniValues);
        if (migrated) {
          items.add(const NotificationItem(
            id: 'lodmod_migration',
            message: AppStrings.notifyLodModMigrated,
            icon: Icons.swap_horiz,
            color: AppColors.success,
            type: NotificationType.migration,
          ));
        }
      }
    } catch (_) {}

    try {
      final hasReShade = await DetectionService.detectReShade(gameDir);
      if (hasReShade) {
        items.add(const NotificationItem(
          id: 'reshade',
          message: AppStrings.notifyReShadeDetected,
          icon: Icons.auto_fix_high,
          color: AppColors.accentPrimary,
          type: NotificationType.reshade,
        ));
      }
    } catch (_) {}

    try {
      final textureFolders = await DetectionService.detectHDTextures(gameDir);
      for (final folder in textureFolders) {
        items.add(NotificationItem(
          id: 'textures_$folder',
          message: AppStrings.notifyTexturesDetected(folder),
          icon: Icons.image,
          color: AppColors.accentPrimary,
          type: NotificationType.textures,
        ));
      }
    } catch (_) {}

    state = items;
  }

  void addNotification(NotificationItem item) {
    state = [...state, item];
  }

  void dismiss(String id) {
    state = state.where((n) => n.id != id).toList();
  }
}
