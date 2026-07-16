import 'package:flutter/material.dart';
import 'package:yp_launcher/services/reveal_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class HeaderInfoIcon extends StatefulWidget {
  final String tooltip;
  final String revealPath;
  final bool isFile;

  const HeaderInfoIcon({
    super.key,
    required this.tooltip,
    required this.revealPath,
    this.isFile = true,
  });

  @override
  State<HeaderInfoIcon> createState() => _HeaderInfoIconState();
}

class _HeaderInfoIconState extends State<HeaderInfoIcon> {
  bool _hovered = false;

  Future<void> _reveal() =>
      revealInFileManager(widget.revealPath, isFile: widget.isFile);

  @override
  Widget build(BuildContext context) {
    return Tooltip(
      message: widget.tooltip,
      child: MouseRegion(
        cursor: SystemMouseCursors.click,
        onEnter: (_) => setState(() => _hovered = true),
        onExit: (_) => setState(() => _hovered = false),
        child: GestureDetector(
          onTap: _reveal,
          child: Padding(
            padding: EdgeInsets.only(left: AppSizes.spacingMD(context)),
            child: Icon(
              Icons.info_outline,
              size: AppSizes.iconSM(context),
              color: _hovered ? AppColors.accentPrimary : AppColors.textMuted,
            ),
          ),
        ),
      ),
    );
  }
}
