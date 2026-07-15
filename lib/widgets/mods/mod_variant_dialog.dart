import 'package:flutter/material.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

Future<List<ModVariant>?> showModVariantDialog(
  BuildContext context, {
  required List<ModVariant> variants,
}) {
  return showDialog<List<ModVariant>>(
    context: context,
    builder: (ctx) => _ModVariantDialog(variants: variants),
  );
}

String _categoryLabel(AppLocalizations l10n, DataCategory? c) {
  switch (c) {
    case DataCategory.player3d:
      return l10n.variantCatPlayer;
    case DataCategory.weapon3d:
      return l10n.variantCatWeapon;
    case DataCategory.accessory3d:
      return l10n.variantCatAccessory;
    case DataCategory.enemy3d:
      return l10n.variantCatEnemy;
    case DataCategory.modelVariant3d:
      return l10n.variantCatModelVariant;
    case DataCategory.item3d:
      return l10n.variantCatItem;
    case DataCategory.worldProp3d:
      return l10n.variantCatWorldProp;
    case DataCategory.map3d:
      return l10n.variantCatMap;
    case DataCategory.effects:
      return l10n.variantCatEffects;
    case DataCategory.scripting:
      return l10n.variantCatScripting;
    case DataCategory.localization:
      return l10n.variantCatLocalization;
    case DataCategory.ui:
      return l10n.variantCatUi;
    case DataCategory.cutscenes:
      return l10n.variantCatCutscenes;
    case DataCategory.audio:
      return l10n.variantCatAudio;
    case DataCategory.misc:
      return l10n.variantCatMisc;
    default:
      return l10n.variantCatOther;
  }
}

class _Row {
  final DataCategory? headerCategory;
  final bool isHeader;
  final int variantIndex;
  const _Row.header(this.headerCategory)
      : isHeader = true,
        variantIndex = -1;
  const _Row.item(this.variantIndex)
      : isHeader = false,
        headerCategory = null;
}

class _ModVariantDialog extends StatefulWidget {
  final List<ModVariant> variants;

  const _ModVariantDialog({required this.variants});

  @override
  State<_ModVariantDialog> createState() => _ModVariantDialogState();
}

class _ModVariantDialogState extends State<_ModVariantDialog> {
  late final Map<DataCategory, List<int>> _groups = _groupByCategory();
  late final bool _grouped = _groups.length > 1;
  late final Set<int> _selected = _initialSelection();
  final _scrollController = ScrollController();

  Map<DataCategory, List<int>> _groupByCategory() {
    final map = <DataCategory, List<int>>{};
    for (var i = 0; i < widget.variants.length; i++) {
      final cat = widget.variants[i].category ?? DataCategory.other;
      map.putIfAbsent(cat, () => []).add(i);
    }
    final ordered = <DataCategory, List<int>>{};
    for (final cat in variantCategoryOrder) {
      if (map.containsKey(cat)) ordered[cat] = map[cat]!;
    }
    for (final entry in map.entries) {
      ordered.putIfAbsent(entry.key, () => entry.value);
    }
    return ordered;
  }

  bool _isExclusive(int i) {
    final cat = widget.variants[i].category;
    return _grouped && mutuallyExclusiveVariantCategories.contains(cat);
  }

  Set<int> _initialSelection() {
    if (!_grouped) {
      return {for (var i = 0; i < widget.variants.length; i++) i};
    }
    return {
      for (final entry in _groups.entries)
        if (!mutuallyExclusiveVariantCategories.contains(entry.key))
          ...entry.value
    };
  }

  List<_Row> _buildRows() {
    if (!_grouped) {
      return [for (var i = 0; i < widget.variants.length; i++) _Row.item(i)];
    }
    final rows = <_Row>[];
    for (final entry in _groups.entries) {
      rows.add(_Row.header(entry.key));
      for (final i in entry.value) {
        rows.add(_Row.item(i));
      }
    }
    return rows;
  }

  void _toggle(int i) {
    setState(() {
      if (_isExclusive(i)) {
        final cat = widget.variants[i].category;
        final wasSelected = _selected.contains(i);
        _selected.removeWhere((j) => widget.variants[j].category == cat);
        if (!wasSelected) _selected.add(i);
      } else if (_selected.contains(i)) {
        _selected.remove(i);
      } else {
        _selected.add(i);
      }
    });
  }

  void _selectAll() {
    setState(() {
      _selected
        ..removeWhere(
            (j) => mutuallyExclusiveVariantCategories.contains(
                widget.variants[j].category))
        ..addAll({
          for (var i = 0; i < widget.variants.length; i++)
            if (!_isExclusive(i)) i
        });
    });
  }

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final rows = _buildRows();
    return AlertDialog(
      backgroundColor: AppColors.backgroundCard,
      title: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            l10n.modVariantDialogTitle,
            style: TextStyle(
              color: AppColors.accentPrimary,
              fontSize: AppSizes.fontLG(context),
            ),
          ),
          const SizedBox(height: 6),
          Text(
            l10n.modVariantDialogSubtitle,
            style: TextStyle(
              color: AppColors.textMuted,
              fontSize: AppSizes.fontSM(context),
            ),
          ),
        ],
      ),
      contentPadding: const EdgeInsets.fromLTRB(12, 12, 12, 8),
      content: SizedBox(
        width: 440,
        height: 400,
        child: Column(
          children: [
            Row(
              children: [
                _miniButton(l10n.modVariantSelectAll, _selectAll),
                const SizedBox(width: 8),
                _miniButton(l10n.modVariantSelectNone, () {
                  setState(_selected.clear);
                }),
              ],
            ),
            const SizedBox(height: 8),
            Expanded(
              child: Scrollbar(
                controller: _scrollController,
                thumbVisibility: true,
                child: ListView.builder(
                  controller: _scrollController,
                  itemCount: rows.length,
                  itemBuilder: (_, i) {
                    final row = rows[i];
                    if (row.isHeader) {
                      return _sectionHeader(l10n, row.headerCategory);
                    }
                    return _variantRow(l10n, row.variantIndex);
                  },
                ),
              ),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: Text(
            l10n.buttonCancel,
            style: TextStyle(
              color: AppColors.textMuted,
              fontSize: AppSizes.fontSM(context),
            ),
          ),
        ),
        TextButton(
          onPressed: _selected.isEmpty
              ? null
              : () => Navigator.of(context)
                  .pop([for (final i in _selected) widget.variants[i]]),
          child: Text(
            l10n.modVariantInstallSelected(_selected.length),
            style: TextStyle(
              color: _selected.isEmpty
                  ? AppColors.textMuted
                  : AppColors.accentPrimary,
              fontSize: AppSizes.fontSM(context),
              fontWeight: FontWeight.bold,
            ),
          ),
        ),
      ],
    );
  }

  Widget _sectionHeader(AppLocalizations l10n, DataCategory? category) {
    var label = _categoryLabel(l10n, category).toUpperCase();
    if (mutuallyExclusiveVariantCategories.contains(category)) {
      label = '$label  ·  ${l10n.variantPickOneSuffix}';
    }
    return Padding(
      padding: const EdgeInsets.fromLTRB(6, 14, 6, 4),
      child: Text(
        label,
        style: TextStyle(
          color: AppColors.textMuted,
          fontSize: AppSizes.fontXS(context),
          fontWeight: FontWeight.w700,
          letterSpacing: 1.1,
        ),
      ),
    );
  }

  Widget _miniButton(String label, VoidCallback onTap) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        child: Text(
          label,
          style: TextStyle(
            color: AppColors.accentPrimary,
            fontSize: AppSizes.fontXS(context),
            fontWeight: FontWeight.w600,
          ),
        ),
      ),
    );
  }

  Widget _variantRow(AppLocalizations l10n, int i) {
    final v = widget.variants[i];
    final selected = _selected.contains(i);
    final radioStyle = _isExclusive(i);
    return InkWell(
      onTap: () => _toggle(i),
      borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 9),
        child: Row(
          children: [
            Icon(
              radioStyle
                  ? (selected
                      ? Icons.radio_button_checked
                      : Icons.radio_button_unchecked)
                  : (selected ? Icons.check_box : Icons.check_box_outline_blank),
              size: 20,
              color: selected ? AppColors.accentPrimary : AppColors.textMuted,
            ),
            const SizedBox(width: 10),
            Expanded(
              child: Text(
                v.label,
                overflow: TextOverflow.ellipsis,
                style: TextStyle(
                  color: AppColors.textPrimary,
                  fontSize: AppSizes.fontMD(context),
                ),
              ),
            ),
            if (v.textureOnly) ...[
              const SizedBox(width: 8),
              Container(
                padding:
                    const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                decoration: BoxDecoration(
                  color: AppColors.surfaceLight,
                  borderRadius: BorderRadius.circular(4),
                ),
                child: Text(
                  l10n.modVariantTexture,
                  style: TextStyle(
                    color: AppColors.textMuted,
                    fontSize: AppSizes.fontXS(context),
                  ),
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
