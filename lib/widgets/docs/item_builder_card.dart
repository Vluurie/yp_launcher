import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/services/docs/item_builder_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/hover_button.dart';
import 'package:yp_launcher/services/docs/thumbnail_service.dart';
import 'package:yp_launcher/services/rust_error_codes.dart';
import 'package:yp_launcher/src/rust/api/texture.dart';
import 'package:yp_launcher/widgets/docs/thumbnail_dialog.dart';

class ItemBuilderCard extends ConsumerStatefulWidget {
  const ItemBuilderCard({super.key});

  @override
  ConsumerState<ItemBuilderCard> createState() => _ItemBuilderCardState();
}

class _ItemBuilderCardState extends ConsumerState<ItemBuilderCard> {
  final _nameController = TextEditingController();
  final _descriptionController = TextEditingController();
  final _modIdController = TextEditingController(text: 'my_mod');
  final _idController = TextEditingController(
    text: '${ItemBuilderService.firstFreeId}',
  );

  Set<int> _takenIds = const {};
  bool get _idTaken => _takenIds.contains(_id);

  int _id = ItemBuilderService.firstFreeId;
  int _carry = 1;
  bool _sellable = false;
  int _sellPrice = 100;
  bool _autoGive = true;

  String? _writtenPath;
  String? _error;
  bool _busy = false;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => _pickId());
  }

  @override
  void dispose() {
    _nameController.dispose();
    _descriptionController.dispose();
    _modIdController.dispose();
    _idController.dispose();
    super.dispose();
  }

  String get _gameDir =>
      ref.read(appStateControllerProvider).selectedDirectory;

  Future<void> _pickId() async {
    final dir = _gameDir;
    if (dir.isEmpty) return;
    final taken = await ItemBuilderService.takenIds(dir);
    final next = await ItemBuilderService.nextFreeId(dir);
    if (!mounted) return;
    setState(() {
      _takenIds = taken;
      _id = next;
      _idController.text = '$next';
    });
  }

  ItemDraft get _draft => ItemDraft(
        displayName: _nameController.text.trim(),
        description: _descriptionController.text.trim(),
        id: _id,
        maxCarry: _carry,
        sellable: _sellable,
        sellPrice: _sellPrice,
        autoGive: _autoGive,
        misctexName: _misctexName,
      );

  ThumbnailChoice? _iconChoice;

  String? get _misctexName => _iconChoice == null
      ? null
      : ThumbnailService.textureNameFor(_id);

  Future<void> _create() async {
    final l10n = AppLocalizations.of(context)!;
    final dir = _gameDir;

    if (dir.isEmpty) {
      setState(() => _error = l10n.itemBuilderNeedsGameDir);
      return;
    }
    if (_nameController.text.trim().isEmpty) {
      setState(() => _error = l10n.itemBuilderNeedsName);
      return;
    }

    setState(() {
      _busy = true;
      _error = null;
      _writtenPath = null;
    });

    try {
      final taken = await ItemBuilderService.takenIds(dir);
      if (taken.contains(_id)) {
        if (!mounted) return;
        setState(() {
          _busy = false;
          _takenIds = taken;
          _error = l10n.itemBuilderExists;
        });
        return;
      }

      final modId = _modIdController.text.trim().isEmpty
          ? 'my_mod'
          : _modIdController.text.trim();

      final icon = _iconChoice;
      if (icon != null) {
        await ThumbnailService.create(
          sourceImagePath: icon.imagePath,
          gameDir: dir,
          modId: modId,
          itemId: _id,
          width: icon.width,
          height: icon.height,
        );
      }

      final file = await ItemBuilderService.write(
        gameDir: dir,
        modId: modId,
        draft: _draft,
      );
      if (!mounted) return;
      setState(() {
        _busy = false;
        _writtenPath = file.path;
        _iconChoice = null;
      });
      await _pickId();
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
          _field(
            context,
            label: l10n.itemBuilderName,
            hint: l10n.itemBuilderNameHint,
            controller: _nameController,
          ),
          _field(
            context,
            label: l10n.itemBuilderDescription,
            hint: l10n.itemBuilderDescriptionHint,
            controller: _descriptionController,
            maxLines: 2,
          ),

          SizedBox(height: AppSizes.spacingMD(context)),
          _carryRow(context, l10n),
          _toggle(
            context,
            label: l10n.itemBuilderAutoGive,
            hint: l10n.itemBuilderAutoGiveHint,
            value: _autoGive,
            onChanged: (v) => setState(() => _autoGive = v),
          ),
          _toggle(
            context,
            label: l10n.itemBuilderSellable,
            value: _sellable,
            onChanged: (v) => setState(() => _sellable = v),
          ),
          if (_sellable) _sellPriceRow(context, l10n),

          SizedBox(height: AppSizes.paddingMD(context)),
          if (_gameDir.isNotEmpty) _iconRow(context, l10n),

          SizedBox(height: AppSizes.paddingMD(context)),
          _advanced(context, l10n),

          SizedBox(height: AppSizes.paddingMD(context)),
          _preview(context, l10n),

          SizedBox(height: AppSizes.paddingMD(context)),
          _actions(context, l10n),
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

  Widget _carryRow(BuildContext context, AppLocalizations l10n) {
    return Padding(
      padding: EdgeInsets.symmetric(vertical: AppSizes.spacingSM(context)),
      child: Row(
        children: [
          Expanded(
            child: Text(
              l10n.itemBuilderCarry,
              style: TextStyle(
                fontSize: AppSizes.fontSM(context),
                color: AppColors.textPrimary,
              ),
            ),
          ),
          _stepper(
            context,
            value: _carry,
            min: 1,
            max: 99,
            onChanged: (v) => setState(() => _carry = v),
          ),
        ],
      ),
    );
  }

  Widget _sellPriceRow(BuildContext context, AppLocalizations l10n) {
    return Padding(
      padding: EdgeInsets.only(
        left: AppSizes.paddingLG(context),
        top: AppSizes.spacingSM(context),
        bottom: AppSizes.spacingSM(context),
      ),
      child: Row(
        children: [
          Expanded(
            child: Text(
              l10n.itemBuilderSellPrice,
              style: TextStyle(
                fontSize: AppSizes.fontSM(context),
                color: AppColors.textSecondary,
              ),
            ),
          ),
          _stepper(
            context,
            value: _sellPrice,
            min: 1,
            max: 99999,
            step: 50,
            onChanged: (v) => setState(() => _sellPrice = v),
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
        _stepButton(
          context,
          icon: Icons.remove,
          onTap: value > min
              ? () => onChanged((value - step).clamp(min, max))
              : null,
        ),
        SizedBox(
          width: 56,
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
        _stepButton(
          context,
          icon: Icons.add,
          onTap: value < max
              ? () => onChanged((value + step).clamp(min, max))
              : null,
        ),
      ],
    );
  }

  Widget _stepButton(
    BuildContext context, {
    required IconData icon,
    required VoidCallback? onTap,
  }) {
    return _StepButton(icon: icon, onTap: onTap);
  }

  Widget _toggle(
    BuildContext context, {
    required String label,
    required bool value,
    required ValueChanged<bool> onChanged,
    String? hint,
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
                if (hint != null)
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

  Future<void> _openIconDialog() async {
    final choice = await ThumbnailDialog.show(
      context,
      subtitle: '${ThumbnailService.misctexNameFor(_id)}.dat / .dtt',
      initial: _iconChoice,
    );
    if (choice != null && mounted) {
      setState(() => _iconChoice = choice);
    }
  }

  void _removeIcon() {
    setState(() => _iconChoice = null);
  }

  Widget _iconRow(BuildContext context, AppLocalizations l10n) {
    final hasIcon = _misctexName != null;

    return Padding(
      padding: EdgeInsets.symmetric(vertical: AppSizes.spacingSM(context)),
      child: Row(
        children: [
          Icon(
            hasIcon ? Icons.image : Icons.image_outlined,
            size: AppSizes.iconMD(context),
            color: hasIcon ? AppColors.success : AppColors.textMuted,
          ),
          SizedBox(width: AppSizes.spacingMD(context)),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  l10n.thumbnailTitle,
                  style: TextStyle(
                    fontSize: AppSizes.fontSM(context),
                    color: AppColors.textPrimary,
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.only(top: 2),
                  child: Text(
                    hasIcon
                        ? '${_iconChoice!.width}×${_iconChoice!.height}  ·  '
                            '${l10n.thumbnailPending}'
                        : l10n.thumbnailDropHint,
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      color: hasIcon
                          ? AppColors.success
                          : AppColors.textMuted,
                      height: 1.3,
                    ),
                  ),
                ),
              ],
            ),
          ),
          SizedBox(width: AppSizes.spacingMD(context)),
          HoverButton(
            label: hasIcon ? l10n.thumbnailChange : l10n.thumbnailOpen,
            color: AppColors.accentPrimary,
            onTap: _openIconDialog,
          ),
          if (hasIcon) ...[
            SizedBox(width: AppSizes.spacingSM(context)),
            HoverButton(
              label: l10n.thumbnailRemove,
              color: AppColors.textMuted,
              onTap: _removeIcon,
            ),
          ],
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
                label: l10n.itemBuilderModId,
                controller: _modIdController,
              ),
            ),
            SizedBox(width: AppSizes.spacingMD(context)),
            Expanded(
              child: _compactField(
                context,
                label: l10n.itemBuilderIdLabel,
                controller: _idController,
                digitsOnly: true,
                onChanged: (text) {
                  final value = int.tryParse(text);
                  if (value != null) setState(() => _id = value);
                },
              ),
            ),
          ],
        ),
        Padding(
          padding: EdgeInsets.only(top: AppSizes.spacingSM(context)),
          child: Text(
            _idTaken ? l10n.itemBuilderExists : l10n.itemBuilderIdHint,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: _idTaken ? AppColors.warning : AppColors.textMuted,
              height: 1.4,
            ),
          ),
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
              '${l10n.itemBuilderPreview}  ',
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textMuted,
              ),
            ),
            Text(
              ItemBuilderService.fileNameFor(_draft),
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.accentPrimary,
                fontFamily: AppSizes.monoFamily,
                fontFamilyFallback: AppSizes.monoFallback,
              ),
            ),
          ],
        ),
        const SizedBox(height: 6),
        Container(
          width: double.infinity,
          padding: EdgeInsets.all(AppSizes.paddingMD(context)),
          decoration: BoxDecoration(
            color: AppColors.surfaceLight,
            borderRadius: BorderRadius.circular(6),
            border: Border.all(color: AppColors.borderLight),
          ),
          child: SelectableText(
            ItemBuilderService.toToml(_draft),
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
                label: l10n.itemBuilderCreate,
                color: AppColors.success,
                onTap: _create,
              ),
            if (_writtenPath != null) ...[
              SizedBox(width: AppSizes.spacingMD(context)),
              HoverButton(
                label: l10n.itemBuilderOpenFolder,
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
                    l10n.itemBuilderCreated(_writtenPath!),
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

class _StepButton extends StatefulWidget {
  final IconData icon;
  final VoidCallback? onTap;

  const _StepButton({required this.icon, required this.onTap});

  @override
  State<_StepButton> createState() => _StepButtonState();
}

class _StepButtonState extends State<_StepButton> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final enabled = widget.onTap != null;
    final active = enabled && _hovered;

    return MouseRegion(
      cursor: enabled ? SystemMouseCursors.click : SystemMouseCursors.basic,
      onEnter: (_) {
        if (enabled) setState(() => _hovered = true);
      },
      onExit: (_) {
        if (_hovered) setState(() => _hovered = false);
      },
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          padding: const EdgeInsets.all(4),
          decoration: BoxDecoration(
            color: active
                ? AppColors.accentPrimary.withValues(alpha: 0.15)
                : AppColors.surfaceLight,
            borderRadius: BorderRadius.circular(4),
            border: Border.all(
              color: active ? AppColors.accentPrimary : AppColors.borderLight,
            ),
          ),
          child: Icon(
            widget.icon,
            size: AppSizes.iconSM(context),
            color: !enabled
                ? AppColors.textMuted
                : active
                    ? AppColors.accentPrimary
                    : AppColors.textSecondary,
          ),
        ),
      ),
    );
  }
}
