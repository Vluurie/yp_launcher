import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class SidebarKeys {
  SidebarKeys._();
  static final GlobalKey texturesTab = GlobalKey(debugLabel: 'sidebar-textures');
  static final GlobalKey cutscenesTab = GlobalKey(debugLabel: 'sidebar-cutscenes');
}

class SidebarTabItem {
  final int index;
  final String label;
  final IconData icon;
  const SidebarTabItem({
    required this.index,
    required this.label,
    required this.icon,
  });
}

class SidebarSection {
  final String title;
  final List<SidebarTabItem> items;
  final bool collapsible;
  final bool defaultCollapsed;
  const SidebarSection({
    required this.title,
    required this.items,
    this.collapsible = false,
    this.defaultCollapsed = false,
  });
}

class LauncherSidebar extends StatefulWidget {
  final List<SidebarSection> sections;
  final int activeIndex;
  final ValueChanged<int> onSelect;

  const LauncherSidebar({
    super.key,
    required this.sections,
    required this.activeIndex,
    required this.onSelect,
  });

  @override
  State<LauncherSidebar> createState() => _LauncherSidebarState();
}

class _LauncherSidebarState extends State<LauncherSidebar> {
  static const _prefKeyPrefix = 'sidebar_section_collapsed_';
  final Map<String, bool> _collapsed = {};

  @override
  void initState() {
    super.initState();
    for (final section in widget.sections) {
      if (section.collapsible) {
        _collapsed[section.title] = section.defaultCollapsed;
      }
    }
    _loadPrefs();
  }

  Future<void> _loadPrefs() async {
    final prefs = await SharedPreferences.getInstance();
    var changed = false;
    for (final section in widget.sections) {
      if (!section.collapsible) continue;
      final key = '$_prefKeyPrefix${section.title}';
      final stored = prefs.getBool(key);
      if (stored != null && stored != _collapsed[section.title]) {
        _collapsed[section.title] = stored;
        changed = true;
      }
    }
    if (mounted && changed) setState(() {});
  }

  bool _rawCollapsed(SidebarSection section) {
    if (!section.collapsible) return false;
    return _collapsed[section.title] ?? section.defaultCollapsed;
  }

  bool _isCollapsed(SidebarSection section) => _rawCollapsed(section);

  void _toggle(SidebarSection section) {
    if (!section.collapsible) return;
    final next = !_rawCollapsed(section);
    setState(() => _collapsed[section.title] = next);
    SharedPreferences.getInstance().then((prefs) {
      prefs.setBool('$_prefKeyPrefix${section.title}', next);
    });
  }

  @override
  Widget build(BuildContext context) {
    final showLabels = AppSizes.sidebarLabelsVisible(context);
    return Container(
      width: AppSizes.sidebarWidth(context),
      decoration: const BoxDecoration(
        color: AppColors.surfaceMedium,
        border: Border(right: BorderSide(color: AppColors.borderLight)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Expanded(
            child: SingleChildScrollView(
              padding:
                  EdgeInsets.symmetric(vertical: AppSizes.paddingMD(context)),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  for (var s = 0; s < widget.sections.length; s++) ...[
                    if (s > 0)
                      SizedBox(height: AppSizes.sidebarSectionGap(context)),
                    if (showLabels)
                      _SectionHeader(
                        section: widget.sections[s],
                        collapsed: _isCollapsed(widget.sections[s]),
                        onToggle: () => _toggle(widget.sections[s]),
                      ),
                    if (!showLabels && s > 0) const _SectionDivider(),
                    if (!showLabels && widget.sections[s].collapsible)
                      _CollapsedToggleIcon(
                        collapsed: _isCollapsed(widget.sections[s]),
                        onToggle: () => _toggle(widget.sections[s]),
                      ),
                    if (!_isCollapsed(widget.sections[s]))
                      for (final item in widget.sections[s].items)
                        KeyedSubtree(
                          key: item.index == 3
                              ? SidebarKeys.texturesTab
                              : item.index == 7
                                  ? SidebarKeys.cutscenesTab
                                  : null,
                          child: _SidebarTab(
                            item: item,
                            active: item.index == widget.activeIndex,
                            showLabel: showLabels,
                            onTap: () => widget.onSelect(item.index),
                          ),
                        ),
                  ],
                ],
              ),
            ),
          ),
          _VersionFooter(showLabel: showLabels),
        ],
      ),
    );
  }
}

class _VersionFooter extends StatelessWidget {
  final bool showLabel;
  const _VersionFooter({required this.showLabel});

  @override
  Widget build(BuildContext context) {
    final label = AppStrings.appVersion;
    return Container(
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.cardPaddingH(context),
        vertical: AppSizes.spacingSM(context),
      ),
      decoration: const BoxDecoration(
        border: Border(top: BorderSide(color: AppColors.borderLight)),
      ),
      child: Tooltip(
        message: showLabel ? '' : label,
        child: Container(
          padding: EdgeInsets.symmetric(
            horizontal: AppSizes.chipPaddingH(context),
            vertical: AppSizes.chipPaddingV(context),
          ),
          decoration: BoxDecoration(
            color: AppColors.accentPrimary.withValues(alpha: 0.10),
            borderRadius: BorderRadius.circular(
              AppSizes.borderRadius(context),
            ),
            border: Border.all(
              color: AppColors.accentPrimary.withValues(alpha: 0.3),
            ),
          ),
          alignment: Alignment.center,
          child: Text(
            label,
            overflow: TextOverflow.ellipsis,
            maxLines: 1,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              fontWeight: FontWeight.bold,
              color: AppColors.accentPrimary,
              letterSpacing: 0.5,
            ),
          ),
        ),
      ),
    );
  }
}

class _SectionHeader extends StatefulWidget {
  final SidebarSection section;
  final bool collapsed;
  final VoidCallback onToggle;
  const _SectionHeader({
    required this.section,
    required this.collapsed,
    required this.onToggle,
  });

  @override
  State<_SectionHeader> createState() => _SectionHeaderState();
}

class _SectionHeaderState extends State<_SectionHeader> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final collapsible = widget.section.collapsible;
    final color = (collapsible && _hovered)
        ? AppColors.textSecondary
        : AppColors.textMuted;

    final title = Text(
      widget.section.title.toUpperCase(),
      style: TextStyle(
        fontSize: AppSizes.fontXS(context),
        fontWeight: FontWeight.bold,
        color: color,
        letterSpacing: 1.0,
      ),
    );

    final padding = EdgeInsets.fromLTRB(
      AppSizes.cardPaddingH(context),
      AppSizes.spacingMD(context),
      AppSizes.cardPaddingH(context),
      AppSizes.spacingSM(context),
    );

    if (!collapsible) {
      return Padding(padding: padding, child: title);
    }

    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onToggle,
        child: Padding(
          padding: padding,
          child: Row(
            children: [
              Expanded(child: title),
              if (widget.collapsed)
                Padding(
                  padding: const EdgeInsets.only(right: 6),
                  child: Text(
                    widget.section.items.length.toString(),
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      color: AppColors.textMuted,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                ),
              AnimatedRotation(
                duration: const Duration(milliseconds: 120),
                turns: widget.collapsed ? -0.25 : 0.0,
                child: Icon(
                  Icons.keyboard_arrow_down,
                  size: AppSizes.iconSM(context),
                  color: color,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _SectionDivider extends StatelessWidget {
  const _SectionDivider();

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.cardPaddingH(context),
        vertical: AppSizes.spacingSM(context),
      ),
      child: Container(height: 1, color: AppColors.borderLight),
    );
  }
}

/// Tiny chevron-only toggle for collapsible sections in icon-only sidebar
/// mode (no labels visible).
class _CollapsedToggleIcon extends StatefulWidget {
  final bool collapsed;
  final VoidCallback onToggle;
  const _CollapsedToggleIcon({
    required this.collapsed,
    required this.onToggle,
  });

  @override
  State<_CollapsedToggleIcon> createState() => _CollapsedToggleIconState();
}

class _CollapsedToggleIconState extends State<_CollapsedToggleIcon> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final color = _hovered ? AppColors.textSecondary : AppColors.textMuted;
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onToggle,
        child: Container(
          alignment: Alignment.center,
          padding: const EdgeInsets.symmetric(vertical: 4),
          child: AnimatedRotation(
            duration: const Duration(milliseconds: 120),
            turns: widget.collapsed ? -0.25 : 0.0,
            child: Icon(
              Icons.keyboard_arrow_down,
              size: AppSizes.iconSM(context),
              color: color,
            ),
          ),
        ),
      ),
    );
  }
}

class _SidebarTab extends StatefulWidget {
  final SidebarTabItem item;
  final bool active;
  final bool showLabel;
  final VoidCallback onTap;

  const _SidebarTab({
    required this.item,
    required this.active,
    required this.showLabel,
    required this.onTap,
  });

  @override
  State<_SidebarTab> createState() => _SidebarTabState();
}

class _SidebarTabState extends State<_SidebarTab> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final iconColor = widget.active
        ? AppColors.accentPrimary
        : _hovered
            ? AppColors.textPrimary
            : AppColors.textMuted;
    final textColor = widget.active
        ? AppColors.accentPrimary
        : _hovered
            ? AppColors.textPrimary
            : AppColors.textSecondary;

    final row = Row(
      mainAxisAlignment: widget.showLabel
          ? MainAxisAlignment.start
          : MainAxisAlignment.center,
      children: [
        SizedBox(width: widget.showLabel ? AppSizes.cardPaddingH(context) : 0),
        Icon(widget.item.icon, size: AppSizes.iconMD(context), color: iconColor),
        if (widget.showLabel) ...[
          SizedBox(width: AppSizes.spacingMD(context)),
          Expanded(
            child: Text(
              widget.item.label,
              style: TextStyle(
                fontSize: AppSizes.fontSM(context),
                fontWeight:
                    widget.active ? FontWeight.w600 : FontWeight.normal,
                color: textColor,
                letterSpacing: 0.3,
              ),
              overflow: TextOverflow.ellipsis,
            ),
          ),
        ],
      ],
    );

    final tab = MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 100),
          height: AppSizes.sidebarRowHeight(context),
          decoration: BoxDecoration(
            color: widget.active
                ? AppColors.accentPrimary.withValues(alpha: 0.10)
                : _hovered
                    ? AppColors.surfaceLight
                    : Colors.transparent,
            border: Border(
              left: BorderSide(
                color: widget.active
                    ? AppColors.accentPrimary
                    : Colors.transparent,
                width: 3,
              ),
            ),
          ),
          child: row,
        ),
      ),
    );

    if (widget.showLabel) return tab;
    return Tooltip(
      message: widget.item.label,
      child: tab,
    );
  }
}

/// Helper to assemble the standard launcher sidebar from app state.
List<SidebarSection> buildLauncherSections(AppLocalizations l10n) {
  return [
    SidebarSection(
      title: l10n.tabSectionGeneral,
      items: [
        SidebarTabItem(
          index: 0,
          label: l10n.tabLauncher,
          icon: Icons.play_circle_outline,
        ),
      ],
    ),
    SidebarSection(
      title: l10n.tabSectionMods,
      items: [
        SidebarTabItem(
          index: 5,
          label: l10n.tabMods,
          icon: Icons.extension_outlined,
        ),
        SidebarTabItem(
          index: 3,
          label: l10n.tabTextures,
          icon: Icons.texture,
        ),
        SidebarTabItem(
          index: 7,
          label: l10n.tabCutscenes,
          icon: Icons.movie_creation_outlined,
        ),
      ],
    ),
    SidebarSection(
      title: l10n.tabSectionConfig,
      collapsible: true,
      defaultCollapsed: true,
      items: [
        SidebarTabItem(
          index: 1,
          label: l10n.tabNams,
          icon: Icons.tune,
        ),
        SidebarTabItem(
          index: 4,
          label: l10n.tabYorhaProtocol,
          icon: Icons.developer_mode,
        ),
        SidebarTabItem(
          index: 2,
          label: l10n.tabLodMod,
          icon: Icons.image_outlined,
        ),
        SidebarTabItem(
          index: 6,
          label: l10n.tabNaiom,
          icon: Icons.mouse_outlined,
        ),
        SidebarTabItem(
          index: 8,
          label: l10n.tabPlugins,
          icon: Icons.power_outlined,
        ),
      ],
    ),
  ];
}
