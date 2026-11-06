import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/providers/nams_settings_state.dart';
import 'package:yp_launcher/services/mods_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/theme/nier_curves.dart';
import 'package:yp_launcher/widgets/app_dialog.dart';

abstract final class OutfitEquipGuide {
  static bool isOutfitMod(InstalledMod mod) {
    final data = mod.data;
    if (data == null) return false;
    return data.players.isNotEmpty ||
        data.entries.any((e) => e.dirName == 'pl');
  }

  static Future<void> watchInstall(
    BuildContext context,
    String? gameDir,
    Future<void> Function() install,
  ) async {
    if (gameDir == null || gameDir.isEmpty || await _wasShown()) {
      await install();
      return;
    }
    final before = {for (final m in await _listQuiet(gameDir)) m.id};
    await install();
    if (!context.mounted) return;
    final added = [
      for (final m in await _listQuiet(gameDir))
        if (!before.contains(m.id) && isOutfitMod(m)) m,
    ];
    if (added.isEmpty || !context.mounted) return;
    if (!await _markShown() || !context.mounted) return;
    await showDialog<void>(
      context: context,
      builder: (_) => _OutfitEquipDialog(mod: added.first),
    );
  }

  static Future<List<InstalledMod>> _listQuiet(String gameDir) async {
    try {
      return await ModsService.listInstalled(gameDir);
    } catch (_) {
      return const [];
    }
  }

  static Future<bool> _wasShown() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      return prefs.getBool(AppStrings.prefKeyOutfitGuideShown) == true;
    } catch (_) {
      return true;
    }
  }

  static Future<bool> _markShown() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      return await prefs.setBool(AppStrings.prefKeyOutfitGuideShown, true);
    } catch (_) {
      return false;
    }
  }
}

class _OutfitEquipDialog extends ConsumerWidget {
  final InstalledMod mod;

  const _OutfitEquipDialog({required this.mod});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l = AppLocalizations.of(context)!;
    final key = YpKeybindFields.openWorkspace.valueIn(
      ref.read(namsSettingsStateControllerProvider).keybinds,
    );
    return AppDialog(
      backgroundColor: AppColors.surfaceMedium,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        side: BorderSide(color: AppColors.borderLight),
      ),
      title: Row(
        children: [
          Icon(Icons.checkroom, size: 20, color: AppColors.accentPrimary),
          SizedBox(width: AppSizes.spacingSM(context)),
          Flexible(
            child: Text(
              l.outfitGuideTitle,
              style: TextStyle(
                fontSize: AppSizes.fontMD(context),
                fontWeight: FontWeight.bold,
                color: AppColors.textPrimary,
              ),
            ),
          ),
        ],
      ),
      content: SizedBox(
        width: (MediaQuery.of(context).size.width * 0.5).clamp(380.0, 560.0),
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(l.outfitGuideIntro(mod.displayName)),
              SizedBox(height: AppSizes.spacingMD(context)),
              _Way(
                icon: Icons.dashboard_customize_outlined,
                title: l.outfitGuideWardrobeTitle,
                body: l.outfitGuideWardrobeBody(key),
              ),
              _Way(
                icon: Icons.person_pin_circle_outlined,
                title: l.outfitGuideNpcTitle,
                body: l.outfitGuideNpcBody,
                trailing: _LinkChip(
                  label: l.outfitGuideNpcLink,
                  url: AppStrings.multiOutfitNpcUrl,
                ),
              ),
              _Way(
                icon: Icons.star_outline,
                title: l.outfitGuideDefaultTitle,
                body: l.outfitGuideDefaultBody,
              ),
              if (mod.data?.hasCompatConfig == true)
                Padding(
                  padding: EdgeInsets.only(top: AppSizes.spacingXS(context)),
                  child: Text(
                    l.outfitGuideCompatNote,
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      color: AppColors.textMuted,
                      fontStyle: FontStyle.italic,
                    ),
                  ),
                ),
            ],
          ),
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: Text(
            l.outfitGuideDone,
            style: TextStyle(color: AppColors.accentSecondary),
          ),
        ),
      ],
    );
  }
}

class _Way extends StatelessWidget {
  final IconData icon;
  final String title;
  final String body;
  final Widget? trailing;

  const _Way({
    required this.icon,
    required this.title,
    required this.body,
    this.trailing,
  });

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: EdgeInsets.only(bottom: AppSizes.spacingMD(context)),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.only(top: 1),
            child: Icon(icon, size: 18, color: AppColors.accentPrimary),
          ),
          SizedBox(width: AppSizes.spacingSM(context)),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  title,
                  style: TextStyle(
                    fontSize: AppSizes.fontSM(context),
                    fontWeight: FontWeight.w700,
                    color: AppColors.textPrimary,
                  ),
                ),
                const SizedBox(height: 2),
                Text(body),
                if (trailing != null) ...[const SizedBox(height: 6), trailing!],
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _LinkChip extends StatefulWidget {
  final String label;
  final String url;

  const _LinkChip({required this.label, required this.url});

  @override
  State<_LinkChip> createState() => _LinkChipState();
}

class _LinkChipState extends State<_LinkChip> {
  bool _hovered = false;

  Future<void> _open() async {
    try {
      final uri = Uri.parse(widget.url);
      if (await canLaunchUrl(uri)) {
        await launchUrl(uri, mode: LaunchMode.externalApplication);
      }
    } catch (_) {}
  }

  @override
  Widget build(BuildContext context) {
    final color = AppColors.accentPrimary;
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: _open,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          curve: NierCurves.smooth,
          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 3),
          decoration: BoxDecoration(
            color: _hovered
                ? color.withValues(alpha: 0.16)
                : Colors.transparent,
            border: Border.all(
              color: color.withValues(alpha: _hovered ? 0.6 : 0.3),
            ),
            borderRadius: BorderRadius.circular(4),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(Icons.open_in_new, size: 11, color: color),
              const SizedBox(width: 3),
              Text(
                widget.label,
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  color: color,
                  fontWeight: FontWeight.w600,
                  decoration: _hovered
                      ? TextDecoration.underline
                      : TextDecoration.none,
                  decorationColor: color,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
