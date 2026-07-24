import 'dart:io';

import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path/path.dart' as p;
import 'package:url_launcher/url_launcher.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/notification_state.dart';
import 'package:yp_launcher/providers/thirdparty_state.dart';
import 'package:yp_launcher/services/archive_service.dart';
import 'package:yp_launcher/services/file_ops.dart';
import 'package:yp_launcher/services/thirdparty/migoto_runtime.dart';
import 'package:yp_launcher/services/thirdparty/reshade_runtime.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_models.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_paths.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/config_field_bool.dart';
import 'package:yp_launcher/widgets/config_field_dropdown.dart';
import 'package:yp_launcher/widgets/hover_button.dart';
import 'package:yp_launcher/widgets/mods/mod_drop_zone.dart';

const _thirdPartyTabIndex = 9;

class ThirdPartyView extends ConsumerStatefulWidget {
  const ThirdPartyView({super.key});

  @override
  ConsumerState<ThirdPartyView> createState() => _ThirdPartyViewState();
}

class _ThirdPartyViewState extends ConsumerState<ThirdPartyView> {
  final _scrollController = ScrollController();
  bool _loaded = false;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_loaded) return;
      _loaded = true;
      ref.read(thirdPartyStateControllerProvider.notifier).refresh();
    });
  }

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  String get _gameDir =>
      ref.read(appStateControllerProvider).selectedDirectory;

  Future<void> _handleDrop(List<String> paths) async {
    if (ref.read(activeTabProvider) != _thirdPartyTabIndex) return;
    for (final path in paths) {
      await _installFromPath(path);
    }
  }

  Future<void> _browse() async {
    final result = await FilePicker.pickFiles(
      type: FileType.custom,
      allowedExtensions: const ['zip', '7z', 'rar', 'ini', 'exe'],
    );
    final path = result?.files.single.path;
    if (path != null) await _installFromPath(path);
  }

  Future<void> _browseFolder() async {
    final path = await FilePicker.getDirectoryPath();
    if (path != null) await _installFromPath(path);
  }

  String _modNameFromPath(String filePath) {
    var name = p.basenameWithoutExtension(filePath);
    name = name.replaceAll(RegExp(r'[_]+'), ' ');
    name = name.replaceAll(RegExp(r'-\d+$'), '');
    name = name.replaceAll(RegExp(r'\s+'), ' ').trim();
    return name.isEmpty ? p.basename(filePath) : name;
  }

  Future<void> _installFromPath(String path) async {
    final notifier = ref.read(thirdPartyStateControllerProvider.notifier);

    String sourceRoot = path;
    Directory? extracted;
    if (ArchiveService.isArchive(path) ||
        path.toLowerCase().endsWith('.exe')) {
      final out = await ArchiveService.extract(path);
      if (out == null) {
        _notify(
          (l10n) => l10n.thirdPartyInstallFailed,
          Icons.error_outline,
          AppColors.error,
        );
        return;
      }
      extracted = Directory(out);
      sourceRoot = out;
    } else if (path.toLowerCase().endsWith('.ini')) {
      final tmp = Directory.systemTemp.createTempSync('yp_tp_ini_');
      FileOps.copyFileInto(path, tmp.path);
      extracted = tmp;
      sourceRoot = tmp.path;
    }

    try {
      final classified = await notifier.classify(sourceRoot);
      if (classified == null) return;
      final c = classified.withSourceName(_modNameFromPath(path));
      switch (c.kind) {
        case ThirdPartyKind.reshadeWholeInstall:
        case ThirdPartyKind.reshadePreset:
        case ThirdPartyKind.migoto:
          final update = await notifier.wouldUpdate(c);
          if (update != null) {
            if (!mounted) return;
            final replace = await _confirmReplace(update);
            if (replace != true) {
              _notify((l10n) => l10n.thirdPartyUpdateSkipped, Icons.info_outline,
                  AppColors.textMuted);
              break;
            }
          }
          final result = await notifier.install(c);
          if (result?.ok == true) {
            _notify((l10n) => l10n.thirdPartyInstalled, Icons.check_circle,
                AppColors.success);
          } else {
            _notify((l10n) => l10n.thirdPartyInstallFailed, Icons.error_outline,
                AppColors.error);
          }
          break;
        case ThirdPartyKind.gameData:
          _notify((l10n) => l10n.thirdPartyRedirectMods, Icons.info_outline,
              AppColors.accentPrimary);
          break;
        case ThirdPartyKind.textures:
          _notify((l10n) => l10n.thirdPartyRedirectTextures, Icons.info_outline,
              AppColors.accentPrimary);
          break;
        case ThirdPartyKind.lodmod:
        case ThirdPartyKind.specialK:
        case ThirdPartyKind.skShaderEdit:
        case ThirdPartyKind.brokenLegacy:
        case ThirdPartyKind.multiBundle:
        case ThirdPartyKind.unknown:
          _notify((l10n) => l10n.thirdPartyUnsupported, Icons.warning_amber,
              AppColors.warning);
          break;
      }
    } finally {
      if (extracted != null) FileOps.deleteDirQuiet(extracted.path);
    }
  }

  String _runtimeName(ThirdPartyRuntime which) =>
      which == ThirdPartyRuntime.reshade ? 'ReShade' : '3DMigoto';

  Widget _updateDiff(
    AppLocalizations l10n,
    BuildContext ctx,
    ThirdPartyUpdateInfo update,
  ) {
    Widget row(String label, String value, Color color) => Padding(
          padding: EdgeInsets.symmetric(vertical: AppSizes.spacingSM(ctx) / 2),
          child: Row(
            children: [
              SizedBox(
                width: 96,
                child: Text(
                  label,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(ctx),
                    color: AppColors.textMuted,
                  ),
                ),
              ),
              Expanded(
                child: Text(
                  value,
                  style: TextStyle(
                    fontSize: AppSizes.fontSM(ctx),
                    color: color,
                    fontWeight: FontWeight.w600,
                  ),
                ),
              ),
            ],
          ),
        );
    return Container(
      padding: EdgeInsets.all(AppSizes.paddingSM(ctx)),
      decoration: BoxDecoration(
        color: AppColors.surfaceMedium,
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(ctx)),
        border: Border.all(color: AppColors.borderLight),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          row(l10n.thirdPartyUpdateInstalledLabel,
              update.installedLabel ?? '—', AppColors.textSecondary),
          row(l10n.thirdPartyUpdateIncomingLabel,
              update.incomingLabel ?? '—', AppColors.accentPrimary),
        ],
      ),
    );
  }

  Future<bool?> _confirmReplace(ThirdPartyUpdateInfo update) {
    final l10n = AppLocalizations.of(context)!;
    final name = _runtimeName(update.runtime);
    return showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: AppColors.backgroundCard,
        title: Text(
          l10n.thirdPartyUpdateTitle(name),
          style: TextStyle(
            fontSize: AppSizes.fontXL(ctx),
            color: AppColors.textPrimary,
          ),
        ),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              l10n.thirdPartyUpdateBody(name),
              style: TextStyle(
                fontSize: AppSizes.fontMD(ctx),
                color: AppColors.textSecondary,
              ),
            ),
            if (update.installedLabel != null || update.incomingLabel != null)
              Padding(
                padding: EdgeInsets.only(top: AppSizes.spacingMD(ctx)),
                child: _updateDiff(l10n, ctx, update),
              ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(false),
            child: Text(
              l10n.thirdPartyUpdateKeep,
              style: const TextStyle(color: AppColors.textMuted),
            ),
          ),
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(true),
            child: Text(
              l10n.thirdPartyUpdateReplace,
              style: const TextStyle(
                color: AppColors.accentPrimary,
                fontWeight: FontWeight.bold,
              ),
            ),
          ),
        ],
      ),
    );
  }

  void _notify(
    String Function(AppLocalizations l10n) message,
    IconData icon,
    Color color,
  ) {
    ref.read(notificationStateControllerProvider.notifier).addNotification(
          NotificationItem(
            id: 'thirdparty_${DateTime.now().millisecondsSinceEpoch}',
            message: message,
            icon: icon,
            color: color,
            type: NotificationType.general,
          ),
        );
  }

  Future<void> _revealShaders() async {
    final dir = p.join(
      ThirdPartyPaths.reshade(),
      'reshade-shaders',
      'Shaders',
    );
    if (Directory(dir).existsSync()) {
      await launchUrl(Uri.file(dir));
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final data = ref.watch(thirdPartyStateControllerProvider);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildHeader(l10n),
        Expanded(
          child: SingleChildScrollView(
            controller: _scrollController,
            padding: EdgeInsets.all(AppSizes.contentPadding(context)),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                _infoBanner(l10n.thirdPartyBanner),
                LayoutBuilder(
              builder: (context, constraints) {
                final stack = constraints.maxWidth < 760;
                final leftCol = Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    _dropCard(l10n),
                    _reshadeCard(l10n, data.reshade),
                  ],
                );
                final rightCol = Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    _migotoCard(l10n, data.migoto),
                  ],
                );
                if (stack) {
                  return Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [leftCol, rightCol],
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
              ],
            ),
          ),
        ),
      ],
    );
  }

  Widget _infoBanner(String text) {
    return Container(
      width: double.infinity,
      margin: EdgeInsets.only(bottom: AppSizes.paddingLG(context)),
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.paddingMD(context),
        vertical: AppSizes.paddingSM(context),
      ),
      decoration: BoxDecoration(
        color: AppColors.accentPrimary.withValues(alpha: 0.06),
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(
          color: AppColors.accentPrimary.withValues(alpha: 0.25),
        ),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(
            Icons.shield_outlined,
            size: AppSizes.iconLG(context),
            color: AppColors.accentPrimary,
          ),
          SizedBox(width: AppSizes.spacingMD(context)),
          Expanded(
            child: Text(
              text,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textSecondary,
                height: 1.4,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildHeader(AppLocalizations l10n) {
    return Padding(
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.contentPadding(context),
        vertical: AppSizes.paddingMD(context),
      ),
      child: Row(
        children: [
          Text(
            l10n.thirdPartyTitle,
            style: TextStyle(
              fontSize: AppSizes.fontXL(context),
              fontWeight: FontWeight.bold,
              color: AppColors.accentPrimary,
            ),
          ),
          SizedBox(width: AppSizes.spacingSM(context)),
          Tooltip(
            message: l10n.thirdPartySubtitle,
            child: Icon(
              Icons.info_outline,
              size: AppSizes.iconMD(context),
              color: AppColors.textMuted,
            ),
          ),
        ],
      ),
    );
  }

  Widget _dropCard(AppLocalizations l10n) {
    return _card(l10n.thirdPartyInstallCard, [
      ModDropZone(
        onDrop: _handleDrop,
        onBrowse: _browse,
        onBrowseFolder: _browseFolder,
        title: l10n.thirdPartyDropHere,
      ),
    ]);
  }

  Widget _reshadeCard(AppLocalizations l10n, ThirdPartyRuntimeStatus status) {
    const runtime = ReShadeRuntime();
    final foundInGame =
        !status.installed && runtime.findGameRootDll(_gameDir) != null;

    if (!status.installed) {
      return _card(l10n.thirdPartyReShadeHeader, [
        _statusRow(l10n, status, foundInGame),
        SizedBox(height: AppSizes.spacingSM(context)),
        _howto(l10n.thirdPartyReShadeHowto),
        Align(
          alignment: Alignment.centerLeft,
          child: HoverButton(
            label: l10n.thirdPartyGetReShade,
            color: AppColors.accentPrimary,
            onTap: () => launchUrl(Uri.parse(AppStrings.reshadeMeUrl)),
          ),
        ),
      ]);
    }

    final info = status.reshadeInfo ?? const ReShadeInfo();
    final reshadeDir = ThirdPartyPaths.reshade();
    return _card(
      l10n.thirdPartyReShadeHeader,
      [
        if (info.version != null || info.dllName != null)
          _kv(
            l10n.thirdPartyVersion,
            [
              if (info.version != null) info.version!,
              if (info.isAddonBuild) l10n.thirdPartyAddonBuild,
            ].join(' · '),
          ),
        _section(
          l10n.thirdPartyPresets,
          info.presets,
          emptyLabel: l10n.thirdPartyNonePresets,
        ),
        _section(
          l10n.thirdPartyShaderRepos,
          [
            ...info.shaderRepos,
          ],
          countSuffix: l10n.thirdPartyShaderCount(info.shaderCount),
          emptyLabel: l10n.thirdPartyNoneShaders,
        ),
        if (info.addons.isNotEmpty)
          _section(
            l10n.thirdPartyAddons,
            info.addons,
          ),
        _reshadeConfig(l10n, info.config),
        if (status.shadersMissing) _shadersMissing(l10n),
        if (info.d3dCompilerMissing) _d3dCompilerMissing(l10n),
      ],
      trailing: _statusChip(l10n, status.enabled),
      actions: _iconActions(l10n, reshadeDir, ThirdPartyRuntime.reshade),
    );
  }

  Widget _reshadeConfig(AppLocalizations l10n, ReShadeConfig c) {
    void apply(ReShadeConfig next) => ref
        .read(thirdPartyStateControllerProvider.notifier)
        .applyReShadeConfig(next);
    return _configGroup(l10n, [
      if (c.activePreset != null)
        _kv(l10n.thirdPartyReShadeActivePreset, c.activePreset!),
      ConfigFieldBool(
        label: l10n.thirdPartyReShadePerformanceMode,
        tooltip: l10n.thirdPartyReShadePerformanceModeHint,
        value: c.performanceMode,
        onChanged: (v) => apply(c.copyWith(performanceMode: v)),
      ),
      ConfigFieldBool(
        label: l10n.thirdPartyReShadeShowFps,
        tooltip: l10n.thirdPartyReShadeShowFpsHint,
        value: c.showFps,
        onChanged: (v) => apply(c.copyWith(showFps: v)),
      ),
      ConfigFieldBool(
        label: l10n.thirdPartyReShadeShowClock,
        tooltip: l10n.thirdPartyReShadeShowClockHint,
        value: c.showClock,
        onChanged: (v) => apply(c.copyWith(showClock: v)),
      ),
    ]);
  }

  Widget _migotoCard(AppLocalizations l10n, ThirdPartyRuntimeStatus status) {
    const runtime = MigotoRuntime();
    final foundInGame =
        !status.installed && runtime.findGameRootDll(_gameDir) != null;

    if (!status.installed) {
      return _card(l10n.thirdPartyMigotoHeader, [
        _statusRow(l10n, status, foundInGame),
        SizedBox(height: AppSizes.spacingSM(context)),
        _howto(l10n.thirdPartyMigotoHowto),
      ]);
    }

    final info = status.migotoInfo ?? const MigotoInfo();
    final migotoDir = ThirdPartyPaths.migoto();
    return _card(
      l10n.thirdPartyMigotoHeader,
      [
        if (!info.loaderTargetOk) _loaderRow(l10n, info),
        _section(
          l10n.thirdPartyShaderFixes,
          info.shaderFixNames,
          countSuffix: l10n.thirdPartyShaderFixCount(info.shaderFixCount),
          emptyLabel: l10n.thirdPartyShaderFixCount(0),
        ),
        _migotoConfig(l10n, info.config),
        _section(
          l10n.thirdPartyFiles,
          info.files,
          emptyLabel: '',
        ),
      ],
      trailing: _statusChip(l10n, status.enabled),
      actions: _iconActions(l10n, migotoDir, ThirdPartyRuntime.migoto),
    );
  }

  Widget _migotoConfig(AppLocalizations l10n, MigotoConfig c) {
    void apply(MigotoConfig next) => ref
        .read(thirdPartyStateControllerProvider.notifier)
        .applyMigotoConfig(next);
    return _configGroup(l10n, [
      ConfigFieldDropdown(
        label: l10n.thirdPartyMigotoHunting,
        tooltip: l10n.thirdPartyMigotoHuntingHint,
        value: c.hunting.index,
        options: const [0, 1, 2],
        labels: {
          MigotoHunting.off.index: l10n.thirdPartyMigotoHuntingOff,
          MigotoHunting.on.index: l10n.thirdPartyMigotoHuntingOn,
          MigotoHunting.noMarking.index: l10n.thirdPartyMigotoHuntingNoMarking,
        },
        onChanged: (v) =>
            apply(c.copyWith(hunting: MigotoHunting.values[v])),
      ),
      if (c.hunting != MigotoHunting.off)
        ConfigFieldDropdown(
          label: l10n.thirdPartyMigotoMarkingMode,
          tooltip: l10n.thirdPartyMigotoMarkingModeHint,
          value: c.markingMode.index,
          options: const [0, 1, 2, 3],
          labels: {
            MigotoMarking.skip.index: l10n.thirdPartyMigotoMarkingSkip,
            MigotoMarking.original.index: l10n.thirdPartyMigotoMarkingOriginal,
            MigotoMarking.pink.index: l10n.thirdPartyMigotoMarkingPink,
            MigotoMarking.mono.index: l10n.thirdPartyMigotoMarkingMono,
          },
          onChanged: (v) =>
              apply(c.copyWith(markingMode: MigotoMarking.values[v])),
        ),
      ConfigFieldBool(
        label: l10n.thirdPartyMigotoCacheShaders,
        tooltip: l10n.thirdPartyMigotoCacheShadersHint,
        value: c.cacheShaders,
        onChanged: (v) => apply(c.copyWith(cacheShaders: v)),
      ),
      ConfigFieldBool(
        label: l10n.thirdPartyMigotoVerboseOverlay,
        tooltip: l10n.thirdPartyMigotoVerboseOverlayHint,
        value: c.verboseOverlay,
        onChanged: (v) => apply(c.copyWith(verboseOverlay: v)),
      ),
      ConfigFieldBool(
        label: l10n.thirdPartyMigotoCheckForegroundWindow,
        tooltip: l10n.thirdPartyMigotoCheckForegroundWindowHint,
        value: c.checkForegroundWindow,
        onChanged: (v) => apply(c.copyWith(checkForegroundWindow: v)),
      ),
    ]);
  }

  Widget _loaderRow(AppLocalizations l10n, MigotoInfo info) {
    final ok = info.loaderTargetOk;
    return Padding(
      padding: EdgeInsets.symmetric(vertical: AppSizes.spacingSM(context)),
      child: Row(
        children: [
          Icon(
            ok ? Icons.check_circle_outline : Icons.error_outline,
            size: AppSizes.iconSM(context),
            color: ok ? AppColors.success : AppColors.error,
          ),
          SizedBox(width: AppSizes.spacingSM(context)),
          Expanded(
            child: Text(
              '${l10n.thirdPartyLoaderTarget}: ${info.loaderTarget ?? '—'}',
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: ok ? AppColors.textSecondary : AppColors.error,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _kv(String key, String value) {
    return Padding(
      padding: EdgeInsets.symmetric(vertical: AppSizes.spacingSM(context)),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            '$key: ',
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: AppColors.textMuted,
            ),
          ),
          Expanded(
            child: Text(
              value,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textSecondary,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _configGroup(AppLocalizations l10n, List<Widget> rows) {
    return Padding(
      padding: EdgeInsets.only(top: AppSizes.spacingMD(context)),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Text(
                l10n.thirdPartyConfigSection,
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  fontWeight: FontWeight.bold,
                  color: AppColors.textMuted,
                  letterSpacing: 0.5,
                ),
              ),
              SizedBox(width: AppSizes.spacingSM(context)),
              Icon(
                Icons.restart_alt,
                size: AppSizes.iconSM(context),
                color: AppColors.textMuted,
              ),
              SizedBox(width: AppSizes.spacingSM(context) / 2),
              Flexible(
                child: Text(
                  l10n.thirdPartyRestartRequired,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.textMuted,
                    fontStyle: FontStyle.italic,
                  ),
                ),
              ),
            ],
          ),
          ...rows,
        ],
      ),
    );
  }

  Widget _section(
    String title,
    List<String> items, {
    String? countSuffix,
    String? emptyLabel,
  }) {
    return Padding(
      padding: EdgeInsets.only(top: AppSizes.spacingSM(context)),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            countSuffix != null ? '$title · $countSuffix' : title,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              fontWeight: FontWeight.bold,
              color: AppColors.textMuted,
              letterSpacing: 0.5,
            ),
          ),
          if (items.isEmpty && (emptyLabel ?? '').isNotEmpty)
            Padding(
              padding: EdgeInsets.only(top: AppSizes.spacingSM(context)),
              child: Text(
                emptyLabel!,
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  color: AppColors.textMuted,
                  fontStyle: FontStyle.italic,
                ),
              ),
            )
          else
            Padding(
              padding: EdgeInsets.only(top: AppSizes.spacingSM(context)),
              child: Wrap(
                spacing: AppSizes.spacingSM(context),
                runSpacing: AppSizes.spacingSM(context),
                children: [for (final it in items) _chip(it)],
              ),
            ),
        ],
      ),
    );
  }

  Widget _chip(String label) {
    return Container(
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.spacingSM(context),
        vertical: AppSizes.spacingSM(context),
      ),
      decoration: BoxDecoration(
        color: AppColors.surfaceMedium,
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: AppColors.borderLight),
      ),
      child: Text(
        label,
        style: TextStyle(
          fontSize: AppSizes.fontXS(context),
          color: AppColors.textSecondary,
        ),
      ),
    );
  }

  Widget _iconActions(
    AppLocalizations l10n,
    String dir,
    ThirdPartyRuntime which,
  ) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        HoverIconButton(
          bordered: false,
          padding: const EdgeInsets.all(4),
          tooltip: l10n.thirdPartyOpenFolder,
          icon: Icon(
            Icons.folder_open,
            size: AppSizes.iconSM(context),
            color: AppColors.textSecondary,
          ),
          onTap: () => _openDir(dir),
        ),
        SizedBox(width: AppSizes.spacingSM(context)),
        HoverIconButton(
          bordered: false,
          padding: const EdgeInsets.all(4),
          tooltip: l10n.thirdPartyRemove,
          icon: Icon(
            Icons.delete_outline,
            size: AppSizes.iconSM(context),
            color: AppColors.error,
          ),
          onTap: () => ref
              .read(thirdPartyStateControllerProvider.notifier)
              .remove(which),
        ),
      ],
    );
  }

  Future<void> _openDir(String dir) async {
    if (Directory(dir).existsSync()) {
      await launchUrl(Uri.file(dir));
    }
  }

  Widget _card(
    String title,
    List<Widget> children, {
    Widget? trailing,
    Widget? actions,
  }) {
    return Container(
      margin: EdgeInsets.only(bottom: AppSizes.paddingLG(context)),
      decoration: BoxDecoration(
        color: AppColors.backgroundCard,
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(color: AppColors.borderLight),
        boxShadow: const [
          BoxShadow(color: AppColors.shadow, blurRadius: 4, offset: Offset(0, 2)),
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
                if (trailing != null) trailing,
                if (actions != null) ...[
                  SizedBox(width: AppSizes.spacingMD(context)),
                  actions,
                ],
              ],
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

  Widget _statusChip(AppLocalizations l10n, bool enabled) {
    final color = enabled ? AppColors.success : AppColors.textMuted;
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Icon(Icons.circle, size: 8, color: color),
        SizedBox(width: AppSizes.spacingSM(context)),
        Text(
          enabled ? l10n.thirdPartyStatusActive : l10n.thirdPartyStatusInactive,
          style: TextStyle(
            fontSize: AppSizes.fontXS(context),
            color: color,
            fontWeight: FontWeight.w600,
            letterSpacing: 0.5,
          ),
        ),
      ],
    );
  }

  Widget _howto(String text) {
    return Padding(
      padding: EdgeInsets.only(bottom: AppSizes.spacingMD(context)),
      child: Text(
        text,
        style: TextStyle(
          fontSize: AppSizes.fontSM(context),
          color: AppColors.textSecondary,
          height: 1.4,
        ),
      ),
    );
  }

  Widget _statusRow(
    AppLocalizations l10n,
    ThirdPartyRuntimeStatus status,
    bool foundInGame,
  ) {
    final String label;
    final Color color;
    if (status.installed) {
      label = status.presetCount > 0
          ? '${l10n.thirdPartyStatusInstalled} · ${l10n.thirdPartyPresetsCount(status.presetCount)}'
          : l10n.thirdPartyStatusInstalled;
      color = AppColors.success;
    } else if (foundInGame) {
      label = l10n.thirdPartyStatusFoundInGame;
      color = AppColors.warning;
    } else {
      label = l10n.thirdPartyStatusNotInstalled;
      color = AppColors.textMuted;
    }
    return Row(
      children: [
        Icon(
          status.installed
              ? Icons.check_circle_outline
              : foundInGame
                  ? Icons.info_outline
                  : Icons.radio_button_unchecked,
          size: AppSizes.iconSM(context),
          color: color,
        ),
        SizedBox(width: AppSizes.spacingSM(context)),
        Expanded(
          child: Text(
            label,
            style: TextStyle(
              fontSize: AppSizes.fontSM(context),
              color: color,
            ),
          ),
        ),
      ],
    );
  }

  Widget _shadersMissing(AppLocalizations l10n) => _warningBox(
        l10n.thirdPartyShadersMissing,
        l10n.thirdPartyOpenFolder,
        _revealShaders,
      );

  Widget _d3dCompilerMissing(AppLocalizations l10n) => _warningBox(
        l10n.thirdPartyD3dCompilerMissing,
        l10n.thirdPartyOpenFolder,
        () => _openDir(ThirdPartyPaths.reshade()),
      );

  Widget _warningBox(String text, String actionLabel, VoidCallback onAction) {
    return Container(
      margin: EdgeInsets.only(top: AppSizes.spacingSM(context)),
      padding: EdgeInsets.all(AppSizes.paddingSM(context)),
      decoration: BoxDecoration(
        color: AppColors.warning.withValues(alpha: 0.10),
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(color: AppColors.warning.withValues(alpha: 0.4)),
      ),
      child: Row(
        children: [
          Icon(Icons.warning_amber_rounded,
              size: AppSizes.iconSM(context), color: AppColors.warning),
          SizedBox(width: AppSizes.spacingSM(context)),
          Expanded(
            child: Text(
              text,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textSecondary,
                height: 1.35,
              ),
            ),
          ),
          TextButton(
            onPressed: onAction,
            child: Text(actionLabel),
          ),
        ],
      ),
    );
  }

}
