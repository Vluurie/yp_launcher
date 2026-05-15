import 'package:flutter/material.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/cutscenes/cutscene_isolates.dart';

Future<String?> showCutsceneMergeDialog(
  BuildContext context,
  List<CutsceneMod> mods,
) {
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
                l10n.cutsceneMergeTitle,
                style: TextStyle(fontSize: AppSizes.fontXL(ctx), color: AppColors.accentPrimary, fontWeight: FontWeight.bold),
              ),
              SizedBox(height: AppSizes.spacingLG(ctx)),
              Text(
                l10n.cutsceneMergeDescription,
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
                        ...mods.map((mod) => Padding(
                          padding: EdgeInsets.only(bottom: AppSizes.spacingSM(ctx)),
                          child: OutlinedButton.icon(
                            onPressed: () => Navigator.of(ctx).pop(mod.name),
                            icon: const Icon(Icons.merge_type, size: 16),
                            label: Text(l10n.cutsceneMergeAddTo(mod.name)),
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
                  label: Text(l10n.cutsceneMergeNewMod),
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

Future<String?> showCutsceneNamingDialog(
  BuildContext context,
  String defaultName,
) async {
  final truncated = defaultName.length > maxModNameLength
      ? defaultName.substring(0, maxModNameLength)
      : defaultName;
  final controller = TextEditingController(text: truncated);
  return showDialog<String>(
    context: context,
    builder: (ctx) => AlertDialog(
      backgroundColor: AppColors.backgroundCard,
      title: Text(
        AppLocalizations.of(ctx)!.cutsceneNamingTitle,
        style: TextStyle(
          fontSize: AppSizes.fontXL(ctx),
          color: AppColors.textPrimary,
        ),
      ),
      content: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            AppLocalizations.of(ctx)!.cutsceneNamingHint(maxModNameLength),
            style: TextStyle(
              fontSize: AppSizes.fontSM(ctx),
              color: AppColors.textSecondary,
            ),
          ),
          const SizedBox(height: 12),
          TextField(
            controller: controller,
            autofocus: true,
            maxLength: maxModNameLength,
            style: TextStyle(
              fontSize: AppSizes.fontMD(ctx),
              color: AppColors.textPrimary,
            ),
            decoration: InputDecoration(
              isDense: true,
              contentPadding: EdgeInsets.symmetric(
                horizontal: AppSizes.paddingSM(ctx),
                vertical: AppSizes.paddingSM(ctx),
              ),
              filled: true,
              fillColor: AppColors.inputBackground,
              border: OutlineInputBorder(
                borderRadius: BorderRadius.circular(
                  AppSizes.borderRadius(ctx),
                ),
                borderSide: const BorderSide(color: AppColors.borderMedium),
              ),
              enabledBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(
                  AppSizes.borderRadius(ctx),
                ),
                borderSide: const BorderSide(color: AppColors.borderMedium),
              ),
              focusedBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(
                  AppSizes.borderRadius(ctx),
                ),
                borderSide: const BorderSide(color: AppColors.accentPrimary),
              ),
              counterStyle: const TextStyle(color: AppColors.textMuted),
            ),
            onSubmitted: (v) => Navigator.of(ctx).pop(v),
          ),
        ],
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(ctx).pop(null),
          child: Text(
            AppLocalizations.of(ctx)!.buttonCancel,
            style: const TextStyle(color: AppColors.textMuted),
          ),
        ),
        TextButton(
          onPressed: () => Navigator.of(ctx).pop(controller.text),
          child: Text(
            AppLocalizations.of(ctx)!.buttonInstall,
            style: const TextStyle(
              color: AppColors.accentPrimary,
              fontWeight: FontWeight.bold,
            ),
          ),
        ),
      ],
    ),
  );
}

Future<bool> showDeleteCutsceneConfirmDialog(
  BuildContext context,
  String modName,
) async {
  final result = await showDialog<bool>(
    context: context,
    builder: (ctx) => AlertDialog(
      backgroundColor: AppColors.backgroundCard,
      title: Text(
        AppLocalizations.of(ctx)!.removeCutsceneModTitle,
        style: TextStyle(
          fontSize: AppSizes.fontXL(ctx),
          color: AppColors.textPrimary,
        ),
      ),
      content: Text(
        AppLocalizations.of(ctx)!.deleteCutsceneConfirm(modName),
        style: TextStyle(
          fontSize: AppSizes.fontMD(ctx),
          color: AppColors.textSecondary,
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(ctx).pop(false),
          child: Text(
            AppLocalizations.of(ctx)!.buttonCancel,
            style: const TextStyle(color: AppColors.textMuted),
          ),
        ),
        TextButton(
          onPressed: () => Navigator.of(ctx).pop(true),
          child: Text(
            AppLocalizations.of(ctx)!.buttonDelete,
            style: const TextStyle(
              color: AppColors.error,
              fontWeight: FontWeight.bold,
            ),
          ),
        ),
      ],
    ),
  );
  return result == true;
}
