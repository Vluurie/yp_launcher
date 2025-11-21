import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:file_picker/file_picker.dart';
import 'package:automato_theme/automato_theme.dart';
import 'package:yp_launcher/services/settings_service.dart';

class ConfigDialog extends ConsumerStatefulWidget {
  const ConfigDialog({super.key});

  @override
  ConsumerState<ConfigDialog> createState() => _ConfigDialogState();
}

class _ConfigDialogState extends ConsumerState<ConfigDialog> {
  String? _launcherExePath;
  String? _modloaderDllPath;
  String? _yorhaDllPath;
  bool _isLoading = true;

  @override
  void initState() {
    super.initState();
    _loadSettings();
  }

  Future<void> _loadSettings() async {
    final settings = await SettingsService.initialize();
    setState(() {
      _launcherExePath = settings.launcherExeOverride;
      _modloaderDllPath = settings.modloaderDllOverride;
      _yorhaDllPath = settings.yorhaDllOverride;
      _isLoading = false;
    });
  }

  @override
  Widget build(BuildContext context) {
    if (_isLoading) {
      return Dialog(
        backgroundColor: AutomatoThemeColors.darkBrown(ref),
        child: Container(
          width: 200,
          height: 200,
          alignment: Alignment.center,
          child: AutomatoLoading(
            color: AutomatoThemeColors.primaryColor(ref),
            translateX: 0,
            svgString: AutomatoSvgStrings.automatoSvgStrHead,
          ),
        ),
      );
    }

    return Dialog(
      backgroundColor: AutomatoThemeColors.darkBrown(ref),
      child: Container(
        width: 700,
        padding: const EdgeInsets.all(24.0),
        decoration: BoxDecoration(
          color: AutomatoThemeColors.darkBrown(ref),
          borderRadius: BorderRadius.circular(12.0),
          border: Border.all(
            color: AutomatoThemeColors.primaryColor(ref).withValues(alpha: 0.3),
            width: 2,
          ),
        ),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Configuration',
              style: TextStyle(
                fontSize: 24,
                fontWeight: FontWeight.bold,
                color: AutomatoThemeColors.bright(ref),
              ),
            ),
            const SizedBox(height: 8),
            Text(
              'Override mod files (optional)',
              style: TextStyle(
                fontSize: 14,
                color: AutomatoThemeColors.primaryColor(ref).withValues(alpha: 0.7),
              ),
            ),
            const SizedBox(height: 24),
            _buildPathSelector(
              label: 'Launcher Executable',
              path: _launcherExePath,
              hint: 'launch_nier.exe',
              onSelect: () => _selectFile(
                'Select Launcher Executable',
                ['exe'],
                (path) => setState(() => _launcherExePath = path),
              ),
              onClear: () => setState(() => _launcherExePath = null),
            ),
            const SizedBox(height: 16),
            _buildPathSelector(
              label: 'Modloader DLL',
              path: _modloaderDllPath,
              hint: 'modloader.dll',
              onSelect: () => _selectFile(
                'Select Modloader DLL',
                ['dll'],
                (path) => setState(() => _modloaderDllPath = path),
              ),
              onClear: () => setState(() => _modloaderDllPath = null),
            ),
            const SizedBox(height: 16),
            _buildPathSelector(
              label: 'YorHa Protocol DLL',
              path: _yorhaDllPath,
              hint: 'yorha_protocol.dll',
              onSelect: () => _selectFile(
                'Select YorHa Protocol DLL',
                ['dll'],
                (path) => setState(() => _yorhaDllPath = path),
              ),
              onClear: () => setState(() => _yorhaDllPath = null),
            ),
            const SizedBox(height: 24),
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                TextButton(
                  onPressed: () => Navigator.of(context).pop(),
                  child: Text(
                    'CANCEL',
                    style: TextStyle(
                      color: AutomatoThemeColors.primaryColor(ref),
                    ),
                  ),
                ),
                const SizedBox(width: 16),
                ElevatedButton(
                  onPressed: _saveSettings,
                  style: ElevatedButton.styleFrom(
                    backgroundColor: AutomatoThemeColors.primaryColor(ref),
                    foregroundColor: AutomatoThemeColors.darkBrown(ref),
                    padding: const EdgeInsets.symmetric(
                      horizontal: 24.0,
                      vertical: 12.0,
                    ),
                  ),
                  child: const Text('SAVE'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildPathSelector({
    required String label,
    required String? path,
    required String hint,
    required VoidCallback onSelect,
    required VoidCallback onClear,
  }) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          label,
          style: TextStyle(
            fontSize: 14,
            fontWeight: FontWeight.bold,
            color: AutomatoThemeColors.bright(ref),
          ),
        ),
        const SizedBox(height: 8),
        Row(
          children: [
            Expanded(
              child: Container(
                padding: const EdgeInsets.symmetric(
                  horizontal: 12.0,
                  vertical: 8.0,
                ),
                decoration: BoxDecoration(
                  color: Colors.black.withValues(alpha: 0.3),
                  borderRadius: BorderRadius.circular(8.0),
                  border: Border.all(
                    color: AutomatoThemeColors.primaryColor(ref)
                        .withValues(alpha: 0.3),
                    width: 1,
                  ),
                ),
                child: Text(
                  path ?? hint,
                  style: TextStyle(
                    color: path != null
                        ? AutomatoThemeColors.bright(ref)
                        : AutomatoThemeColors.primaryColor(ref)
                            .withValues(alpha: 0.5),
                    fontSize: 12,
                  ),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
            ),
            const SizedBox(width: 8),
            IconButton(
              onPressed: onSelect,
              icon: Icon(
                Icons.folder_open,
                color: AutomatoThemeColors.primaryColor(ref),
              ),
              tooltip: 'Select file',
            ),
            if (path != null)
              IconButton(
                onPressed: onClear,
                icon: Icon(
                  Icons.clear,
                  color: AutomatoThemeColors.dangerZone(ref),
                ),
                tooltip: 'Clear',
              ),
          ],
        ),
      ],
    );
  }

  Future<void> _selectFile(
    String title,
    List<String> extensions,
    Function(String) onSelected,
  ) async {
    final result = await FilePicker.platform.pickFiles(
      dialogTitle: title,
      type: FileType.custom,
      allowedExtensions: extensions,
    );

    if (result != null && result.files.single.path != null) {
      final path = result.files.single.path!;
      if (await File(path).exists()) {
        onSelected(path);
      }
    }
  }

  Future<void> _saveSettings() async {
    final settings = await SettingsService.initialize();
    await settings.setLauncherExeOverride(_launcherExePath);
    await settings.setModloaderDllOverride(_modloaderDllPath);
    await settings.setYorhaDllOverride(_yorhaDllPath);

    if (mounted) {
      Navigator.of(context).pop();
    }
  }
}
