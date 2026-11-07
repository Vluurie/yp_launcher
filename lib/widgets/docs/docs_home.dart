import 'package:flutter/material.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/services/docs/docs_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class DocsHome extends StatelessWidget {
  final List<DocPage> pages;
  final void Function(String fileName) onOpenFile;
  final VoidCallback onBrowseAll;

  const DocsHome({
    super.key,
    required this.pages,
    required this.onOpenFile,
    required this.onBrowseAll,
  });

  DocPage? _find(String fileName) {
    for (final page in pages) {
      if (page.path.endsWith('/$fileName') && !page.isResource) return page;
    }
    for (final page in pages) {
      if (page.path.endsWith('/$fileName')) return page;
    }
    return null;
  }

  String _titleOf(String fileName, String fallback) =>
      _find(fileName)?.title ?? fallback;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final guides = pages.where((p) => !p.isResource).length;

    return SingleChildScrollView(
      padding: EdgeInsets.all(AppSizes.contentPadding(context)),
      child: Align(
        alignment: Alignment.topCenter,
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 1000),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _hero(context, l10n),
              SizedBox(height: AppSizes.paddingLG(context)),
              _pathCards(context, l10n),
              SizedBox(height: AppSizes.paddingXL(context)),
              _categoryBlock(
                context,
                l10n.docsCategoryNative,
                l10n.docsNativeBlurb,
                Icons.inventory_2_outlined,
                _nativeEntries,
              ),
              SizedBox(height: AppSizes.paddingXL(context)),
              _categoryBlock(
                context,
                l10n.docsCategoryScripting,
                l10n.docsScriptingBlurb,
                Icons.movie_filter_outlined,
                _scriptingEntries,
              ),
              SizedBox(height: AppSizes.paddingXL(context)),
              _referenceCard(context, l10n),
              SizedBox(height: AppSizes.paddingLG(context)),
              _browseAll(context, l10n, guides),
            ],
          ),
        ),
      ),
    );
  }

  Widget _hero(BuildContext context, AppLocalizations l10n) {
    return Container(
      width: double.infinity,
      padding: EdgeInsets.all(AppSizes.paddingXL(context)),
      decoration: BoxDecoration(
        gradient: LinearGradient(
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
          colors: [
            AppColors.accentPrimary.withValues(
              alpha: AppColors.isLight ? 0.30 : 0.16,
            ),
            AppColors.surfaceMedium.withValues(
              alpha: AppColors.isLight ? 0.75 : 0.4,
            ),
          ],
        ),
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context) * 2),
        border: Border.all(
          color: AppColors.accentPrimary.withValues(alpha: 0.35),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            l10n.docsHeroTitle,
            style: TextStyle(
              fontSize: AppSizes.fontXL(context),
              fontWeight: FontWeight.bold,
              color: AppColors.textPrimary,
              letterSpacing: 0.5,
            ),
          ),
          SizedBox(height: AppSizes.spacingMD(context)),
          ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 620),
            child: Text(
              l10n.docsHeroBody,
              style: TextStyle(
                fontSize: AppSizes.fontSM(context),
                color: AppColors.textSecondary,
                height: 1.6,
              ),
            ),
          ),
          SizedBox(height: AppSizes.paddingMD(context)),
          _HeroButton(
            label: l10n.docsHeroCta,
            badge: l10n.docsHeroTime,
            onTap: () => onOpenFile('01_first_mod.md'),
          ),
          if (Localizations.localeOf(context).languageCode != 'en') ...[
            SizedBox(height: AppSizes.paddingMD(context)),
            Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(
                  Icons.info_outline,
                  size: AppSizes.iconSM(context),
                  color: AppColors.textMuted,
                ),
                SizedBox(width: AppSizes.spacingSM(context)),
                Text(
                  l10n.docsEnglishOnly,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.textMuted,
                  ),
                ),
              ],
            ),
          ],
        ],
      ),
    );
  }

  Widget _pathCards(BuildContext context, AppLocalizations l10n) {
    final cards = [
      _PathCard(
        icon: Icons.rocket_launch_outlined,
        title: l10n.docsPathBeginner,
        body: l10n.docsPathBeginnerBody,
        onTap: () => onOpenFile('01_first_mod.md'),
      ),
      _PathCard(
        icon: Icons.inventory_2_outlined,
        title: l10n.docsPathContent,
        body: l10n.docsPathContentBody,
        onTap: () => onOpenFile('README.md'),
      ),
      _PathCard(
        icon: Icons.movie_filter_outlined,
        title: l10n.docsPathScripting,
        body: l10n.docsPathScriptingBody,
        onTap: () => onOpenFile('scripting/README.md'),
      ),
    ];

    return LayoutBuilder(
      builder: (context, constraints) {
        final stacked = constraints.maxWidth < 680;
        if (stacked) {
          return Column(
            children: [
              for (final card in cards)
                Padding(
                  padding: EdgeInsets.only(bottom: AppSizes.spacingMD(context)),
                  child: card,
                ),
            ],
          );
        }
        return IntrinsicHeight(
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              for (var i = 0; i < cards.length; i++) ...[
                if (i > 0) SizedBox(width: AppSizes.spacingMD(context)),
                Expanded(child: cards[i]),
              ],
            ],
          ),
        );
      },
    );
  }

  static const _nativeEntries = <_GuideEntry>[
    _GuideEntry('03_item.md', Icons.category_outlined),
    _GuideEntry('04_weapon.md', Icons.gavel_outlined),
    _GuideEntry('05_outfit.md', Icons.checkroom_outlined),
    _GuideEntry('06_accessory.md', Icons.auto_awesome_outlined),
    _GuideEntry('07_quest.md', Icons.flag_outlined),
    _GuideEntry('27_custom_music.md', Icons.music_note_outlined),
    _GuideEntry('12_translate.md', Icons.translate_outlined),
    _GuideEntry('18_cheat_sheet.md', Icons.bolt_outlined),
  ];

  static const _scriptingEntries = <_GuideEntry>[
    _GuideEntry('scripting/README.md', Icons.menu_book_outlined),
    _GuideEntry('01_hap.md', Icons.list_alt_outlined),
    _GuideEntry('00_hap_system.md', Icons.schema_outlined),
    _GuideEntry('03_getting_started.md', Icons.edit_outlined),
    _GuideEntry('02_action.md', Icons.play_circle_outline),
    _GuideEntry('04_scripts.md', Icons.code),
    _GuideEntry('_signals.md', Icons.podcasts_outlined),
    _GuideEntry('_layouts.md', Icons.grid_view_outlined),
    _GuideEntry('scripting.md', Icons.functions),
  ];

  Widget _categoryBlock(
    BuildContext context,
    String title,
    String blurb,
    IconData icon,
    List<_GuideEntry> entries,
  ) {
    final available = entries.where((e) => _find(e.fileName) != null).toList();
    if (available.isEmpty) return const SizedBox.shrink();

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Icon(
              icon,
              size: AppSizes.iconMD(context),
              color: AppColors.accentPrimary,
            ),
            SizedBox(width: AppSizes.spacingMD(context)),
            Text(
              title.toUpperCase(),
              style: TextStyle(
                fontSize: AppSizes.fontSM(context),
                fontWeight: FontWeight.bold,
                color: AppColors.textPrimary,
                letterSpacing: 1.2,
              ),
            ),
          ],
        ),
        SizedBox(height: AppSizes.spacingSM(context)),
        ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 620),
          child: Text(
            blurb,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: AppColors.textMuted,
              height: 1.5,
            ),
          ),
        ),
        SizedBox(height: AppSizes.spacingLG(context)),
        _tileGrid(context, available),
      ],
    );
  }

  Widget _tileGrid(BuildContext context, List<_GuideEntry> available) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final columns = constraints.maxWidth < 520
            ? 2
            : constraints.maxWidth < 820
            ? 3
            : 4;
        final spacing = AppSizes.spacingMD(context);
        final width =
            (constraints.maxWidth - spacing * (columns - 1)) / columns;

        return Wrap(
          spacing: spacing,
          runSpacing: spacing,
          children: [
            for (final entry in available)
              SizedBox(
                width: width,
                child: _GuideTile(
                  icon: entry.icon,
                  title: _titleOf(
                    entry.fileName,
                    DocsService.prettyName(entry.fileName),
                  ),
                  onTap: () => onOpenFile(entry.fileName),
                ),
              ),
          ],
        );
      },
    );
  }

  Widget _referenceCard(BuildContext context, AppLocalizations l10n) {
    final resources = pages.where((p) => p.isResource).toList();
    if (resources.isEmpty) return const SizedBox.shrink();

    return Container(
      width: double.infinity,
      padding: EdgeInsets.all(AppSizes.paddingLG(context)),
      decoration: BoxDecoration(
        color: AppColors.surfaceMedium,
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(color: AppColors.borderLight),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(
                Icons.table_chart_outlined,
                size: AppSizes.iconMD(context),
                color: AppColors.accentPrimary,
              ),
              SizedBox(width: AppSizes.spacingMD(context)),
              Text(
                l10n.docsSectionReference,
                style: TextStyle(
                  fontSize: AppSizes.fontMD(context),
                  fontWeight: FontWeight.w700,
                  color: AppColors.textPrimary,
                ),
              ),
            ],
          ),
          SizedBox(height: AppSizes.spacingSM(context)),
          Text(
            l10n.docsReferenceBody,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: AppColors.textMuted,
              height: 1.5,
            ),
          ),
          SizedBox(height: AppSizes.spacingMD(context)),
          Wrap(
            spacing: AppSizes.spacingSM(context),
            runSpacing: AppSizes.spacingSM(context),
            children: [
              for (final page in resources)
                _Chip(
                  label: page.title,
                  onTap: () => onOpenFile(page.fileName),
                ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _browseAll(
    BuildContext context,
    AppLocalizations l10n,
    int guideCount,
  ) {
    return Center(
      child: _TextAction(
        label: '${l10n.docsBrowseAll}  ·  ${l10n.docsPagesCount(pages.length)}',
        onTap: onBrowseAll,
      ),
    );
  }
}

class _GuideEntry {
  final String fileName;
  final IconData icon;

  const _GuideEntry(this.fileName, this.icon);
}

class _HeroButton extends StatefulWidget {
  final String label;
  final String badge;
  final VoidCallback onTap;

  const _HeroButton({
    required this.label,
    required this.badge,
    required this.onTap,
  });

  @override
  State<_HeroButton> createState() => _HeroButtonState();
}

class _HeroButtonState extends State<_HeroButton> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          padding: EdgeInsets.symmetric(
            horizontal: AppSizes.paddingLG(context),
            vertical: AppSizes.paddingMD(context),
          ),
          decoration: BoxDecoration(
            color: _hovered
                ? AppColors.accentPrimary
                : AppColors.accentPrimary.withValues(alpha: 0.85),
            borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(
                Icons.play_arrow_rounded,
                size: AppSizes.iconMD(context),
                color: AppColors.backgroundPrimary,
              ),
              SizedBox(width: AppSizes.spacingSM(context)),
              Text(
                widget.label,
                style: TextStyle(
                  fontSize: AppSizes.fontMD(context),
                  fontWeight: FontWeight.w700,
                  color: AppColors.backgroundPrimary,
                ),
              ),
              SizedBox(width: AppSizes.spacingMD(context)),
              Container(
                padding: EdgeInsets.symmetric(
                  horizontal: AppSizes.chipPaddingH(context),
                  vertical: AppSizes.chipPaddingV(context),
                ),
                decoration: BoxDecoration(
                  color: AppColors.backgroundPrimary.withValues(alpha: 0.22),
                  borderRadius: BorderRadius.circular(999),
                ),
                child: Text(
                  widget.badge,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    fontWeight: FontWeight.w700,
                    color: AppColors.backgroundPrimary,
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _PathCard extends StatefulWidget {
  final IconData icon;
  final String title;
  final String body;
  final VoidCallback onTap;

  const _PathCard({
    required this.icon,
    required this.title,
    required this.body,
    required this.onTap,
  });

  @override
  State<_PathCard> createState() => _PathCardState();
}

class _PathCardState extends State<_PathCard> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          padding: EdgeInsets.all(AppSizes.paddingLG(context)),
          decoration: BoxDecoration(
            color: _hovered ? AppColors.surfaceLight : AppColors.surfaceMedium,
            borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
            border: Border.all(
              color: _hovered
                  ? AppColors.accentPrimary.withValues(alpha: 0.5)
                  : AppColors.borderLight,
            ),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Icon(
                widget.icon,
                size: AppSizes.iconLG(context),
                color: AppColors.accentPrimary,
              ),
              SizedBox(height: AppSizes.spacingMD(context)),
              Text(
                widget.title,
                style: TextStyle(
                  fontSize: AppSizes.fontMD(context),
                  fontWeight: FontWeight.w700,
                  color: AppColors.textPrimary,
                ),
              ),
              SizedBox(height: AppSizes.spacingSM(context)),
              Text(
                widget.body,
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  color: AppColors.textMuted,
                  height: 1.5,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _GuideTile extends StatefulWidget {
  final IconData icon;
  final String title;
  final VoidCallback onTap;

  const _GuideTile({
    required this.icon,
    required this.title,
    required this.onTap,
  });

  @override
  State<_GuideTile> createState() => _GuideTileState();
}

class _GuideTileState extends State<_GuideTile> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          padding: EdgeInsets.symmetric(
            horizontal: AppSizes.paddingMD(context),
            vertical: AppSizes.paddingMD(context),
          ),
          decoration: BoxDecoration(
            color: _hovered ? AppColors.surfaceLight : Colors.transparent,
            borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
            border: Border.all(
              color: _hovered
                  ? AppColors.accentPrimary.withValues(alpha: 0.45)
                  : AppColors.borderLight,
            ),
          ),
          child: Row(
            children: [
              Icon(
                widget.icon,
                size: AppSizes.iconMD(context),
                color: _hovered
                    ? AppColors.accentPrimary
                    : AppColors.textSecondary,
              ),
              SizedBox(width: AppSizes.spacingMD(context)),
              Expanded(
                child: Text(
                  widget.title,
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.textPrimary,
                    height: 1.35,
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _Chip extends StatefulWidget {
  final String label;
  final VoidCallback onTap;

  const _Chip({required this.label, required this.onTap});

  @override
  State<_Chip> createState() => _ChipState();
}

class _ChipState extends State<_Chip> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          padding: EdgeInsets.symmetric(
            horizontal: AppSizes.chipPaddingH(context),
            vertical: AppSizes.chipPaddingV(context),
          ),
          decoration: BoxDecoration(
            color: _hovered
                ? AppColors.accentPrimary.withValues(alpha: 0.16)
                : AppColors.surfaceLight,
            borderRadius: BorderRadius.circular(999),
            border: Border.all(
              color: _hovered
                  ? AppColors.accentPrimary.withValues(alpha: 0.5)
                  : AppColors.borderLight,
            ),
          ),
          child: Text(
            widget.label,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: _hovered
                  ? AppColors.accentPrimary
                  : AppColors.textSecondary,
            ),
          ),
        ),
      ),
    );
  }
}

class _TextAction extends StatefulWidget {
  final String label;
  final VoidCallback onTap;

  const _TextAction({required this.label, required this.onTap});

  @override
  State<_TextAction> createState() => _TextActionState();
}

class _TextActionState extends State<_TextAction> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: Padding(
          padding: EdgeInsets.symmetric(
            horizontal: AppSizes.paddingMD(context),
            vertical: AppSizes.spacingMD(context),
          ),
          child: Text(
            widget.label,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: _hovered ? AppColors.accentPrimary : AppColors.textMuted,
              decoration: _hovered ? TextDecoration.underline : null,
              decorationColor: AppColors.accentPrimary,
            ),
          ),
        ),
      ),
    );
  }
}
