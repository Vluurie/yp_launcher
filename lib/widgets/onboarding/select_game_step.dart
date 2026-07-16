import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:file_picker/file_picker.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/nier_installation.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/notification_state.dart';
import 'package:yp_launcher/services/nams_config_service.dart';
import 'package:yp_launcher/services/platform/platform_adapter.dart';
import 'package:yp_launcher/services/win10_compat_setup_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/onboarding/shared.dart';

class SelectGameStep extends ConsumerStatefulWidget {
  final String? selectedPath;
  final ValueChanged<String> onPathSelected;
  final VoidCallback onNext;
  final VoidCallback onBack;
  const SelectGameStep({
    super.key,
    required this.selectedPath,
    required this.onPathSelected,
    required this.onNext,
    required this.onBack,
  });

  @override
  ConsumerState<SelectGameStep> createState() => _SelectGameStepState();
}

class _SelectGameStepState extends ConsumerState<SelectGameStep> {
  bool _searching = false;
  bool _deepSearching = false;

  Future<void> _runAutoSearch() async {
    if (_searching) return;
    setState(() {
      _searching = true;
      _deepSearching = false;
    });

    try {
      final fastResults = await PlatformAdapter.current.discoverFast();
      if (!mounted) return;

      if (fastResults.isNotEmpty) {
        if (fastResults.length == 1) {
          await _applyPath(fastResults.first.path);
        } else {
          final selected = await _showInstallationsDialog(fastResults);
          if (selected != null) await _applyPath(selected);
        }
      } else {
        setState(() => _deepSearching = true);
        final deepResults = await PlatformAdapter.current.discoverDeep();
        if (!mounted) return;

        if (deepResults.isNotEmpty) {
          if (deepResults.length == 1) {
            await _applyPath(deepResults.first.path);
          } else {
            final selected = await _showInstallationsDialog(deepResults);
            if (selected != null) await _applyPath(selected);
          }
        }
      }
    } catch (_) {}

    if (mounted) {
      setState(() {
        _searching = false;
        _deepSearching = false;
      });
    }
  }

  Future<String?> _showInstallationsDialog(
    List<NierInstallation> installations,
  ) {
    return showDialog<String>(
      context: context,
      builder: (ctx) {
        final l = AppLocalizations.of(ctx)!;
        return AlertDialog(
          backgroundColor: AppColors.backgroundCard,
          title: Text(
            l.onboardingSelectInstallation,
            style: TextStyle(
              color: AppColors.accentPrimary,
              fontSize: AppSizes.fontXL(ctx),
            ),
          ),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            children: installations
                .map(
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
                                style: TextStyle(
                                  fontSize: AppSizes.fontSM(ctx),
                                ),
                              ),
                            ),
                            const SizedBox(width: 10),
                            const Icon(
                              Icons.arrow_forward,
                              size: 16,
                              color: AppColors.accentPrimary,
                            ),
                          ],
                        ),
                      ),
                    ),
                  ),
                )
                .toList(),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(ctx).pop(),
              child: Text(
                l.buttonCancel,
                style: const TextStyle(color: AppColors.textMuted),
              ),
            ),
          ],
        );
      },
    );
  }

  Future<void> _applyPath(String directory) async {
    final controller = ref.read(appStateControllerProvider.notifier);
    await controller.setDirectory(directory);
    await NamsConfigService.ensureConfigs(directory);
    await Win10CompatSetupService.ensureRanForDirectory(directory);
    ref
        .read(notificationStateControllerProvider.notifier)
        .refreshDetections(directory);
    if (mounted) widget.onPathSelected(directory);
  }

  Future<void> _selectManually() async {
    try {
      final result = await FilePicker.pickFiles(
        dialogTitle: AppLocalizations.of(context)!.filePickerTitle,
        type: FileType.custom,
        allowedExtensions: [AppStrings.allowedExtension],
      );

      if (result != null && result.files.single.path != null) {
        final filePath = result.files.single.path!;
        final file = File(filePath);

        if (await file.exists()) {
          final isValid = await compute(_validateNierExe, filePath);
          if (!isValid || !mounted) return;
          if (_rejectOutsidePrefix(filePath)) return;
          await _applyPath(file.parent.path);
        }
      }
    } catch (_) {}
  }

  bool _rejectOutsidePrefix(String exePath) {
    final reason = PlatformAdapter.current
        .rejectGameSelection(exePath, AppLocalizations.of(context)!);
    if (reason == null) return false;

    ref.read(notificationStateControllerProvider.notifier).addNotification(
          NotificationItem(
            id: 'game_outside_prefix',
            message: reason,
            icon: Icons.warning_amber,
            color: AppColors.warning,
            type: NotificationType.general,
          ),
        );
    return true;
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final selectedPath = widget.selectedPath;
    return OnboardingStepCard(
      children: [
        Text(
          l.onboardingSelectTitle,
          style: TextStyle(
            fontSize: AppSizes.fontXL(context),
            fontWeight: FontWeight.bold,
            color: AppColors.accentPrimary,
          ),
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 28),
        if (selectedPath != null) ...[
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
            decoration: BoxDecoration(
              color: AppColors.inputBackground,
              borderRadius: BorderRadius.circular(8),
              border: Border.all(color: AppColors.success),
            ),
            child: Row(
              children: [
                const Icon(
                  Icons.check_circle,
                  color: AppColors.success,
                  size: 22,
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: SelectableText(
                    selectedPath,
                    style: TextStyle(
                      fontSize: AppSizes.fontSM(context),
                      color: AppColors.textPrimary,
                    ),
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 20),
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              OnboardingBackButton(onTap: widget.onBack),
              const SizedBox(width: 12),
              SizedBox(
                height: 44,
                child: OutlinedButton(
                  onPressed: _selectManually,
                  style: OutlinedButton.styleFrom(
                    foregroundColor: AppColors.textSecondary,
                    side: const BorderSide(color: AppColors.borderMedium),
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(6),
                    ),
                  ),
                  child: Text(
                    l.buttonSelectManually,
                    style: TextStyle(fontSize: AppSizes.fontMD(context)),
                  ),
                ),
              ),
              const SizedBox(width: 12),
              SizedBox(
                width: 160,
                height: 48,
                child: ElevatedButton(
                  onPressed: widget.onNext,
                  style: ElevatedButton.styleFrom(
                    backgroundColor: AppColors.buttonBackground,
                    foregroundColor: AppColors.buttonText,
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(6),
                    ),
                  ),
                  child: Text(
                    l.buttonContinue,
                    style: TextStyle(
                      fontSize: AppSizes.fontLG(context),
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ),
              ),
            ],
          ),
        ] else ...[
          if (_searching) ...[
            Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                SizedBox(
                  width: 20,
                  height: 20,
                  child: CircularProgressIndicator(
                    strokeWidth: 2,
                    color: AppColors.accentPrimary,
                  ),
                ),
                const SizedBox(width: 12),
                Text(
                  _deepSearching
                      ? l.onboardingSearchingDrives
                      : l.onboardingSearchingNier,
                  style: TextStyle(
                    fontSize: AppSizes.fontMD(context),
                    color: AppColors.textMuted,
                    fontStyle: FontStyle.italic,
                  ),
                ),
              ],
            ),
          ] else ...[
            SizedBox(
              width: double.infinity,
              height: 48,
              child: ElevatedButton.icon(
                onPressed: _runAutoSearch,
                icon: const Icon(Icons.search, size: 20),
                label: Text(
                  l.buttonAutoDetect,
                  style: TextStyle(
                    fontSize: AppSizes.fontLG(context),
                    fontWeight: FontWeight.bold,
                  ),
                ),
                style: ElevatedButton.styleFrom(
                  backgroundColor: AppColors.buttonBackground,
                  foregroundColor: AppColors.buttonText,
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(6),
                  ),
                ),
              ),
            ),
            const SizedBox(height: 12),
            SizedBox(
              width: double.infinity,
              height: 44,
              child: OutlinedButton(
                onPressed: _selectManually,
                style: OutlinedButton.styleFrom(
                  foregroundColor: AppColors.textSecondary,
                  side: const BorderSide(color: AppColors.borderMedium),
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(6),
                  ),
                ),
                child: Text(
                  l.buttonSelectManually,
                  style: TextStyle(fontSize: AppSizes.fontMD(context)),
                ),
              ),
            ),
          ],
          const SizedBox(height: 20),
          OnboardingBackButton(onTap: widget.onBack),
        ],
      ],
    );
  }
}

Future<bool> _validateNierExe(String filePath) async {
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
