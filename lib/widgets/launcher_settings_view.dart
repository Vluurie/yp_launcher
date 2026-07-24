import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart' show Clipboard, ClipboardData;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path/path.dart' as p;
import 'package:url_launcher/url_launcher.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/verify_result.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/notification_state.dart';
import 'package:yp_launcher/services/cache_service.dart';
import 'package:yp_launcher/services/detection/game_detection.dart';
import 'package:yp_launcher/services/launch_wrapper_service.dart';
import 'package:yp_launcher/services/launcher_setup_service.dart';
import 'package:yp_launcher/services/log_service.dart';
import 'package:yp_launcher/services/nams_cli_service.dart';
import 'package:yp_launcher/services/platform_gate.dart';
import 'package:yp_launcher/services/process_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/widgets/diagnostics_dialog.dart';
import 'package:yp_launcher/widgets/hover_button.dart';

class LauncherSettingsView extends ConsumerStatefulWidget {
  const LauncherSettingsView({super.key});

  @override
  ConsumerState<LauncherSettingsView> createState() =>
      _LauncherSettingsViewState();
}

class _LauncherSettingsViewState extends ConsumerState<LauncherSettingsView> {
  final _scrollController = ScrollController();
  final _wrapperController = TextEditingController();
  bool _wrapperLoaded = false;
  bool _verifying = false;
  VerifyOutcome? _verifyOutcome;

  List<String> _missingFiles = const [];
  ExeVariant? _exeVariant;
  List<LogEntry> _recentErrors = const [];
  Map<String, String> _launcherPaths = const {};

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => _loadHealth());
  }

  @override
  void dispose() {
    _scrollController.dispose();
    _wrapperController.dispose();
    super.dispose();
  }

  Future<void> _loadHealth() async {
    final gameDir = ref.read(appStateControllerProvider).selectedDirectory;
    List<String> missing = const [];
    Map<String, String> paths = const {};
    ExeVariant? variant;
    List<LogEntry> errors = const [];
    try {
      missing = await LauncherSetupService.findMissingFiles();
    } catch (_) {}
    try {
      paths = await LauncherSetupService.getLauncherPaths();
    } catch (_) {}
    if (gameDir.isNotEmpty) {
      try {
        variant = await GameDetection.detectExeVariant(gameDir);
      } catch (_) {}
    }
    try {
      final entries = await LogService.readLog(AppStrings.namsLogName);
      final errs =
          entries.where((e) => e.level == 'ERROR' || e.level == 'WARN').toList();
      errors = errs.length > 8 ? errs.sublist(errs.length - 8) : errs;
    } catch (_) {}
    if (!mounted) return;
    setState(() {
      _missingFiles = missing;
      _launcherPaths = paths;
      _exeVariant = variant;
      _recentErrors = errors;
    });
  }

  Future<void> _openDir(String? dir) async {
    if (dir == null || dir.isEmpty) return;
    final directory = Directory(dir);
    try {
      if (!directory.existsSync()) {
        directory.createSync(recursive: true);
      }
      await launchUrl(Uri.file(dir));
    } catch (_) {}
  }

  Future<void> _loadWrapper() async {
    if (_wrapperLoaded) return;
    _wrapperLoaded = true;
    final value = await LaunchWrapperService.read();
    if (!mounted) return;
    setState(() => _wrapperController.text = value);
  }

  Future<void> _runVerify() async {
    final l10n = AppLocalizations.of(context)!;
    final gameDir = ref.read(appStateControllerProvider).selectedDirectory;
    if (gameDir.isEmpty) {
      setState(() => _verifyOutcome =
          const VerifyOutcome(status: VerifyStatus.error));
      return;
    }
    setState(() {
      _verifying = true;
      _verifyOutcome = null;
    });
    final outcome = await NamsCliService.verify(gameDir, l10n);
    if (!mounted) return;
    setState(() {
      _verifying = false;
      _verifyOutcome = outcome;
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return Container(
      color: AppColors.backgroundPrimary,
      child: Column(
        children: [
          _header(context, l10n),
          Expanded(
            child: Scrollbar(
              controller: _scrollController,
              thumbVisibility: true,
              child: SingleChildScrollView(
                controller: _scrollController,
                padding: EdgeInsets.all(AppSizes.contentPadding(context)),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    LayoutBuilder(
                      builder: (context, constraints) {
                        final gap = AppSizes.paddingLG(context);
                        final twoCol = constraints.maxWidth >= 720;
                        final w = twoCol
                            ? (constraints.maxWidth - gap) / 2
                            : constraints.maxWidth;
                        final legacyExe =
                            _exeVariant == ExeVariant.legacyWindows7;
                        return Wrap(
                          spacing: gap,
                          runSpacing: gap,
                          children: [
                            if (legacyExe)
                              SizedBox(
                                  width: constraints.maxWidth,
                                  child: _legacyExeCard(context, l10n)),
                            if (_missingFiles.isNotEmpty)
                              SizedBox(
                                  width: constraints.maxWidth,
                                  child: _missingFilesCard(context, l10n)),
                            SizedBox(width: w, child: _verifyCard(context, l10n)),
                            SizedBox(
                                width: w, child: _diagnosticsCard(context, l10n)),
                            SizedBox(
                                width: w,
                                child: _copyCommandCard(context, l10n)),
                            SizedBox(
                                width: w, child: _foldersCard(context, l10n)),
                            SizedBox(
                                width: w,
                                child: _clearCacheCard(context, l10n)),
                            if (_recentErrors.isNotEmpty)
                              SizedBox(
                                  width: w,
                                  child: _recentErrorsCard(context, l10n)),
                          ],
                        );
                      },
                    ),
                    if (PlatformGate.isLinux) _wrapperCard(context, l10n),
                  ],
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _header(BuildContext context, AppLocalizations l10n) {
    return Container(
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.cardPaddingH(context),
        vertical: AppSizes.cardPaddingV(context),
      ),
      decoration: const BoxDecoration(
        color: AppColors.surfaceMedium,
        border: Border(bottom: BorderSide(color: AppColors.borderLight)),
      ),
      child: Row(
        children: [
          Text(
            l10n.tabLauncherSettings,
            style: TextStyle(
              fontSize: AppSizes.fontXL(context),
              fontWeight: FontWeight.bold,
              color: AppColors.textPrimary,
              letterSpacing: 1.0,
            ),
          ),
        ],
      ),
    );
  }

  Widget _verifyCard(BuildContext context, AppLocalizations l10n) {
    return _card(context, l10n.verifyInstallTitle, [
      Text(
        l10n.verifyInstallDesc,
        style: TextStyle(
          fontSize: AppSizes.fontXS(context),
          color: AppColors.textMuted,
          height: 1.4,
        ),
      ),
      SizedBox(height: AppSizes.spacingMD(context)),
      Row(
        children: [
          if (_verifying) ...[
            SizedBox(
              width: AppSizes.iconSM(context),
              height: AppSizes.iconSM(context),
              child: const CircularProgressIndicator(
                strokeWidth: 2,
                color: AppColors.accentPrimary,
              ),
            ),
            SizedBox(width: AppSizes.spacingSM(context)),
            Text(
              l10n.verifyInstallRunning,
              style: TextStyle(
                fontSize: AppSizes.fontSM(context),
                color: AppColors.textMuted,
              ),
            ),
          ] else
            HoverButton(
              label: l10n.verifyInstallButton,
              color: AppColors.accentPrimary,
              onTap: _runVerify,
            ),
        ],
      ),
      if (_verifyOutcome != null && !_verifying) ...[
        SizedBox(height: AppSizes.spacingMD(context)),
        ..._verifyResultWidgets(context, l10n, _verifyOutcome!),
      ],
    ]);
  }

  Widget _diagnosticsCard(BuildContext context, AppLocalizations l10n) {
    return _card(context, l10n.diagnosticsTitle, [
      Text(
        l10n.diagnosticsSubtitle,
        style: TextStyle(
          fontSize: AppSizes.fontXS(context),
          color: AppColors.textMuted,
          height: 1.4,
        ),
      ),
      SizedBox(height: AppSizes.spacingMD(context)),
      HoverButton(
        label: l10n.diagnosticsButton,
        color: AppColors.accentPrimary,
        onTap: () => showDiagnosticsDialog(context, ref),
      ),
    ]);
  }

  Widget _copyCommandCard(BuildContext context, AppLocalizations l10n) {
    return _card(context, l10n.copyCommandTitle, [
      Text(
        l10n.copyCommandDesc,
        style: TextStyle(
          fontSize: AppSizes.fontXS(context),
          color: AppColors.textMuted,
          height: 1.4,
        ),
      ),
      SizedBox(height: AppSizes.spacingMD(context)),
      HoverButton(
        label: l10n.copyCommandButton,
        color: AppColors.accentPrimary,
        onTap: _copyLaunchCommand,
      ),
    ]);
  }

  Future<void> _copyLaunchCommand() async {
    final gameDir = ref.read(appStateControllerProvider).selectedDirectory;
    final notifier = ref.read(notificationStateControllerProvider.notifier);
    final cmd = await ProcessService.buildLaunchCommandPreview(
      installDirectory: gameDir,
      l10n: AppLocalizations.of(context)!,
    );
    if (cmd == null) {
      notifier.addNotification(
        NotificationItem(
          id: 'launch_cmd_${DateTime.now().millisecondsSinceEpoch}',
          message: (l10n) => l10n.notificationCommandNotReady,
          icon: Icons.error_outline,
          color: AppColors.error,
          type: NotificationType.general,
        ),
      );
      return;
    }
    await Clipboard.setData(ClipboardData(text: cmd));
    notifier.addNotification(
      NotificationItem(
        id: 'launch_cmd_${DateTime.now().millisecondsSinceEpoch}',
        message: (l10n) => l10n.notificationCommandCopied,
        icon: Icons.check_circle,
        color: AppColors.success,
        type: NotificationType.general,
      ),
    );
  }

  Widget _legacyExeCard(BuildContext context, AppLocalizations l10n) {
    return _card(context, l10n.troubleWrongExeTitle, [
      _statusLine(context, false, l10n.troubleWrongExeSummary),
      SizedBox(height: AppSizes.spacingMD(context)),
      Text(
        l10n.detectionExeLegacyWin7Tooltip,
        style: TextStyle(
          fontSize: AppSizes.fontXS(context),
          color: AppColors.textSecondary,
          height: 1.5,
        ),
      ),
    ], accent: AppColors.error);
  }

  Widget _missingFilesCard(BuildContext context, AppLocalizations l10n) {
    return _card(context, l10n.troubleMissingFilesTitle, [
      _statusLine(context, false,
          l10n.troubleMissingFilesSummary(_missingFiles.join(', '))),
      SizedBox(height: AppSizes.spacingSM(context)),
      Text(
        l10n.troubleMissingFilesDesc,
        style: TextStyle(
          fontSize: AppSizes.fontXS(context),
          color: AppColors.textSecondary,
          height: 1.5,
        ),
      ),
      SizedBox(height: AppSizes.spacingMD(context)),
      HoverButton(
        label: l10n.troubleOpenBins,
        color: AppColors.accentPrimary,
        onTap: () => _openDir(_launcherPaths['launcherDir']),
      ),
    ], accent: AppColors.error);
  }

  Widget _recentErrorsCard(BuildContext context, AppLocalizations l10n) {
    return _card(context, l10n.troubleRecentErrorsTitle, [
      for (final e in _recentErrors.reversed)
        Padding(
          padding: EdgeInsets.only(bottom: AppSizes.paddingXS(context)),
          child: Text(
            '${e.level} ${e.module}: ${e.message}',
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: e.level == 'ERROR'
                  ? AppColors.error
                  : AppColors.warning,
              fontFamily: 'monospace',
              height: 1.35,
            ),
          ),
        ),
    ], accent: AppColors.warning);
  }

  Widget _clearCacheCard(BuildContext context, AppLocalizations l10n) {
    return _card(context, l10n.troubleClearCacheTitle, [
      Text(
        l10n.troubleClearCacheDesc,
        style: TextStyle(
          fontSize: AppSizes.fontXS(context),
          color: AppColors.textMuted,
          height: 1.4,
        ),
      ),
      SizedBox(height: AppSizes.spacingMD(context)),
      HoverButton(
        label: l10n.troubleClearCacheButton,
        color: AppColors.warning,
        onTap: _clearCache,
      ),
    ]);
  }

  Future<void> _clearCache() async {
    final l10n = AppLocalizations.of(context)!;
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: AppColors.backgroundCard,
        title: Text(
          l10n.troubleClearCacheTitle,
          style: TextStyle(
            fontSize: AppSizes.fontXL(ctx),
            color: AppColors.textPrimary,
          ),
        ),
        content: Text(
          l10n.troubleClearCacheConfirm,
          style: TextStyle(
            fontSize: AppSizes.fontMD(ctx),
            color: AppColors.textSecondary,
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(false),
            child: Text(l10n.diagnosticsClose,
                style: const TextStyle(color: AppColors.textMuted)),
          ),
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(true),
            child: Text(
              l10n.troubleClearCacheButton,
              style: const TextStyle(
                color: AppColors.warning,
                fontWeight: FontWeight.bold,
              ),
            ),
          ),
        ],
      ),
    );
    if (confirmed != true) return;
    final gameDir = ref.read(appStateControllerProvider).selectedDirectory;
    final notifier = ref.read(notificationStateControllerProvider.notifier);
    final result = await CacheService.clearAll(gameDir);
    if (!mounted) return;
    await _loadHealth();
    notifier.addNotification(
      NotificationItem(
        id: 'cache_cleared_${DateTime.now().millisecondsSinceEpoch}',
        message: (l10n) => l10n.troubleClearCacheDone(result.logsDeleted),
        icon: Icons.check_circle,
        color: AppColors.success,
        type: NotificationType.general,
      ),
    );
  }

  Widget _foldersCard(BuildContext context, AppLocalizations l10n) {
    final gameDir = ref.watch(appStateControllerProvider).selectedDirectory;
    final launcherDir = _launcherPaths['launcherDir'];
    return _card(context, l10n.troubleFoldersTitle, [
      Text(
        l10n.troubleFoldersDesc,
        style: TextStyle(
          fontSize: AppSizes.fontXS(context),
          color: AppColors.textMuted,
          height: 1.4,
        ),
      ),
      SizedBox(height: AppSizes.spacingMD(context)),
      Wrap(
        spacing: AppSizes.spacingSM(context),
        runSpacing: AppSizes.spacingSM(context),
        children: [
          if (gameDir.isNotEmpty)
            HoverButton(
              label: l10n.troubleOpenGame,
              color: AppColors.textSecondary,
              onTap: () => _openDir(gameDir),
            ),
          if (gameDir.isNotEmpty)
            HoverButton(
              label: l10n.troubleOpenNamsConfig,
              color: AppColors.textSecondary,
              onTap: () => _openDir(p.join(gameDir, 'nams')),
            ),
          HoverButton(
            label: l10n.troubleOpenLogs,
            color: AppColors.textSecondary,
            onTap: () => _openDir(LogService.logsDirectory),
          ),
          if (launcherDir != null)
            HoverButton(
              label: l10n.troubleOpenBins,
              color: AppColors.textSecondary,
              onTap: () => _openDir(launcherDir),
            ),
        ],
      ),
    ]);
  }

  List<Widget> _verifyResultWidgets(
    BuildContext context,
    AppLocalizations l10n,
    VerifyOutcome outcome,
  ) {
    switch (outcome.status) {
      case VerifyStatus.ok:
        return [_statusLine(context, true, l10n.verifyInstallOk)];
      case VerifyStatus.failed:
        return [
          _statusLine(context, false, l10n.verifyInstallFailed),
          SizedBox(height: AppSizes.spacingSM(context)),
          ...?outcome.result?.checks.map((c) => _checkRow(context, l10n, c)),
        ];
      case VerifyStatus.noRuntime:
        return [_statusLine(context, false, l10n.verifyNoRuntime)];
      case VerifyStatus.steamNotRunning:
        return [_statusLine(context, false, l10n.verifySteamNotRunning)];
      case VerifyStatus.cannotParse:
      case VerifyStatus.error:
        return [_statusLine(context, false, l10n.verifyInstallError)];
    }
  }

  Widget _statusLine(BuildContext context, bool ok, String text) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Icon(
          ok ? Icons.check_circle : Icons.error_outline,
          size: AppSizes.iconSM(context),
          color: ok ? AppColors.success : AppColors.warning,
        ),
        SizedBox(width: AppSizes.spacingSM(context)),
        Expanded(
          child: Text(
            text,
            style: TextStyle(
              fontSize: AppSizes.fontSM(context),
              color: ok ? AppColors.success : AppColors.warning,
              height: 1.35,
            ),
          ),
        ),
      ],
    );
  }

  Widget _checkRow(
    BuildContext context,
    AppLocalizations l10n,
    VerifyCheck check,
  ) {
    return Padding(
      padding: EdgeInsets.only(bottom: AppSizes.paddingXS(context)),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(
            check.ok ? Icons.check_circle_outline : Icons.error_outline,
            size: AppSizes.iconSM(context),
            color: check.ok ? AppColors.success : AppColors.error,
          ),
          SizedBox(width: AppSizes.spacingSM(context)),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  _checkLabel(l10n, check.name),
                  style: TextStyle(
                    fontSize: AppSizes.fontSM(context),
                    color: AppColors.textSecondary,
                  ),
                ),
                if (check.detail != null && check.detail!.isNotEmpty)
                  Text(
                    check.detail!,
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      color: AppColors.textMuted,
                      height: 1.35,
                    ),
                  ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  String _checkLabel(AppLocalizations l10n, String name) {
    switch (name) {
      case 'steam_install':
        return l10n.verifyCheckSteamInstall;
      case 'nier_exe':
        return l10n.verifyCheckNierExe;
      case 'steam_api64_source':
        return l10n.verifyCheckSteamApi64;
      case 'runtime_writable':
        return l10n.verifyCheckRuntimeWritable;
      case 'runtime_library_cached':
        return l10n.verifyCheckRuntimeCached;
      default:
        return name;
    }
  }

  Widget _wrapperCard(BuildContext context, AppLocalizations l10n) {
    if (!_wrapperLoaded) {
      WidgetsBinding.instance.addPostFrameCallback((_) => _loadWrapper());
    }
    return _card(context, l10n.launchWrapperTitle, [
      Text(
        l10n.launchWrapperDesc,
        style: TextStyle(
          fontSize: AppSizes.fontXS(context),
          color: AppColors.textMuted,
          height: 1.4,
        ),
      ),
      SizedBox(height: AppSizes.spacingMD(context)),
      TextField(
        controller: _wrapperController,
        onChanged: (v) => LaunchWrapperService.save(v),
        style: TextStyle(
          fontSize: AppSizes.fontSM(context),
          color: AppColors.textPrimary,
          fontFamily: 'monospace',
        ),
        decoration: InputDecoration(
          isDense: true,
          hintText: l10n.launchWrapperHint,
          hintStyle: TextStyle(
            fontSize: AppSizes.fontSM(context),
            color: AppColors.textMuted.withValues(alpha: 0.6),
            fontFamily: 'monospace',
          ),
          filled: true,
          fillColor: AppColors.backgroundPrimary,
          contentPadding: EdgeInsets.symmetric(
            horizontal: AppSizes.paddingSM(context),
            vertical: AppSizes.paddingSM(context),
          ),
          enabledBorder: OutlineInputBorder(
            borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
            borderSide: const BorderSide(color: AppColors.borderLight),
          ),
          focusedBorder: OutlineInputBorder(
            borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
            borderSide: const BorderSide(color: AppColors.accentPrimary),
          ),
        ),
      ),
      SizedBox(height: AppSizes.spacingSM(context)),
      Text(
        l10n.launchWrapperExample,
        style: TextStyle(
          fontSize: AppSizes.fontXS(context),
          color: AppColors.textMuted.withValues(alpha: 0.8),
          fontFamily: 'monospace',
          height: 1.4,
        ),
      ),
    ]);
  }

  Widget _card(BuildContext context, String title, List<Widget> children,
      {Color? accent}) {
    final titleColor = accent ?? AppColors.accentPrimary;
    return Container(
      width: double.infinity,
      decoration: BoxDecoration(
        color: AppColors.backgroundCard,
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(
          color: accent?.withValues(alpha: 0.5) ?? AppColors.borderLight,
        ),
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
              color: accent?.withValues(alpha: 0.08) ?? AppColors.surfaceMedium,
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
                color: titleColor,
                letterSpacing: 0.5,
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
