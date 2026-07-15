import 'dart:async';
import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:file_picker/file_picker.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/notification_state.dart';
import 'package:yp_launcher/services/mods_service.dart';
import 'package:yp_launcher/services/nams_config_service.dart';
import 'package:yp_launcher/services/nier_finder_service.dart';
import 'package:yp_launcher/services/win10_compat_setup_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class DirectorySelector extends ConsumerStatefulWidget {
  const DirectorySelector({super.key});

  @override
  ConsumerState<DirectorySelector> createState() => _DirectorySelectorState();
}

class _DirectorySelectorState extends ConsumerState<DirectorySelector> {
  bool _hovered = false;
  bool _autoSearchTriggered = false;

  bool get _searching => ref.watch(autoSearchingProvider);

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _maybeAutoSearch();
    });
  }

  Future<void> _maybeAutoSearch() async {
    if (_autoSearchTriggered) return;
    _autoSearchTriggered = true;

    final appState = ref.read(appStateControllerProvider);
    if (!appState.isDirectorySelected) {
      await _runAutoSearch();
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final appState = ref.watch(appStateControllerProvider);
    final controller = ref.read(appStateControllerProvider.notifier);

    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 8.0, vertical: 4.0),
        decoration: BoxDecoration(
          color: _hovered
              ? AppColors.backgroundCard.withValues(alpha: 0.95)
              : AppColors.backgroundCard,
          borderRadius: BorderRadius.circular(6.0),
          border: Border.all(
            color: _hovered ? AppColors.borderMedium : AppColors.borderLight,
          ),
        ),
        child: Row(
          children: [
            Expanded(
              child: Container(
                padding: const EdgeInsets.symmetric(
                  horizontal: 8.0,
                  vertical: 5.0,
                ),
                decoration: BoxDecoration(
                  color: AppColors.inputBackground,
                  borderRadius: BorderRadius.circular(4.0),
                  border: Border.all(
                    color: appState.isDirectorySelected
                        ? AppColors.success
                        : AppColors.borderMedium,
                  ),
                ),
                child: _searching
                    ? Row(
                        children: [
                          const SizedBox(
                            width: 12,
                            height: 12,
                            child: CircularProgressIndicator(
                              strokeWidth: 2,
                              color: AppColors.accentPrimary,
                            ),
                          ),
                          const SizedBox(width: 8),
                          Text(
                            l10n.searchingForNier,
                            style: TextStyle(
                              color: AppColors.textMuted,
                              fontSize: AppSizes.fontXS(context),
                              fontStyle: FontStyle.italic,
                            ),
                          ),
                        ],
                      )
                    : SelectableText(
                        appState.isDirectorySelected
                            ? appState.selectedDirectory
                            : l10n.noFileSelected,
                        style: TextStyle(
                          color: appState.isDirectorySelected
                              ? AppColors.textPrimary
                              : AppColors.textMuted,
                          fontSize: AppSizes.fontXS(context),
                          fontStyle: appState.isDirectorySelected
                              ? FontStyle.normal
                              : FontStyle.italic,
                        ),
                        maxLines: 1,
                      ),
              ),
            ),
            const SizedBox(width: 6),
            SizedBox(
              height: 26,
              child: ElevatedButton(
                onPressed: _searching ? null : () => _runAutoSearch(),
                style: ElevatedButton.styleFrom(
                  backgroundColor: AppColors.surfaceLight,
                  foregroundColor: AppColors.accentPrimary,
                  padding: const EdgeInsets.symmetric(horizontal: 10.0),
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(4.0),
                  ),
                  minimumSize: const Size(0, 26),
                  tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                ),
                child: Text(
                  l10n.autoButton,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    fontWeight: FontWeight.bold,
                    letterSpacing: 0.4,
                  ),
                ),
              ),
            ),
            const SizedBox(width: 4),
            SizedBox(
              height: 26,
              child: ElevatedButton(
                onPressed: _searching
                    ? null
                    : () => _selectDirectory(controller, ref),
                style: ElevatedButton.styleFrom(
                  backgroundColor: AppColors.buttonBackground,
                  foregroundColor: AppColors.buttonText,
                  padding: const EdgeInsets.symmetric(horizontal: 12.0),
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(4.0),
                  ),
                  minimumSize: const Size(0, 26),
                  tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                ),
                child: Text(
                  l10n.selectButton,
                  style: TextStyle(
                    fontSize: AppSizes.fontSM(context),
                    fontWeight: FontWeight.bold,
                    letterSpacing: 0.4,
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _runAutoSearch() async {
    if (_searching) return;

    ref.read(autoSearchingProvider.notifier).state = true;

    try {
      final fastResults = await NierFinderService.findFast();

      if (!mounted) return;

      if (fastResults.isNotEmpty) {
        final result = await _showFoundDialog(fastResults);
        if (result == 'deep' && mounted) {
          await _runDeepSearch();
        } else if (result == 'manual' && mounted) {
          await _selectDirectory(
            ref.read(appStateControllerProvider.notifier),
            ref,
          );
        } else if (result != null &&
            result.isNotEmpty &&
            result != 'deep' &&
            result != 'manual') {
          await _applyDirectory(result);
        }
      } else {
        final shouldDeep = await _showNotFoundDialog();
        if (shouldDeep == true && mounted) {
          await _runDeepSearch();
        } else if (mounted) {
          await _selectDirectory(
            ref.read(appStateControllerProvider.notifier),
            ref,
          );
        }
      }
    } catch (_) {
      if (mounted) {
        await _selectDirectory(
          ref.read(appStateControllerProvider.notifier),
          ref,
        );
      }
    } finally {
      if (mounted) {
        ref.read(autoSearchingProvider.notifier).state = false;
      }
    }
  }

  Future<void> _runDeepSearch() async {
    final deepResults = await NierFinderService.findDeep();
    if (!mounted) return;

    if (deepResults.isEmpty) {
      await _selectDirectory(
        ref.read(appStateControllerProvider.notifier),
        ref,
      );
    } else if (deepResults.length == 1) {
      final confirmed = await _showConfirmDialog(deepResults.first);
      if (confirmed == true) {
        await _applyDirectory(deepResults.first.path);
      } else if (confirmed == false && mounted) {
        await _selectDirectory(
          ref.read(appStateControllerProvider.notifier),
          ref,
        );
      }
    } else {
      final selected = await _showInstallationDialog(deepResults);
      if (selected != null && selected.isNotEmpty) {
        await _applyDirectory(selected);
      } else if (mounted) {
        await _selectDirectory(
          ref.read(appStateControllerProvider.notifier),
          ref,
        );
      }
    }
  }

  Future<String?> _showFoundDialog(List<NierInstallation> installations) {
    return showDialog<String>(
      context: context,
      builder: (ctx) {
        final dl10n = AppLocalizations.of(ctx)!;
        return AlertDialog(
          backgroundColor: AppColors.backgroundCard,
          title: Text(
            dl10n.nierFound,
            style: TextStyle(
              color: AppColors.accentPrimary,
              fontSize: AppSizes.fontXL(ctx),
            ),
          ),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                dl10n.selectInstallation,
                style: TextStyle(
                  color: AppColors.textSecondary,
                  fontSize: AppSizes.fontMD(ctx),
                ),
              ),
              const SizedBox(height: 12),
              ...installations.map(
                (i) => Padding(
                  padding: const EdgeInsets.only(bottom: 8),
                  child: SizedBox(
                    width: double.infinity,
                    child: ElevatedButton(
                      onPressed: () => Navigator.of(ctx).pop(i.path),
                      style: ElevatedButton.styleFrom(
                        backgroundColor: AppColors.surfaceLight,
                        foregroundColor: AppColors.textPrimary,
                        padding: const EdgeInsets.symmetric(
                          horizontal: 14,
                          vertical: 12,
                        ),
                        shape: RoundedRectangleBorder(
                          borderRadius: BorderRadius.circular(6),
                          side: BorderSide(
                            color: i.hasData
                                ? AppColors.success
                                : AppColors.warning,
                          ),
                        ),
                      ),
                      child: Row(
                        children: [
                          Icon(
                            Icons.folder,
                            size: 18,
                            color: i.hasData
                                ? AppColors.success
                                : AppColors.warning,
                          ),
                          const SizedBox(width: 10),
                          Flexible(
                            child: Text(
                              i.path,
                              style: TextStyle(fontSize: AppSizes.fontSM(ctx)),
                            ),
                          ),
                          const SizedBox(width: 10),
                          Icon(
                            Icons.arrow_forward,
                            size: 16,
                            color: AppColors.accentPrimary,
                          ),
                        ],
                      ),
                    ),
                  ),
                ),
              ),
              const SizedBox(height: 8),
              const Divider(color: AppColors.borderLight),
              const SizedBox(height: 4),
              Text(
                dl10n.notYourGame,
                style: TextStyle(
                  color: AppColors.textMuted,
                  fontSize: AppSizes.fontXS(ctx),
                ),
              ),
            ],
          ),
          actions: [
            OutlinedButton(
              onPressed: () => Navigator.of(ctx).pop('deep'),
              style: OutlinedButton.styleFrom(
                foregroundColor: AppColors.accentPrimary,
                side: const BorderSide(color: AppColors.accentPrimary),
              ),
              child: Text(dl10n.searchAllDrives),
            ),
            OutlinedButton(
              onPressed: () => Navigator.of(ctx).pop('manual'),
              style: OutlinedButton.styleFrom(
                foregroundColor: AppColors.textSecondary,
                side: BorderSide(color: AppColors.borderLight),
              ),
              child: Text(dl10n.selectManually),
            ),
          ],
        );
      },
    );
  }

  Future<bool?> _showNotFoundDialog() {
    return showDialog<bool>(
      context: context,
      builder: (ctx) {
        final dl10n = AppLocalizations.of(ctx)!;
        return AlertDialog(
          backgroundColor: AppColors.backgroundCard,
          title: Text(
            dl10n.notFoundTitle,
            style: TextStyle(
              color: AppColors.accentPrimary,
              fontSize: AppSizes.fontXL(ctx),
            ),
          ),
          content: Text(
            dl10n.notFoundMessage,
            style: TextStyle(
              color: AppColors.textSecondary,
              fontSize: AppSizes.fontMD(ctx),
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(ctx).pop(false),
              child: Text(
                dl10n.selectManually,
                style: TextStyle(color: AppColors.textMuted),
              ),
            ),
            TextButton(
              onPressed: () => Navigator.of(ctx).pop(true),
              child: Text(
                dl10n.scanDrives,
                style: TextStyle(
                  color: AppColors.accentPrimary,
                  fontWeight: FontWeight.bold,
                ),
              ),
            ),
          ],
        );
      },
    );
  }

  Future<bool?> _showConfirmDialog(NierInstallation installation) {
    return showDialog<bool>(
      context: context,
      builder: (ctx) {
        final dl10n = AppLocalizations.of(ctx)!;
        return AlertDialog(
          backgroundColor: AppColors.backgroundCard,
          title: Text(
            dl10n.nierFound,
            style: TextStyle(
              color: AppColors.accentPrimary,
              fontSize: AppSizes.fontXL(ctx),
            ),
          ),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                dl10n.confirmInstallation,
                style: TextStyle(
                  color: AppColors.textSecondary,
                  fontSize: AppSizes.fontMD(ctx),
                ),
              ),
              const SizedBox(height: 12),
              Row(
                children: [
                  Icon(
                    Icons.folder,
                    size: 16,
                    color: installation.hasData
                        ? AppColors.success
                        : AppColors.warning,
                  ),
                  const SizedBox(width: 8),
                  Flexible(
                    child: Text(
                      installation.path,
                      style: TextStyle(
                        color: AppColors.textPrimary,
                        fontSize: AppSizes.fontSM(ctx),
                      ),
                    ),
                  ),
                ],
              ),
            ],
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(ctx).pop(null),
              child: Text(
                dl10n.cancelButton,
                style: TextStyle(color: AppColors.textMuted),
              ),
            ),
            TextButton(
              onPressed: () => Navigator.of(ctx).pop(false),
              child: Text(
                dl10n.noSelectManually,
                style: TextStyle(color: AppColors.textMuted),
              ),
            ),
            TextButton(
              onPressed: () => Navigator.of(ctx).pop(true),
              child: Text(
                dl10n.yesButton,
                style: TextStyle(
                  color: AppColors.accentPrimary,
                  fontWeight: FontWeight.bold,
                ),
              ),
            ),
          ],
        );
      },
    );
  }

  Future<String?> _showInstallationDialog(
    List<NierInstallation> installations,
  ) {
    return showDialog<String>(
      context: context,
      barrierDismissible: true,
      builder: (ctx) {
        final dl10n = AppLocalizations.of(ctx)!;
        final withData = installations.where((i) => i.hasData).toList();
        final withoutData = installations.where((i) => !i.hasData).toList();

        return Dialog(
          backgroundColor: AppColors.backgroundPrimary,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(12),
            side: BorderSide(color: AppColors.borderMedium),
          ),
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 600, maxHeight: 500),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Container(
                  padding: const EdgeInsets.all(20),
                  decoration: BoxDecoration(
                    color: AppColors.backgroundCard,
                    borderRadius: const BorderRadius.vertical(
                      top: Radius.circular(12),
                    ),
                  ),
                  child: Text(
                    dl10n.installationsFoundTitle,
                    style: TextStyle(
                      color: AppColors.textPrimary,
                      fontSize: AppSizes.fontXL(ctx),
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ),
                Flexible(
                  child: SingleChildScrollView(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 16,
                      vertical: 12,
                    ),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        if (withData.isNotEmpty) ...[
                          Padding(
                            padding: const EdgeInsets.only(bottom: 8),
                            child: Row(
                              children: [
                                Icon(
                                  Icons.check_circle,
                                  color: AppColors.success,
                                  size: 18,
                                ),
                                const SizedBox(width: 8),
                                Text(
                                  dl10n.validInstallations,
                                  style: TextStyle(
                                    color: AppColors.success,
                                    fontSize: AppSizes.fontSM(ctx),
                                    fontWeight: FontWeight.bold,
                                  ),
                                ),
                              ],
                            ),
                          ),
                          ...withData.map(
                            (i) => _installationTile(
                              ctx,
                              i.path,
                              AppColors.success,
                              Icons.folder,
                            ),
                          ),
                          const SizedBox(height: 12),
                        ],
                        if (withoutData.isNotEmpty) ...[
                          Padding(
                            padding: const EdgeInsets.only(bottom: 8),
                            child: Row(
                              children: [
                                Icon(
                                  Icons.warning_rounded,
                                  color: AppColors.warning,
                                  size: 18,
                                ),
                                const SizedBox(width: 8),
                                Text(
                                  dl10n.withoutDataFolder,
                                  style: TextStyle(
                                    color: AppColors.warning,
                                    fontSize: AppSizes.fontSM(ctx),
                                    fontWeight: FontWeight.bold,
                                  ),
                                ),
                              ],
                            ),
                          ),
                          ...withoutData.map(
                            (i) => _installationTile(
                              ctx,
                              i.path,
                              AppColors.warning,
                              Icons.folder_open,
                            ),
                          ),
                        ],
                      ],
                    ),
                  ),
                ),
                Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 16,
                    vertical: 12,
                  ),
                  decoration: BoxDecoration(
                    color: AppColors.backgroundCard,
                    borderRadius: const BorderRadius.vertical(
                      bottom: Radius.circular(12),
                    ),
                  ),
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.end,
                    children: [
                      TextButton(
                        onPressed: () => Navigator.pop(ctx),
                        child: Text(
                          dl10n.cancelButton,
                          style: TextStyle(color: AppColors.textMuted),
                        ),
                      ),
                      const SizedBox(width: 8),
                      ElevatedButton(
                        onPressed: () => Navigator.pop(ctx, null),
                        style: ElevatedButton.styleFrom(
                          backgroundColor: AppColors.surfaceLight,
                          foregroundColor: AppColors.textSecondary,
                        ),
                        child: Text(dl10n.selectManually),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _installationTile(
    BuildContext ctx,
    String path,
    Color color,
    IconData icon,
  ) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 6),
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          onTap: () => Navigator.pop(ctx, path),
          borderRadius: BorderRadius.circular(6),
          hoverColor: AppColors.surfaceLight,
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            decoration: BoxDecoration(
              border: Border.all(color: AppColors.borderLight),
              borderRadius: BorderRadius.circular(6),
            ),
            child: Row(
              children: [
                Icon(icon, color: color, size: 20),
                const SizedBox(width: 10),
                Expanded(
                  child: Text(
                    path,
                    style: TextStyle(
                      color: color,
                      fontSize: AppSizes.fontSM(ctx),
                    ),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                Icon(
                  Icons.arrow_forward_ios,
                  color: AppColors.textMuted,
                  size: 14,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Future<void> _applyDirectory(String directory) async {
    final controller = ref.read(appStateControllerProvider.notifier);
    await controller.setDirectory(directory);
    await NamsConfigService.ensureConfigs(directory);
    await Win10CompatSetupService.ensureRanForDirectory(directory);
    await ModsService.syncDlcSlots(directory);
    await ref
        .read(notificationStateControllerProvider.notifier)
        .refreshDetections(directory);
    ref.read(detectionRefreshProvider.notifier).state++;
  }

  Future<void> _selectDirectory(
    AppStateController controller,
    WidgetRef ref,
  ) async {
    final l10n = AppLocalizations.of(context)!;
    try {
      final result = await FilePicker.pickFiles(
        dialogTitle: l10n.filePickerTitle,
        type: FileType.custom,
        allowedExtensions: [AppStrings.allowedExtension],
      );

      if (result != null && result.files.single.path != null) {
        final filePath = result.files.single.path!;
        final file = File(filePath);

        if (await file.exists()) {
          final isValid = await compute(_validateNierExeInIsolate, filePath);

          if (isValid) {
            final directory = file.parent.path;
            await _applyDirectory(directory);
          } else {
            controller.setError(l10n.errorInvalidExe);
          }
        } else {
          controller.setError(l10n.errorFileNotExist);
        }
      }
    } catch (e) {
      controller.setError('${l10n.errorSelectFailed}: $e');
    }
  }
}

Future<bool> _validateNierExeInIsolate(String filePath) async {
  try {
    final file = File(filePath);
    final raf = await file.open();
    try {
      final header = await raf.read(2);
      if (header.length < 2 || header[0] != 0x4D || header[1] != 0x5A) {
        return false;
      }

      const chunkSize = 65536;
      final searchBytes = AppStrings.gameSignature.codeUnits;
      int position = 0;

      while (true) {
        await raf.setPosition(position);
        final chunk = await raf.read(chunkSize);

        if (chunk.isEmpty) break;

        for (int i = 0; i <= chunk.length - searchBytes.length; i++) {
          if (chunk[i] == searchBytes[0]) {
            bool match = true;
            for (int j = 1; j < searchBytes.length; j++) {
              if (chunk[i + j] != searchBytes[j]) {
                match = false;
                break;
              }
            }
            if (match) return true;
          }
        }

        position += chunkSize;
        if (chunk.length < chunkSize) break;
      }

      return false;
    } finally {
      await raf.close();
    }
  } catch (e) {
    return true;
  }
}
