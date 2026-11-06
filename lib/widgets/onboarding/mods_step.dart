import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:file_picker/file_picker.dart';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/providers/mods_state.dart';
import 'package:yp_launcher/services/archive_service.dart';
import 'package:yp_launcher/services/mods_service.dart';
import 'package:yp_launcher/services/texture_install_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/mods/mod_drop_zone.dart';
import 'package:yp_launcher/widgets/mods/mod_naming.dart';
import 'package:yp_launcher/widgets/mods/mod_variant_dialog.dart';
import 'package:yp_launcher/widgets/mods/outfit_equip_guide.dart';
import 'package:yp_launcher/widgets/onboarding/shared.dart';
import 'package:yp_launcher/widgets/textures/texture_pack_install.dart';

class ModsStep extends ConsumerStatefulWidget {
  final String? gameDir;
  final VoidCallback onNext;
  final VoidCallback onBack;
  const ModsStep({
    super.key,
    required this.gameDir,
    required this.onNext,
    required this.onBack,
  });

  @override
  ConsumerState<ModsStep> createState() => _ModsStepState();
}

class _ModsStepState extends ConsumerState<ModsStep> {
  bool? _wantsModInstall;
  bool _busy = false;
  String _busyMessage = '';
  String? _error;
  String? _warning;
  final List<String> _installedNames = [];

  @override
  Widget build(BuildContext context) {
    if (_wantsModInstall == true) return _buildInstall();
    return _buildAsk();
  }

  Widget _buildAsk() {
    final l = AppLocalizations.of(context)!;
    return OnboardingStepCard(
      children: [
        Text(
          l.onboardingModInstallAskTitle,
          style: TextStyle(
            fontSize: AppSizes.fontXL(context),
            fontWeight: FontWeight.bold,
            color: AppColors.accentPrimary,
          ),
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 12),
        Text(
          l.onboardingModInstallAskBody,
          style: TextStyle(
            fontSize: AppSizes.fontSM(context),
            color: AppColors.textSecondary,
            height: 1.5,
          ),
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 28),
        OnboardingChoiceButton(
          label: l.onboardingModInstallYes,
          selected: false,
          onTap: () => setState(() => _wantsModInstall = true),
        ),
        const SizedBox(height: 10),
        OnboardingChoiceButton(
          label: l.onboardingModInstallSkip,
          selected: false,
          onTap: () {
            setState(() => _wantsModInstall = false);
            widget.onNext();
          },
        ),
        const SizedBox(height: 28),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [OnboardingBackButton(onTap: widget.onBack)],
        ),
      ],
    );
  }

  Widget _buildInstall() {
    final l = AppLocalizations.of(context)!;
    return OnboardingStepCard(
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.download_for_offline_outlined,
              size: 22,
              color: AppColors.accentPrimary,
            ),
            const SizedBox(width: 8),
            Text(
              l.onboardingModInstallTitle,
              style: TextStyle(
                fontSize: AppSizes.fontLG(context),
                fontWeight: FontWeight.bold,
                color: AppColors.accentPrimary,
                letterSpacing: 0.4,
              ),
            ),
          ],
        ),
        const SizedBox(height: 4),
        Text(
          l.onboardingModInstallBody,
          style: TextStyle(
            fontSize: AppSizes.fontSM(context),
            color: AppColors.textSecondary,
          ),
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 2),
        Text(
          l.onboardingModInstallSubBody,
          style: TextStyle(
            fontSize: AppSizes.fontXS(context),
            color: AppColors.textMuted,
            fontStyle: FontStyle.italic,
          ),
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 16),
        if (_busy)
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 24),
            child: Column(
              children: [
                CircularProgressIndicator(color: AppColors.accentPrimary),
                const SizedBox(height: 10),
                Text(
                  _busyMessage.isEmpty
                      ? l.onboardingModInstallBusy
                      : _busyMessage,
                  style: TextStyle(
                    fontSize: AppSizes.fontSM(context),
                    color: AppColors.textSecondary,
                  ),
                ),
              ],
            ),
          )
        else
          ModDropZone(onDrop: _installAll, onBrowse: _pickAndInstall),
        if (_error != null) ...[
          const SizedBox(height: 10),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
            decoration: BoxDecoration(
              color: AppColors.error.withValues(alpha: 0.10),
              border: Border.all(color: AppColors.error.withValues(alpha: 0.5)),
              borderRadius: BorderRadius.circular(6),
            ),
            child: Row(
              children: [
                Icon(Icons.error_outline, size: 16, color: AppColors.error),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    _error!,
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      color: AppColors.error,
                      height: 1.35,
                    ),
                  ),
                ),
              ],
            ),
          ),
        ],
        if (_warning != null) ...[
          const SizedBox(height: 10),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
            decoration: BoxDecoration(
              color: AppColors.warning.withValues(alpha: 0.10),
              border: Border.all(
                color: AppColors.warning.withValues(alpha: 0.5),
              ),
              borderRadius: BorderRadius.circular(6),
            ),
            child: Row(
              children: [
                Icon(Icons.warning_amber, size: 16, color: AppColors.warning),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    _warning!,
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      color: AppColors.warning,
                      height: 1.35,
                    ),
                  ),
                ),
              ],
            ),
          ),
        ],
        if (_installedNames.isNotEmpty) ...[
          const SizedBox(height: 12),
          Wrap(
            spacing: 6,
            runSpacing: 6,
            alignment: WrapAlignment.center,
            children: [
              for (final name in _installedNames)
                Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 10,
                    vertical: 5,
                  ),
                  decoration: BoxDecoration(
                    color: AppColors.success.withValues(alpha: 0.10),
                    border: Border.all(
                      color: AppColors.success.withValues(alpha: 0.45),
                    ),
                    borderRadius: BorderRadius.circular(20),
                  ),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(
                        Icons.check_circle,
                        size: 13,
                        color: AppColors.success,
                      ),
                      const SizedBox(width: 6),
                      ConstrainedBox(
                        constraints: const BoxConstraints(maxWidth: 220),
                        child: Text(
                          name,
                          overflow: TextOverflow.ellipsis,
                          style: TextStyle(
                            fontSize: AppSizes.fontXS(context),
                            color: AppColors.textPrimary,
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
            ],
          ),
        ],
        const SizedBox(height: 20),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            OnboardingBackButton(onTap: _busy ? null : widget.onBack),
            const SizedBox(width: 12),
            SizedBox(
              width: 200,
              height: 48,
              child: ElevatedButton(
                onPressed: _busy ? null : widget.onNext,
                style: ElevatedButton.styleFrom(
                  backgroundColor: AppColors.buttonBackground,
                  foregroundColor: AppColors.buttonText,
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(6),
                  ),
                ),
                child: Text(
                  _installedNames.isEmpty
                      ? l.onboardingModInstallSkipButton
                      : l.onboardingModInstallDoneButton,
                  style: TextStyle(
                    fontSize: AppSizes.fontLG(context),
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ),
            ),
          ],
        ),
      ],
    );
  }

  Future<void> _pickAndInstall() async {
    final l = AppLocalizations.of(context)!;
    final result = await FilePicker.pickFiles(
      dialogTitle: l.onboardingModInstallPickerTitle,
      type: FileType.custom,
      allowedExtensions: const ['zip', '7z'],
    );
    if (result == null || result.files.isEmpty) {
      if (!mounted) return;
      final dir = await FilePicker.getDirectoryPath(
        dialogTitle: l.onboardingModInstallFolderPickerTitle,
      );
      if (dir == null) return;
      await _installAll([dir]);
      return;
    }
    final filePath = result.files.first.path;
    if (filePath == null) return;
    await _installAll([filePath]);
  }

  Future<void> _installAll(List<String> paths) =>
      OutfitEquipGuide.watchInstall(context, widget.gameDir, () async {
        for (final path in paths) {
          if (!mounted) return;
          await _install(path);
        }
      });

  Future<void> _install(String sourcePath) async {
    final l = AppLocalizations.of(context)!;
    if (widget.gameDir == null) {
      setState(() => _error = l.onboardingModInstallNoGameDir);
      return;
    }
    setState(() {
      _busy = true;
      _busyMessage = l.onboardingModInstallInspecting;
      _error = null;
    });

    final detect = await ModsService.detectDrop(
      sourcePath,
      onExtractProgress: (percent, currentFile) {
        if (!mounted) return;
        setState(() {
          _busyMessage = currentFile == null || currentFile.isEmpty
              ? l.onboardingModInstallExtractingPercent((percent * 100).round())
              : l.onboardingModInstallExtractingFile(
                  (percent * 100).round(),
                  currentFile,
                );
        });
      },
    );
    if (!mounted) return;

    if (detect.hasVariants) {
      setState(() => _busy = false);
      await _installVariants(sourcePath, detect, l);
      return;
    }

    if (detect.kind == ModKind.unknown) {
      if (detect.errorReason == 'texture_only') {
        await _installTexturePack(sourcePath, l);
        return;
      }
      setState(() {
        _busy = false;
        _error = l.onboardingModInstallNotAMod;
      });
      return;
    }

    final manifestId = detect.manifest?.id?.trim();
    String? requestedName;
    if (manifestId != null && manifestId.isNotEmpty) {
      requestedName = manifestId;
      setState(() {
        _busyMessage = l.onboardingModInstallBusy;
      });
    } else {
      setState(() => _busy = false);
      requestedName = await showModNamingDialog(
        context,
        initial: prettifyModId(detect.suggestedId),
      );
      if (!mounted) return;
      if (requestedName == null) return;
      setState(() {
        _busy = true;
        _busyMessage = l.onboardingModInstallBusy;
      });
    }

    final result = await ModsService.install(
      widget.gameDir!,
      sourcePath,
      requestedName: requestedName,
      onExtractProgress: (percent, currentFile) {
        if (!mounted) return;
        setState(() {
          _busyMessage = currentFile == null || currentFile.isEmpty
              ? l.onboardingModInstallExtractingPercent((percent * 100).round())
              : l.onboardingModInstallExtractingFile(
                  (percent * 100).round(),
                  currentFile,
                );
        });
      },
    );
    if (!mounted) return;

    if (!result.success) {
      setState(() {
        _busy = false;
        _error = l.onboardingModInstallFailed(
          result.errorMessage ?? 'unknown error',
        );
      });
      return;
    }

    final displayNameRaw = detect.manifest?.displayName?.trim();
    final displayName = displayNameRaw != null && displayNameRaw.isNotEmpty
        ? displayNameRaw
        : (result.installedId ?? 'Unnamed mod');

    setState(() {
      _busy = false;
      _error = null;
      _warning = result.unpairedWarnings.isNotEmpty
          ? l.modLooseUnpairedWarning(result.unpairedWarnings.join(', '))
          : null;
      _installedNames.add(displayName);
    });

    ref.read(modsStateControllerProvider.notifier).loadMods(widget.gameDir!);
  }

  Future<void> _installTexturePack(
    String sourcePath,
    AppLocalizations l,
  ) async {
    final name = ArchiveService.isArchive(sourcePath)
        ? path.basenameWithoutExtension(sourcePath)
        : path.basename(sourcePath);
    int? count;
    try {
      count = await TexturePackInstall.run(
        context,
        paths: [sourcePath],
        textureDir: textureInjectDir(widget.gameDir!),
        onExtractProgress: (percent, currentFile) {
          if (!mounted) return;
          setState(() {
            _busyMessage = currentFile == null || currentFile.isEmpty
                ? l.onboardingModInstallExtractingPercent(
                    (percent * 100).round(),
                  )
                : l.onboardingModInstallExtractingFile(
                    (percent * 100).round(),
                    currentFile,
                  );
          });
        },
        onCopyStart: () {
          if (mounted) setState(() => _busyMessage = l.texturesProgressCopying);
        },
      );
    } catch (e) {
      if (mounted) {
        setState(() {
          _busy = false;
          _error = l.onboardingModInstallFailed('$e');
        });
      }
      return;
    }
    if (!mounted) return;
    setState(() {
      _busy = false;
      if (count == null) return;
      if (count > 0) {
        _installedNames.add(l.onboardingTexturePackChip(name));
      } else {
        _warning = l.noTextureFilesFound;
      }
    });
  }

  Future<void> _installVariants(
    String sourcePath,
    DetectedDrop detect,
    AppLocalizations l,
  ) async {
    final chosen = await showModVariantDialog(
      context,
      variants: detect.variants,
    );
    if (chosen == null || chosen.isEmpty || !mounted) return;

    final base = await showModNamingDialog(
      context,
      initial: prettifyModId(detect.suggestedId),
    );
    if (base == null || !mounted) return;

    for (final variant in chosen) {
      if (!mounted) return;
      setState(() {
        _busy = true;
        _busyMessage = l.onboardingModInstallBusy;
      });
      var name = '$base - ${variant.label}';
      InstallResult result;
      var attempt = 1;
      while (true) {
        result = await ModsService.install(
          widget.gameDir!,
          sourcePath,
          requestedName: name,
          variantSubPath: variant.subPath,
        );
        if (result.success ||
            result.errorMessage?.startsWith('exists:') != true) {
          break;
        }
        attempt++;
        name = '$base - ${variant.label} $attempt';
      }
      if (!mounted) return;
      if (result.success) {
        setState(() {
          _installedNames.add(name);
          if (result.unpairedWarnings.isNotEmpty) {
            _warning = l.modLooseUnpairedWarning(
              result.unpairedWarnings.join(', '),
            );
          }
        });
      } else {
        setState(
          () => _error = l.onboardingModInstallFailed(
            result.errorMessage ?? 'unknown error',
          ),
        );
      }
    }

    if (!mounted) return;
    setState(() => _busy = false);
    ref.read(modsStateControllerProvider.notifier).loadMods(widget.gameDir!);
  }
}
