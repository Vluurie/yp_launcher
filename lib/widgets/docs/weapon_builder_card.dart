import 'package:flutter/material.dart';
import 'dart:async';

import 'package:desktop_drop/desktop_drop.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/services/docs/outfit_builder_service.dart';
import 'package:yp_launcher/services/docs/weapon_builder_service.dart';
import 'package:yp_launcher/services/docs/weapon_convert_service.dart';
import 'package:yp_launcher/services/docs/thumbnail_service.dart';
import 'package:yp_launcher/services/rust_error_codes.dart';
import 'package:yp_launcher/src/rust/api/texture.dart';
import 'package:yp_launcher/widgets/docs/thumbnail_dialog.dart';
import 'package:yp_launcher/widgets/docs/weapon_preview.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/app_dropdown.dart';
import 'package:yp_launcher/widgets/docs/step_button.dart';
import 'package:yp_launcher/widgets/hover_button.dart';

class WeaponBuilderCard extends ConsumerStatefulWidget {
  const WeaponBuilderCard({super.key});

  @override
  ConsumerState<WeaponBuilderCard> createState() => _WeaponBuilderCardState();
}

class _WeaponBuilderCardState extends ConsumerState<WeaponBuilderCard> {
  final _nameController = TextEditingController();
  final _descriptionController = TextEditingController();
  final _modIdController = TextEditingController(text: 'my_weapon');
  final _helpShortController = TextEditingController();
  final _skill1NameController = TextEditingController();
  final _skill1DescController = TextEditingController();
  final _skill2NameController = TextEditingController();
  final _skill2DescController = TextEditingController();
  final _storyController = TextEditingController();

  final List<WeaponUpgrade> _upgrades = [];
  final List<WeaponEffect> _effects = [];
  int? _traitId;
  int? _estIdNormal;
  int? _estIdBerserk;

  ThumbnailChoice? _iconChoice;
  ThumbnailChoice? _storyChoice;

  String get _iconTextureName =>
      WeaponBuilderService.iconTextureNameFor(_id ?? WeaponBuilderService.firstId);

  String get _storyTextureName => WeaponBuilderService.storyTextureNameFor(
        _id ?? WeaponBuilderService.firstId,
      );

  WeaponType _type = WeaponType.smallSword;
  int? _id;
  int? _objId;
  int _sellPrice = 500;
  bool _inShop = false;
  String _shopName = OutfitBuilderService.shopNames.first;
  final Set<int> _shopStates = {0};
  int _buyPrice = 5000;

  final List<WeaponLevel> _levels = [
    const WeaponLevel(minDamage: 100, maxDamage: 140),
  ];

  Set<int> _takenObjIds = const {};
  WeaponSource? _model;
  bool _dragging = false;
  String? _dropError;
  bool _busy = false;
  String? _error;
  String? _writtenPath;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => _pickIds());
  }

  @override
  void dispose() {
    _nameController.dispose();
    _descriptionController.dispose();
    _modIdController.dispose();
    _helpShortController.dispose();
    _skill1NameController.dispose();
    _skill1DescController.dispose();
    _skill2NameController.dispose();
    _skill2DescController.dispose();
    _storyController.dispose();
    super.dispose();
  }

  String get _gameDir =>
      ref.read(appStateControllerProvider).selectedDirectory;

  WeaponDraft get _draft => WeaponDraft(
        displayName: _nameController.text.trim(),
        description: _descriptionController.text.trim(),
        type: _type,
        id: _id ?? WeaponBuilderService.firstId,
        objId: _objId ?? _type.objIdStart,
        sellPrice: _sellPrice,
        levels: _levels,
        shopName: _inShop ? _shopName : null,
        shopStates: _shopStates,
        buyPrice: _buyPrice,
        iconName: _iconChoice == null ? null : _iconTextureName,
        storyTextureSuffix: _storyChoice == null ? null : _id,
        helpShort: _helpShortController.text.trim(),
        skill1Name: _skill1NameController.text.trim(),
        skill1Desc: _skill1DescController.text.trim(),
        skill2Name: _skill2NameController.text.trim(),
        skill2Desc: _skill2DescController.text.trim(),
        story: _storyController.text.trim().isEmpty
            ? const []
            : [_storyController.text.trim()],
        upgrades: _upgrades,
        traitId: _traitId,
        effects: _effects,
        estIdNormal: _estIdNormal,
        estIdBerserk: _estIdBerserk,
      );

  Future<void> _pickIds() async {
    final dir = _gameDir;
    if (dir.isEmpty) return;

    final seed = _modIdController.text.trim();
    final takenObj = await WeaponBuilderService.takenObjIds(dir);
    final id = await WeaponBuilderService.nextFreeId(dir, seed: seed);
    if (!mounted) return;

    setState(() {
      _takenObjIds = takenObj;
      _id = id;
      _objId = WeaponBuilderService.allocateObjId(
        _type,
        takenObj,
        seed: _modIdController.text.trim(),
      );
    });
  }

  Future<void> _create() async {
    final l10n = AppLocalizations.of(context)!;
    final dir = _gameDir;

    if (dir.isEmpty) {
      setState(() => _error = l10n.weaponBuilderNeedsGameDir);
      return;
    }
    if (_nameController.text.trim().isEmpty) {
      setState(() => _error = l10n.weaponBuilderNeedsName);
      return;
    }
    if (_model == null) {
      setState(() => _error = l10n.weaponBuilderNeedsModel);
      return;
    }
    if (_objId == null || _id == null) {
      setState(() => _error = l10n.weaponBuilderNoFreeSlot);
      return;
    }

    setState(() {
      _busy = true;
      _error = null;
      _writtenPath = null;
    });

    try {
      final modId = _modIdController.text.trim().isEmpty
          ? 'my_weapon'
          : _modIdController.text.trim();

      final model = _model;
      if (model != null) {
        await WeaponConvertService.convert(
          source: model,
          targetObjId: _objId!,
          gameDir: dir,
          modId: modId,
        );
      }

      final outputDir = ThumbnailService.outputDirFor(dir, modId);

      final icon = _iconChoice;
      if (icon != null) {
        await ThumbnailService.createNamed(
          sourceImagePath: icon.imagePath,
          outputDir: outputDir,
          textureName: _iconTextureName,
          width: icon.width,
          height: icon.height,
        );
      }

      final story = _storyChoice;
      if (story != null) {
        await ThumbnailService.createNamed(
          sourceImagePath: story.imagePath,
          outputDir: outputDir,
          textureName: _storyTextureName,
          width: story.width,
          height: story.height,
        );
      }

      final file = await WeaponBuilderService.write(
        gameDir: dir,
        modId: modId,
        draft: _draft,
      );

      if (!mounted) return;
      setState(() {
        _busy = false;
        _writtenPath = file.path;
        _iconChoice = null;
        _storyChoice = null;
      });
      await _pickIds();
    } on MiscTexError catch (e) {
      if (!mounted) return;
      setState(() {
        _busy = false;
        _error = RustErrorCodes.message(l10n, e.code, e.detail);
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _busy = false;
        _error = '$e';
      });
    }
  }

  Future<void> _openFolder() async {
    final path = _writtenPath;
    if (path == null) return;
    try {
      await launchUrl(Uri.file(path).resolve('.'));
    } catch (_) {}
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _modelZone(context, l10n),
        if (_model == null) const SizedBox.shrink() else ..._rest(context, l10n),
      ],
    );
  }

  List<Widget> _rest(BuildContext context, AppLocalizations l10n) {
    return [
        SizedBox(height: AppSizes.paddingMD(context)),
        WeaponPreview(
          key: ValueKey(_model!.datPath),
          datPath: _model!.datPath,
          dttPath: _model!.dttPath,
        ),
        SizedBox(height: AppSizes.paddingMD(context)),
        _field(
          context,
          label: l10n.weaponBuilderName,
          hint: l10n.weaponBuilderNameHint,
          controller: _nameController,
        ),
        _field(
          context,
          label: l10n.weaponBuilderDescription,
          hint: l10n.weaponBuilderDescriptionHint,
          controller: _descriptionController,
          maxLines: 2,
        ),
        SizedBox(height: AppSizes.paddingMD(context)),
        _levelsSection(context, l10n),
        SizedBox(height: AppSizes.paddingMD(context)),
        _detailsSection(context, l10n),
        SizedBox(height: AppSizes.paddingMD(context)),
        if (_gameDir.isNotEmpty) ...[
          _iconRow(
            context,
            label: l10n.weaponBuilderIcon,
            hint: l10n.weaponBuilderIconHint,
            textureName: _iconTextureName,
            choice: _iconChoice,
            onPick: () => _pickTexture(isStory: false),
            onRemove: () => setState(() => _iconChoice = null),
          ),
          _iconRow(
            context,
            label: l10n.weaponBuilderStoryIcon,
            hint: l10n.weaponBuilderStoryIconHint,
            textureName: _storyTextureName,
            choice: _storyChoice,
            onPick: () => _pickTexture(isStory: true),
            onRemove: () => setState(() => _storyChoice = null),
          ),
          SizedBox(height: AppSizes.paddingMD(context)),
        ],
        _upgradesSection(context, l10n),
        SizedBox(height: AppSizes.paddingMD(context)),
        _effectsSection(context, l10n),
        SizedBox(height: AppSizes.paddingMD(context)),
        _shopSection(context, l10n),
        SizedBox(height: AppSizes.paddingMD(context)),
        _advanced(context, l10n),
        SizedBox(height: AppSizes.paddingMD(context)),
        _preview(context, l10n),
        SizedBox(height: AppSizes.paddingMD(context)),
        _actions(context, l10n),
    ];
  }

  Future<void> _pickModel() async {
    final result = await FilePicker.pickFiles(
      type: FileType.custom,
      allowedExtensions: const ['dat', 'dtt'],
    );
    final path = result?.files.single.path;
    if (path != null) await _acceptModel(path);
  }

  Future<void> _acceptModel(String path) async {
    final l10n = AppLocalizations.of(context)!;

    if (!WeaponConvertService.isWeaponFile(path)) {
      setState(() => _dropError = l10n.weaponConvertDropInvalid);
      return;
    }
    final source = await WeaponConvertService.inspect(path);
    if (!mounted) return;
    if (source == null) {
      setState(() => _dropError = l10n.weaponConvertNeedsBothFiles);
      return;
    }

    setState(() {
      _model = source;
      _dropError = null;
      _type = source.type;
      _estIdNormal = null;
      _estIdBerserk = null;
      _objId = WeaponBuilderService.allocateObjId(
        source.type,
        _takenObjIds,
        seed: _modIdController.text.trim(),
      );
    });
  }

  Widget _modelZone(BuildContext context, AppLocalizations l10n) {
    final model = _model;

    return DropTarget(
      onDragEntered: (_) => setState(() => _dragging = true),
      onDragExited: (_) => setState(() => _dragging = false),
      onDragDone: (detail) {
        setState(() => _dragging = false);
        final dropped = detail.files.map((f) => f.path).toList();
        if (dropped.isNotEmpty) unawaited(_acceptModel(dropped.first));
      },
      child: MouseRegion(
        cursor: SystemMouseCursors.click,
        child: GestureDetector(
          onTap: _pickModel,
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 120),
            width: double.infinity,
            padding: EdgeInsets.all(AppSizes.paddingLG(context)),
            decoration: BoxDecoration(
              color: _dragging
                  ? AppColors.accentPrimary.withValues(alpha: 0.10)
                  : AppColors.surfaceLight,
              borderRadius: BorderRadius.circular(
                AppSizes.borderRadius(context),
              ),
              border: Border.all(
                color: _dragging
                    ? AppColors.accentPrimary
                    : model != null
                        ? AppColors.success.withValues(alpha: 0.5)
                        : AppColors.borderMedium,
              ),
            ),
            child: Row(
              children: [
                Icon(
                  model != null
                      ? Icons.hardware
                      : Icons.file_download_outlined,
                  size: AppSizes.iconLG(context),
                  color:
                      model != null ? AppColors.success : AppColors.textMuted,
                ),
                SizedBox(width: AppSizes.spacingMD(context)),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          Flexible(
                            child: Text(
                              model?.stem ?? l10n.weaponBuilderDropTitle,
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                              style: TextStyle(
                                fontSize: AppSizes.fontSM(context),
                                fontWeight: FontWeight.w600,
                                fontFamily:
                                    model != null ? AppSizes.monoFamily : null,
                                fontFamilyFallback: model != null
                                    ? AppSizes.monoFallback
                                    : null,
                                color: model != null
                                    ? AppColors.success
                                    : AppColors.textPrimary,
                              ),
                            ),
                          ),
                          if (model != null) ...[
                            SizedBox(width: AppSizes.spacingSM(context)),
                            Container(
                              padding: EdgeInsets.symmetric(
                                horizontal: AppSizes.chipPaddingH(context),
                                vertical: AppSizes.chipPaddingV(context),
                              ),
                              decoration: BoxDecoration(
                                color: AppColors.accentPrimary
                                    .withValues(alpha: 0.15),
                                borderRadius: BorderRadius.circular(999),
                                border: Border.all(
                                  color: AppColors.accentPrimary
                                      .withValues(alpha: 0.4),
                                ),
                              ),
                              child: Text(
                                _typeLabel(l10n, model.type),
                                style: TextStyle(
                                  fontSize: AppSizes.fontXS(context),
                                  fontWeight: FontWeight.w700,
                                  color: AppColors.accentPrimary,
                                ),
                              ),
                            ),
                          ],
                        ],
                      ),
                      const SizedBox(height: 2),
                      Text(
                        _dropError ??
                            (model != null
                                ? l10n.weaponBuilderModelReady
                                : l10n.weaponBuilderDropHint),
                        style: TextStyle(
                          fontSize: AppSizes.fontXS(context),
                          color: _dropError != null
                              ? AppColors.error
                              : AppColors.textMuted,
                          height: 1.4,
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  String _typeLabel(AppLocalizations l10n, WeaponType type) {
    return switch (type) {
      WeaponType.smallSword => l10n.weaponTypeSmallSword,
      WeaponType.largeSword => l10n.weaponTypeLargeSword,
      WeaponType.spear => l10n.weaponTypeSpear,
      WeaponType.combatBracer => l10n.weaponTypeCombatBracer,
    };
  }

  Widget _levelsSection(BuildContext context, AppLocalizations l10n) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Expanded(
              child: Text(
                l10n.weaponBuilderLevels,
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  fontWeight: FontWeight.w700,
                  color: AppColors.textSecondary,
                  letterSpacing: 0.5,
                ),
              ),
            ),
            if (_levels.length < WeaponBuilderService.maxLevel)
              HoverButton(
                label: l10n.weaponBuilderAddLevel,
                color: AppColors.accentPrimary,
                onTap: () => setState(() {
                  final last = _levels.last;
                  _levels.add(
                    WeaponLevel(
                      minDamage: (last.minDamage * 1.4).round(),
                      maxDamage: (last.maxDamage * 1.4).round(),
                    ),
                  );
                }),
              ),
          ],
        ),
        SizedBox(height: AppSizes.spacingSM(context)),
        for (var i = 0; i < _levels.length; i++)
          _levelRow(context, l10n, i),
      ],
    );
  }

  Widget _levelRow(BuildContext context, AppLocalizations l10n, int index) {
    final level = _levels[index];

    return Padding(
      padding: EdgeInsets.only(bottom: AppSizes.spacingSM(context)),
      child: Row(
        children: [
          SizedBox(
            width: 64,
            child: Text(
              l10n.weaponBuilderLevelNumber(index + 1),
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textMuted,
              ),
            ),
          ),
          Expanded(
            child: Text(
              l10n.weaponBuilderDamage,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textSecondary,
              ),
            ),
          ),
          StepButton(
            icon: Icons.remove,
            onTap: level.minDamage > 1
                ? () => _setLevel(index, level.minDamage - 10, level.maxDamage)
                : null,
          ),
          SizedBox(
            width: 48,
            child: Text(
              '${level.minDamage}',
              textAlign: TextAlign.center,
              style: _numberStyle(context),
            ),
          ),
          StepButton(
            icon: Icons.add,
            onTap: () =>
                _setLevel(index, level.minDamage + 10, level.maxDamage),
          ),
          SizedBox(width: AppSizes.spacingSM(context)),
          Text('-', style: TextStyle(color: AppColors.textMuted)),
          SizedBox(width: AppSizes.spacingSM(context)),
          StepButton(
            icon: Icons.remove,
            onTap: level.maxDamage > level.minDamage
                ? () => _setLevel(index, level.minDamage, level.maxDamage - 10)
                : null,
          ),
          SizedBox(
            width: 48,
            child: Text(
              '${level.maxDamage}',
              textAlign: TextAlign.center,
              style: _numberStyle(context),
            ),
          ),
          StepButton(
            icon: Icons.add,
            onTap: () =>
                _setLevel(index, level.minDamage, level.maxDamage + 10),
          ),
          if (_levels.length > 1) ...[
            SizedBox(width: AppSizes.spacingSM(context)),
            StepButton(
              icon: Icons.close,
              onTap: () => setState(() => _levels.removeAt(index)),
            ),
          ],
        ],
      ),
    );
  }

  Widget _effectsSection(BuildContext context, AppLocalizations l10n) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Expanded(
              child: Text(
                l10n.weaponBuilderEffects,
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  fontWeight: FontWeight.w700,
                  color: AppColors.textSecondary,
                  letterSpacing: 0.5,
                ),
              ),
            ),
            HoverButton(
              label: l10n.weaponBuilderAddEffect,
              color: AppColors.accentPrimary,
              onTap: _addEffect,
            ),
          ],
        ),
        SizedBox(height: AppSizes.spacingSM(context)),
        _estIdRow(
          context,
          label: l10n.weaponBuilderTrailNormal,
          value: _draft.effectiveEstIdNormal,
          onChanged: (v) => setState(() => _estIdNormal = v),
        ),
        _estIdRow(
          context,
          label: l10n.weaponBuilderTrailBerserk,
          value: _draft.effectiveEstIdBerserk,
          onChanged: (v) => setState(() => _estIdBerserk = v),
        ),
        Padding(
          padding: EdgeInsets.only(bottom: AppSizes.spacingMD(context)),
          child: Text(
            l10n.weaponBuilderTrailHint,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: AppColors.textMuted,
              height: 1.4,
            ),
          ),
        ),
        if (_effects.isEmpty)
          Text(
            l10n.weaponBuilderEffectsHint,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: AppColors.textMuted,
              height: 1.4,
            ),
          )
        else
          for (var i = 0; i < _effects.length; i++)
            _effectRow(context, l10n, i),
      ],
    );
  }

  Widget _estIdRow(
    BuildContext context, {
    required String label,
    required int value,
    required ValueChanged<int> onChanged,
  }) {
    return Padding(
      padding: EdgeInsets.only(bottom: AppSizes.spacingSM(context)),
      child: Row(
        children: [
          Expanded(
            child: Text(
              label,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textSecondary,
              ),
            ),
          ),
          StepButton(
            icon: Icons.remove,
            onTap: value > WeaponBuilderService.minEstId
                ? () => onChanged(
                      WeaponBuilderService.clampEstId(value - 1, up: false),
                    )
                : null,
          ),
          SizedBox(
            width: 64,
            child: Text(
              '$value',
              textAlign: TextAlign.center,
              style: _numberStyle(context),
            ),
          ),
          StepButton(
            icon: Icons.add,
            onTap: value < WeaponBuilderService.maxEstId
                ? () => onChanged(
                      WeaponBuilderService.clampEstId(value + 1, up: true),
                    )
                : null,
          ),
        ],
      ),
    );
  }

  void _addEffect() {
    final used = _effects.map((e) => e.path.path).toSet();
    final free = WeaponBuilderService.effectPaths
        .where((p) => !used.contains(p.path))
        .toList();
    if (free.isEmpty) return;

    setState(() {
      _effects.add(
        WeaponEffect(
          path: free.first,
          levels: free.first.perLevel ? _defaultLevels(free.first) : const {},
          value: free.first.op == EffectOp.set ? 1.0 : 1.5,
        ),
      );
    });
  }

  Map<int, double> _defaultLevels(EffectPath path) {
    final base = path.op == EffectOp.add ? 0.1 : 1.1;
    return {
      for (var lv = 1; lv <= _levels.length; lv++)
        lv: double.parse((base * lv).toStringAsFixed(2)),
    };
  }

  Widget _effectRow(BuildContext context, AppLocalizations l10n, int index) {
    final effect = _effects[index];
    final used = _effects
        .where((e) => e != effect)
        .map((e) => e.path.path)
        .toSet();
    final choices = WeaponBuilderService.effectPaths
        .where((p) => p == effect.path || !used.contains(p.path))
        .toList();

    return Padding(
      padding: EdgeInsets.only(bottom: AppSizes.spacingMD(context)),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Expanded(
                child: AppDropdown<EffectPath>(
                  value: effect.path,
                  items: choices,
                  itemLabel: (p) => p.path,
                  onChanged: (path) => setState(() {
                    _effects[index] = WeaponEffect(
                      path: path,
                      levels:
                          path.perLevel ? _defaultLevels(path) : const {},
                      value: path.op == EffectOp.set ? 1.0 : 1.5,
                      whenPlayer: effect.whenPlayer,
                    );
                  }),
                  maxWidth: 340,
                  minWidth: 340,
                ),
              ),
              SizedBox(width: AppSizes.spacingSM(context)),
              StepButton(
                icon: Icons.close,
                onTap: () => setState(() => _effects.removeAt(index)),
              ),
            ],
          ),
          SizedBox(height: AppSizes.spacingSM(context)),
          Padding(
            padding: EdgeInsets.only(left: AppSizes.paddingMD(context)),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                if (effect.path.perLevel)
                  for (final lv in effect.levels.keys.toList()..sort())
                    _effectLevelRow(context, l10n, index, lv)
                else
                  _effectValueRow(context, l10n, index),
                SizedBox(height: AppSizes.spacingSM(context)),
                Row(
                  children: [
                    Expanded(
                      child: Text(
                        l10n.weaponBuilderEffectWhen,
                        style: TextStyle(
                          fontSize: AppSizes.fontXS(context),
                          color: AppColors.textSecondary,
                        ),
                      ),
                    ),
                    AppDropdown<String>(
                      value: effect.whenPlayer ?? '',
                      items: ['', ...WeaponBuilderService.whenPlayers],
                      itemLabel: (v) =>
                          v.isEmpty ? l10n.weaponBuilderEffectWhenAny : v,
                      onChanged: (v) => setState(() {
                        _effects[index] = WeaponEffect(
                          path: effect.path,
                          levels: effect.levels,
                          value: effect.value,
                          whenPlayer: v.isEmpty ? null : v,
                        );
                      }),
                      maxWidth: 180,
                      minWidth: 180,
                    ),
                  ],
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _effectLevelRow(
    BuildContext context,
    AppLocalizations l10n,
    int index,
    int lv,
  ) {
    final effect = _effects[index];
    final value = effect.levels[lv] ?? 1.0;

    return Padding(
      padding: EdgeInsets.only(bottom: AppSizes.spacingSM(context)),
      child: Row(
        children: [
          SizedBox(
            width: 64,
            child: Text(
              l10n.weaponBuilderLevelNumber(lv),
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textMuted,
              ),
            ),
          ),
          const Spacer(),
          StepButton(
            icon: Icons.remove,
            onTap: () => _setEffectLevel(index, lv, value - 0.1),
          ),
          SizedBox(
            width: 64,
            child: Text(
              value.toStringAsFixed(2),
              textAlign: TextAlign.center,
              style: _numberStyle(context),
            ),
          ),
          StepButton(
            icon: Icons.add,
            onTap: () => _setEffectLevel(index, lv, value + 0.1),
          ),
        ],
      ),
    );
  }

  Widget _effectValueRow(
    BuildContext context,
    AppLocalizations l10n,
    int index,
  ) {
    final effect = _effects[index];
    final on = effect.value != 0;

    return Row(
      children: [
        Expanded(
          child: Text(
            l10n.weaponBuilderEffectOn,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: AppColors.textSecondary,
            ),
          ),
        ),
        Switch(
          value: on,
          onChanged: (v) => setState(() {
            _effects[index] = WeaponEffect(
              path: effect.path,
              levels: effect.levels,
              value: v ? 1.0 : 0.0,
              whenPlayer: effect.whenPlayer,
            );
          }),
          activeThumbColor: AppColors.accentPrimary,
        ),
      ],
    );
  }

  void _setEffectLevel(int index, int lv, double value) {
    final effect = _effects[index];
    final clamped = value < 0 ? 0.0 : value;

    setState(() {
      _effects[index] = WeaponEffect(
        path: effect.path,
        levels: {
          ...effect.levels,
          lv: double.parse(clamped.toStringAsFixed(2)),
        },
        value: effect.value,
        whenPlayer: effect.whenPlayer,
      );
    });
  }

  Widget _iconRow(
    BuildContext context, {
    required String label,
    required String hint,
    required String textureName,
    required ThumbnailChoice? choice,
    required VoidCallback onPick,
    required VoidCallback onRemove,
  }) {
    final l10n = AppLocalizations.of(context)!;
    final has = choice != null;

    return Padding(
      padding: EdgeInsets.symmetric(vertical: AppSizes.spacingSM(context)),
      child: Row(
        children: [
          Icon(
            has ? Icons.image : Icons.image_outlined,
            size: AppSizes.iconMD(context),
            color: has ? AppColors.success : AppColors.textMuted,
          ),
          SizedBox(width: AppSizes.spacingMD(context)),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  label,
                  style: TextStyle(
                    fontSize: AppSizes.fontSM(context),
                    color: AppColors.textPrimary,
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.only(top: 2),
                  child: Text(
                    has
                        ? '${choice.width}×${choice.height}  ·  '
                            'misctex_$textureName'
                        : hint,
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      color: has ? AppColors.success : AppColors.textMuted,
                      height: 1.3,
                    ),
                  ),
                ),
              ],
            ),
          ),
          SizedBox(width: AppSizes.spacingMD(context)),
          HoverButton(
            label: has ? l10n.thumbnailChange : l10n.thumbnailOpen,
            color: AppColors.accentPrimary,
            onTap: onPick,
          ),
          if (has) ...[
            SizedBox(width: AppSizes.spacingSM(context)),
            HoverButton(
              label: l10n.thumbnailRemove,
              color: AppColors.textMuted,
              onTap: onRemove,
            ),
          ],
        ],
      ),
    );
  }

  Future<void> _pickTexture({required bool isStory}) async {
    final name = isStory ? _storyTextureName : _iconTextureName;
    final choice = await ThumbnailDialog.show(
      context,
      subtitle: 'misctex_$name.dat / .dtt',
      initial: isStory ? _storyChoice : _iconChoice,
      fixedSize: isStory
          ? (
              width: ThumbnailService.storyWidth,
              height: ThumbnailService.storyHeight,
            )
          : null,
    );
    if (choice == null || !mounted) return;

    setState(() {
      if (isStory) {
        _storyChoice = choice;
      } else {
        _iconChoice = choice;
      }
    });
  }

  Widget _detailsSection(BuildContext context, AppLocalizations l10n) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          l10n.weaponBuilderDetails,
          style: TextStyle(
            fontSize: AppSizes.fontXS(context),
            fontWeight: FontWeight.w700,
            color: AppColors.textSecondary,
            letterSpacing: 0.5,
          ),
        ),
        SizedBox(height: AppSizes.spacingSM(context)),
        _field(
          context,
          label: l10n.weaponBuilderHelpShort,
          hint: l10n.weaponBuilderHelpShortHint,
          controller: _helpShortController,
          maxLines: 2,
        ),
        _field(
          context,
          label: l10n.weaponBuilderStory,
          hint: l10n.weaponBuilderStoryHint,
          controller: _storyController,
          maxLines: 3,
        ),
        _field(
          context,
          label: l10n.weaponBuilderSkillName(1),
          hint: l10n.weaponBuilderSkillNameHint,
          controller: _skill1NameController,
        ),
        if (_skill1NameController.text.trim().isNotEmpty)
          _field(
            context,
            label: l10n.weaponBuilderSkillDesc(1),
            hint: l10n.weaponBuilderSkillDescHint,
            controller: _skill1DescController,
            maxLines: 2,
          ),
        _field(
          context,
          label: l10n.weaponBuilderSkillName(2),
          hint: l10n.weaponBuilderSkillNameHint,
          controller: _skill2NameController,
        ),
        if (_skill2NameController.text.trim().isNotEmpty)
          _field(
            context,
            label: l10n.weaponBuilderSkillDesc(2),
            hint: l10n.weaponBuilderSkillDescHint,
            controller: _skill2DescController,
            maxLines: 2,
          ),
      ],
    );
  }

  Widget _upgradesSection(BuildContext context, AppLocalizations l10n) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Expanded(
              child: Text(
                l10n.weaponBuilderUpgrades,
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  fontWeight: FontWeight.w700,
                  color: AppColors.textSecondary,
                  letterSpacing: 0.5,
                ),
              ),
            ),
            if (_upgrades.length < WeaponBuilderService.maxLevel - 1)
              HoverButton(
                label: l10n.weaponBuilderAddUpgrade,
                color: AppColors.accentPrimary,
                onTap: () => setState(() {
                  final lv = _upgrades.length + 2;
                  _upgrades.add(
                    WeaponUpgrade(lv: lv, cost: 2500 * (lv - 1)),
                  );
                }),
              ),
          ],
        ),
        SizedBox(height: AppSizes.spacingSM(context)),
        if (_upgrades.isEmpty)
          Text(
            l10n.weaponBuilderUpgradeHint,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: AppColors.textMuted,
              height: 1.4,
            ),
          )
        else
          for (var i = 0; i < _upgrades.length; i++)
            _upgradeRow(context, l10n, i),
      ],
    );
  }

  Widget _upgradeRow(BuildContext context, AppLocalizations l10n, int index) {
    final upgrade = _upgrades[index];

    return Padding(
      padding: EdgeInsets.only(bottom: AppSizes.spacingSM(context)),
      child: Row(
        children: [
          SizedBox(
            width: 96,
            child: Text(
              l10n.weaponBuilderUpgradeLevel(upgrade.lv),
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textMuted,
              ),
            ),
          ),
          Expanded(
            child: Text(
              l10n.weaponBuilderUpgradeCost,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textSecondary,
              ),
            ),
          ),
          StepButton(
            icon: Icons.remove,
            onTap: upgrade.cost > 500
                ? () => _setUpgradeCost(index, upgrade.cost - 500)
                : null,
          ),
          SizedBox(
            width: 72,
            child: Text(
              '${upgrade.cost}',
              textAlign: TextAlign.center,
              style: _numberStyle(context),
            ),
          ),
          StepButton(
            icon: Icons.add,
            onTap: () => _setUpgradeCost(index, upgrade.cost + 500),
          ),
          if (index == _upgrades.length - 1) ...[
            SizedBox(width: AppSizes.spacingSM(context)),
            StepButton(
              icon: Icons.close,
              onTap: () => setState(() => _upgrades.removeAt(index)),
            ),
          ],
        ],
      ),
    );
  }

  void _setUpgradeCost(int index, int cost) {
    setState(() {
      _upgrades[index] = WeaponUpgrade(
        lv: _upgrades[index].lv,
        cost: cost,
        materials: _upgrades[index].materials,
      );
    });
  }

  void _setLevel(int index, int min, int max) {
    setState(() {
      _levels[index] = WeaponLevel(
        minDamage: min,
        maxDamage: max < min ? min : max,
      );
    });
  }

  TextStyle _numberStyle(BuildContext context) => TextStyle(
        fontSize: AppSizes.fontSM(context),
        fontWeight: FontWeight.w600,
        color: AppColors.accentPrimary,
        fontFamily: AppSizes.monoFamily,
        fontFamilyFallback: AppSizes.monoFallback,
      );

  Widget _shopSection(BuildContext context, AppLocalizations l10n) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    l10n.outfitBuilderInShop,
                    style: TextStyle(
                      fontSize: AppSizes.fontSM(context),
                      color: AppColors.textPrimary,
                    ),
                  ),
                  Padding(
                    padding: const EdgeInsets.only(top: 2),
                    child: Text(
                      l10n.outfitBuilderInShopHint,
                      style: TextStyle(
                        fontSize: AppSizes.fontXS(context),
                        color: AppColors.textMuted,
                        height: 1.3,
                      ),
                    ),
                  ),
                ],
              ),
            ),
            Switch(
              value: _inShop,
              onChanged: (v) => setState(() => _inShop = v),
              activeThumbColor: AppColors.accentPrimary,
            ),
          ],
        ),
        if (_inShop) ...[
          SizedBox(height: AppSizes.spacingSM(context)),
          Padding(
            padding: EdgeInsets.only(left: AppSizes.paddingLG(context)),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Expanded(
                      child: Text(
                        l10n.outfitBuilderShopName,
                        style: TextStyle(
                          fontSize: AppSizes.fontSM(context),
                          color: AppColors.textSecondary,
                        ),
                      ),
                    ),
                    AppDropdown<String>(
                      value: _shopName,
                      items: OutfitBuilderService.shopNames,
                      itemLabel: (name) => name,
                      onChanged: (name) => setState(() => _shopName = name),
                      maxWidth: 220,
                      minWidth: 220,
                    ),
                  ],
                ),
                SizedBox(height: AppSizes.spacingSM(context)),
                Row(
                  children: [
                    Expanded(
                      child: Text(
                        l10n.outfitBuilderBuyPrice,
                        style: TextStyle(
                          fontSize: AppSizes.fontSM(context),
                          color: AppColors.textSecondary,
                        ),
                      ),
                    ),
                    StepButton(
                      icon: Icons.remove,
                      onTap: _buyPrice > 500
                          ? () => setState(() => _buyPrice -= 500)
                          : null,
                    ),
                    SizedBox(
                      width: 72,
                      child: Text(
                        '$_buyPrice',
                        textAlign: TextAlign.center,
                        style: _numberStyle(context),
                      ),
                    ),
                    StepButton(
                      icon: Icons.add,
                      onTap: () => setState(() => _buyPrice += 500),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ],
      ],
    );
  }

  Widget _advanced(BuildContext context, AppLocalizations l10n) {
    final objId = _objId;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Expanded(
              flex: 2,
              child: _compactField(
                context,
                label: l10n.outfitBuilderModId,
                controller: _modIdController,
                onChanged: (_) => unawaited(_pickIds()),
              ),
            ),
            SizedBox(width: AppSizes.spacingMD(context)),
            Expanded(
              child: _readOnlyField(
                context,
                label: l10n.weaponBuilderItemId,
                value: _id == null ? '-' : '$_id',
              ),
            ),
            SizedBox(width: AppSizes.spacingMD(context)),
            Expanded(
              child: _readOnlyField(
                context,
                label: l10n.weaponBuilderObjId,
                value: objId == null
                    ? '-'
                    : '0x${objId.toRadixString(16).toUpperCase()}',
              ),
            ),
          ],
        ),
        Padding(
          padding: EdgeInsets.only(top: AppSizes.spacingSM(context)),
          child: Text(
            l10n.weaponBuilderObjIdHint,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: AppColors.textMuted,
              height: 1.4,
            ),
          ),
        ),
      ],
    );
  }

  Widget _readOnlyField(
    BuildContext context, {
    required String label,
    required String value,
  }) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          label,
          style: TextStyle(
            fontSize: AppSizes.fontXS(context),
            color: AppColors.textSecondary,
          ),
        ),
        const SizedBox(height: 4),
        Container(
          width: double.infinity,
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
          decoration: BoxDecoration(
            color: AppColors.surfaceLight,
            borderRadius: BorderRadius.circular(6),
            border: Border.all(color: AppColors.borderLight),
          ),
          child: Text(value, style: _numberStyle(context)),
        ),
      ],
    );
  }

  Widget _field(
    BuildContext context, {
    required String label,
    required String hint,
    required TextEditingController controller,
    int maxLines = 1,
  }) {
    return Padding(
      padding: EdgeInsets.only(bottom: AppSizes.spacingMD(context)),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            label,
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
              controller: controller,
              maxLines: maxLines,
              onChanged: (_) => setState(() {}),
              style: TextStyle(
                fontSize: AppSizes.fontSM(context),
                color: AppColors.textPrimary,
              ),
              decoration: InputDecoration(
                isDense: true,
                hintText: hint,
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
        ],
      ),
    );
  }

  Widget _compactField(
    BuildContext context, {
    required String label,
    required TextEditingController controller,
    ValueChanged<String>? onChanged,
  }) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          label,
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
            controller: controller,
            onChanged: (text) {
              if (onChanged != null) {
                onChanged(text);
              } else {
                setState(() {});
              }
            },
            style: TextStyle(
              fontSize: AppSizes.fontSM(context),
              color: AppColors.textPrimary,
              fontFamily: AppSizes.monoFamily,
              fontFamilyFallback: AppSizes.monoFallback,
            ),
            decoration: const InputDecoration(
              isDense: true,
              border: InputBorder.none,
              contentPadding: EdgeInsets.symmetric(
                horizontal: 10,
                vertical: 8,
              ),
            ),
          ),
        ),
      ],
    );
  }

  Widget _preview(BuildContext context, AppLocalizations l10n) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Icon(
              Icons.description_outlined,
              size: AppSizes.iconSM(context),
              color: AppColors.textMuted,
            ),
            SizedBox(width: AppSizes.spacingSM(context)),
            Text(
              l10n.outfitBuilderPreview,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textMuted,
              ),
            ),
          ],
        ),
        const SizedBox(height: 6),
        Container(
          width: double.infinity,
          decoration: BoxDecoration(
            color: AppColors.surfaceLight,
            borderRadius: BorderRadius.circular(6),
            border: Border.all(color: AppColors.borderLight),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Container(
                padding: EdgeInsets.symmetric(
                  horizontal: AppSizes.paddingMD(context),
                  vertical: AppSizes.paddingXS(context),
                ),
                decoration: BoxDecoration(
                  color: AppColors.surfaceMedium,
                  borderRadius: const BorderRadius.vertical(
                    top: Radius.circular(5),
                  ),
                ),
                child: Text(
                  WeaponBuilderService.fileNameFor(_draft),
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.accentPrimary,
                    fontFamily: AppSizes.monoFamily,
                    fontFamilyFallback: AppSizes.monoFallback,
                  ),
                ),
              ),
              Padding(
                padding: EdgeInsets.all(AppSizes.paddingMD(context)),
                child: SelectableText(
                  WeaponBuilderService.toToml(_draft).trimRight(),
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    fontFamily: AppSizes.monoFamily,
                    fontFamilyFallback: AppSizes.monoFallback,
                    color: AppColors.textSecondary,
                    height: 1.4,
                  ),
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _actions(BuildContext context, AppLocalizations l10n) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            if (_busy)
              SizedBox(
                width: AppSizes.iconMD(context),
                height: AppSizes.iconMD(context),
                child: CircularProgressIndicator(
                  strokeWidth: 2,
                  color: AppColors.accentPrimary,
                ),
              )
            else
              HoverButton(
                label: l10n.weaponBuilderCreate,
                color: AppColors.success,
                onTap: _create,
              ),
            if (_writtenPath != null) ...[
              SizedBox(width: AppSizes.spacingMD(context)),
              HoverButton(
                label: l10n.outfitBuilderOpenFolder,
                color: AppColors.accentPrimary,
                onTap: _openFolder,
              ),
            ],
          ],
        ),
        if (_error != null)
          Padding(
            padding: EdgeInsets.only(top: AppSizes.spacingMD(context)),
            child: Text(
              _error!,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.error,
              ),
            ),
          ),
        if (_writtenPath != null)
          Padding(
            padding: EdgeInsets.only(top: AppSizes.spacingMD(context)),
            child: Row(
              children: [
                Icon(
                  Icons.check_circle_outline,
                  size: AppSizes.iconSM(context),
                  color: AppColors.success,
                ),
                SizedBox(width: AppSizes.spacingSM(context)),
                Expanded(
                  child: Text(
                    l10n.outfitBuilderCreated(_writtenPath!),
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      color: AppColors.success,
                      height: 1.4,
                    ),
                  ),
                ),
              ],
            ),
          ),
      ],
    );
  }
}
