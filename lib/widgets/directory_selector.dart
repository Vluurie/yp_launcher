import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:file_picker/file_picker.dart';
import 'package:automato_theme/automato_theme.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/services/logging_service.dart';

class DirectorySelector extends ConsumerWidget {
  const DirectorySelector({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final appState = ref.watch(appStateControllerProvider);
    final controller = ref.read(appStateControllerProvider.notifier);

    return Container(
      padding: const EdgeInsets.all(24.0),
      decoration: BoxDecoration(
        color: AutomatoThemeColors.darkBrown(ref).withValues(alpha: 0.3),
        borderRadius: BorderRadius.circular(12.0),
        border: Border.all(
          color: AutomatoThemeColors.primaryColor(ref).withValues(alpha: 0.3),
          width: 2,
        ),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            'NieR:Automata Executable',
            style: TextStyle(
              fontSize: 18,
              fontWeight: FontWeight.bold,
              color: AutomatoThemeColors.bright(ref),
            ),
          ),
          const SizedBox(height: 16),
          Row(
            children: [
              Expanded(
                child: Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 16.0,
                    vertical: 12.0,
                  ),
                  decoration: BoxDecoration(
                    color: AutomatoThemeColors.darkBrown(ref),
                    borderRadius: BorderRadius.circular(8.0),
                    border: Border.all(
                      color: appState.isDirectorySelected
                          ? AutomatoThemeColors.saveZone(ref)
                          : AutomatoThemeColors.primaryColor(ref)
                              .withValues(alpha: 0.5),
                      width: 2,
                    ),
                  ),
                  child: Text(
                    appState.isDirectorySelected
                        ? appState.selectedDirectory
                        : 'No file selected',
                    style: TextStyle(
                      color: appState.isDirectorySelected
                          ? AutomatoThemeColors.bright(ref)
                          : AutomatoThemeColors.primaryColor(ref)
                              .withValues(alpha: 0.5),
                      fontSize: 14,
                    ),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
              ),
              const SizedBox(width: 16),
              ElevatedButton(
                onPressed: () => _selectDirectory(controller),
                style: ElevatedButton.styleFrom(
                  backgroundColor: AutomatoThemeColors.primaryColor(ref),
                  foregroundColor: AutomatoThemeColors.darkBrown(ref),
                  padding: const EdgeInsets.symmetric(
                    horizontal: 24.0,
                    vertical: 16.0,
                  ),
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(8.0),
                  ),
                ),
                child: const Text(
                  'SELECT',
                  style: TextStyle(
                    fontSize: 16,
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ),
            ],
          ),
          if (appState.errorMessage != null) ...[
            const SizedBox(height: 12),
            Container(
              padding: const EdgeInsets.all(12.0),
              decoration: BoxDecoration(
                color: AutomatoThemeColors.dangerZone(ref).withValues(alpha: 0.2),
                borderRadius: BorderRadius.circular(8.0),
                border: Border.all(
                  color: AutomatoThemeColors.dangerZone(ref),
                  width: 1,
                ),
              ),
              child: Row(
                children: [
                  Icon(
                    Icons.error_outline,
                    color: AutomatoThemeColors.dangerZone(ref),
                    size: 20,
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      appState.errorMessage!,
                      style: TextStyle(
                        color: AutomatoThemeColors.dangerZone(ref),
                        fontSize: 14,
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ],
      ),
    );
  }

  Future<void> _selectDirectory(AppStateController controller) async {
    try {
      await LoggingService.log('Opening file picker for NieR:Automata executable...');
      final result = await FilePicker.platform.pickFiles(
        dialogTitle: 'Select NieR:Automata Executable',
        type: FileType.custom,
        allowedExtensions: ['exe'],
      );

      if (result != null && result.files.single.path != null) {
        final filePath = result.files.single.path!;
        await LoggingService.log('Selected file: $filePath');
        final file = File(filePath);

        if (await file.exists()) {
          await LoggingService.log('Validating NieR:Automata executable...');
          // Validate in background isolate
          final isValid = await compute(_validateNierExeInIsolate, filePath);

          if (isValid) {
            // Get the directory containing the executable
            final directory = file.parent.path;
            await LoggingService.log('Validation successful, setting directory: $directory');
            await controller.setDirectory(directory);
          } else {
            await LoggingService.log('Validation failed: Not a NieR:Automata executable');
            controller.setError(
              'This is not NieR:Automata executable',
            );
          }
        } else {
          await LoggingService.log('Error: Selected file does not exist');
          controller.setError(
            'Selected file does not exist',
          );
        }
      } else {
        await LoggingService.log('File picker cancelled or no file selected');
      }
    } catch (e) {
      await LoggingService.logError('Failed to select file', e);
      controller.setError('Failed to select file: $e');
    }
  }

}

// Top-level function for isolate
Future<bool> _validateNierExeInIsolate(String filePath) async {
  try {
    final file = File(filePath);
    final raf = await file.open();
    try {
      // Check PE signature
      final header = await raf.read(2);
      if (header.length < 2 || header[0] != 0x4D || header[1] != 0x5A) {
        return false;
      }

      // Read file in 64KB chunks and search for PRJ_028
      const chunkSize = 65536;
      final searchBytes = 'PRJ_028'.codeUnits;
      int position = 0;

      while (true) {
        await raf.setPosition(position);
        final chunk = await raf.read(chunkSize);

        if (chunk.isEmpty) break;

        // Simple byte search
        for (int i = 0; i <= chunk.length - searchBytes.length; i++) {
          if (chunk[i] == searchBytes[0]) {
            bool match = true;
            for (int j = 1; j < searchBytes.length; j++) {
              if (chunk[i + j] != searchBytes[j]) {
                match = false;
                break;
              }
            }
            if (match) return true;
          }
        }

        position += chunkSize;
        if (chunk.length < chunkSize) break;
      }

      return false;
    } finally {
      await raf.close();
    }
  } catch (e) {
    return true;
  }
}
