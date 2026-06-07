import 'package:flutter/material.dart';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/providers/config_state.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/texture_widgets.dart';

class TextureInjectCard extends StatelessWidget {
  final Map<String, dynamic> tex;
  final ConfigStateController notifier;
  final String gameDir;
  final List<String> installedTextures;
  final List<String> detectedFolders;
  final Map<String, List<String>> conflicts;
  final Map<String, String> bundledOriginByPack;
  final Future<void> Function(String name) onDelete;

  const TextureInjectCard({
    super.key,
    required this.tex,
    required this.notifier,
    required this.gameDir,
    required this.installedTextures,
    required this.detectedFolders,
    required this.conflicts,
    required this.bundledOriginByPack,
    required this.onDelete,
  });

  String get _textureDir => path.join(gameDir, 'nams', 'inject', 'textures');

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final disabledPacks = <String>{
      ...((tex[TextureInjectionFields.disabledPacks.key] as List?) ??
          const [])
          .whereType<String>(),
    };
    final loadOrder = List<String>.from(
      ((tex[TextureInjectionFields.loadOrder.key] as List?) ?? const [])
          .whereType<String>()
          .where((p) => !disabledPacks.contains(p)),
    );

    return Container(
      margin: EdgeInsets.only(bottom: AppSizes.paddingLG(context)),
      decoration: BoxDecoration(
        color: AppColors.backgroundCard,
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(color: AppColors.borderLight),
        boxShadow: const [
          BoxShadow(
            color: AppColors.shadow,
            blurRadius: 4,
            offset: Offset(0, 2),
          ),
        ],
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            width: double.infinity,
            padding: EdgeInsets.symmetric(
              horizontal: AppSizes.cardPaddingH(context),
              vertical: AppSizes.cardPaddingV(context),
            ),
            decoration: BoxDecoration(
              color: AppColors.surfaceMedium,
              borderRadius: BorderRadius.only(
                topLeft: Radius.circular(AppSizes.borderRadius(context)),
                topRight: Radius.circular(AppSizes.borderRadius(context)),
              ),
            ),
            child: Text(
              'nams/inject/textures/ (${installedTextures.where((n) => !disabledPacks.contains(n)).length})',
              style: TextStyle(
                fontSize: AppSizes.fontSM(context),
                fontWeight: FontWeight.bold,
                color: AppColors.accentPrimary,
                letterSpacing: 1.0,
              ),
            ),
          ),
          Padding(
            padding: EdgeInsets.all(AppSizes.cardPaddingH(context)),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                if (conflicts.isNotEmpty)
                  Container(
                    padding: EdgeInsets.all(AppSizes.paddingSM(context)),
                    margin: EdgeInsets.only(
                      bottom: AppSizes.spacingMD(context),
                    ),
                    decoration: BoxDecoration(
                      color: AppColors.accentPrimary.withValues(alpha: 0.04),
                      borderRadius: BorderRadius.circular(
                        AppSizes.borderRadius(context),
                      ),
                    ),
                    child: Row(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Icon(
                          Icons.swap_vert,
                          size: AppSizes.iconSM(context),
                          color: AppColors.textMuted,
                        ),
                        SizedBox(width: AppSizes.spacingMD(context)),
                        Expanded(
                          child: Text(
                            l10n.textureConflictHint,
                            style: TextStyle(
                              fontSize: AppSizes.fontXS(context),
                              color: AppColors.textSecondary,
                            ),
                          ),
                        ),
                      ],
                    ),
                  )
                else if (loadOrder.length > 1)
                  Padding(
                    padding: EdgeInsets.only(
                      bottom: AppSizes.spacingMD(context),
                    ),
                    child: Text(
                      l10n.noConflictsFound,
                      style: TextStyle(
                        fontSize: AppSizes.fontXS(context),
                        color: AppColors.success,
                      ),
                    ),
                  ),
                if (installedTextures.isEmpty)
                  Center(
                    child: Padding(
                      padding: EdgeInsets.symmetric(
                        vertical: AppSizes.paddingSM(context),
                      ),
                      child: Text(
                        l10n.noTexturesInstalled,
                        style: TextStyle(
                          color: AppColors.textMuted,
                          fontSize: AppSizes.fontSM(context),
                        ),
                      ),
                    ),
                  )
                else ...[
                  if (loadOrder.isNotEmpty)
                    ReorderableListView.builder(
                      shrinkWrap: true,
                      physics: const NeverScrollableScrollPhysics(),
                      buildDefaultDragHandles: false,
                      itemCount: loadOrder.length,
                      proxyDecorator: (child, index, animation) =>
                          Material(color: Colors.transparent, child: child),
                      onReorderItem: (oldIndex, newIndex) {
                        final updated = List<String>.from(loadOrder);
                        final item = updated.removeAt(oldIndex);
                        updated.insert(newIndex, item);
                        notifier.updateTextureInjection(
                          TextureInjectionFields.loadOrder.key,
                          updated,
                        );
                      },
                      itemBuilder: (context, index) {
                        final folder = loadOrder[index];
                        final exists = detectedFolders.contains(folder);
                        return LoadOrderItem(
                          key: ValueKey(folder),
                          index: index,
                          folder: folder,
                          folderPath: path.join(
                            gameDir,
                            'nams',
                            'inject',
                            'textures',
                            folder,
                          ),
                          priority: loadOrder.length > 1
                              ? (index == 0
                                    ? l10n.priorityHighest
                                    : index == loadOrder.length - 1
                                    ? l10n.priorityLowest
                                    : null)
                              : null,
                          exists: exists,
                          conflictsWith: conflicts[folder],
                          bundledWithModId: bundledOriginByPack[folder],
                          onRemove: () async {
                            await onDelete(folder);
                            final updated = List<String>.from(loadOrder)
                              ..remove(folder);
                            notifier.updateTextureInjectionSilent(
                              TextureInjectionFields.loadOrder.key,
                              updated,
                            );
                            await notifier.saveConfigs(gameDir);
                          },
                        );
                      },
                    )
                  else
                    ...installedTextures
                        .where((n) => !disabledPacks.contains(n))
                        .map(
                      (name) => Padding(
                        padding: EdgeInsets.only(
                          bottom: AppSizes.paddingXS(context),
                        ),
                        child: Row(
                          children: [
                            const Icon(
                              Icons.image,
                              size: 14,
                              color: AppColors.textMuted,
                            ),
                            SizedBox(width: AppSizes.spacingMD(context)),
                            Expanded(
                              child: ClickableName(
                                name: name,
                                folderPath: path.join(
                                  _textureDir,
                                  name,
                                ),
                              ),
                            ),
                            InkWell(
                              onTap: () => onDelete(name),
                              borderRadius: BorderRadius.circular(
                                AppSizes.borderRadius(context),
                              ),
                              hoverColor: AppColors.error.withValues(
                                alpha: 0.15,
                              ),
                              splashColor: AppColors.error.withValues(
                                alpha: 0.2,
                              ),
                              child: Padding(
                                padding: EdgeInsets.all(
                                  AppSizes.paddingSM(context),
                                ),
                                child: Icon(
                                  Icons.close,
                                  size: AppSizes.iconSM(context),
                                  color: AppColors.error,
                                ),
                              ),
                            ),
                          ],
                        ),
                      ),
                    ),
                ],
              ],
            ),
          ),
        ],
      ),
    );
  }
}
