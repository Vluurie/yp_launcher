import 'dart:async';
import 'dart:io';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/models/mod_grouping.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/default_mods_state.dart';
import 'package:yp_launcher/providers/disabled_mods_state.dart';
import 'package:yp_launcher/providers/mod_load_order_state.dart';
import 'package:yp_launcher/providers/mod_names_state.dart';
import 'package:yp_launcher/providers/mod_profiles_state.dart';
import 'package:yp_launcher/widgets/mods/mod_group_header.dart';
import 'package:yp_launcher/widgets/mods/mod_naming.dart';
import 'package:yp_launcher/widgets/mods/mod_variant_dialog.dart';
import 'package:yp_launcher/widgets/mods/mod_profile_selector.dart';
import 'package:yp_launcher/providers/mods_state.dart';
import 'package:yp_launcher/services/archive_service.dart';
import 'package:yp_launcher/services/isolate_service.dart';
import 'package:yp_launcher/services/mods_service.dart';
import 'package:yp_launcher/providers/notification_state.dart';
import 'package:yp_launcher/widgets/hover_button.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/busy_guard.dart';
import 'package:yp_launcher/widgets/busy_overlay.dart';
import 'package:yp_launcher/widgets/mods/mod_detail_panel.dart';
import 'package:yp_launcher/widgets/launcher_sidebar.dart';
import 'package:yp_launcher/widgets/mods/mod_drop_zone.dart';
import 'package:yp_launcher/widgets/mods/mod_list_row.dart';
import 'package:yp_launcher/widgets/mods/mods_tutorial.dart';

class ModsView extends ConsumerStatefulWidget {
  const ModsView({super.key});

  @override
  ConsumerState<ModsView> createState() => _ModsViewState();
}

class _ModsViewState extends ConsumerState<ModsView> {
  String _loadedForDir = '';
  String? _selectedId;
  String _filter = '';
  final _collapsedGroups = <ModGroupKind>{};
  final _searchController = TextEditingController();
  final _listScrollController = ScrollController();

  StreamSubscription<FileSystemEvent>? _watchSub;
  String? _watchedDir;
  Timer? _reloadDebounce;
  bool _busy = false;
  String _busyMessage = '';

  final _dropZoneKey = GlobalKey();
  final _listKey = GlobalKey();
  final _detailKey = GlobalKey();
  final _profileSelectorKey = GlobalKey();
  final _helpIconKey = GlobalKey();

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _maybeReload();
      ref.listenManual(appStateControllerProvider, (_, __) => _maybeReload());
    });
  }

  @override
  void dispose() {
    _reloadDebounce?.cancel();
    _watchSub?.cancel();
    _searchController.dispose();
    _listScrollController.dispose();
    super.dispose();
  }

  String get _gameDir => ref.read(appStateControllerProvider).selectedDirectory;

  void _maybeReload() {
    final dir = _gameDir;
    if (dir.isEmpty || dir == _loadedForDir) return;
    _loadedForDir = dir;
    ModsService.migrateBundledTexturesInPlace(dir).whenComplete(() {
      if (!mounted) return;
      ref.read(modProfilesStateControllerProvider.notifier).load(dir);
      ref.read(modsStateControllerProvider.notifier).loadMods(dir);
      ref.read(disabledModsStateControllerProvider.notifier).load(dir);
      ref.read(defaultModsStateControllerProvider.notifier).load(dir);
      ref.read(modLoadOrderStateControllerProvider.notifier).load(dir);
      ref.read(modNamesStateControllerProvider.notifier).load(dir);
      _restartWatcher(dir);
    });
  }

  void _restartWatcher(String gameDir) {
    final modsDir = p.join(gameDir, 'nams', 'mods');
    if (_watchedDir == modsDir && _watchSub != null) return;
    _watchSub?.cancel();
    _watchSub = null;
    _watchedDir = null;

    final dir = Directory(modsDir);
    if (!dir.existsSync()) return;

    try {
      _watchSub = dir.watch(recursive: true).listen(
        (_) => _scheduleWatcherReload(),
        onError: (_) {},
      );
      _watchedDir = modsDir;
    } catch (_) {}
  }

  void _scheduleWatcherReload() {
    _reloadDebounce?.cancel();
    _reloadDebounce = Timer(const Duration(milliseconds: 400), () {
      if (!mounted) return;
      final dir = _gameDir;
      if (dir.isEmpty) return;
      ref.read(modsStateControllerProvider.notifier).loadMods(dir);
    });
  }

  Future<void> _handleDrop(List<String> paths) async {
    if (ref.read(activeTabProvider) != 5) return;
    for (final path in paths) {
      if (!mounted) return;
      await _installFromPath(path);
    }
  }

  Future<void> _handleBrowseFile() async {
    final result = await FilePicker.pickFiles(
      type: FileType.custom,
      allowedExtensions: const ['zip', '7z', 'rar'],
    );
    final picked = result?.files.single.path;
    if (picked == null) return;
    await _installFromPath(picked);
  }

  Future<void> _handleBrowseFolder() async {
    final result = await FilePicker.getDirectoryPath();
    if (result == null) return;
    await _installFromPath(result);
  }

  Future<void> _installFromPath(String sourcePath) async {
    final l10n = AppLocalizations.of(context)!;
    final notifier = ref.read(modsStateControllerProvider.notifier);
    final notif = ref.read(notificationStateControllerProvider.notifier);

    setState(() {
      _busy = true;
      _busyMessage = l10n.modDropAnalyzing;
    });

    final classification = await _dropLooksLikeMod(sourcePath);
    if (!mounted) return;
    if (!classification.valid) {
      setState(() => _busy = false);
      final message = classification.misroutedTab == 'cutscenes'
          ? l10n.modDropMisroutedCutscenes
          : l10n.modDropNotAMod;
      notif.addNotification(NotificationItem(
        id: 'mod_drop_reject_${DateTime.now().millisecondsSinceEpoch}',
        message: (l10n) => message,
        icon: Icons.error_outline,
        color: AppColors.error,
        type: NotificationType.general,
      ));
      return;
    }

    setState(() {
      _busyMessage = l10n.modInspectBusy;
    });

    final detect = await withBusyGuard(
      ref,
      () => ModsService.detectDrop(
        sourcePath,
        onExtractProgress: (percent, currentFile) {
          if (!mounted) return;
          setState(() {
            _busyMessage = currentFile == null || currentFile.isEmpty
                ? l10n.extractingArchivePercent((percent * 100).round())
                : l10n.extractingArchivePercentFile(
                    (percent * 100).round(), currentFile);
          });
        },
      ),
    );
    if (!mounted) return;
    setState(() => _busy = false);

    if (detect.hasVariants) {
      await _installVariants(sourcePath, detect, notifier, notif, l10n);
      return;
    }

    if (detect.kind == ModKind.unknown) {
      final isTextureOnly = detect.errorReason == 'texture_only';
      notif.addNotification(NotificationItem(
        id: 'mod_install_fail_${DateTime.now().millisecondsSinceEpoch}',
        message: (l10n) => isTextureOnly
            ? l10n.modInstallReasonTextureOnly
            : l10n.modInstallFailed(
                _localizeReason(l10n, detect.errorReason ?? 'unknown_drop')),
        icon: isTextureOnly ? Icons.warning_amber : Icons.error_outline,
        color: isTextureOnly ? AppColors.warning : AppColors.error,
        type: NotificationType.general,
      ));
      return;
    }

    List<String>? texturePackSubPaths;
    if (detect.hasTextureVariants) {
      final chosen = await showModVariantDialog(
        context,
        variants: [
          for (final t in detect.textureVariants)
            ModVariant(
              subPath: p.relative(t.path, from: detect.unwrappedRoot),
              label: t.label,
              kind: ModKind.texture,
              textureOnly: true,
            ),
        ],
      );
      if (chosen == null || chosen.isEmpty || !mounted) return;
      texturePackSubPaths = [for (final v in chosen) v.subPath];
    }

    final manifestId = detect.manifest?.id?.trim();
    String? requestedName;
    if (manifestId != null && manifestId.isNotEmpty) {
      requestedName = manifestId;
    } else {
      requestedName = await showModNamingDialog(
        context,
        initial: prettifyModId(detect.suggestedId),
      );
      if (requestedName == null || !mounted) return;
    }

    setState(() {
      _busy = true;
      _busyMessage = l10n.modInstallBusy;
    });

    void onExtract(double percent, String? currentFile) {
      if (!mounted) return;
      setState(() {
        _busyMessage = currentFile == null || currentFile.isEmpty
            ? l10n.extractingArchivePercent((percent * 100).round())
            : l10n.extractingArchivePercentFile(
                (percent * 100).round(), currentFile);
      });
    }

    InstallResult result;
    try {
      result = await withBusyGuard(
        ref,
        () => notifier.installMod(
          _gameDir,
          sourcePath,
          requestedName: requestedName,
          texturePackSubPaths: texturePackSubPaths,
          onExtractProgress: onExtract,
        ),
      );

      while (!result.success && result.errorMessage?.startsWith('exists:') == true) {
        if (!mounted) return;
        setState(() => _busy = false);
        requestedName = await showModNamingDialog(
          context,
          initial: requestedName ?? '',
          existsId: result.errorMessage!.substring('exists:'.length),
        );
        if (requestedName == null) return;
        if (!mounted) return;
        setState(() {
          _busy = true;
          _busyMessage = l10n.modInstallBusy;
        });
        result = await withBusyGuard(
          ref,
          () => notifier.installMod(
            _gameDir,
            sourcePath,
            requestedName: requestedName,
            onExtractProgress: onExtract,
          ),
        );
      }
    } finally {
      if (mounted) setState(() => _busy = false);
    }

    if (!mounted) return;
    if (result.success) {
      setState(() => _selectedId = result.installedId);
      notif.addNotification(NotificationItem(
        id: 'mod_installed_${DateTime.now().millisecondsSinceEpoch}',
        message: (l10n) => l10n.modInstalled(result.installedId ?? ''),
        icon: Icons.check_circle,
        color: AppColors.success,
        type: NotificationType.general,
      ));
      if (result.sideInstalledTexturePacks.isNotEmpty) {
        notif.addNotification(NotificationItem(
          id: 'mod_side_textures_${DateTime.now().millisecondsSinceEpoch}',
          message: (l10n) => l10n.modSideInstalledTextures(
            result.sideInstalledTexturePacks.join(', '),
          ),
          icon: Icons.texture,
          color: AppColors.accentPrimary,
          type: NotificationType.general,
        ));
      }
      if (result.unpairedWarnings.isNotEmpty) {
        notif.addNotification(NotificationItem(
          id: 'mod_unpaired_${DateTime.now().millisecondsSinceEpoch}',
          message: (l10n) =>
              l10n.modLooseUnpairedWarning(result.unpairedWarnings.join(', ')),
          icon: Icons.warning_amber,
          color: AppColors.warning,
          type: NotificationType.general,
        ));
      }
    } else {
      notif.addNotification(NotificationItem(
        id: 'mod_install_fail_${DateTime.now().millisecondsSinceEpoch}',
        message: (l10n) => l10n.modInstallFailed(_localizeReason(l10n, result.errorMessage ?? 'unknown')),
        icon: Icons.error_outline,
        color: AppColors.error,
        type: NotificationType.general,
      ));
    }
  }

  // Special edge case for "2P - The Mock Machine - All in One" (Nexus 335).
  // This mod ships several 2P outfit variants (Classic, Final Fantasy,
  // Soulcalibur, or-not-2P) plus separate texture-set folders under Textures/.
  // Per the mod's readme, each outfit uses the shared set ("2P I Shared
  // Textures") plus its own same-named set (e.g. "2P - Classic" -> "2P I
  // Classic"). The launcher cannot derive this from the model data because all
  // variants replace the same pl000d slot with the same vanilla hashes, so the
  // only authoritative mapping is the author's readme naming. We therefore
  // bundle, per outfit, the shared set plus the name-matching set only, instead
  // of dumping all sets into every outfit (which produced 29 colliding dds).
  List<String> _texturesForOutfit(ModVariant outfit, DetectedDrop detect) {
    final outfitKey = _outfitMatchKey(outfit.label);
    final result = <String>[];
    for (final t in detect.textureVariants) {
      final label = t.label.toLowerCase();
      final isShared = label.contains('shared');
      final matches = _outfitMatchKey(t.label) == outfitKey;
      if (isShared || matches) {
        result.add(p.relative(t.path, from: detect.unwrappedRoot));
      }
    }
    return result;
  }

  String _outfitMatchKey(String label) {
    var s = label.toLowerCase();
    for (final noise in ['2p', 'i', '-', '/', 'shared', 'textures', 'outfit']) {
      s = s.replaceAll(noise, ' ');
    }
    return s.replaceAll(RegExp(r'\s+'), ' ').trim();
  }

  Future<void> _installVariants(
    String sourcePath,
    DetectedDrop detect,
    dynamic notifier,
    dynamic notif,
    AppLocalizations l10n,
  ) async {
    final chosen = await showModVariantDialog(
      context,
      variants: detect.variants,
    );
    if (chosen == null || chosen.isEmpty || !mounted) return;

    final base = await showModNamingDialog(
      context,
      initial: prettifyModId(detect.suggestedId),
    );
    if (base == null || !mounted) return;

    final requests = [
      for (final variant in chosen)
        VariantInstallRequest(
          requestedName: '$base - ${variant.label}',
          variantSubPath: variant.subPath,
          texturePackSubPaths: variant.isWeapon
              ? const []
              : _texturesForOutfit(variant, detect),
        ),
    ];

    void onExtract(double percent, String? currentFile) {
      if (!mounted) return;
      setState(() {
        _busyMessage = currentFile == null || currentFile.isEmpty
            ? l10n.extractingArchivePercent((percent * 100).round())
            : l10n.extractingArchivePercentFile(
                (percent * 100).round(), currentFile);
      });
    }

    setState(() {
      _busy = true;
      _busyMessage = l10n.modInstallBusy;
    });

    final results = await withBusyGuard(
      ref,
      () => notifier.installVariants(
        _gameDir,
        sourcePath,
        requests,
        onExtractProgress: onExtract,
      ),
    ) as List<InstallResult>;

    if (!mounted) return;

    var installed = 0;
    String? lastId;
    final failures = <String>[];
    for (var i = 0; i < results.length; i++) {
      final r = results[i];
      if (r.success) {
        installed++;
        lastId = r.installedId;
      } else {
        failures.add('${chosen[i].label}: '
            '${_localizeReason(l10n, r.errorMessage ?? 'unknown')}');
      }
    }

    setState(() {
      _busy = false;
      if (lastId != null) _selectedId = lastId;
    });

    if (installed > 0) {
      notif.addNotification(NotificationItem(
        id: 'mod_variants_${DateTime.now().millisecondsSinceEpoch}',
        message: (l10n) => l10n.modVariantInstalledToast(installed),
        icon: Icons.check_circle,
        color: AppColors.success,
        type: NotificationType.general,
      ));
    }
    for (final f in failures) {
      notif.addNotification(NotificationItem(
        id: 'mod_variant_fail_${DateTime.now().millisecondsSinceEpoch}_$f',
        message: (l10n) => l10n.modInstallFailed(f),
        icon: Icons.error_outline,
        color: AppColors.error,
        type: NotificationType.general,
      ));
    }
  }

  String _localizeReason(AppLocalizations l10n, String code) {
    switch (code) {
      case 'unknown_drop': return l10n.modInstallReasonUnknownDrop;
      case 'invalid_mixed': return l10n.modInstallReasonInvalidMixed;
      case 'native_empty': return l10n.modInstallReasonNativeEmpty;
      case 'data_empty': return l10n.modInstallReasonDataEmpty;
      case 'texture_only': return l10n.modInstallReasonTextureOnly;
      case 'archive_extract_failed': return l10n.modInstallReasonArchiveExtractFailed;
      default:
        if (code.startsWith('move_failed')) return l10n.modInstallReasonMoveFailed;
        return code;
    }
  }

  Future<void> _confirmAndUninstall(InstalledMod mod) async {
    final l10n = AppLocalizations.of(context)!;
    final hasBundled = mod.bundledTexturePacks.isNotEmpty;
    var deleteBundled = hasBundled; // checked by default
    final ok = await showDialog<bool>(
      context: context,
      builder: (ctx) => StatefulBuilder(
        builder: (ctx, setLocal) => AlertDialog(
          backgroundColor: AppColors.backgroundCard,
          title: Text(
            l10n.modUninstallConfirmTitle,
            style: TextStyle(color: AppColors.error, fontSize: AppSizes.fontLG(ctx)),
          ),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                l10n.modUninstallConfirmBody(mod.id),
                style: TextStyle(color: AppColors.textSecondary, fontSize: AppSizes.fontSM(ctx)),
              ),
              if (hasBundled) ...[
                SizedBox(height: AppSizes.spacingMD(ctx)),
                InkWell(
                  onTap: () => setLocal(() => deleteBundled = !deleteBundled),
                  child: Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Checkbox(
                        value: deleteBundled,
                        onChanged: (v) =>
                            setLocal(() => deleteBundled = v ?? false),
                        activeColor: AppColors.error,
                      ),
                      SizedBox(width: AppSizes.spacingSM(ctx)),
                      Expanded(
                        child: Padding(
                          padding: EdgeInsets.only(top: AppSizes.paddingSM(ctx)),
                          child: Text(
                            l10n.modUninstallAlsoTexturesLabel(
                              mod.bundledTexturePacks.join(', '),
                            ),
                            style: TextStyle(
                              color: AppColors.textSecondary,
                              fontSize: AppSizes.fontSM(ctx),
                            ),
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ],
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(ctx).pop(false),
              child: Text(AppLocalizations.of(ctx)!.actionCancel, style: TextStyle(color: AppColors.textMuted, fontSize: AppSizes.fontSM(ctx))),
            ),
            TextButton(
              onPressed: () => Navigator.of(ctx).pop(true),
              child: Text(
                l10n.modUninstall,
                style: TextStyle(color: AppColors.error, fontSize: AppSizes.fontSM(ctx), fontWeight: FontWeight.bold),
              ),
            ),
          ],
        ),
      ),
    );
    if (ok != true) return;
    if (!mounted) return;
    setState(() {
      _busy = true;
      _busyMessage = l10n.modUninstallBusy;
    });
    try {
      await withBusyGuard(
        ref,
        () => ref
            .read(modsStateControllerProvider.notifier)
            .uninstallMod(
              _gameDir,
              mod.id,
              deleteBundledTextures: deleteBundled,
            ),
      );
    } finally {
      if (mounted) setState(() => _busy = false);
    }
    if (!mounted) return;
    if (_selectedId == mod.id) setState(() => _selectedId = null);
    ref.read(notificationStateControllerProvider.notifier).addNotification(NotificationItem(
          id: 'mod_uninstalled_${DateTime.now().millisecondsSinceEpoch}',
          message: (l10n) => l10n.modUninstalled(mod.id),
          icon: Icons.delete_outline,
          color: AppColors.textMuted,
          type: NotificationType.general,
        ));
  }

  List<InstalledMod> _applyFilter(
    List<InstalledMod> mods,
    ModNamesData names,
  ) {
    String shownName(InstalledMod m) =>
        names.customNameOf(m.id) ?? m.displayName;

    final filtered = _filter.isEmpty
        ? List<InstalledMod>.of(mods)
        : mods.where((m) {
            final f = _filter.toLowerCase();
            return m.id.toLowerCase().contains(f) ||
                m.displayName.toLowerCase().contains(f) ||
                shownName(m).toLowerCase().contains(f);
          }).toList();
    filtered.sort((a, b) {
      final tierDiff = _sortTier(a) - _sortTier(b);
      if (tierDiff != 0) return tierDiff;
      return shownName(a).toLowerCase().compareTo(shownName(b).toLowerCase());
    });
    return filtered;
  }

  static int _sortTier(InstalledMod m) {
    if (m.kind == ModKind.native) return 0;
    if (m.data?.hasCompatConfig == true) return 1;
    return 2;
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final data = ref.watch(modsStateControllerProvider);

    final pending = ref.watch(pendingTabSelectionProvider);
    if (pending != null && pending.tabIndex == 5) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (!mounted) return;
        if (data.mods.any((m) => m.id == pending.key)) {
          if (_selectedId != pending.key) {
            setState(() => _selectedId = pending.key);
          }
          ref.read(pendingTabSelectionProvider.notifier).state = null;
        }
      });
    }

    final names = ref.watch(modNamesStateControllerProvider);
    final filtered = _applyFilter(data.mods, names);
    InstalledMod? selected;
    if (_selectedId != null) {
      try {
        selected = data.mods.firstWhere((m) => m.id == _selectedId);
      } catch (_) {
        selected = null;
      }
    }

    return Stack(
      children: [
        Container(
          color: AppColors.backgroundPrimary,
          child: Padding(
            padding: EdgeInsets.all(AppSizes.contentPadding(context)),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Expanded(
                  flex: 6,
                  child: LayoutBuilder(
                    builder: (context, constraints) {
                      const estimatedRowHeight = 52.0;
                      final headerReserve =
                          AppSizes.spacingMD(context) +
                              AppSizes.spacingSM(context) +
                              48; // selector + spacing fudge
                      final approxListHeight =
                          constraints.maxHeight - headerReserve;
                      final wouldOverflow =
                          data.mods.length * estimatedRowHeight >
                              approxListHeight;
                      final showSearch = wouldOverflow ||
                          (_filter.isNotEmpty && data.mods.isNotEmpty);
                      return Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Row(
                            children: [
                              Expanded(
                                child: KeyedSubtree(
                                  key: _profileSelectorKey,
                                  child: const ModProfileSelector(),
                                ),
                              ),
                              SizedBox(width: AppSizes.spacingSM(context)),
                              _bulkInstallButton(l10n),
                              SizedBox(width: AppSizes.spacingSM(context)),
                              _looseInstallButton(l10n),
                              SizedBox(width: AppSizes.spacingSM(context)),
                              _helpIconButton(l10n),
                            ],
                          ),
                          SizedBox(height: AppSizes.spacingMD(context)),
                          if (showSearch) ...[
                            _searchBar(l10n),
                            SizedBox(height: AppSizes.spacingSM(context)),
                          ],
                          Expanded(
                            child: KeyedSubtree(
                              key: _listKey,
                              child: _list(filtered, data.isLoading, l10n),
                            ),
                          ),
                        ],
                      );
                    },
                  ),
                ),
                SizedBox(width: AppSizes.spacingLG(context)),
                Expanded(
                  flex: 5,
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      SizedBox(
                        height: 120,
                        child: KeyedSubtree(
                          key: _dropZoneKey,
                          child: ModDropZone(
                            onDrop: _handleDrop,
                            onBrowse: _handleBrowseFile,
                            onBrowseFolder: _handleBrowseFolder,
                          ),
                        ),
                      ),
                      SizedBox(height: AppSizes.spacingMD(context)),
                      Expanded(
                        child: KeyedSubtree(
                          key: _detailKey,
                          child: Container(
                            decoration: BoxDecoration(
                              color: AppColors.backgroundCard,
                              borderRadius: BorderRadius.circular(
                                AppSizes.borderRadius(context),
                              ),
                              border: Border.all(color: AppColors.borderLight),
                            ),
                            child: ModDetailPanel(
                              mod: selected,
                              onUninstall: _confirmAndUninstall,
                            ),
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ),
        if (_busy) BusyOverlay(message: _busyMessage),
      ],
    );
  }

  Widget _bulkInstallButton(AppLocalizations l10n) {
    return HoverIconButton(
      tooltip: l10n.modBulkInstall,
      bordered: false,
      padding: EdgeInsets.all(AppSizes.paddingXS(context)),
      icon: Icon(
        Icons.library_add_outlined,
        size: AppSizes.iconLG(context),
        color: AppColors.textMuted,
      ),
      onTap: _handleBulkInstall,
    );
  }

  Future<void> _handleBulkInstall() async {
    final l10n = AppLocalizations.of(context)!;
    final notif = ref.read(notificationStateControllerProvider.notifier);

    final folder = await FilePicker.getDirectoryPath();
    if (folder == null || !mounted) return;

    setState(() {
      _busy = true;
      _busyMessage = l10n.modBulkInstallScanning;
    });

    final archives = await IsolateService.run(_findArchivesSync, folder);
    if (!mounted) return;
    if (archives.isEmpty) {
      setState(() => _busy = false);
      notif.addNotification(NotificationItem(
        id: 'bulk_none_${DateTime.now().millisecondsSinceEpoch}',
        message: (l10n) => l10n.modBulkInstallNone,
        icon: Icons.info_outline,
        color: AppColors.warning,
        type: NotificationType.general,
      ));
      return;
    }

    var installed = 0;
    final failures = <String>[];
    try {
      for (var i = 0; i < archives.length; i++) {
        if (!mounted) return;
        final archive = archives[i];
        final baseName = p.basenameWithoutExtension(archive);
        setState(() {
          _busyMessage = l10n.modBulkInstallBusy(i + 1, archives.length, baseName);
        });

        final result = await withBusyGuard(
          ref,
          () => _installOneForBulk(archive, prettifyModId(baseName)),
        );
        if (result.success) {
          installed++;
        } else {
          failures.add('$baseName: ${result.errorMessage}');
        }
      }
    } finally {
      if (mounted) setState(() => _busy = false);
    }
    if (!mounted) return;

    await ref.read(modsStateControllerProvider.notifier).loadMods(_gameDir);
    if (!mounted) return;

    notif.addNotification(NotificationItem(
      id: 'bulk_done_${DateTime.now().millisecondsSinceEpoch}',
      message: (l10n) => l10n.modBulkInstallDone(installed, archives.length),
      icon: Icons.check_circle,
      color: installed == archives.length ? AppColors.success : AppColors.warning,
      type: NotificationType.general,
    ));
    for (final f in failures) {
      notif.addNotification(NotificationItem(
        id: 'bulk_fail_${DateTime.now().millisecondsSinceEpoch}_$f',
        message: (l10n) => l10n.modInstallFailed(f),
        icon: Icons.error_outline,
        color: AppColors.error,
        type: NotificationType.general,
      ));
    }
  }

  Widget _looseInstallButton(AppLocalizations l10n) {
    return HoverIconButton(
      tooltip: l10n.modLooseInstall,
      bordered: false,
      padding: EdgeInsets.all(AppSizes.paddingXS(context)),
      icon: Icon(
        Icons.note_add_outlined,
        size: AppSizes.iconLG(context),
        color: AppColors.textMuted,
      ),
      onTap: _handleLooseInstall,
    );
  }

  Future<void> _handleLooseInstall() async {
    final l10n = AppLocalizations.of(context)!;
    final notif = ref.read(notificationStateControllerProvider.notifier);

    final folder = await FilePicker.getDirectoryPath();
    if (folder == null || !mounted) return;

    setState(() {
      _busy = true;
      _busyMessage = l10n.modLooseInstallScanning;
    });

    final count = await IsolateService.run(_countLooseFilesSync, folder);
    if (!mounted) return;
    if (count == 0) {
      setState(() => _busy = false);
      notif.addNotification(NotificationItem(
        id: 'loose_none_${DateTime.now().millisecondsSinceEpoch}',
        message: (l10n) => l10n.modLooseInstallNone,
        icon: Icons.info_outline,
        color: AppColors.warning,
        type: NotificationType.general,
      ));
      return;
    }

    setState(() => _busyMessage = l10n.modLooseInstallBusy(count));

    InstallResult result;
    try {
      result = await withBusyGuard(
        ref,
        () => _installLooseFolder(
          folder,
          (done) {
            if (mounted) {
              setState(() =>
                  _busyMessage = l10n.modLooseInstallProgress(done, count));
            }
          },
          () {
            if (mounted) {
              setState(() => _busyMessage = l10n.modLooseInstallFinalizing);
            }
          },
        ),
      );
    } finally {
      if (mounted) setState(() => _busy = false);
    }
    if (!mounted) return;

    await ref.read(modsStateControllerProvider.notifier).loadMods(_gameDir);
    if (!mounted) return;

    if (result.success) {
      notif.addNotification(NotificationItem(
        id: 'loose_done_${DateTime.now().millisecondsSinceEpoch}',
        message: (l10n) =>
            l10n.modLooseInstallDone(count, result.installedId ?? ''),
        icon: Icons.check_circle,
        color: AppColors.success,
        type: NotificationType.general,
      ));
      if (result.unpairedWarnings.isNotEmpty) {
        notif.addNotification(NotificationItem(
          id: 'loose_unpaired_${DateTime.now().millisecondsSinceEpoch}',
          message: (l10n) =>
              l10n.modLooseUnpairedWarning(result.unpairedWarnings.join(', ')),
          icon: Icons.warning_amber,
          color: AppColors.warning,
          type: NotificationType.general,
        ));
      }
    } else {
      notif.addNotification(NotificationItem(
        id: 'loose_fail_${DateTime.now().millisecondsSinceEpoch}',
        message: (l10n) =>
            l10n.modInstallFailed(_localizeReason(l10n, result.errorMessage ?? 'unknown')),
        icon: Icons.error_outline,
        color: AppColors.error,
        type: NotificationType.general,
      ));
    }
  }

  Future<InstallResult> _installLooseFolder(
    String folder,
    void Function(int done) onCopyProgress,
    VoidCallback onFinalizing,
  ) async {
    final staged = await IsolateService.runWithProgress<String, String?>(
      _stageLooseFilesSync,
      folder,
      (msg) {
        final done = int.tryParse(msg);
        if (done != null) onCopyProgress(done);
      },
    );
    if (staged == null) return const InstallResult.fail('data_empty');
    onFinalizing();
    try {
      final baseName = prettifyModId(p.basename(folder));
      var name = baseName;
      for (var attempt = 2; attempt < 100; attempt++) {
        final result = await ModsService.install(
          _gameDir,
          staged,
          requestedName: name,
        );
        if (result.success ||
            result.errorMessage?.startsWith('exists:') != true) {
          return result;
        }
        name = '$baseName $attempt';
      }
      return const InstallResult.fail('exists');
    } finally {
      try {
        Directory(staged).deleteSync(recursive: true);
      } catch (_) {}
    }
  }

  /// Installs one archive without any dialogs: name collisions get a numeric
  /// suffix, and multi-variant packs install every variant.
  Future<InstallResult> _installOneForBulk(
    String archive,
    String requestedName,
  ) async {
    final detect = await ModsService.detectDrop(archive);

    if (detect.hasVariants) {
      final requests = [
        for (final v in detect.variants)
          VariantInstallRequest(
            requestedName: '$requestedName - ${v.label}',
            variantSubPath: v.subPath,
          ),
      ];
      final results =
          await ModsService.installVariants(_gameDir, archive, requests);
      final ok = results.where((r) => r.success).length;
      if (ok == 0) {
        return InstallResult.fail(
          results.firstOrNull?.errorMessage ?? 'unknown',
        );
      }
      return InstallResult.ok(results.firstWhere((r) => r.success).installedId!);
    }

    var name = requestedName;
    for (var attempt = 2; attempt < 100; attempt++) {
      final result = await ModsService.install(
        _gameDir,
        archive,
        requestedName: name,
      );
      if (result.success ||
          result.errorMessage?.startsWith('exists:') != true) {
        return result;
      }
      name = '$requestedName $attempt';
    }
    return const InstallResult.fail('exists');
  }

  Widget _helpIconButton(AppLocalizations l10n) {
    return KeyedSubtree(
      key: _helpIconKey,
      child: Tooltip(
        message: l10n.modsTutorialHelpTooltip,
        child: PopupMenuButton<TutorialKind>(
          icon: Icon(
            Icons.help_outline,
            size: AppSizes.iconLG(context),
            color: AppColors.textMuted,
          ),
          color: AppColors.backgroundCard,
          onSelected: (kind) => _startTutorial(kind),
          itemBuilder: (ctx) => [
            PopupMenuItem(
              value: TutorialKind.ecosystem,
              child: Row(
                children: [
                  Icon(Icons.hub_outlined,
                      size: 16, color: AppColors.accentPrimary),
                  const SizedBox(width: 8),
                  Text(
                    l10n.modsTutorialMenuEcosystem,
                    style: TextStyle(
                      color: AppColors.textPrimary,
                      fontSize: AppSizes.fontSM(context),
                    ),
                  ),
                ],
              ),
            ),
            PopupMenuItem(
              value: TutorialKind.installMod,
              child: Row(
                children: [
                  Icon(Icons.download_outlined,
                      size: 16, color: AppColors.accentPrimary),
                  const SizedBox(width: 8),
                  Text(
                    l10n.modsTutorialMenuInstall,
                    style: TextStyle(
                      color: AppColors.textPrimary,
                      fontSize: AppSizes.fontSM(context),
                    ),
                  ),
                ],
              ),
            ),
            PopupMenuItem(
              value: TutorialKind.profiles,
              child: Row(
                children: [
                  Icon(Icons.layers_outlined,
                      size: 16, color: AppColors.accentPrimary),
                  const SizedBox(width: 8),
                  Text(
                    l10n.modsTutorialMenuProfiles,
                    style: TextStyle(
                      color: AppColors.textPrimary,
                      fontSize: AppSizes.fontSM(context),
                    ),
                  ),
                ],
              ),
            ),
            PopupMenuItem(
              value: TutorialKind.supporting,
              child: Row(
                children: [
                  Icon(Icons.favorite_outline,
                      size: 16, color: AppColors.accentPrimary),
                  const SizedBox(width: 8),
                  Text(
                    l10n.modsTutorialMenuSupporting,
                    style: TextStyle(
                      color: AppColors.textPrimary,
                      fontSize: AppSizes.fontSM(context),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  void _startTutorial(TutorialKind kind) {
    showModsTutorial(
      context,
      kind: kind,
      anchors: ModsTutorialAnchors(
        dropZoneKey: _dropZoneKey,
        listKey: _listKey,
        detailKey: _detailKey,
        profileSelectorKey: _profileSelectorKey,
        helpIconKey: _helpIconKey,
        texturesTabKey: SidebarKeys.texturesTab,
        cutscenesTabKey: SidebarKeys.cutscenesTab,
      ),
    );
  }

  Widget _searchBar(AppLocalizations l10n) {
    return TextField(
      controller: _searchController,
      onChanged: (v) => setState(() => _filter = v),
      style: TextStyle(color: AppColors.textPrimary, fontSize: AppSizes.fontMD(context)),
      decoration: InputDecoration(
        isDense: true,
        contentPadding: EdgeInsets.symmetric(
          horizontal: AppSizes.cardPaddingH(context),
          vertical: AppSizes.cardPaddingV(context),
        ),
        hintText: l10n.modSearchPlaceholder,
        hintStyle: TextStyle(color: AppColors.textMuted, fontSize: AppSizes.fontMD(context)),
        filled: true,
        fillColor: AppColors.inputBackground,
        prefixIcon: Icon(Icons.search, size: AppSizes.iconSM(context), color: AppColors.textMuted),
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
          borderSide: BorderSide.none,
        ),
      ),
    );
  }

  Widget _list(
    List<InstalledMod> mods,
    bool loading,
    AppLocalizations l10n,
  ) {
    if (loading && mods.isEmpty) {
      return Center(
        child: SizedBox(
          width: AppSizes.iconLG(context),
          height: AppSizes.iconLG(context),
          child: const CircularProgressIndicator(strokeWidth: 2, color: AppColors.accentPrimary),
        ),
      );
    }
    if (mods.isEmpty) {
      return Center(
        child: Padding(
          padding: EdgeInsets.all(AppSizes.paddingLG(context)),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                l10n.modListEmpty,
                style: TextStyle(
                  color: AppColors.textMuted,
                  fontStyle: FontStyle.italic,
                  fontSize: AppSizes.fontMD(context),
                ),
              ),
              SizedBox(height: AppSizes.spacingMD(context)),
              Text(
                l10n.modListEmptyHint,
                style: TextStyle(color: AppColors.textMuted, fontSize: AppSizes.fontSM(context)),
                textAlign: TextAlign.center,
              ),
            ],
          ),
        ),
      );
    }

    return Container(
      decoration: BoxDecoration(
        color: AppColors.backgroundCard,
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(color: AppColors.borderLight),
      ),
      child: Scrollbar(
        controller: _listScrollController,
        thumbVisibility: true,
        child: ListView(
          controller: _listScrollController,
          padding: EdgeInsets.zero,
          children: _listChildren(mods),
        ),
      ),
    );
  }

  List<Widget> _listChildren(List<InstalledMod> mods) {
    if (_filter.isNotEmpty) {
      return [for (final m in mods) _row(m)];
    }
    final grouped = groupMods(mods);
    final children = <Widget>[];
    for (final entry in grouped.entries) {
      final collapsed = _collapsedGroups.contains(entry.key);
      children.add(ModGroupHeader(
        key: ValueKey('group_${entry.key.name}'),
        kind: entry.key,
        count: entry.value.length,
        collapsed: collapsed,
        onTap: () => setState(() {
          if (!_collapsedGroups.remove(entry.key)) {
            _collapsedGroups.add(entry.key);
          }
        }),
      ));
      if (collapsed) continue;
      for (final m in entry.value) {
        children.add(_row(m, group: entry.key));
      }
    }
    return children;
  }

  Widget _row(InstalledMod m, {ModGroupKind? group}) {
    return ModListRow(
      key: ValueKey('row_${group?.name ?? 'flat'}_${m.id}'),
      mod: m,
      selected: m.id == _selectedId,
      onTap: () => setState(() => _selectedId = m.id),
    );
  }

  /// Returns the classification for a drop. `valid` is true when it's a
  /// NAMS mod. `misroutedTab` is non-null when the drop looks like content
  /// for a different tab (e.g. a `data/movie/` only overlay → Cutscenes).
  Future<_ModDropClassification> _dropLooksLikeMod(String sourcePath) async {
    if (ArchiveService.isArchive(sourcePath)) {
      final entries = await ArchiveService.listEntries(sourcePath);
      if (entries == null) return const _ModDropClassification(valid: false);
      return _classifyEntryPaths(entries);
    }
    if (FileSystemEntity.isFileSync(sourcePath)) {
      return _ModDropClassification(
        valid: _isLoosePlFile(p.basename(sourcePath)),
      );
    }
    if (FileSystemEntity.isDirectorySync(sourcePath)) {
      return IsolateService.run(_dirLooksLikeModSync, sourcePath);
    }
    return const _ModDropClassification(valid: false);
  }
}

class _ModDropClassification {
  final bool valid;
  final String? misroutedTab;
  const _ModDropClassification({required this.valid, this.misroutedTab});
}

const _bulkArchiveExts = {'.zip', '.7z', '.rar'};

/// Collects mod archives from a folder and its immediate subfolders, which is
/// how Nexus downloads are usually laid out (one subfolder per mod page).
List<String> _findArchivesSync(String root) {
  final out = <String>[];
  void scan(Directory dir, int depth) {
    if (depth > 2) return;
    List<FileSystemEntity> entries;
    try {
      entries = dir.listSync(followLinks: false);
    } catch (_) {
      return;
    }
    for (final entity in entries) {
      if (entity is Directory) {
        scan(entity, depth + 1);
      } else if (entity is File &&
          _bulkArchiveExts.contains(p.extension(entity.path).toLowerCase())) {
        out.add(entity.path);
      }
    }
  }

  final dir = Directory(root);
  if (!dir.existsSync()) return out;
  scan(dir, 1);
  out.sort((a, b) => a.toLowerCase().compareTo(b.toLowerCase()));
  return out;
}

int _countLooseFilesSync(String root) {
  final dir = Directory(root);
  if (!dir.existsSync()) return 0;
  var count = 0;
  for (final entity in dir.listSync(followLinks: false).whereType<File>()) {
    final ext = p.extension(entity.path).toLowerCase();
    if (ext == '.dat' || ext == '.dtt') count++;
  }
  return count;
}

String? _stageLooseFilesSync(String root, void Function(String) report) {
  final dir = Directory(root);
  if (!dir.existsSync()) return null;
  final loose = dir
      .listSync(followLinks: false)
      .whereType<File>()
      .where((f) {
        final ext = p.extension(f.path).toLowerCase();
        return ext == '.dat' || ext == '.dtt';
      })
      .toList();
  if (loose.isEmpty) return null;
  final staged = Directory.systemTemp.createTempSync('yp_loose_stage_');
  var done = 0;
  for (final f in loose) {
    f.copySync(p.join(staged.path, p.basename(f.path)));
    done++;
    if (done % 100 == 0 || done == loose.length) {
      report('$done');
    }
  }
  return staged.path;
}

/// Subdirs accepted INSIDE a `data/` overlay. Includes everything NAMS
/// recognises.
const _modDataSubdirs = <String>{
  'pl', 'wp', 'em', 'ba', 'bg', 'bh', 'et', 'it', 'um',
  'wd1', 'wd2', 'wd3', 'wd4', 'wd5', 'wda',
  'core', 'ph1', 'ph2', 'ph3', 'phf', 'quest', 'st1', 'st2', 'st5',
  'novel', 'subtitle', 'txtmess', 'ui',
  'effect', 'enlighten', 'font', 'misctex', 'movie', 'sound',
};

/// Subdirs accepted at the ROOT of a drop (bare-outfit / weapon / effect /
/// etc. legacy layouts). Excludes generic names like `movie/`, `sound/`,
/// `font/`, `ui/` that are ambiguous and belong on other tabs (e.g. a loose
/// `movie/` at the root is a Cutscenes tab drop, not a NAMS mod).
const _modRootDataSubdirs = <String>{
  'pl', 'wp', 'em', 'ba', 'bg', 'bh', 'et', 'it', 'um',
  'wd1', 'wd2', 'wd3', 'wd4', 'wd5', 'wda',
  'core', 'ph1', 'ph2', 'ph3', 'phf', 'quest', 'st1', 'st2', 'st5',
  'effect', 'enlighten', 'misctex',
};

bool _isLoosePlFile(String fileName) {
  final lower = fileName.toLowerCase();
  return lower.startsWith('pl') &&
      (lower.endsWith('.dat') || lower.endsWith('.dtt'));
}

/// Inspects every entry path in a drop and decides whether it's a real
/// NAMS mod, ignorable, or content for a different tab.
///
/// Strong "real NAMS mod" signals: `mod.toml`, `entities/`, `wax/`,
/// `data/<datasubdir except 'movie' alone>/`, root-level legacy data dirs
/// (`pl/`, `wp/`, etc.), loose `pl*.dat`/`.dtt`.
///
/// Misroute redirect: drop has only `data/movie/...` (cutscenes-as-overlay)
/// or only a loose `movie/` at root, with no other mod content → tell the
/// user to use the Cutscenes tab.
_ModDropClassification _classifyEntryPaths(Iterable<String> rawEntries) {
  // Strip wrapper if all paths share the same first segment.
  final entries = _stripCommonWrapper(
    rawEntries.map((e) => e.toLowerCase().replaceAll('\\', '/')).toList(),
  );

  bool hasRealModSignal = false;
  bool hasOnlyDataMovie = false;
  bool hasOtherDataContent = false;
  bool hasLooseMovie = false;
  bool hasOtherContent = false;

  for (final entry in entries) {
    if (entry.isEmpty) continue;
    final segments = entry.split('/');
    final first = segments[0];
    if (first.isEmpty) continue;

    // Nested variant signals: a pl model file, a mod.toml, a .dds texture,
    // or a data/<subdir> anywhere below the root means at least one installable
    // variant exists. detectDrop resolves which; here we just don't reject.
    final last = segments.last;
    if (_isLoosePlFile(last) ||
        last == 'mod.toml' ||
        last.endsWith('.dds') ||
        last.endsWith('.cpk')) {
      hasRealModSignal = true;
      continue;
    }
    for (var i = 0; i < segments.length - 1; i++) {
      final seg = segments[i];
      if (seg == 'entities' || seg == 'wax') {
        hasRealModSignal = true;
        break;
      }
      if (seg == 'data' &&
          i + 1 < segments.length &&
          segments[i + 1] != 'movie' &&
          _modDataSubdirs.contains(segments[i + 1])) {
        hasRealModSignal = true;
        break;
      }
    }
    if (hasRealModSignal) continue;

    // Strong NAMS-mod root signals.
    if (first == 'mod.toml' || first == 'entities' || first == 'wax') {
      hasRealModSignal = true;
      continue;
    }
    if (_isLoosePlFile(first)) {
      hasRealModSignal = true;
      continue;
    }
    if (_modRootDataSubdirs.contains(first)) {
      hasRealModSignal = true;
      continue;
    }

    // `data/<subdir>/...` — distinguish "only data/movie" from real overlays.
    if (first == 'data' && segments.length >= 2) {
      final child = segments[1];
      if (child == 'movie') {
        hasOnlyDataMovie = true;
      } else if (_modDataSubdirs.contains(child)) {
        hasOtherDataContent = true;
        hasRealModSignal = true;
      }
      continue;
    }

    // Loose `movie/` at root (no `data/` wrapper) → cutscene-tab content.
    if (first == 'movie') {
      hasLooseMovie = true;
      continue;
    }

    // Anything else at root that isn't an obvious NAMS marker counts as
    // "other content" — used to detect mods that ship cutscenes alongside
    // unrelated junk vs cleanly cutscene-only drops.
    if (segments.length == 1) {
      // It's a top-level file or dir name we don't recognise.
      hasOtherContent = true;
    }
  }

  if (hasRealModSignal) {
    return const _ModDropClassification(valid: true);
  }

  // Redirect heuristic: drop is purely cutscene content.
  if ((hasOnlyDataMovie || hasLooseMovie) && !hasOtherDataContent) {
    return const _ModDropClassification(
      valid: false,
      misroutedTab: 'cutscenes',
    );
  }

  return _ModDropClassification(
    valid: false,
    misroutedTab: hasOtherContent ? null : null,
  );
}

/// If every entry shares the same first path segment AND it's not a
/// recognised NAMS root marker, treat that segment as a wrapper and strip
/// it. Mirrors `_unwrapSingleChild` in mods_service.
List<String> _stripCommonWrapper(List<String> entries) {
  var current = entries;
  for (var i = 0; i < 4; i++) {
    final stripped = _stripOneWrapper(current);
    if (identical(stripped, current)) return current;
    current = stripped;
  }
  return current;
}

List<String> _stripOneWrapper(List<String> entries) {
  if (entries.isEmpty) return entries;
  String? wrapper;
  for (final e in entries) {
    final firstSlash = e.indexOf('/');
    final root = firstSlash == -1 ? e : e.substring(0, firstSlash);
    if (wrapper == null) {
      wrapper = root;
    } else if (wrapper != root) {
      return entries;
    }
  }
  if (wrapper == null || wrapper.isEmpty) return entries;
  if (wrapper == 'mod.toml' ||
      wrapper == 'entities' ||
      wrapper == 'wax' ||
      wrapper == 'data' ||
      wrapper == 'movie' ||
      _modRootDataSubdirs.contains(wrapper) ||
      wrapper.endsWith('.cpk') ||
      _isLoosePlFile(wrapper)) {
    return entries;
  }
  return entries
      .map((e) {
        if (e == wrapper) return '';
        if (e.startsWith('$wrapper/')) {
          return e.substring(wrapper!.length + 1);
        }
        return e;
      })
      .where((e) => e.isNotEmpty)
      .toList();
}

_ModDropClassification _dirLooksLikeModSync(String dirPath) {
  try {
    final dir = Directory(dirPath);
    if (!dir.existsSync()) return const _ModDropClassification(valid: false);
    final paths = _collectShallowPaths(dir, 4);
    return _classifyEntryPaths(paths);
  } catch (_) {}
  return const _ModDropClassification(valid: false);
}

List<String> _collectShallowPaths(Directory root, int maxDepth) {
  final out = <String>[];
  void walk(Directory dir, String prefix, int depth) {
    if (depth > maxDepth) return;
    List<FileSystemEntity> entries;
    try {
      entries = dir.listSync(followLinks: false);
    } catch (_) {
      return;
    }
    for (final entity in entries) {
      final name = p.basename(entity.path);
      final rel = prefix.isEmpty ? name : '$prefix/$name';
      out.add(rel);
      if (entity is Directory) {
        walk(entity, rel, depth + 1);
      }
    }
  }
  walk(root, '', 1);
  return out;
}
