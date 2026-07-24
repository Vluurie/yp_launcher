import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/notification_state.dart';
import 'package:yp_launcher/services/diagnostics_service.dart';
import 'package:yp_launcher/services/process_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

Future<void> showDiagnosticsDialog(BuildContext context, WidgetRef ref) async {
  final l10n = AppLocalizations.of(context)!;
  final gameDir = ref.read(appStateControllerProvider).selectedDirectory;
  final messenger = Navigator.of(context, rootNavigator: true);

  showDialog(
    context: context,
    barrierDismissible: false,
    builder: (_) => const _DiagnosticsLoadingDialog(),
  );

  DiagnosticsReport report;
  try {
    final launchPreview = gameDir.isEmpty
        ? null
        : await ProcessService.buildLaunchCommandPreview(
            installDirectory: gameDir, l10n: l10n);
    report = await DiagnosticsService.collect(gameDir,
        launchCommandPreview: launchPreview);
  } catch (e) {
    if (!context.mounted) return;
    messenger.pop();
    ref.read(notificationStateControllerProvider.notifier).addNotification(
          NotificationItem(
            id: 'diag_fail_${DateTime.now().millisecondsSinceEpoch}',
            message: (l10n) => '$e',
            icon: Icons.error_outline,
            color: AppColors.error,
            type: NotificationType.general,
          ),
        );
    return;
  }

  if (!context.mounted) return;
  messenger.pop();
  if (!context.mounted) return;

  showDialog(
    context: context,
    builder: (ctx) => _DiagnosticsDialog(report: report, l10n: l10n),
  );
}

class _DiagnosticsLoadingDialog extends StatelessWidget {
  const _DiagnosticsLoadingDialog();

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return Dialog(
      backgroundColor: Colors.transparent,
      child: Container(
        padding: EdgeInsets.symmetric(
          horizontal: AppSizes.paddingXL(context),
          vertical: AppSizes.paddingLG(context),
        ),
        decoration: BoxDecoration(
          color: AppColors.backgroundCard,
          borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
          border: Border.all(
            color: AppColors.accentPrimary.withValues(alpha: 0.4),
          ),
          boxShadow: const [
            BoxShadow(
              color: AppColors.shadow,
              blurRadius: 12,
              offset: Offset(0, 4),
            ),
          ],
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            const SizedBox(
              width: 22,
              height: 22,
              child: CircularProgressIndicator(
                strokeWidth: 2,
                color: AppColors.accentPrimary,
              ),
            ),
            SizedBox(width: AppSizes.spacingLG(context)),
            Text(
              l10n.diagnosticsCollecting,
              style: TextStyle(
                fontSize: AppSizes.fontMD(context),
                color: AppColors.textPrimary,
                letterSpacing: 0.4,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _DiagnosticsDialog extends ConsumerStatefulWidget {
  final DiagnosticsReport report;
  final AppLocalizations l10n;
  const _DiagnosticsDialog({required this.report, required this.l10n});

  @override
  ConsumerState<_DiagnosticsDialog> createState() => _DiagnosticsDialogState();
}

class _DiagnosticsDialogState extends ConsumerState<_DiagnosticsDialog> {
  bool _saving = false;
  final ScrollController _scrollController = ScrollController();

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  String _localTime(DateTime dt) {
    String two(int n) => n.toString().padLeft(2, '0');
    return '${dt.year}-${two(dt.month)}-${two(dt.day)} '
        '${two(dt.hour)}:${two(dt.minute)}:${two(dt.second)}';
  }

  @override
  Widget build(BuildContext context) {
    final l10n = widget.l10n;
    final r = widget.report;
    final summary = DiagnosticsService.formatSummary(r);

    return Dialog(
      backgroundColor: Colors.transparent,
      insetPadding: EdgeInsets.symmetric(
        horizontal: AppSizes.paddingLG(context),
        vertical: AppSizes.paddingLG(context),
      ),
      child: Container(
        width: double.infinity,
        height: double.infinity,
        decoration: BoxDecoration(
          color: AppColors.backgroundCard,
          borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
          border: Border.all(
            color: AppColors.accentPrimary.withValues(alpha: 0.45),
          ),
          boxShadow: const [
            BoxShadow(
              color: AppColors.shadow,
              blurRadius: 18,
              offset: Offset(0, 6),
            ),
          ],
        ),
        child: Column(
          children: [
            _buildHeader(context, l10n, r),
            Expanded(
              child: Scrollbar(
                controller: _scrollController,
                thumbVisibility: true,
                child: SingleChildScrollView(
                  controller: _scrollController,
                  padding: EdgeInsets.symmetric(
                    horizontal: AppSizes.paddingLG(context),
                    vertical: AppSizes.paddingMD(context),
                  ),
                  child: Center(
                    child: ConstrainedBox(
                      constraints: const BoxConstraints(maxWidth: 1100),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: _buildSections(context, r),
                      ),
                    ),
                  ),
                ),
              ),
            ),
            Container(
              height: 1,
              color: AppColors.borderLight,
            ),
            Padding(
              padding: EdgeInsets.symmetric(
                horizontal: AppSizes.paddingLG(context),
                vertical: AppSizes.paddingSM(context),
              ),
              child: Row(
                children: [
                  _diagButton(
                    context: context,
                    icon: Icons.copy,
                    label: l10n.diagnosticsCopy,
                    onTap: () async {
                      await Clipboard.setData(ClipboardData(text: summary));
                      ref
                          .read(notificationStateControllerProvider.notifier)
                          .addNotification(NotificationItem(
                            id: 'diag_copied_${DateTime.now().millisecondsSinceEpoch}',
                            message: (l10n) => l10n.diagnosticsCopied,
                            icon: Icons.check_circle,
                            color: AppColors.success,
                            type: NotificationType.general,
                          ));
                    },
                  ),
                  SizedBox(width: AppSizes.spacingMD(context)),
                  _diagButton(
                    context: context,
                    icon: Icons.save,
                    label: l10n.diagnosticsSaveFull,
                    primary: true,
                    busy: _saving,
                    onTap: _saving
                        ? null
                        : () async {
                            setState(() => _saving = true);
                            try {
                              final outPath =
                                  await DiagnosticsService.writeFullReport(r);
                              if (!mounted) return;
                              ref
                                  .read(notificationStateControllerProvider.notifier)
                                  .addNotification(NotificationItem(
                                    id: 'diag_saved_${DateTime.now().millisecondsSinceEpoch}',
                                    message: (l10n) => l10n.diagnosticsSavedAt(outPath),
                                    icon: Icons.save,
                                    color: AppColors.success,
                                    type: NotificationType.general,
                                  ));
                            } finally {
                              if (mounted) setState(() => _saving = false);
                            }
                          },
                  ),
                  const Spacer(),
                  TextButton(
                    onPressed: () => Navigator.of(context).pop(),
                    child: Text(
                      l10n.diagnosticsClose,
                      style: TextStyle(
                        color: AppColors.textMuted,
                        fontSize: AppSizes.fontSM(context),
                        letterSpacing: 0.4,
                      ),
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

  Widget _buildHeader(
    BuildContext context,
    AppLocalizations l10n,
    DiagnosticsReport r,
  ) {
    return Container(
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.paddingLG(context),
        vertical: AppSizes.paddingMD(context),
      ),
      decoration: BoxDecoration(
        color: AppColors.surfaceMedium,
        border: const Border(
          bottom: BorderSide(color: AppColors.borderLight),
        ),
        borderRadius: BorderRadius.only(
          topLeft: Radius.circular(AppSizes.borderRadius(context)),
          topRight: Radius.circular(AppSizes.borderRadius(context)),
        ),
      ),
      child: Row(
        children: [
          Container(
            width: 36,
            height: 36,
            alignment: Alignment.center,
            decoration: BoxDecoration(
              color: AppColors.accentPrimary.withValues(alpha: 0.12),
              borderRadius: BorderRadius.circular(6),
              border: Border.all(
                color: AppColors.accentPrimary.withValues(alpha: 0.5),
              ),
            ),
            child: const Icon(
              Icons.medical_information_outlined,
              size: 20,
              color: AppColors.accentPrimary,
            ),
          ),
          SizedBox(width: AppSizes.spacingMD(context)),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  l10n.diagnosticsTitle.toUpperCase(),
                  style: TextStyle(
                    fontSize: AppSizes.fontLG(context),
                    fontWeight: FontWeight.bold,
                    color: AppColors.textPrimary,
                    letterSpacing: 1.2,
                  ),
                ),
                SizedBox(height: AppSizes.spacingSM(context) / 2),
                Text(
                  '${r.launcherInfo['Launcher version'] ?? '?'}  ·  ${_localTime(r.generatedAt)}',
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.textMuted,
                    letterSpacing: 0.3,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  List<Widget> _buildSections(BuildContext context, DiagnosticsReport r) {
    final gap = AppSizes.spacingMD(context);
    return [
      LayoutBuilder(
        builder: (context, constraints) {
          final cards = _diagnosticCards(context, r);
          final twoCol = constraints.maxWidth >= 720;
          final cardWidth = twoCol
              ? (constraints.maxWidth - gap) / 2
              : constraints.maxWidth;
          return Wrap(
            spacing: gap,
            runSpacing: gap,
            children: [
              for (final card in cards)
                SizedBox(width: cardWidth, child: card),
            ],
          );
        },
      ),
      SizedBox(height: gap),
    ];
  }

  List<Widget> _diagnosticCards(BuildContext context, DiagnosticsReport r) {
    final l10n = AppLocalizations.of(context)!;
    final modKinds = <ModKind, int>{};
    for (final m in r.mods) {
      modKinds[m.kind] = (modKinds[m.kind] ?? 0) + 1;
    }
    final gi = r.gameIdentity;
    final rs = r.thirdParty.reshade;
    final mi = r.thirdParty.migoto;
    final rsInfo = rs.reshadeInfo;
    final miInfo = mi.migotoInfo;

    return [
      _sectionCard(
        context,
        icon: Icons.computer_outlined,
        title: l10n.logSectionSystem,
        children: [
          _kvRow(context, l10n.logDetailOs,
              '${r.systemInfo['OS'] ?? '?'} ${r.systemInfo['OS version'] ?? ''}'),
          _kvRow(context, l10n.logDetailLocale, r.systemInfo['Locale'] ?? '?'),
          if (r.preferDedicatedGpu)
            Padding(
              padding: EdgeInsets.only(top: AppSizes.spacingSM(context)),
              child: _chip(context, 'dGPU', AppColors.accentPrimary),
            ),
        ],
      ),
      _sectionCard(
        context,
        icon: Icons.videogame_asset_outlined,
        title: l10n.diagnosticsSectionGameIdentity,
        accent: gi.exeVariantSupported ? null : AppColors.warning,
        children: [
          _kvRow(context, l10n.diagnosticsExeVariant(gi.exeVariant),
              gi.exeVariantSupported ? '' : l10n.diagnosticsExeUnsupported),
          Wrap(
            spacing: 6,
            runSpacing: 6,
            children: [
              _flagChip(context, l10n.diagnosticsDlcPresent, gi.hasDlc),
              if (gi.exeSize != null)
                _chip(context, _mb(gi.exeSize!), AppColors.textMuted),
            ],
          ),
        ],
      ),
      _sectionCard(
        context,
        icon: Icons.health_and_safety_outlined,
        title: l10n.diagnosticsSectionNamsHealth,
        accent: r.namsHealth.missingFiles.isEmpty ? null : AppColors.error,
        children: [
          Wrap(
            spacing: 6,
            runSpacing: 6,
            children: [
              _flagChip(
                  context, l10n.diagnosticsNamsPresent, r.namsHealth.namsExePresent),
              if (r.namsHealth.missingFiles.isNotEmpty)
                _chip(
                  context,
                  l10n.diagnosticsMissingFiles(r.namsHealth.missingFiles.length),
                  AppColors.error,
                ),
            ],
          ),
        ],
      ),
      _sectionCard(
        context,
        icon: Icons.extension_outlined,
        title: l10n.logSectionModsNams,
        trailing: _countBadge(r.mods.length),
        children: [
          if (modKinds.isEmpty)
            _mutedLine(context, l10n.logNoModsInstalled)
          else
            Wrap(
              spacing: 6,
              runSpacing: 6,
              children: modKinds.entries
                  .map((e) => _chip(
                        context,
                        '${e.key.name}: ${e.value}',
                        AppColors.accentPrimary,
                      ))
                  .toList(),
            ),
          if (r.disabledModsEntries.isNotEmpty) ...[
            SizedBox(height: AppSizes.spacingSM(context)),
            _mutedLine(context,
                '${r.disabledModsEntries.length} disabled prefix(es) in disabled_mods.toml'),
          ],
        ],
      ),
      if (rs.installed)
        _sectionCard(
          context,
          icon: Icons.auto_awesome,
          title: l10n.diagnosticsSectionReshade,
          trailing: _countBadge(rsInfo?.presets.length ?? 0),
          children: [
            Wrap(
              spacing: 6,
              runSpacing: 6,
              children: [
                _flagChip(context, l10n.diagnosticsEnabled, rs.enabled),
                if (rsInfo?.version != null)
                  _chip(context, rsInfo!.version!, AppColors.accentPrimary),
                _chip(context, 'presets: ${rs.presetCount}',
                    AppColors.accentPrimary),
                _chip(context, 'shaders: ${rsInfo?.shaderCount ?? 0}',
                    AppColors.accentPrimary),
                if (rs.shadersMissing || (rsInfo?.d3dCompilerMissing ?? false))
                  _chip(context, l10n.diagnosticsShadersMissing,
                      AppColors.warning),
              ],
            ),
          ],
        ),
      if (mi.installed)
        _sectionCard(
          context,
          icon: Icons.blur_on,
          title: l10n.diagnosticsSectionMigoto,
          trailing: _countBadge(miInfo?.shaderFixCount ?? 0),
          children: [
            Wrap(
              spacing: 6,
              runSpacing: 6,
              children: [
                _flagChip(context, l10n.diagnosticsEnabled, mi.enabled),
                _flagChip(context, 'loader', miInfo?.loaderTargetOk ?? false),
              ],
            ),
            if ((miInfo?.shaderFixNames ?? const []).isNotEmpty) ...[
              SizedBox(height: AppSizes.spacingSM(context)),
              _mutedLine(context, miInfo!.shaderFixNames.join(', ')),
            ],
          ],
        ),
      _sectionCard(
        context,
        icon: Icons.texture,
        title: l10n.diagnosticsSectionTexturePacks,
        trailing:
            r.texturePacksAvailable ? _countBadge(r.texturePacks.length) : null,
        children: [
          if (!r.texturePacksAvailable)
            _mutedLine(context, l10n.diagnosticsTexturePacksUnavailable)
          else
            Wrap(
              spacing: 6,
              runSpacing: 6,
              children: [
                _chip(context, 'NAMS list: ${r.texturePacks.length}',
                    AppColors.accentPrimary),
                _chip(context, 'nams/inject: ${r.namsTextures.length}',
                    AppColors.textMuted),
                if (r.skResTextures.isNotEmpty)
                  _chip(context, 'SK_Res: ${r.skResTextures.length}',
                      AppColors.warning),
                if (r.waxTextures.isNotEmpty)
                  _chip(context, 'WAX: ${r.waxTextures.length}',
                      AppColors.warning),
              ],
            ),
        ],
      ),
      _sectionCard(
        context,
        icon: Icons.movie_creation_outlined,
        title: l10n.logSectionCutscenes,
        trailing: _countBadge(r.cutsceneMods.length),
        children: [
          Wrap(
            spacing: 6,
            runSpacing: 6,
            children: [
              _flagChip(context, 'HD', r.cutsceneDetection.hasHdCutscenes),
              _flagChip(context, 'H264', r.cutsceneDetection.needsH264),
              if (r.directOverrides.isNotEmpty)
                _chip(
                  context,
                  'data/movie overrides: ${r.directOverrides.length}',
                  AppColors.warning,
                ),
            ],
          ),
        ],
      ),
      if (r.vanillaDropped.isNotEmpty)
        _sectionCard(
          context,
          icon: Icons.folder_zip_outlined,
          title: l10n.diagnosticsSectionVanillaDrops,
          accent: AppColors.warning,
          trailing: _countBadge(r.vanillaDropped.length),
          children: [
            for (final f in r.vanillaDropped.take(12))
              _kvRow(context, f.path, _mb(f.sizeBytes)),
            if (r.vanillaDropped.length > 12)
              _mutedLine(
                context,
                l10n.diagnosticsMoreItems(r.vanillaDropped.length - 12),
              ),
          ],
        ),
      if (r.configDeltas.isNotEmpty)
        _sectionCard(
          context,
          icon: Icons.tune,
          title: l10n.diagnosticsSectionNonDefault,
          trailing: _countBadge(r.configDeltas.length),
          children: [
            for (final d in r.configDeltas.take(14))
              _kvRow(context, d.key, d.value),
            if (r.configDeltas.length > 14)
              _mutedLine(
                context,
                l10n.diagnosticsMoreItems(r.configDeltas.length - 14),
              ),
          ],
        ),
      if (r.recentLogIssues.isNotEmpty)
        _sectionCard(
          context,
          icon: Icons.report_gmailerrorred_outlined,
          title: l10n.diagnosticsSectionRecentIssues,
          accent: AppColors.error,
          trailing: _countBadge(r.recentLogIssues.length),
          children: [
            for (final e in r.recentLogIssues.reversed.take(8))
              _mutedLine(context, '${e.level} ${e.module}: ${e.message}'),
          ],
        ),
      if (r.dataDirContents.isNotEmpty)
        _sectionCard(
          context,
          icon: Icons.warning_amber_rounded,
          title: l10n.diagnosticsSectionDataOverlay,
          accent: AppColors.warning,
          children: [
            for (final e in (r.dataDirContents.entries.toList()
                  ..sort((a, b) => b.value.compareTo(a.value)))
                .take(8))
              _kvRow(context, '${e.key}/', l10n.diagnosticsFileCount(e.value)),
            if (r.dataDirContents.length > 8)
              _mutedLine(
                context,
                l10n.diagnosticsMoreItems(r.dataDirContents.length - 8),
              ),
          ],
        ),
      if (r.waxModsRoot.isNotEmpty ||
          r.skResPacks.isNotEmpty ||
          r.reshadeContents.isNotEmpty)
        _sectionCard(
          context,
          icon: Icons.folder_outlined,
          title: l10n.diagnosticsSectionExternalLegacy,
          children: [
            if (r.waxModsRoot.isNotEmpty)
              _kvRow(context, 'wax/mods/', r.waxModsRoot.join(', ')),
            if (r.skResPacks.isNotEmpty)
              _kvRow(context, 'SK_Res packs', r.skResPacks.join(', ')),
            if (r.reshadeContents.isNotEmpty)
              _kvRow(context, 'reshade/', r.reshadeContents.join(', ')),
          ],
        ),
      if (r.gameRootExtras.isNotEmpty)
        _sectionCard(
          context,
          icon: Icons.travel_explore,
          title: l10n.diagnosticsSectionGameRootExtras,
          accent: AppColors.accentPrimary,
          children: [
            for (final name in r.gameRootExtras.take(20))
              _mutedLine(context, '• $name'),
            if (r.gameRootExtras.length > 20)
              _mutedLine(
                context,
                l10n.diagnosticsMoreItems(r.gameRootExtras.length - 20),
              ),
          ],
        ),
    ];
  }

  String _mb(int bytes) {
    if (bytes >= 1024 * 1024 * 1024) {
      return '${(bytes / (1024 * 1024 * 1024)).toStringAsFixed(2)} GB';
    }
    if (bytes >= 1024 * 1024) {
      return '${(bytes / (1024 * 1024)).toStringAsFixed(1)} MB';
    }
    if (bytes >= 1024) return '${(bytes / 1024).round()} KB';
    return '$bytes B';
  }

  Widget _sectionCard(
    BuildContext context, {
    required IconData icon,
    required String title,
    Color? accent,
    Widget? trailing,
    required List<Widget> children,
  }) {
    final color = accent ?? AppColors.accentPrimary;
    return Container(
      decoration: BoxDecoration(
        color: AppColors.surfaceLight,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: AppColors.borderLight),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Container(
            padding: EdgeInsets.symmetric(
              horizontal: AppSizes.paddingMD(context),
              vertical: AppSizes.paddingSM(context),
            ),
            decoration: BoxDecoration(
              color: color.withValues(alpha: 0.08),
              border: const Border(
                bottom: BorderSide(color: AppColors.borderLight),
              ),
              borderRadius: const BorderRadius.only(
                topLeft: Radius.circular(6),
                topRight: Radius.circular(6),
              ),
            ),
            child: Row(
              children: [
                Icon(icon, size: 14, color: color),
                SizedBox(width: AppSizes.spacingSM(context)),
                Expanded(
                  child: Text(
                    title,
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      fontWeight: FontWeight.bold,
                      color: color,
                      letterSpacing: 1.0,
                    ),
                  ),
                ),
                if (trailing != null) trailing,
              ],
            ),
          ),
          Padding(
            padding: EdgeInsets.symmetric(
              horizontal: AppSizes.paddingMD(context),
              vertical: AppSizes.paddingSM(context),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: children,
            ),
          ),
        ],
      ),
    );
  }

  Widget _countBadge(int n) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 1),
      decoration: BoxDecoration(
        color: AppColors.accentPrimary.withValues(alpha: 0.18),
        borderRadius: BorderRadius.circular(3),
        border: Border.all(
          color: AppColors.accentPrimary.withValues(alpha: 0.4),
        ),
      ),
      child: Text(
        '$n',
        style: const TextStyle(
          fontSize: 10,
          fontWeight: FontWeight.bold,
          color: AppColors.accentPrimary,
          fontFamily: 'monospace',
        ),
      ),
    );
  }

  Widget _chip(BuildContext context, String label, Color color) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.12),
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: color.withValues(alpha: 0.4)),
      ),
      child: Text(
        label,
        style: TextStyle(
          fontSize: AppSizes.fontXS(context),
          color: color,
          fontWeight: FontWeight.w600,
          letterSpacing: 0.4,
        ),
      ),
    );
  }

  Widget _flagChip(BuildContext context, String label, bool on) {
    final color = on ? AppColors.success : AppColors.textMuted;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.12),
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: color.withValues(alpha: 0.4)),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(
            on ? Icons.check_circle : Icons.remove_circle_outline,
            size: 12,
            color: color,
          ),
          const SizedBox(width: 4),
          Text(
            label,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: color,
              fontWeight: FontWeight.bold,
              letterSpacing: 0.4,
            ),
          ),
        ],
      ),
    );
  }

  Widget _kvRow(BuildContext context, String key, String value) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 6),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            key,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: AppColors.textMuted,
              fontWeight: FontWeight.w600,
            ),
          ),
          if (value.isNotEmpty)
            SelectableText(
              value,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textSecondary,
                fontFamily: 'monospace',
                height: 1.35,
              ),
            ),
        ],
      ),
    );
  }

  Widget _mutedLine(BuildContext context, String text) {
    return Text(
      text,
      style: TextStyle(
        fontSize: AppSizes.fontXS(context),
        color: AppColors.textMuted,
        fontStyle: FontStyle.italic,
      ),
    );
  }

  Widget _diagButton({
    required BuildContext context,
    required IconData icon,
    required String label,
    required VoidCallback? onTap,
    bool primary = false,
    bool busy = false,
  }) {
    final foreground = primary ? AppColors.buttonText : AppColors.accentPrimary;
    final background =
        primary ? AppColors.accentPrimary : Colors.transparent;
    final border = primary
        ? AppColors.accentPrimary
        : AppColors.accentPrimary.withValues(alpha: 0.5);
    return Material(
      color: Colors.transparent,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(4),
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          decoration: BoxDecoration(
            color: background,
            borderRadius: BorderRadius.circular(4),
            border: Border.all(color: border),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              busy
                  ? SizedBox(
                      width: 14,
                      height: 14,
                      child: CircularProgressIndicator(
                        strokeWidth: 2,
                        color: foreground,
                      ),
                    )
                  : Icon(icon, size: 14, color: foreground),
              const SizedBox(width: 6),
              Text(
                label,
                style: TextStyle(
                  fontSize: AppSizes.fontSM(context),
                  fontWeight: FontWeight.bold,
                  color: foreground,
                  letterSpacing: 0.4,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
