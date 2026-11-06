import 'dart:io';

import 'package:flutter/material.dart';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/services/archive_service.dart';
import 'package:yp_launcher/services/mods_service.dart';
import 'package:yp_launcher/services/texture_install_service.dart';
import 'package:yp_launcher/widgets/mods/mod_variant_dialog.dart';

abstract final class TexturePackInstall {
  static Future<int?> run(
    BuildContext context, {
    required List<String> paths,
    required String textureDir,
    void Function(double percent, String? currentFile)? onExtractProgress,
    void Function(String archivePath)? onExtractFailed,
    Future<String?> Function(String fallback)? promptName,
    Future<String?> Function()? chooseMergeTarget,
    VoidCallback? onCopyStart,
    void Function(TextureCopyProgress progress)? onCopyProgress,
  }) async {
    final tempDirs = <String>[];
    try {
      final effectivePaths = <String>[
        for (final p in paths)
          if (!isArchive(p)) p,
      ];
      final names = <String, String>{};

      for (final archivePath in paths.where(isArchive)) {
        final extracted = await ArchiveService.extract(
          archivePath,
          onProgress: onExtractProgress,
        );
        if (extracted == null) {
          onExtractFailed?.call(archivePath);
          continue;
        }
        tempDirs.add(extracted);

        final packs = await ModsService.detectTexturePacks(extracted);
        if (!context.mounted) return null;
        final fallback = path.basenameWithoutExtension(archivePath);

        if (packs.length <= 1) {
          var name = packs.isNotEmpty ? packs.first.label : fallback;
          if (name == path.basename(extracted)) {
            name = await promptName?.call(fallback) ?? fallback;
          }
          if (!context.mounted) return null;
          effectivePaths.add(extracted);
          names[extracted] = name;
          continue;
        }

        final chosen = await showModVariantDialog(
          context,
          variants: [
            for (final p in packs)
              ModVariant(
                subPath: p.path,
                label: p.label,
                kind: ModKind.texture,
                textureOnly: true,
              ),
          ],
        );
        if (chosen == null || chosen.isEmpty || !context.mounted) return null;
        final packName = chosen.length == 1 ? chosen.first.label : fallback;
        for (final v in chosen) {
          effectivePaths.add(v.subPath);
          names[v.subPath] = packName;
        }
      }

      if (effectivePaths.isEmpty) return null;

      if (chooseMergeTarget != null && effectivePaths.length == 1) {
        final choice = await chooseMergeTarget();
        if (choice == null) return null;
        if (choice.isNotEmpty) {
          for (final ep in effectivePaths) {
            names[ep] = choice;
          }
        }
      }

      onCopyStart?.call();
      return await installTexturesInBackground(
        [
          for (final ep in effectivePaths)
            names[ep] != null ? '$ep|${names[ep]}' : ep,
        ],
        textureDir,
        onProgress: onCopyProgress,
      );
    } finally {
      for (final dir in tempDirs) {
        try {
          await Directory(dir).delete(recursive: true);
        } catch (_) {}
      }
    }
  }
}
