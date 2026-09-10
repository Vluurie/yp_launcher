import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/l10n/app_localizations_en.dart';
import 'package:yp_launcher/providers/locale_state.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/hover_button.dart';

class LanguageSelector extends ConsumerWidget {
  const LanguageSelector({super.key, this.menuKey});

  final GlobalKey<PopupMenuButtonState<Locale>>? menuKey;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final current =
        ref.watch(localeControllerProvider) ?? Localizations.localeOf(context);
    final effectiveKey = menuKey ?? GlobalKey<PopupMenuButtonState<Locale>>();
    return PopupMenuButton<Locale>(
      key: effectiveKey,
      tooltip: '',
      color: AppColors.backgroundCard,
      position: PopupMenuPosition.under,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        side: BorderSide(color: AppColors.borderLight),
      ),
      onSelected: (locale) =>
          ref.read(localeControllerProvider.notifier).setLocale(locale),
      itemBuilder: (context) => [
        for (final locale in kSupportedLocales)
          PopupMenuItem<Locale>(
            value: locale,
            child: Row(
              children: [
                Icon(
                  locale.languageCode == current.languageCode
                      ? Icons.check
                      : Icons.language,
                  size: AppSizes.iconSM(context),
                  color: locale.languageCode == current.languageCode
                      ? AppColors.accentPrimary
                      : AppColors.textSecondary,
                ),
                SizedBox(width: AppSizes.spacingSM(context)),
                Text(
                  localeDisplayName(locale),
                  style: TextStyle(
                    color: locale.languageCode == current.languageCode
                        ? AppColors.accentPrimary
                        : AppColors.textPrimary,
                  ),
                ),
              ],
            ),
          ),
        const PopupMenuDivider(),
        PopupMenuItem<Locale>(
          enabled: false,
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 240),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Icon(
                      Icons.info_outline,
                      size: AppSizes.iconSM(context),
                      color: AppColors.warning,
                    ),
                    SizedBox(width: AppSizes.spacingSM(context)),
                    Expanded(
                      child: Text(
                        l10n.languageSupportNotice,
                        style: TextStyle(
                          fontSize: AppSizes.fontXS(context),
                          color: AppColors.textMuted,
                          height: 1.35,
                        ),
                      ),
                    ),
                  ],
                ),
                if (current.languageCode != 'en') ...[
                  SizedBox(height: AppSizes.spacingSM(context) / 2),
                  Padding(
                    padding: EdgeInsets.only(
                      left:
                          AppSizes.iconSM(context) +
                          AppSizes.spacingSM(context),
                    ),
                    child: Text(
                      AppLocalizationsEn().languageSupportNotice,
                      style: TextStyle(
                        fontSize: AppSizes.fontXS(context),
                        color: AppColors.textMuted.withValues(alpha: 0.7),
                        height: 1.35,
                        fontStyle: FontStyle.italic,
                      ),
                    ),
                  ),
                ],
              ],
            ),
          ),
        ),
      ],
      child: HoverIconButton(
        tooltip: l10n.tooltipLanguage,
        bordered: false,
        padding: EdgeInsets.all(AppSizes.paddingXS(context)),
        radius: AppSizes.borderRadius(context),
        onTap: () => effectiveKey.currentState?.showButtonMenu(),
        icon: Icon(
          Icons.language,
          size: AppSizes.iconMD(context),
          color: AppColors.accentPrimary,
        ),
      ),
    );
  }
}
