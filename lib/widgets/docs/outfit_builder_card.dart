import 'dart:async';

import 'package:desktop_drop/desktop_drop.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path/path.dart' as p;
import 'package:url_launcher/url_launcher.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/services/docs/item_builder_service.dart';
import 'package:yp_launcher/services/docs/outfit_builder_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/app_dropdown.dart';
import 'package:yp_launcher/widgets/docs/step_button.dart';
import 'package:yp_launcher/widgets/docs/outfit_mesh_tuner.dart';
import 'package:yp_launcher/widgets/hover_button.dart';

class OutfitBuilderCard extends ConsumerStatefulWidget {
  const OutfitBuilderCard({super.key});

  @override
  ConsumerState<OutfitBuilderCard> createState() => _OutfitBuilderCardState();
}

class _OutfitBuilderCardState extends ConsumerState<OutfitBuilderCard> {
  final _nameController = TextEditingController();
  final _descriptionController = TextEditingController();
  final _modIdController = TextEditingController(text: 'my_outfit');
  final _outfitIdController = TextEditingController();
  final _itemIdController = TextEditingController();

  OutfitCharacter _character = OutfitCharacter.twoB;
  int _outfitId = OutfitCharacter.twoB.firstOutfitId;
  int _itemId = ItemBuilderService.firstFreeId;
  String? _modelPath;
  bool _copyModel = true;
  bool _autoGive = true;
  bool _inShop = false;
  String _shopName = OutfitBuilderService.shopNames.first;
  final Set<int> _shopStates = {0};
  int _buyPrice = 5000;
  Map<String, MeshRule> _meshRules = const {};

  Set<int> _takenOutfitIds = const {};
  bool _dragging = false;
  bool _busy = false;
  String? _dropError;
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
    _outfitIdController.dispose();
    _itemIdController.dispose();
    super.dispose();
  }

  String get _gameDir =>
      ref.read(appStateControllerProvider).selectedDirectory;

  bool get _idTaken => _takenOutfitIds.contains(_outfitId);

  OutfitDraft get _draft => OutfitDraft(
        displayName: _nameController.text.trim(),
        description: _descriptionController.text.trim(),
        character: _character,
        outfitId: _outfitId,
        itemId: _itemId,
        modelSourcePath: _modelPath,
        meshes: OutfitBuilderService.listsFromRules(_character, _meshRules),
        autoGive: _autoGive,
        shopName: _inShop ? _shopName : null,
        shopStates: _shopStates,
        buyPrice: _buyPrice,
      );

  Future<void> _pickIds() async {
    final dir = _gameDir;
    if (dir.isEmpty) return;

    final taken = await OutfitBuilderService.takenOutfitIds(dir, _character);
    final outfitId =
        await OutfitBuilderService.nextFreeOutfitId(dir, _character);
    final itemId = await ItemBuilderService.nextFreeId(dir);
    if (!mounted) return;

    setState(() {
      _takenOutfitIds = taken;
      _outfitId = outfitId;
      _outfitIdController.text = '$outfitId';
      _itemId = itemId;
      _itemIdController.text = '$itemId';
    });
  }

  Future<void> _pickModel() async {
    final result = await FilePicker.pickFiles(
      type: FileType.custom,
      allowedExtensions: const ['dat', 'dtt'],
    );
    final path = result?.files.single.path;
    if (path != null) _acceptModel(path);
  }

  void _acceptModel(String path) {
    final l10n = AppLocalizations.of(context)!;

    if (!OutfitBuilderService.isModelFile(path)) {
      setState(() => _dropError = l10n.outfitBuilderDropInvalid);
      return;
    }

    final character = OutfitBuilderService.characterFromModelPath(path);
    if (character == null) {
      setState(() => _dropError = l10n.outfitBuilderDropUnknownModel);
      return;
    }

    setState(() {
      _modelPath = path;
      _dropError = null;
      _meshRules = const {};
      _character = character;
    });

    unawaited(_pickIds());
  }

  Future<void> _create() async {
    final l10n = AppLocalizations.of(context)!;
    final dir = _gameDir;

    if (dir.isEmpty) {
      setState(() => _error = l10n.outfitBuilderNeedsGameDir);
      return;
    }
    if (_nameController.text.trim().isEmpty) {
      setState(() => _error = l10n.outfitBuilderNeedsName);
      return;
    }
    if (_modelPath == null) {
      setState(() => _error = l10n.outfitBuilderNeedsModel);
      return;
    }

    setState(() {
      _busy = true;
      _error = null;
      _writtenPath = null;
    });

    try {
      final taken =
          await OutfitBuilderService.takenOutfitIds(dir, _character);
      if (taken.contains(_outfitId)) {
        if (!mounted) return;
        setState(() {
          _busy = false;
          _takenOutfitIds = taken;
          _error = l10n.outfitBuilderIdTaken;
        });
        return;
      }

      final modId = _modIdController.text.trim().isEmpty
          ? 'my_outfit'
          : _modIdController.text.trim();

      await ItemBuilderService.write(
        gameDir: dir,
        modId: modId,
        draft: ItemDraft(
          displayName: _nameController.text.trim(),
          description: _descriptionController.text.trim(),
          id: _itemId,
          autoGive: _autoGive,
          buyPrice: _inShop ? _buyPrice : null,
          category: OutfitBuilderService.outfitItemCategory,
        ),
      );

      final file = await OutfitBuilderService.write(
        gameDir: dir,
        modId: modId,
        draft: _draft,
      );

      if (_inShop) {
        await OutfitBuilderService.writeShopItem(
          gameDir: dir,
          modId: modId,
          draft: _draft,
        );
      }

      if (_copyModel) {
        await OutfitBuilderService.copyModel(
          gameDir: dir,
          modId: modId,
          sourcePath: _modelPath!,
        );
      }

      if (!mounted) return;
      setState(() {
        _busy = false;
        _writtenPath = file.path;
      });
      await _pickIds();
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
        _dropZone(context, l10n),
        SizedBox(height: AppSizes.paddingMD(context)),
        _field(
          context,
          label: l10n.outfitBuilderName,
          hint: l10n.outfitBuilderNameHint,
          controller: _nameController,
        ),
        _field(
          context,
          label: l10n.outfitBuilderDescription,
          hint: l10n.outfitBuilderDescriptionHint,
          controller: _descriptionController,
          maxLines: 2,
        ),
        if (_modelPath != null) ...[
          SizedBox(height: AppSizes.paddingMD(context)),
          _meshSection(context, l10n),
        ],
        SizedBox(height: AppSizes.paddingMD(context)),
        _acquisition(context, l10n),
        SizedBox(height: AppSizes.paddingMD(context)),
        _advanced(context, l10n),
        SizedBox(height: AppSizes.paddingMD(context)),
        _preview(context, l10n),
        SizedBox(height: AppSizes.paddingMD(context)),
        _actions(context, l10n),
      ],
    );
  }

  Widget _dropZone(BuildContext context, AppLocalizations l10n) {
    final path = _modelPath;
    final stem = path == null ? null : p.basenameWithoutExtension(path);

    return DropTarget(
      onDragEntered: (_) => setState(() => _dragging = true),
      onDragExited: (_) => setState(() => _dragging = false),
      onDragDone: (detail) {
        setState(() => _dragging = false);
        final dropped = detail.files.map((f) => f.path).toList();
        if (dropped.isNotEmpty) _acceptModel(dropped.first);
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
                    : path != null
                        ? AppColors.success.withValues(alpha: 0.5)
                        : AppColors.borderMedium,
              ),
            ),
            child: Row(
              children: [
                Icon(
                  path != null
                      ? Icons.view_in_ar
                      : Icons.file_download_outlined,
                  size: AppSizes.iconLG(context),
                  color: path != null
                      ? AppColors.success
                      : AppColors.textMuted,
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
                              path != null
                                  ? stem!
                                  : l10n.outfitBuilderDropTitle,
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                              style: TextStyle(
                                fontSize: AppSizes.fontSM(context),
                                fontWeight: FontWeight.w600,
                                fontFamily:
                                    path != null ? AppSizes.monoFamily : null,
                                fontFamilyFallback:
                                    path != null ? AppSizes.monoFallback : null,
                                color: path != null
                                    ? AppColors.success
                                    : AppColors.textPrimary,
                              ),
                            ),
                          ),
                          if (path != null) ...[
                            SizedBox(width: AppSizes.spacingSM(context)),
                            _characterBadge(context),
                          ],
                        ],
                      ),
                      const SizedBox(height: 2),
                      Text(
                        _dropError ??
                            (path != null
                                ? l10n.outfitBuilderCharacterDetected
                                : l10n.outfitBuilderDropHint),
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

  Widget _acquisition(BuildContext context, AppLocalizations l10n) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          l10n.outfitBuilderAcquisitionTitle,
          style: TextStyle(
            fontSize: AppSizes.fontXS(context),
            fontWeight: FontWeight.w700,
            color: AppColors.textSecondary,
            letterSpacing: 0.5,
          ),
        ),
        SizedBox(height: AppSizes.spacingSM(context)),
        _toggleRow(
          context,
          label: l10n.outfitBuilderAutoGive,
          hint: l10n.outfitBuilderAutoGiveHint,
          value: _autoGive,
          onChanged: (v) => setState(() => _autoGive = v),
        ),
        _toggleRow(
          context,
          label: l10n.outfitBuilderInShop,
          hint: l10n.outfitBuilderInShopHint,
          value: _inShop,
          onChanged: (v) => setState(() => _inShop = v),
        ),
        if (_inShop) _shopFields(context, l10n),
      ],
    );
  }

  Widget _shopFields(BuildContext context, AppLocalizations l10n) {
    return Padding(
      padding: EdgeInsets.only(
        left: AppSizes.paddingLG(context),
        top: AppSizes.spacingSM(context),
      ),
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
                menuMatchesTriggerWidth: true,
              ),
            ],
          ),
          SizedBox(height: AppSizes.spacingSM(context)),
          Text(
            l10n.outfitBuilderShopState,
            style: TextStyle(
              fontSize: AppSizes.fontSM(context),
              color: AppColors.textSecondary,
            ),
          ),
          Padding(
            padding: const EdgeInsets.only(top: 2),
            child: Text(
              l10n.outfitBuilderShopStateHint,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textMuted,
                height: 1.3,
              ),
            ),
          ),
          SizedBox(height: AppSizes.spacingSM(context)),
          for (var state = 0;
              state <= OutfitBuilderService.maxShopState;
              state++)
            _StateToggle(
              key: ValueKey(state),
              number: state,
              label: _shopStateLabel(l10n, state),
              selected: _shopStates.contains(state),
              onTap: () => setState(() {
                if (!_shopStates.remove(state)) _shopStates.add(state);
              }),
            ),
          SizedBox(height: AppSizes.spacingSM(context)),
          _AllStatesButton(
            label: l10n.outfitBuilderShopStateAll,
            allSelected:
                _shopStates.length == OutfitBuilderService.maxShopState + 1,
            onTap: () => setState(() {
              final all = OutfitBuilderService.maxShopState + 1;
              if (_shopStates.length == all) {
                _shopStates
                  ..clear()
                  ..add(0);
              } else {
                _shopStates
                  ..clear()
                  ..addAll(List.generate(all, (i) => i));
              }
            }),
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
              _stepper(
                context,
                value: _buyPrice,
                min: 1,
                max: 999999,
                step: 500,
                onChanged: (v) => setState(() => _buyPrice = v),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _toggleRow(
    BuildContext context, {
    required String label,
    required String hint,
    required bool value,
    required ValueChanged<bool> onChanged,
  }) {
    return Padding(
      padding: EdgeInsets.symmetric(vertical: AppSizes.spacingSM(context)),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
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
                    hint,
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
            value: value,
            onChanged: onChanged,
            activeThumbColor: AppColors.accentPrimary,
          ),
        ],
      ),
    );
  }

  Widget _stepper(
    BuildContext context, {
    required int value,
    required int min,
    required int max,
    required ValueChanged<int> onChanged,
    int step = 1,
  }) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        StepButton(
          icon: Icons.remove,
          onTap: value > min
              ? () => onChanged((value - step).clamp(min, max))
              : null,
        ),
        SizedBox(
          width: 64,
          child: Text(
            '$value',
            textAlign: TextAlign.center,
            style: TextStyle(
              fontSize: AppSizes.fontSM(context),
              fontWeight: FontWeight.w600,
              color: AppColors.accentPrimary,
              fontFamily: AppSizes.monoFamily,
              fontFamilyFallback: AppSizes.monoFallback,
            ),
          ),
        ),
        StepButton(
          icon: Icons.add,
          onTap: value < max
              ? () => onChanged((value + step).clamp(min, max))
              : null,
        ),
      ],
    );
  }

  String _shopStateLabel(AppLocalizations l10n, int state) {
    return switch (OutfitBuilderService.shopStateKeys[state]) {
      'routeABeforeRuinsCollapse' => l10n.shopStateRouteABeforeRuinsCollapse,
      'routeAAfterRuinsCollapse' => l10n.shopStateRouteAAfterRuinsCollapse,
      'routeBBeforeRuinsCollapse' => l10n.shopStateRouteBBeforeRuinsCollapse,
      'routeBAfterRuinsCollapse' => l10n.shopStateRouteBAfterRuinsCollapse,
      'routeC' => l10n.shopStateRouteC,
      'endingCOrD' => l10n.shopStateEndingCOrD,
      _ => l10n.shopStateEndingCAndD,
    };
  }

  Widget _characterBadge(BuildContext context) {
    return Container(
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.chipPaddingH(context),
        vertical: AppSizes.chipPaddingV(context),
      ),
      decoration: BoxDecoration(
        color: AppColors.accentPrimary.withValues(alpha: 0.15),
        borderRadius: BorderRadius.circular(999),
        border: Border.all(
          color: AppColors.accentPrimary.withValues(alpha: 0.4),
        ),
      ),
      child: Text(
        _character.tomlValue,
        style: TextStyle(
          fontSize: AppSizes.fontXS(context),
          fontWeight: FontWeight.w700,
          color: AppColors.accentPrimary,
          letterSpacing: 0.5,
        ),
      ),
    );
  }

  Widget _meshSection(BuildContext context, AppLocalizations l10n) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        OutfitMeshTuner(
          key: ValueKey(_modelPath),
          modelPath: _modelPath!,
          character: _character,
          rules: _meshRules,
          onChanged: (rules) => setState(() => _meshRules = rules),
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

  Widget _advanced(BuildContext context, AppLocalizations l10n) {
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
              ),
            ),
            SizedBox(width: AppSizes.spacingMD(context)),
            Expanded(
              child: _compactField(
                context,
                label: l10n.outfitBuilderOutfitId,
                controller: _outfitIdController,
                digitsOnly: true,
                onChanged: (text) {
                  final value = int.tryParse(text);
                  if (value != null) setState(() => _outfitId = value);
                },
              ),
            ),
            SizedBox(width: AppSizes.spacingMD(context)),
            Expanded(
              child: _compactField(
                context,
                label: l10n.outfitBuilderItemId,
                controller: _itemIdController,
                digitsOnly: true,
                onChanged: (text) {
                  final value = int.tryParse(text);
                  if (value != null) setState(() => _itemId = value);
                },
              ),
            ),
          ],
        ),
        Padding(
          padding: EdgeInsets.only(top: AppSizes.spacingSM(context)),
          child: Text(
            _idTaken ? l10n.outfitBuilderIdTaken : l10n.outfitBuilderIdHint,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: _idTaken ? AppColors.warning : AppColors.textMuted,
              height: 1.4,
            ),
          ),
        ),
        SizedBox(height: AppSizes.spacingMD(context)),
        _copyModelToggle(context, l10n),
      ],
    );
  }

  Widget _copyModelToggle(BuildContext context, AppLocalizations l10n) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                l10n.outfitBuilderCopyModel,
                style: TextStyle(
                  fontSize: AppSizes.fontSM(context),
                  color: AppColors.textPrimary,
                ),
              ),
              Padding(
                padding: const EdgeInsets.only(top: 2),
                child: Text(
                  l10n.outfitBuilderCopyModelHint,
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
          value: _copyModel,
          onChanged: (v) => setState(() => _copyModel = v),
          activeThumbColor: AppColors.accentPrimary,
        ),
      ],
    );
  }

  Widget _compactField(
    BuildContext context, {
    required String label,
    required TextEditingController controller,
    bool digitsOnly = false,
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
            keyboardType: digitsOnly ? TextInputType.number : null,
            inputFormatters: digitsOnly
                ? [FilteringTextInputFormatter.digitsOnly]
                : null,
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
    final draft = _draft;
    final itemDraft = ItemDraft(
      displayName: draft.displayName,
      description: draft.description,
      id: _itemId,
      autoGive: _autoGive,
      buyPrice: _inShop ? _buyPrice : null,
      category: OutfitBuilderService.outfitItemCategory,
    );

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
        _previewFile(
          context,
          OutfitBuilderService.fileNameFor(draft),
          OutfitBuilderService.toToml(draft),
        ),
        SizedBox(height: AppSizes.spacingSM(context)),
        _previewFile(
          context,
          ItemBuilderService.fileNameFor(itemDraft),
          ItemBuilderService.toToml(itemDraft),
        ),
        if (draft.sellsInShop) ...[
          SizedBox(height: AppSizes.spacingSM(context)),
          _previewFile(
            context,
            OutfitBuilderService.shopFileNameFor(draft),
            OutfitBuilderService.shopItemToml(draft),
          ),
        ],
      ],
    );
  }

  Widget _previewFile(BuildContext context, String fileName, String content) {
    return Container(
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
              fileName,
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
              content.trimRight(),
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
                label: l10n.outfitBuilderCreate,
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

class _StateToggle extends StatefulWidget {
  final int number;
  final String label;
  final bool selected;
  final VoidCallback onTap;

  const _StateToggle({
    super.key,
    required this.number,
    required this.label,
    required this.selected,
    required this.onTap,
  });

  @override
  State<_StateToggle> createState() => _StateToggleState();
}

class _StateToggleState extends State<_StateToggle> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final selected = widget.selected;
    final active = selected || _hovered;

    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        behavior: HitTestBehavior.opaque,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          margin: const EdgeInsets.only(bottom: 3),
          padding: EdgeInsets.symmetric(
            horizontal: AppSizes.paddingSM(context),
            vertical: AppSizes.paddingXS(context),
          ),
          decoration: BoxDecoration(
            color: selected
                ? AppColors.accentPrimary.withValues(alpha: 0.12)
                : _hovered
                ? AppColors.surfaceLight
                : Colors.transparent,
            borderRadius: BorderRadius.circular(4),
            border: Border.all(
              color: selected
                  ? AppColors.accentPrimary.withValues(alpha: 0.5)
                  : Colors.transparent,
            ),
          ),
          child: Row(
            children: [
              Icon(
                selected ? Icons.check_box : Icons.check_box_outline_blank,
                size: AppSizes.iconSM(context),
                color: selected
                    ? AppColors.accentPrimary
                    : AppColors.textMuted,
              ),
              SizedBox(width: AppSizes.spacingSM(context)),
              Text(
                '${widget.number}',
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  fontWeight: FontWeight.w700,
                  color: active
                      ? AppColors.accentPrimary
                      : AppColors.textMuted,
                  fontFamily: AppSizes.monoFamily,
                  fontFamilyFallback: AppSizes.monoFallback,
                ),
              ),
              SizedBox(width: AppSizes.spacingSM(context)),
              Expanded(
                child: Text(
                  widget.label,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: selected
                        ? AppColors.textPrimary
                        : AppColors.textMuted,
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

class _AllStatesButton extends StatefulWidget {
  final String label;
  final bool allSelected;
  final VoidCallback onTap;

  const _AllStatesButton({
    required this.label,
    required this.allSelected,
    required this.onTap,
  });

  @override
  State<_AllStatesButton> createState() => _AllStatesButtonState();
}

class _AllStatesButtonState extends State<_AllStatesButton> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        behavior: HitTestBehavior.opaque,
        child: Padding(
          padding: EdgeInsets.symmetric(
            horizontal: AppSizes.paddingSM(context),
            vertical: AppSizes.paddingXS(context),
          ),
          child: Text(
            widget.label,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              fontWeight: FontWeight.w700,
              color: _hovered
                  ? AppColors.accentPrimary
                  : AppColors.textSecondary,
              decoration: TextDecoration.underline,
              decorationColor: AppColors.accentPrimary.withValues(alpha: 0.4),
            ),
          ),
        ),
      ),
    );
  }
}
