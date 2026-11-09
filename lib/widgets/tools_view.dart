import 'dart:async';

import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/services/docs/thumbnail_service.dart';
import 'package:yp_launcher/services/rust_error_codes.dart';
import 'package:yp_launcher/widgets/collapsible_card.dart';
import 'package:yp_launcher/widgets/hover_button.dart';
import 'package:yp_launcher/src/rust/api/texture.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/docs/thumbnail_dialog.dart';
import 'package:yp_launcher/widgets/icon_name_dialog.dart';

class ToolsView extends ConsumerStatefulWidget {
  const ToolsView({super.key});

  @override
  ConsumerState<ToolsView> createState() => _ToolsViewState();
}

class _ToolsViewState extends ConsumerState<ToolsView> {
  final _scrollController = ScrollController();

  String? _error;
  String? _busyTexture;

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  String get _gameDir =>
      ref.read(appStateControllerProvider).selectedDirectory;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final gameDir = ref.watch(appStateControllerProvider).selectedDirectory;

    return Container(
      color: AppColors.backgroundPrimary,
      child: Column(
        children: [
          _header(context, l10n),
          Expanded(
            child: gameDir.isEmpty
                ? _needsGameDir(context, l10n)
                : _content(context, l10n),
          ),
        ],
      ),
    );
  }

  Widget _header(BuildContext context, AppLocalizations l10n) {
    return Container(
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.cardPaddingH(context),
        vertical: AppSizes.cardPaddingV(context),
      ),
      decoration: BoxDecoration(
        color: AppColors.surfaceMedium,
        border: Border(bottom: BorderSide(color: AppColors.borderLight)),
      ),
      child: Row(
        children: [
          Text(
            l10n.tabTools.toUpperCase(),
            style: TextStyle(
              fontSize: AppSizes.fontXL(context),
              fontWeight: FontWeight.bold,
              color: AppColors.textPrimary,
              letterSpacing: 1.0,
            ),
          ),
        ],
      ),
    );
  }

  Widget _needsGameDir(BuildContext context, AppLocalizations l10n) {
    return Center(
      child: Padding(
        padding: EdgeInsets.all(AppSizes.contentPadding(context)),
        child: Text(
          l10n.toolsNeedsGameDir,
          textAlign: TextAlign.center,
          style: TextStyle(
            fontSize: AppSizes.fontSM(context),
            color: AppColors.warning,
          ),
        ),
      ),
    );
  }

  Widget _content(BuildContext context, AppLocalizations l10n) {
    return Scrollbar(
      controller: _scrollController,
      thumbVisibility: true,
      child: SingleChildScrollView(
        controller: _scrollController,
        padding: EdgeInsets.all(AppSizes.contentPadding(context)),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (_error != null)
              Padding(
                padding: EdgeInsets.only(bottom: AppSizes.paddingMD(context)),
                child: Text(
                  _error!,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.error,
                    height: 1.4,
                  ),
                ),
              ),
            CollapsibleCard(
              title: l10n.toolsIconsTitle,
              children: [_iconsSection(context, l10n)],
            ),
          ],
        ),
      ),
    );
  }

  Widget _iconsSection(BuildContext context, AppLocalizations l10n) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Expanded(
              child: Text(
                l10n.toolsIconsDesc,
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  color: AppColors.textMuted,
                  height: 1.5,
                ),
              ),
            ),
            SizedBox(width: AppSizes.spacingSM(context)),
            Tooltip(
              message: l10n.toolsIconsHotReload,
              child: Icon(
                Icons.bolt_outlined,
                size: AppSizes.iconSM(context),
                color: AppColors.success,
              ),
            ),
          ],
        ),
        SizedBox(height: AppSizes.paddingMD(context)),
        Row(
          children: [
            if (_busyTexture != null)
              SizedBox(
                width: AppSizes.iconMD(context),
                height: AppSizes.iconMD(context),
                child: CircularProgressIndicator(
                  strokeWidth: 2,
                  color: AppColors.accentPrimary,
                ),
              )
            else ...[
              HoverButton(
                label: l10n.toolsIconsConvert,
                color: AppColors.accentPrimary,
                onTap: _addIcon,
              ),
              SizedBox(width: AppSizes.spacingMD(context)),
              HoverButton(
                label: l10n.toolsIconsAdjust,
                color: AppColors.accentPrimary,
                onTap: _adjustIcon,
              ),
            ],
          ],
        ),
      ],
    );
  }


  Future<void> _adjustIcon() async {
    final l10n = AppLocalizations.of(context)!;

    final result = await FilePicker.pickFiles(
      type: FileType.custom,
      allowedExtensions: const ['dat'],
    );
    final path = result?.files.single.path;
    if (path == null || !mounted) return;

    final name = p.basenameWithoutExtension(path);
    if (!name.startsWith('misctex_')) {
      setState(() => _error = l10n.toolsIconsNotMisctex);
      return;
    }

    final textureName = name.substring('misctex_'.length);
    final choice = await ThumbnailDialog.show(
      context,
      subtitle: '$name.dat / .dtt',
    );
    if (choice == null || !mounted) return;

    setState(() {
      _busyTexture = textureName;
      _error = null;
    });

    try {
      await ThumbnailService.createNamed(
        sourceImagePath: choice.imagePath,
        outputDir: p.dirname(path),
        textureName: textureName,
        width: choice.width,
        height: choice.height,
      );
      if (!mounted) return;
      setState(() => _busyTexture = null);
    } on MiscTexError catch (e) {
      if (!mounted) return;
      setState(() {
        _busyTexture = null;
        _error = RustErrorCodes.message(l10n, e.code, e.detail);
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _busyTexture = null;
        _error = '$e';
      });
    }
  }

  Future<void> _addIcon() async {
    final dir = _gameDir;
    if (dir.isEmpty) return;

    final target = await IconNameDialog.show(context, gameDir: dir);
    if (target == null || !mounted) return;

    final choice = await ThumbnailDialog.show(
      context,
      subtitle: 'misctex_${target.textureName}  ·  ${target.modId}',
    );
    if (choice == null || !mounted) return;

    setState(() {
      _busyTexture = target.textureName;
      _error = null;
    });

    try {
      await ThumbnailService.createNamed(
        sourceImagePath: choice.imagePath,
        outputDir: target.directory,
        textureName: target.textureName,
        width: choice.width,
        height: choice.height,
      );
      if (!mounted) return;
      setState(() => _busyTexture = null);
    } on MiscTexError catch (e) {
      if (!mounted) return;
      final l10n = AppLocalizations.of(context)!;
      setState(() {
        _busyTexture = null;
        _error = RustErrorCodes.message(l10n, e.code, e.detail);
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _busyTexture = null;
        _error = '$e';
      });
    }
  }

}
