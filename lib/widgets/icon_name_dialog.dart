import 'dart:io';

import 'package:flutter/material.dart';
import 'package:yp_launcher/widgets/app_dialog.dart';
import 'package:flutter/services.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/services/mods_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/app_dropdown.dart';

class IconTarget {
  final String modId;
  final String textureName;
  final String directory;

  const IconTarget({
    required this.modId,
    required this.textureName,
    required this.directory,
  });
}

class IconNameDialog extends StatefulWidget {
  final List<String> modIds;

  const IconNameDialog({super.key, required this.modIds});

  static Future<IconTarget?> show(
    BuildContext context, {
    required String gameDir,
  }) async {
    final mods = await _listMods(gameDir);
    if (!context.mounted) return null;

    final result = await showDialog<_Result>(
      context: context,
      builder: (_) => IconNameDialog(modIds: mods),
    );
    if (result == null) return null;

    final directory = p.join(
      ModsService.modsDir(gameDir),
      result.modId,
      'data',
      'misctex',
    );
    await Directory(directory).create(recursive: true);

    return IconTarget(
      modId: result.modId,
      textureName: result.textureName,
      directory: directory,
    );
  }

  static Future<List<String>> _listMods(String gameDir) async {
    final root = Directory(ModsService.modsDir(gameDir));
    if (!await root.exists()) return const [];

    final ids = <String>[];
    await for (final entry in root.list(followLinks: false)) {
      if (entry is Directory) ids.add(p.basename(entry.path));
    }
    ids.sort();
    return ids;
  }

  @override
  State<IconNameDialog> createState() => _IconNameDialogState();
}

class _Result {
  final String modId;
  final String textureName;

  const _Result(this.modId, this.textureName);
}

class _IconNameDialogState extends State<IconNameDialog> {
  final _nameController = TextEditingController();
  late String _modId = widget.modIds.isEmpty ? '' : widget.modIds.first;

  @override
  void dispose() {
    _nameController.dispose();
    super.dispose();
  }

  String get _name => _nameController.text.trim();

  bool get _valid => _modId.isNotEmpty && _name.isNotEmpty;

  void _confirm() {
    if (!_valid) return;
    Navigator.of(context).pop(_Result(_modId, _name));
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    return AppDialog(
      backgroundColor: AppColors.backgroundCard,
      title: Text(
        l10n.toolsIconsNewTitle,
        style: TextStyle(
          fontSize: AppSizes.fontMD(context),
          color: AppColors.textPrimary,
        ),
      ),
      content: SizedBox(
        width: 420,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              l10n.toolsIconsNewMod,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textSecondary,
              ),
            ),
            const SizedBox(height: 4),
            if (widget.modIds.isEmpty)
              Text(
                l10n.toolsIconsNewNoMods,
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  color: AppColors.warning,
                  height: 1.4,
                ),
              )
            else
              AppDropdown<String>(
                value: _modId,
                items: widget.modIds,
                itemLabel: (id) => id,
                onChanged: (id) => setState(() => _modId = id),
                maxWidth: 420,
                minWidth: 420,
              ),
            SizedBox(height: AppSizes.paddingMD(context)),
            Text(
              l10n.toolsIconsNewName,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textSecondary,
              ),
            ),
            const SizedBox(height: 4),
            Container(
              decoration: BoxDecoration(
                color: AppColors.inputBackground,
                borderRadius: BorderRadius.circular(6),
                border: Border.all(color: AppColors.borderMedium),
              ),
              child: TextField(
                controller: _nameController,
                autofocus: true,
                inputFormatters: [
                  FilteringTextInputFormatter.allow(RegExp(r'[a-zA-Z0-9_]')),
                ],
                onChanged: (_) => setState(() {}),
                onSubmitted: (_) => _confirm(),
                style: TextStyle(
                  fontSize: AppSizes.fontSM(context),
                  color: AppColors.textPrimary,
                  fontFamily: AppSizes.monoFamily,
                  fontFamilyFallback: AppSizes.monoFallback,
                ),
                decoration: InputDecoration(
                  isDense: true,
                  hintText: l10n.toolsIconsNewNameHint,
                  hintStyle: TextStyle(
                    fontSize: AppSizes.fontSM(context),
                    color: AppColors.textMuted.withValues(alpha: 0.5),
                  ),
                  border: InputBorder.none,
                  contentPadding: const EdgeInsets.symmetric(
                    horizontal: 10,
                    vertical: 8,
                  ),
                ),
              ),
            ),
            const SizedBox(height: 6),
            Text(
              _name.isEmpty
                  ? l10n.toolsIconsNewHint
                  : 'misctex_$_name.dat / .dtt',
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textMuted,
                fontFamily: _name.isEmpty ? null : AppSizes.monoFamily,
                fontFamilyFallback: _name.isEmpty ? null : AppSizes.monoFallback,
                height: 1.4,
              ),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: Text(
            l10n.cancelButton,
            style: TextStyle(color: AppColors.textMuted),
          ),
        ),
        TextButton(
          onPressed: _valid ? _confirm : null,
          child: Text(
            l10n.toolsIconsNewCreate,
            style: TextStyle(
              color: _valid ? AppColors.accentPrimary : AppColors.textMuted,
              fontWeight: FontWeight.bold,
            ),
          ),
        ),
      ],
    );
  }
}
