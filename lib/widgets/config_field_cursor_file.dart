import 'dart:io';

import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/services/wine/wine_paths.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/hover_button.dart';

class ConfigFieldCursorFile extends StatefulWidget {
  final String label;
  final String value;
  final ValueChanged<String> onChanged;
  final String? tooltip;

  const ConfigFieldCursorFile({
    super.key,
    required this.label,
    required this.value,
    required this.onChanged,
    this.tooltip,
  });

  static bool isValidCursorFile(String path) {
    final file = File(path);
    if (!file.existsSync()) return false;
    final ext = p.extension(path).toLowerCase();
    if (ext != '.cur' && ext != '.ani') return false;
    final List<int> header;
    try {
      final raf = file.openSync();
      try {
        header = raf.readSync(4);
      } finally {
        raf.closeSync();
      }
    } catch (_) {
      return false;
    }
    if (header.length < 4) return false;
    if (ext == '.cur') {
      return header[0] == 0x00 &&
          header[1] == 0x00 &&
          header[2] == 0x02 &&
          header[3] == 0x00;
    }
    return header[0] == 0x52 &&
        header[1] == 0x49 &&
        header[2] == 0x46 &&
        header[3] == 0x46;
  }

  @override
  State<ConfigFieldCursorFile> createState() => _ConfigFieldCursorFileState();
}

class _ConfigFieldCursorFileState extends State<ConfigFieldCursorFile> {
  bool _invalid = false;

  Future<void> _pick() async {
    final result = await FilePicker.pickFiles(
      type: FileType.custom,
      allowedExtensions: ['cur', 'ani'],
    );
    final path = result?.files.single.path;
    if (path == null) return;
    final absolute = p.normalize(p.absolute(path));
    if (!ConfigFieldCursorFile.isValidCursorFile(absolute)) {
      setState(() => _invalid = true);
      return;
    }
    setState(() => _invalid = false);
    widget.onChanged(Platform.isWindows ? absolute : toWinePath(absolute));
  }

  String _displayName(String storedPath) {
    final lastSep = storedPath.lastIndexOf(RegExp(r'[\\/]'));
    return lastSep == -1 ? storedPath : storedPath.substring(lastSep + 1);
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final hasFile = widget.value.isNotEmpty;

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4, horizontal: 8),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            widget.label,
            style: TextStyle(
              fontSize: AppSizes.fontMD(context),
              color: AppColors.textPrimary,
            ),
          ),
          if (widget.tooltip != null)
            Padding(
              padding: const EdgeInsets.only(top: 3),
              child: Text(
                widget.tooltip!,
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  color: AppColors.textMuted,
                  height: 1.3,
                ),
              ),
            ),
          const SizedBox(height: 6),
          Row(
            children: [
              Expanded(
                child: Text(
                  hasFile ? _displayName(widget.value) : l10n.keybindUnbound,
                  overflow: TextOverflow.ellipsis,
                  style: TextStyle(
                    fontSize: AppSizes.fontSM(context),
                    color: hasFile
                        ? AppColors.textSecondary
                        : AppColors.textMuted,
                    fontStyle: hasFile ? FontStyle.normal : FontStyle.italic,
                  ),
                ),
              ),
              const SizedBox(width: 8),
              HoverButton(
                label: l10n.naiomCursorPick,
                color: AppColors.accentPrimary,
                onTap: _pick,
              ),
              if (hasFile) ...[
                const SizedBox(width: 4),
                HoverButton(
                  label: l10n.naiomCursorClear,
                  color: AppColors.textMuted,
                  onTap: () {
                    setState(() => _invalid = false);
                    widget.onChanged('');
                  },
                ),
              ],
            ],
          ),
          if (_invalid)
            Padding(
              padding: const EdgeInsets.only(top: 4),
              child: Row(
                children: [
                  Icon(
                    Icons.error_outline,
                    size: 13,
                    color: AppColors.error,
                  ),
                  const SizedBox(width: 4),
                  Expanded(
                    child: Text(
                      l10n.naiomCursorInvalid,
                      style: TextStyle(
                        fontSize: AppSizes.fontXS(context),
                        color: AppColors.error,
                      ),
                    ),
                  ),
                ],
              ),
            ),
        ],
      ),
    );
  }
}
