import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:file_picker/file_picker.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/theme/app_colors.dart';

class DirectorySelector extends ConsumerWidget {
  const DirectorySelector({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final appState = ref.watch(appStateControllerProvider);
    final controller = ref.read(appStateControllerProvider.notifier);

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 12.0),
      decoration: BoxDecoration(
        color: AppColors.backgroundCard,
        borderRadius: BorderRadius.circular(8.0),
        border: Border.all(color: AppColors.borderLight),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Row(
            children: [
              Expanded(
                child: Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 12.0,
                    vertical: 8.0,
                  ),
                  decoration: BoxDecoration(
                    color: AppColors.inputBackground,
                    borderRadius: BorderRadius.circular(6.0),
                    border: Border.all(
                      color: appState.isDirectorySelected
                          ? AppColors.success
                          : AppColors.borderMedium,
                    ),
                  ),
                  child: SelectableText(
                    appState.isDirectorySelected
                        ? appState.selectedDirectory
                        : AppStrings.noFileSelected,
                    style: TextStyle(
                      color: appState.isDirectorySelected
                          ? AppColors.textPrimary
                          : AppColors.textMuted,
                      fontSize: 12,
                    ),
                    maxLines: 1,
                  ),
                ),
              ),
              const SizedBox(width: 10),
              ElevatedButton(
                onPressed: () => _selectDirectory(controller),
                style: ElevatedButton.styleFrom(
                  backgroundColor: AppColors.buttonBackground,
                  foregroundColor: AppColors.buttonText,
                  padding: const EdgeInsets.symmetric(
                    horizontal: 16.0,
                    vertical: 10.0,
                  ),
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(6.0),
                  ),
                ),
                child: const Text(
                  AppStrings.selectButton,
                  style: TextStyle(fontSize: 13, fontWeight: FontWeight.bold),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Future<void> _selectDirectory(AppStateController controller) async {
    try {
      final result = await FilePicker.platform.pickFiles(
        dialogTitle: AppStrings.filePickerTitle,
        type: FileType.custom,
        allowedExtensions: [AppStrings.allowedExtension],
      );

      if (result != null && result.files.single.path != null) {
        final filePath = result.files.single.path!;
        final file = File(filePath);

        if (await file.exists()) {
          final isValid = await compute(_validateNierExeInIsolate, filePath);

          if (isValid) {
            final directory = file.parent.path;
            await controller.setDirectory(directory);
          } else {
            controller.setError(AppStrings.errorInvalidExe);
          }
        } else {
          controller.setError(AppStrings.errorFileNotExist);
        }
      }
    } catch (e) {
      controller.setError('${AppStrings.errorSelectFailed}: $e');
    }
  }
}

Future<bool> _validateNierExeInIsolate(String filePath) async {
  try {
    final file = File(filePath);
    final raf = await file.open();
    try {
      final header = await raf.read(2);
      if (header.length < 2 || header[0] != 0x4D || header[1] != 0x5A) {
        return false;
      }

      const chunkSize = 65536;
      final searchBytes = AppStrings.gameSignature.codeUnits;
      int position = 0;

      while (true) {
        await raf.setPosition(position);
        final chunk = await raf.read(chunkSize);

        if (chunk.isEmpty) break;

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
