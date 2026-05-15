import 'package:flutter/material.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

Future<bool?> showTextureCrossInstallDialog({
  required BuildContext context,
  required String message,
}) {
  final l10n = AppLocalizations.of(context)!;
  return showDialog<bool>(
    context: context,
    builder: (ctx) => AlertDialog(
      backgroundColor: AppColors.backgroundCard,
      title: Text(
        l10n.mixedModDetected,
        style: TextStyle(
          color: AppColors.accentPrimary,
          fontSize: AppSizes.fontLG(ctx),
        ),
      ),
      content: Text(
        message,
        style: TextStyle(
          color: AppColors.textPrimary,
          fontSize: AppSizes.fontMD(ctx),
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(ctx).pop(false),
          child: Text(
            l10n.buttonNo,
            style: const TextStyle(color: AppColors.textMuted),
          ),
        ),
        TextButton(
          onPressed: () => Navigator.of(ctx).pop(true),
          child: Text(
            l10n.buttonYes,
            style: const TextStyle(color: AppColors.accentPrimary),
          ),
        ),
      ],
    ),
  );
}

Future<String?> showTextureNamingDialog({
  required BuildContext context,
  required String defaultName,
  required String character,
}) {
  final l10n = AppLocalizations.of(context)!;
  final controller = TextEditingController(text: defaultName);
  return showDialog<String>(
    context: context,
    builder: (ctx) => AlertDialog(
      backgroundColor: AppColors.backgroundCard,
      title: Text(
        l10n.nameOutfitTitle(character),
        style: TextStyle(
          color: AppColors.accentPrimary,
          fontSize: AppSizes.fontLG(ctx),
        ),
      ),
      content: TextField(
        controller: controller,
        autofocus: true,
        style: const TextStyle(color: AppColors.textPrimary),
        decoration: InputDecoration(
          hintText: l10n.outfitNameHint,
          hintStyle: const TextStyle(color: AppColors.textMuted),
          enabledBorder: UnderlineInputBorder(
            borderSide: BorderSide(color: AppColors.borderLight),
          ),
          focusedBorder: const UnderlineInputBorder(
            borderSide: BorderSide(color: AppColors.accentPrimary),
          ),
        ),
        onSubmitted: (v) => Navigator.of(ctx).pop(v),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(ctx).pop(null),
          child: Text(
            l10n.buttonCancel,
            style: const TextStyle(color: AppColors.textMuted),
          ),
        ),
        TextButton(
          onPressed: () => Navigator.of(ctx).pop(controller.text),
          child: Text(
            l10n.buttonInstall,
            style: const TextStyle(color: AppColors.accentPrimary),
          ),
        ),
      ],
    ),
  );
}

Future<String?> showTextureMergeDialog({
  required BuildContext context,
  required List<String> detectedFolders,
}) {
  final l10n = AppLocalizations.of(context)!;
  return showDialog<String>(
    context: context,
    builder: (ctx) => Dialog(
      backgroundColor: AppColors.backgroundCard,
      insetPadding: const EdgeInsets.all(40),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
      child: ConstrainedBox(
        constraints: BoxConstraints(
          maxWidth: 420,
          maxHeight: MediaQuery.of(ctx).size.height - 120,
        ),
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                l10n.textureMergeTitle,
                style: TextStyle(fontSize: AppSizes.fontXL(ctx), color: AppColors.accentPrimary, fontWeight: FontWeight.bold),
              ),
              SizedBox(height: AppSizes.spacingLG(ctx)),
              Text(
                l10n.textureMergeDescription,
                style: TextStyle(fontSize: AppSizes.fontSM(ctx), color: AppColors.textSecondary),
              ),
              SizedBox(height: AppSizes.paddingLG(ctx)),
              Flexible(
                child: Scrollbar(
                  thumbVisibility: true,
                  child: SingleChildScrollView(
                    padding: const EdgeInsets.only(right: 12),
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: [
                        ...detectedFolders.map((folder) => Padding(
                          padding: EdgeInsets.only(bottom: AppSizes.spacingSM(ctx)),
                          child: OutlinedButton.icon(
                            onPressed: () => Navigator.of(ctx).pop(folder),
                            icon: const Icon(Icons.merge_type, size: 16),
                            label: Text(l10n.textureMergeAddTo(folder)),
                            style: OutlinedButton.styleFrom(
                              foregroundColor: AppColors.accentPrimary,
                              side: const BorderSide(color: AppColors.accentPrimary),
                              padding: EdgeInsets.symmetric(vertical: AppSizes.paddingMD(ctx), horizontal: AppSizes.paddingLG(ctx)),
                              alignment: Alignment.centerLeft,
                            ),
                          ),
                        )),
                      ],
                    ),
                  ),
                ),
              ),
              SizedBox(height: AppSizes.spacingSM(ctx)),
              SizedBox(
                width: double.infinity,
                child: ElevatedButton.icon(
                  onPressed: () => Navigator.of(ctx).pop(''),
                  icon: const Icon(Icons.add, size: 16),
                  label: Text(l10n.textureMergeNewMod),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: AppColors.surfaceLight,
                    foregroundColor: AppColors.textPrimary,
                    padding: EdgeInsets.symmetric(vertical: AppSizes.paddingMD(ctx), horizontal: AppSizes.paddingLG(ctx)),
                    alignment: Alignment.centerLeft,
                  ),
                ),
              ),
              SizedBox(height: AppSizes.paddingLG(ctx)),
              Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  TextButton(
                    onPressed: () => Navigator.of(ctx).pop(null),
                    child: Text(l10n.cancelButton, style: const TextStyle(color: AppColors.textMuted)),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    ),
  );
}

