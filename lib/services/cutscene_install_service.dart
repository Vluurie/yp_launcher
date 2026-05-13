import 'dart:io';
import 'dart:isolate';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:file_picker/file_picker.dart';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/config_state.dart';
import 'package:yp_launcher/providers/notification_state.dart';
import 'package:yp_launcher/services/archive_service.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/services/isolate_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/widgets/cutscenes/cutscene_dialogs.dart';
import 'package:yp_launcher/widgets/cutscenes/cutscene_isolates.dart';

class CutsceneInstallService {
  final WidgetRef ref;
  final BuildContext context;
  final String cutscenesDir;
  final String originalMovieDir;
  final void Function(bool installing, String progressText, double progressPercent) onProgress;
  final Future<void> Function() onReload;
  final List<CutsceneMod> Function() getMods;

  CutsceneInstallService({
    required this.ref,
    required this.context,
    required this.cutscenesDir,
    required this.originalMovieDir,
    required this.onProgress,
    required this.onReload,
    required this.getMods,
  });

  Future<bool> _dropContainsUsm(String sourcePath) async {
    if (ArchiveService.isArchive(sourcePath)) {
      final found = await ArchiveService.archiveContainsExtension(
          sourcePath, ['.usm']);
      return found == true;
    }
    if (FileSystemEntity.isFileSync(sourcePath)) {
      return sourcePath.toLowerCase().endsWith('.usm');
    }
    if (FileSystemEntity.isDirectorySync(sourcePath)) {
      return IsolateService.run(_dirContainsUsmSync, sourcePath);
    }
    return false;
  }

  Future<void> _refreshCutsceneConfig() async {
    final gameDir = path.normalize(path.join(cutscenesDir, '..', '..'));
    await ref
        .read(configStateControllerProvider.notifier)
        .autoDetectCutscenes(gameDir);
  }

  void notify(String message, IconData icon, Color color) {
    ref
        .read(notificationStateControllerProvider.notifier)
        .addNotification(
          NotificationItem(
            id: 'cutscenes_${DateTime.now().millisecondsSinceEpoch}',
            message: message,
            icon: icon,
            color: color,
            type: NotificationType.shortcut,
          ),
        );
  }

  Future<void> handleDrop(List<String> paths) async {
    if (paths.isEmpty) return;
    await installFromPath(paths.first);
  }

  Future<void> handleBrowse() async {
    final result = await FilePicker.platform.getDirectoryPath(
      dialogTitle: AppLocalizations.of(context)!.selectCutsceneModFolder,
    );
    if (result == null) return;
    await installFromPath(result);
  }

  Future<void> installFromPath(String sourcePath) async {
    final busy = ref.read(busyOperationsProvider.notifier);
    busy.state = busy.state + 1;
    try {
      await _installFromPathInner(sourcePath);
    } finally {
      busy.state = busy.state - 1;
    }
  }

  Future<void> _installFromPathInner(String sourcePath) async {
    final existingMods = getMods();
    var effectivePath = sourcePath;
    String? tempExtractDir;

    final l10n = AppLocalizations.of(context)!;
    onProgress(true, l10n.cutsceneDropAnalyzing, 0);
    final hasUsm = await _dropContainsUsm(sourcePath);
    if (!hasUsm) {
      notify(l10n.cutsceneDropNoUsm, Icons.error_outline, AppColors.error);
      onProgress(false, '', 0);
      return;
    }

    if (ArchiveService.isArchive(sourcePath)) {
      onProgress(true, l10n.extractingArchive, 0);
      notify(l10n.extractingArchive, Icons.hourglass_top, AppColors.accentPrimary);
      final extracted = await ArchiveService.extract(
        sourcePath,
        onProgress: (percent, currentFile) {
          final label = currentFile == null || currentFile.isEmpty
              ? l10n.extractingArchivePercent((percent * 100).round())
              : l10n.extractingArchivePercentFile(
                  (percent * 100).round(), currentFile);
          onProgress(true, label, percent);
        },
      );
      if (extracted == null) {
        final detail = ArchiveService.lastExtractError ?? '';
        final isDiskFull = detail.toLowerCase().contains('disk full') ||
            detail.toLowerCase().contains('no space') ||
            detail.toLowerCase().contains('not enough space') ||
            detail.contains('exit 7') ||
            detail.contains('exit 8');
        final msg = isDiskFull
            ? l10n.extractFailedDiskFull
            : detail.isNotEmpty
                ? l10n.extractFailedDetail(detail)
                : l10n.failedToExtractArchive;
        notify(msg, Icons.error_outline, AppColors.error);
        onProgress(false, '', 0);
        return;
      }
      effectivePath = extracted;
      tempExtractDir = extracted;
    }

    try {
      await _runInstall(sourcePath, effectivePath, existingMods);
    } finally {
      if (tempExtractDir != null) {
        try {
          await Directory(tempExtractDir).delete(recursive: true);
        } catch (_) {}
      }
    }
  }

  Future<void> _runInstall(
    String sourcePath,
    String effectivePath,
    List<CutsceneMod> existingMods,
  ) async {

    onProgress(
      true,
      AppLocalizations.of(context)!.cutsceneScanningArchive,
      0,
    );
    final movieDir = await findMovieDir(effectivePath);
    if (movieDir == null) {
      notify(AppLocalizations.of(context)!.noMovieFolderFound, Icons.movie_filter, AppColors.warning);
      onProgress(false, '', 0);
      return;
    }

    final defaultName = sourcePath
        .split(RegExp(r'[\\/]'))
        .last
        .replaceAll(RegExp(r'\.(zip|7z|rar|tar|gz|bz2)$', caseSensitive: false), '');

    String? name;
    if (existingMods.isNotEmpty) {
      final mergeChoice = await showCutsceneMergeDialog(context, existingMods);
      if (mergeChoice == null) {
        onProgress(false, '', 0);
        return;
      }
      if (mergeChoice.isNotEmpty) {
        name = mergeChoice;
      }
    }
    name ??= await showCutsceneNamingDialog(context, defaultName);

    if (name == null || name.trim().isEmpty) {
      onProgress(false, '', 0);
      return;
    }

    final trimmed = name.trim();
    if (trimmed.length > maxModNameLength) {
      notify(
        AppLocalizations.of(context)!.cutsceneNameTooLong(maxModNameLength),
        Icons.error_outline,
        AppColors.warning,
      );
      onProgress(false, '', 0);
      return;
    }

    onProgress(true, AppLocalizations.of(context)!.preparingInstall, 0);

    final targetDir = path.join(cutscenesDir, trimmed);
    final receivePort = ReceivePort();

    final isolate = await Isolate.spawn(
      installWithProgress,
      ProgressInstallParams(
        sourcePath: effectivePath,
        movieDir: movieDir,
        targetDir: targetDir,
        sendPort: receivePort.sendPort,
      ),
    );

    int count = 0;
    await for (final message in receivePort) {
      if (message is Map<String, dynamic>) {
        final type = message['type'] as String;
        if (type == 'progress') {
          final l10n = AppLocalizations.of(context)!;
          final current = message['current'] as int;
          final total = message['total'] as int;
          final name = message['name'] as String;
          final mbDone = message['mbDone'] as String;
          final mbTotal = message['mbTotal'] as String;
          final text = mbDone == '0'
              ? l10n.cutsceneCopyingFile(current, total, name)
              : l10n.cutsceneCopyingFileBytes(
                  current, total, name, mbDone, mbTotal);
          onProgress(true, text, message['percent'] as double);
        } else if (type == 'done') {
          count = message['count'] as int;
          break;
        }
      }
    }

    isolate.kill(priority: Isolate.immediate);
    receivePort.close();
    onProgress(false, '', 0);

    if (count > 0) {
      notify(
        AppLocalizations.of(context)!.installedCutsceneMod(trimmed, count),
        Icons.check_circle,
        AppColors.success,
      );
      await _refreshCutsceneConfig();
      await onReload();
      ref.read(detectionRefreshProvider.notifier).state++;
    } else {
      notify(AppLocalizations.of(context)!.noUsmFilesFound, Icons.error_outline, AppColors.error);
    }
  }

  Future<void> deleteMod(CutsceneMod mod) async {
    final confirm = await showDeleteCutsceneConfirmDialog(context, mod.name);
    if (!confirm) return;

    final l10n = AppLocalizations.of(context)!;
    final busy = ref.read(busyOperationsProvider.notifier);
    busy.state = busy.state + 1;
    onProgress(true, l10n.busyDeletingCutscene, 0);
    try {
      await IsolateService.run(deleteCutsceneModIsolate, mod.fullPath);
      notify(
        l10n.removedItem(mod.name),
        Icons.delete_outline,
        AppColors.textMuted,
      );
      await _refreshCutsceneConfig();
      await onReload();
      ref.read(detectionRefreshProvider.notifier).state++;
    } finally {
      onProgress(false, '', 0);
      busy.state = busy.state - 1;
    }
  }
}

bool _dirContainsUsmSync(String dirPath) {
  try {
    final dir = Directory(dirPath);
    if (!dir.existsSync()) return false;
    for (final entity in dir.listSync(recursive: true, followLinks: false)) {
      if (entity is File && entity.path.toLowerCase().endsWith('.usm')) {
        return true;
      }
    }
  } catch (_) {}
  return false;
}
