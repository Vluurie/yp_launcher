import 'dart:io';
import 'package:flutter/material.dart';
import 'package:path/path.dart' as p;
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

  Future<void> _reveal() async {
    final path = widget.revealPath;
    if (path.isEmpty) return;
    if (Platform.isWindows) {
      if (widget.isFile) {
        await Process.run('explorer', ['/select,', path]);
      } else {
        await Process.run('explorer', [path]);
      }
    } else if (Platform.isMacOS) {
      if (widget.isFile) {
        await Process.run('open', ['-R', path]);
      } else {
        await Process.run('open', [path]);
      }
    } else {
      final dir = widget.isFile ? p.dirname(path) : path;
      await Process.run('xdg-open', [dir]);
    }
  }

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
