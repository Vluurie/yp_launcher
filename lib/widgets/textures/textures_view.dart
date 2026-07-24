import 'dart:io';
import 'dart:isolate';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/services/isolate_service.dart';
import 'package:yp_launcher/services/toml_service.dart';
import 'package:yp_launcher/providers/config_state.dart';
import 'package:yp_launcher/providers/disabled_mods_state.dart';
import 'package:yp_launcher/providers/mods_state.dart';
import 'package:yp_launcher/providers/notification_state.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/busy_guard.dart';
import 'package:yp_launcher/widgets/busy_overlay.dart';
import 'package:yp_launcher/widgets/config_field_bool.dart';
import 'package:yp_launcher/services/archive_service.dart';
import 'package:yp_launcher/widgets/config_field_dropdown.dart';
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/services/texture_install_service.dart';
import 'package:yp_launcher/services/mods_service.dart';
import 'package:yp_launcher/services/nams_cli_service.dart';
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/widgets/mods/mod_variant_dialog.dart';
import 'package:yp_launcher/widgets/texture_widgets.dart';
import 'package:yp_launcher/widgets/textures/texture_dialogs.dart';
import 'package:yp_launcher/widgets/textures/texture_header.dart';
import 'package:yp_launcher/widgets/textures/texture_drop_zone.dart';
import 'package:yp_launcher/widgets/textures/texture_inject_card.dart';


class TexturesView extends ConsumerStatefulWidget {
  const TexturesView({super.key});

  @override
  ConsumerState<TexturesView> createState() => _TexturesViewState();
}

class _TexturesViewState extends ConsumerState<TexturesView> {
  final _scrollController = ScrollController();
  String _loadedForDir = '';
  bool _installing = false;
  String _installPhase = '';
  int _progressBytes = 0;
  int _progressTotalBytes = 0;
  int _progressFiles = 0;
  int _progressTotalFiles = 0;
  double _progressPercent = 0;
  List<String> _installedTextures = [];
  List<String> _skResTextures = [];
  List<String> _waxTextures = [];
  List<String> _detectedFolders = [];
  List<NamsTexturePack> _outfitPacks = [];
  Map<String, List<String>> _conflicts = {};
  bool _loadingTextures = true;
  bool _deleting = false;
  String _deleteMessage = '';

  String get _gameDir => ref.read(appStateControllerProvider).selectedDirectory;
  String get _textureDir => path.join(_gameDir, 'nams', 'inject', 'textures');

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _maybeReload();
      ref.listenManual(appStateControllerProvider, (prev, next) {
        _maybeReload();
      });
      ref.listenManual<int>(detectionRefreshProvider, (_, __) async {
        if (_gameDir.isEmpty) return;
        await _refreshDisabledPacksFromDisk();
        _loadTextures();
        ref.read(modsStateControllerProvider.notifier).loadMods(_gameDir);
      });
    });
  }

  void _maybeReload() {
    final dir = _gameDir;
    if (dir.isEmpty || dir == _loadedForDir) return;
    _loadedForDir = dir;
    _initLoad();
  }

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  Future<void> _initLoad() async {
    await ref
        .read(configStateControllerProvider.notifier)
        .loadConfigs(_gameDir);
    await _loadTextures();
    ref.read(disabledModsStateControllerProvider.notifier).load(_gameDir);
    ref.read(modsStateControllerProvider.notifier).loadMods(_gameDir);
  }

  /// Re-reads only the `disabled_packs` value from texture_injection.toml
  /// and merges it into the in-memory config state. Used after profile
  /// switches so the texture view picks up the new blacklist without
  /// clobbering any unsaved user edits to other texture-injection fields.
  Future<void> _refreshDisabledPacksFromDisk() async {
    try {
      final tomlPath =
          path.join(_gameDir, 'nams', 'texture_injection.toml');
      final raw = await TomlService.readTomlFile(tomlPath);
      if (raw.isEmpty) return;
      final parsed = TomlService.parse(raw);
      final disabled = parsed[TextureInjectionFields.disabledPacks.key];
      ref
          .read(configStateControllerProvider.notifier)
          .updateTextureInjectionSilent(
            TextureInjectionFields.disabledPacks.key,
            disabled is List ? List<String>.from(disabled.whereType<String>()) : const <String>[],
          );
    } catch (_) {}
  }

  Future<void> _loadTextures() async {
    final isFirstLoad = _installedTextures.isEmpty &&
        _skResTextures.isEmpty &&
        _waxTextures.isEmpty;
    if (isFirstLoad) {
      setState(() => _loadingTextures = true);
    }

    final config = ref.read(configStateControllerProvider);
    final disabledPacks = List<String>.from(
      (config.textureInjectionValues[TextureInjectionFields.disabledPacks.key]
              as List?)
              ?.whereType<String>() ??
          const <String>[],
    );

    final result = await IsolateService.run(
      loadTexturesIsolate,
      LoadTexturesParams(
        textureDir: _textureDir,
        skResDir: path.join(
          _gameDir,
          'SK_Res',
          'inject',
          'textures',
          'NieRAutomata.exe',
        ),
        waxModsDir: path.join(_gameDir, 'wax', 'mods'),
        disabledPacks: disabledPacks,
      ),
    );

    final folders = result['namsFolders'] as List<String>;
    final conflictStrings = result['conflicts'] as List<String>? ?? [];
    final parsedConflicts = <String, List<String>>{};
    for (final s in conflictStrings) {
      final parts = s.split('|');
      if (parts.length == 2) {
        parsedConflicts[parts[0]] = parts[1].split(',');
      }
    }

    final namsPacks = await NamsCliService.texturesList(_gameDir);
    if (!mounted) return;

    setState(() {
      _installedTextures = result['nams'] as List<String>;
      _skResTextures = result['sk'] as List<String>;
      _waxTextures = (result['wax'] as List<String>?) ?? [];
      _detectedFolders = folders;
      _conflicts = parsedConflicts;
      _outfitPacks = (namsPacks ?? const [])
          .where((p) => p.outfitConditional)
          .toList();
      _loadingTextures = false;
    });

    if (folders.isNotEmpty) {
      final config = ref.read(configStateControllerProvider);
      final currentOrder = List<String>.from(
        (config.textureInjectionValues[TextureInjectionFields.loadOrder.key]
                as List?) ??
            [],
      );
      final deduped = currentOrder.toSet().toList();
      final missing = folders.where((f) => !deduped.contains(f)).toList();
      final needsUpdate = missing.isNotEmpty || deduped.length != currentOrder.length;
      if (needsUpdate) {
        final updated = List<String>.from(deduped)..addAll(missing);
        final notifier = ref.read(configStateControllerProvider.notifier);
        notifier.updateTextureInjectionSilent(
          TextureInjectionFields.loadOrder.key,
          updated,
        );
        await notifier.saveConfigs(_gameDir);
      }
    }
  }

  Future<void> _handleDrop(List<String> paths) async {
    if (_installing) return;
    await withBusyGuard(ref, () => _handleDropInner(paths));
  }

  Future<void> _handleDropInner(List<String> paths) async {
    final l10n = AppLocalizations.of(context)!;
    setState(() {
      _installing = true;
      _installPhase = l10n.textureDropAnalyzing;
    });
    ref.read(texturesBusyProvider.notifier).state = true;

    final hasDds = await _dropContainsDds(paths);
    if (!mounted) return;
    if (!hasDds) {
      _notify(l10n.textureDropNoDds, Icons.error_outline, AppColors.error);
      setState(() { _installing = false; _installPhase = ''; });
      ref.read(texturesBusyProvider.notifier).state = false;
      return;
    }

    setState(() {
      _installPhase = l10n.texturesProgressExtracting;
    });
    _notify(
      l10n.installingTextures,
      Icons.hourglass_top,
      AppColors.accentPrimary,
    );

    final effectivePaths = <String>[];
    final archiveNames = <String, String>{};
    final tempExtractDirs = <String>[];

    final archivePaths = paths.where(isArchive).toList();
    final nonArchivePaths = paths.where((p) => !isArchive(p)).toList();
    effectivePaths.addAll(nonArchivePaths);

    for (final archivePath in archivePaths) {
      final extracted = await ArchiveService.extract(
        archivePath,
        onProgress: (percent, currentFile) {
          if (!mounted) return;
          setState(() {
            _installPhase = currentFile == null || currentFile.isEmpty
                ? l10n.extractingArchivePercent((percent * 100).round())
                : l10n.extractingArchivePercentFile(
                    (percent * 100).round(), currentFile);
            _progressPercent = percent;
          });
        },
      );
      if (extracted == null) {
        _notify(l10n.failedToExtractArchive, Icons.error_outline, AppColors.error);
        continue;
      }
      tempExtractDirs.add(extracted);

      final packs = await ModsService.detectTexturePacks(extracted);
      if (!mounted) return;

      if (packs.length <= 1) {
        effectivePaths.add(extracted);
        archiveNames[extracted] = packs.isNotEmpty
            ? packs.first.label
            : path.basenameWithoutExtension(archivePath);
      } else {
        final chosen = await showModVariantDialog(
          context,
          variants: [
            for (final p in packs)
              ModVariant(
                  subPath: p.path,
                  label: p.label,
                  kind: ModKind.texture,
                  textureOnly: true),
          ],
        );
        if (chosen == null || chosen.isEmpty) {
          setState(() { _installing = false; _installPhase = ''; });
          ref.read(texturesBusyProvider.notifier).state = false;
          return;
        }
        final packName = chosen.length == 1
            ? chosen.first.label
            : path.basenameWithoutExtension(archivePath);
        for (final v in chosen) {
          effectivePaths.add(v.subPath);
          archiveNames[v.subPath] = packName;
        }
      }
    }

    if (effectivePaths.isEmpty) {
      setState(() { _installing = false; _installPhase = ''; });
      ref.read(texturesBusyProvider.notifier).state = false;
      return;
    }

    try {
      if (_detectedFolders.isNotEmpty && effectivePaths.length == 1) {
        final choice = await showTextureMergeDialog(
          context: context,
          detectedFolders: _detectedFolders,
        );
        if (choice == null) {
          setState(() => _installing = false);
          ref.read(texturesBusyProvider.notifier).state = false;
          return;
        }
        if (choice.isNotEmpty) {
          for (var i = 0; i < effectivePaths.length; i++) {
            final ep = effectivePaths[i];
            archiveNames[ep] = choice;
          }
        }
      }

      if (mounted) {
        setState(() {
          _installPhase = l10n.texturesProgressCopying;
          _progressBytes = 0;
          _progressTotalBytes = 0;
          _progressFiles = 0;
          _progressTotalFiles = 0;
          _progressPercent = 0;
        });
      }

      final installPaths = effectivePaths.map((ep) {
        final name = archiveNames[ep];
        return name != null ? '$ep|$name' : ep;
      }).toList();

      final receivePort = ReceivePort();
      final isolate = await Isolate.spawn(
        installTexturesWithProgress,
        TextureInstallProgressParams(
          paths: installPaths,
          textureDir: _textureDir,
          sendPort: receivePort.sendPort,
        ),
      );

      int count = 0;
      await for (final message in receivePort) {
        if (message is Map<String, dynamic>) {
          final type = message['type'] as String;
          if (type == 'progress') {
            if (mounted) {
              final copied = message['copied'] as int;
              final total = message['total'] as int;
              setState(() {
                _progressBytes = copied;
                _progressTotalBytes = total;
                _progressFiles = message['files'] as int;
                _progressTotalFiles = message['totalFiles'] as int;
                _progressPercent = total > 0 ? copied / total : 0;
              });
            }
          } else if (type == 'done') {
            count = message['count'] as int;
            break;
          }
        }
      }

      isolate.kill(priority: Isolate.immediate);
      receivePort.close();

      if (count > 0)
        _notify(
          l10n.installedTextureCount(count),
          Icons.check_circle,
          AppColors.success,
        );
      else
        _notify(l10n.noTextureFilesFound, Icons.warning, AppColors.warning);
    } catch (e) {
      _notify(
        l10n.installationFailed('$e'),
        Icons.error_outline,
        AppColors.error,
      );
    }

    await _loadTextures();
    ref.read(detectionRefreshProvider.notifier).state++;
    if (mounted) {
      setState(() {
        _installing = false;
        _installPhase = '';
      });
    }
    ref.read(texturesBusyProvider.notifier).state = false;

    for (final dir in tempExtractDirs) {
      try {
        await Directory(dir).delete(recursive: true);
      } catch (_) {}
    }
  }

  Future<bool> _dropContainsDds(List<String> paths) async {
    for (final p in paths) {
      if (isArchive(p)) {
        final found = await ArchiveService.archiveContainsExtension(p, ['.dds']);
        if (found == true) return true;
      } else if (FileSystemEntity.isFileSync(p)) {
        if (p.toLowerCase().endsWith('.dds')) return true;
      } else if (FileSystemEntity.isDirectorySync(p)) {
        final found = await IsolateService.run(_dirContainsDdsSync, p);
        if (found) return true;
      }
    }
    return false;
  }

  Map<String, String> _bundledOriginByPack() {
    final mods = ref.watch(modsStateControllerProvider).mods;
    final out = <String, String>{};
    for (final m in mods) {
      for (final pack in m.bundledTexturePacks) {
        out[pack] = m.id;
      }
    }
    return out;
  }

  Future<void> _deleteTexture(String name) async {
    final l10n = AppLocalizations.of(context)!;
    final ok = await showDeleteConfirm(
      context,
      title: l10n.textureDeleteConfirmTitle,
      body: l10n.textureDeleteConfirmBody(name),
      deleteLabel: l10n.textureDeleteLabel,
    );
    if (!ok || !mounted) return;
    setState(() {
      _deleting = true;
      _deleteMessage = l10n.busyDeletingTexture;
    });
    try {
      await withBusyGuard(
        ref,
        () => IsolateService.run(
          deleteTextureIsolate,
          path.join(_textureDir, name),
        ),
      );
      _notify(
        l10n.removedItem(name),
        Icons.delete_outline,
        AppColors.textMuted,
      );
      await _loadTextures();
      ref.read(detectionRefreshProvider.notifier).state++;
    } finally {
      if (mounted) setState(() => _deleting = false);
    }
  }

  Future<void> _handleBrowse() async {
    final l10n = AppLocalizations.of(context)!;
    final result = await FilePicker.pickFiles(
      dialogTitle: l10n.selectTextureFiles,
      type: FileType.any,
    );
    if (result == null || result.files.isEmpty) return;
    final paths = result.files
        .where((f) => f.path != null)
        .map((f) => f.path!)
        .toList();
    await _handleDrop(paths);
  }

  Future<void> _handleBrowseFolder() async {
    final result = await FilePicker.getDirectoryPath();
    if (result == null) return;
    await _handleDrop([result]);
  }

  void _notify(String message, IconData icon, Color color) {
    ref
        .read(notificationStateControllerProvider.notifier)
        .addNotification(
          NotificationItem(
            id: 'texture_${DateTime.now().millisecondsSinceEpoch}',
            message: (l10n) => message,
            icon: icon,
            color: color,
            type: NotificationType.textures,
          ),
        );
  }

  String _buildProgressText(AppLocalizations l10n) {
    if (_installPhase == l10n.texturesProgressCopying && _progressTotalBytes > 0) {
      return l10n.texturesInstallProgress(
        _progressFiles,
        _progressTotalFiles,
        _progressBytes ~/ (1024 * 1024),
        _progressTotalBytes ~/ (1024 * 1024),
      );
    }
    if (_installPhase == 'cleaning') {
      return l10n.cleaningTextures;
    }
    return '';
  }

  @override
  Widget build(BuildContext context) {
    final config = ref.watch(configStateControllerProvider);
    final notifier = ref.read(configStateControllerProvider.notifier);
    final gameDir = ref.watch(appStateControllerProvider).selectedDirectory;
    final tex = config.textureInjectionValues;

    return Stack(
      children: [
        Container(
          color: AppColors.backgroundPrimary,
          child: Column(
            children: [
              TextureHeader(
                config: config,
                notifier: notifier,
                gameDir: gameDir,
              ),
          Expanded(
            child: config.isLoading
                ? const Center(
                    child: CircularProgressIndicator(
                      color: AppColors.accentPrimary,
                    ),
                  )
                : Scrollbar(
                    controller: _scrollController,
                    thumbVisibility: true,
                    child: SingleChildScrollView(
                      controller: _scrollController,
                      padding: EdgeInsets.all(AppSizes.contentPadding(context)),
                      child: LayoutBuilder(
                        builder: (context, constraints) {
                      final stack = constraints.maxWidth < 760;
                      final leftCol = Column(
                              children: [
                                _card(
                                  context,
                                  AppLocalizations.of(context)!.cardInstallTextures,
                                  [
                                    TextureDropZone(
                                      installing: _installing,
                                      installPhase: _installPhase,
                                      progressPercent: _progressPercent,
                                      buildProgressText: _buildProgressText,
                                      onDrop: (paths) {
                                        if (ref.read(activeTabProvider) != 3) return;
                                        _handleDrop(paths);
                                      },
                                      onBrowse: _handleBrowse,
                                      onBrowseFolder: _handleBrowseFolder,
                                    ),
                                  ],
                                ),
                                _card(
                                  context,
                                  AppLocalizations.of(context)!.cardTextureConfig,
                                  [
                                    ConfigFieldDropdown(
                                      label: TextureInjectionFields
                                          .vramBudgetMb
                                          .label(AppLocalizations.of(context)!),
                                      value:
                                          (tex[TextureInjectionFields
                                                  .vramBudgetMb
                                                  .key]
                                              as int?) ??
                                          TextureInjectionFields
                                              .vramBudgetMb
                                              .defaultValue,
                                      options: const [
                                        0,
                                        1024,
                                        2048,
                                        4096,
                                        6144,
                                        8192,
                                        12288,
                                        16384,
                                      ],
                                      labels: {
                                        0: AppLocalizations.of(
                                          context,
                                        )!.textureAutoRecommended,
                                      },
                                      onChanged: (v) =>
                                          notifier.updateTextureInjection(
                                            TextureInjectionFields
                                                .vramBudgetMb
                                                .key,
                                            v,
                                          ),
                                      tooltip:
                                          TextureInjectionFields
                                              .vramBudgetMb
                                              .tooltip!(
                                            AppLocalizations.of(context)!,
                                          ),
                                    ),
                                    ConfigFieldBool(
                                      label: TextureInjectionFields
                                          .streamingEnabled
                                          .label(AppLocalizations.of(context)!),
                                      value:
                                          tex[TextureInjectionFields
                                              .streamingEnabled
                                              .key] !=
                                          false,
                                      onChanged: (v) =>
                                          notifier.updateTextureInjection(
                                            TextureInjectionFields
                                                .streamingEnabled
                                                .key,
                                            v,
                                          ),
                                      tooltip:
                                          TextureInjectionFields
                                              .streamingEnabled
                                              .tooltip!(
                                            AppLocalizations.of(context)!,
                                          ),
                                    ),
                                    ConfigFieldBool(
                                      label: TextureInjectionFields
                                          .loadOnlyRelevant
                                          .label(AppLocalizations.of(context)!),
                                      value:
                                          tex[TextureInjectionFields
                                              .loadOnlyRelevant
                                              .key] ==
                                          true,
                                      onChanged: (v) =>
                                          notifier.updateTextureInjection(
                                            TextureInjectionFields
                                                .loadOnlyRelevant
                                                .key,
                                            v,
                                          ),
                                      tooltip:
                                          TextureInjectionFields
                                              .loadOnlyRelevant
                                              .tooltip!(
                                            AppLocalizations.of(context)!,
                                          ),
                                    ),
                                  ],
                                ),
                              ],
                            );
                      final rightCol = Column(
                              children: [
                                if (!_loadingTextures) ...[
                                  if (_installedTextures.isEmpty &&
                                      _skResTextures.isEmpty &&
                                      _waxTextures.isEmpty)
                                    Padding(
                                      padding: EdgeInsets.symmetric(
                                        vertical: AppSizes.paddingXL(context) * 2,
                                      ),
                                      child: Center(
                                        child: Text(
                                          AppLocalizations.of(context)!.noTexturesInstalled,
                                          style: TextStyle(
                                            fontSize: AppSizes.fontMD(context),
                                            color: AppColors.textMuted.withValues(alpha: 0.5),
                                            fontStyle: FontStyle.italic,
                                          ),
                                        ),
                                      ),
                                    ),
                                  if (_installedTextures.isNotEmpty)
                                    TextureInjectCard(
                                      tex: tex,
                                      notifier: notifier,
                                      gameDir: _gameDir,
                                      installedTextures: _installedTextures,
                                      detectedFolders: _detectedFolders,
                                      conflicts: _conflicts,
                                      bundledOriginByPack: _bundledOriginByPack(),
                                      onDelete: _deleteTexture,
                                    ),
                                  if (_outfitPacks.isNotEmpty)
                                    _buildOutfitLinkedCard(),
                                  if (_skResTextures.isNotEmpty)
                                    _buildExternalSourceCard(
                                      title: 'SK_Res/ (${_skResTextures.length})',
                                      priority: AppLocalizations.of(context)!.priorityMedium,
                                      entries: _skResTextures,
                                      folderOf: (name) => path.join(
                                        _gameDir,
                                        'SK_Res',
                                        'inject',
                                        'textures',
                                        'NieRAutomata.exe',
                                        name,
                                      ),
                                    ),
                                  if (_waxTextures.isNotEmpty)
                                    _buildExternalSourceCard(
                                      title: 'wax/mods/ (${_waxTextures.length})',
                                      priority: AppLocalizations.of(context)!.priorityHighest,
                                      entries: _waxTextures,
                                      folderOf: (name) => path.join(
                                        _gameDir,
                                        'wax',
                                        'mods',
                                        name,
                                      ),
                                    ),
                                ],
                              ],
                            );
                      if (stack) {
                        return Column(
                          crossAxisAlignment: CrossAxisAlignment.stretch,
                          children: [
                            leftCol,
                            SizedBox(height: AppSizes.spacingLG(context)),
                            rightCol,
                          ],
                        );
                      }
                      return Row(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Expanded(child: leftCol),
                          SizedBox(width: AppSizes.spacingLG(context)),
                          Expanded(child: rightCol),
                        ],
                      );
                        },
                      ),
                    ),
                  ),
          ),
            ],
          ),
        ),
        if (_deleting) BusyOverlay(message: _deleteMessage),
      ],
    );
  }

  Widget _buildExternalSourceCard({
    required String title,
    required String priority,
    required List<String> entries,
    required String Function(String name) folderOf,
  }) {
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
            child: Row(
              children: [
                Expanded(
                  child: Text(
                    title,
                    style: TextStyle(
                      fontSize: AppSizes.fontSM(context),
                      fontWeight: FontWeight.bold,
                      color: AppColors.accentPrimary,
                      letterSpacing: 1.0,
                    ),
                  ),
                ),
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                  decoration: BoxDecoration(
                    color: AppColors.textMuted.withValues(alpha: 0.15),
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Text(
                    priority,
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      fontWeight: FontWeight.bold,
                      color: AppColors.textMuted,
                      letterSpacing: 0.8,
                    ),
                  ),
                ),
              ],
            ),
          ),
          Padding(
            padding: EdgeInsets.all(AppSizes.cardPaddingH(context)),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: entries.map((name) => Padding(
                padding: EdgeInsets.only(bottom: AppSizes.paddingXS(context)),
                child: Row(
                  children: [
                    const Icon(Icons.image, size: 14, color: AppColors.textMuted),
                    SizedBox(width: AppSizes.spacingMD(context)),
                    Expanded(
                      child: ClickableName(
                        name: name,
                        folderPath: folderOf(name),
                      ),
                    ),
                  ],
                ),
              )).toList(),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildOutfitLinkedCard() {
    final l10n = AppLocalizations.of(context)!;
    return Container(
      margin: EdgeInsets.only(bottom: AppSizes.paddingLG(context)),
      decoration: BoxDecoration(
        color: AppColors.backgroundCard,
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(color: AppColors.borderLight),
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
            child: Row(
              children: [
                Icon(
                  Icons.checkroom,
                  size: AppSizes.iconSM(context),
                  color: AppColors.accentPrimary,
                ),
                SizedBox(width: AppSizes.spacingSM(context)),
                Expanded(
                  child: Text(
                    '${l10n.textureOutfitLinkedTitle} (${_outfitPacks.length})',
                    style: TextStyle(
                      fontSize: AppSizes.fontSM(context),
                      fontWeight: FontWeight.bold,
                      color: AppColors.accentPrimary,
                      letterSpacing: 1.0,
                    ),
                  ),
                ),
              ],
            ),
          ),
          Padding(
            padding: EdgeInsets.all(AppSizes.cardPaddingH(context)),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  l10n.textureOutfitLinkedSubtitle,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.textMuted,
                    height: 1.3,
                  ),
                ),
                SizedBox(height: AppSizes.spacingMD(context)),
                ..._outfitPacks.map((p) => Padding(
                      padding:
                          EdgeInsets.only(bottom: AppSizes.paddingXS(context)),
                      child: Row(
                        children: [
                          const Icon(Icons.link,
                              size: 14, color: AppColors.accentPrimary),
                          SizedBox(width: AppSizes.spacingMD(context)),
                          Expanded(
                            child: ClickableName(
                              name: p.character != null
                                  ? '${p.name} (${p.character})'
                                  : p.name,
                              folderPath: p.path,
                            ),
                          ),
                          Text(
                            l10n.textureOutfitLinkedEntry(p.ddsCount),
                            style: TextStyle(
                              fontSize: AppSizes.fontXS(context),
                              color: AppColors.textMuted,
                            ),
                          ),
                        ],
                      ),
                    )),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _card(BuildContext context, String title, List<Widget> children) {
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
              title,
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
              children: children,
            ),
          ),
        ],
      ),
    );
  }
}

bool _dirContainsDdsSync(String dirPath) {
  try {
    final dir = Directory(dirPath);
    if (!dir.existsSync()) return false;
    for (final entity in dir.listSync(recursive: true, followLinks: false)) {
      if (entity is File && entity.path.toLowerCase().endsWith('.dds')) {
        return true;
      }
    }
  } catch (_) {}
  return false;
}
