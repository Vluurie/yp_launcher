import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:automato_theme/automato_theme.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:window_manager/window_manager.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/widgets/directory_selector.dart';
import 'package:yp_launcher/widgets/hover_button.dart';
import 'package:yp_launcher/widgets/play_button.dart';
import 'package:yp_launcher/widgets/textures/textures_view.dart';
import 'package:yp_launcher/services/platform_gate.dart';
import 'package:yp_launcher/widgets/windows_title_bar.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/config_state.dart';
import 'package:yp_launcher/providers/nams_settings_state.dart';
import 'package:yp_launcher/providers/log_state.dart';
import 'package:yp_launcher/providers/notification_state.dart';
import 'package:yp_launcher/services/process_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/log_panel.dart';
import 'package:yp_launcher/widgets/notification_banner.dart';
import 'package:yp_launcher/widgets/settings_view.dart';
import 'package:yp_launcher/widgets/lodmod_view.dart';
import 'package:yp_launcher/widgets/yorha_protocol_view.dart';
import 'package:yp_launcher/widgets/mods/mods_view.dart';
import 'package:yp_launcher/widgets/cutscenes_view.dart';
import 'package:yp_launcher/widgets/naiom_view.dart';
import 'package:yp_launcher/widgets/onboarding_wizard.dart';
import 'package:yp_launcher/widgets/info_bar.dart';
import 'package:yp_launcher/widgets/launcher_sidebar.dart';
import 'package:shared_preferences/shared_preferences.dart';

class LauncherScreen extends ConsumerStatefulWidget {
  const LauncherScreen({super.key});

  @override
  ConsumerState<LauncherScreen> createState() => _LauncherScreenState();
}

class _LauncherScreenState extends ConsumerState<LauncherScreen>
    with WindowListener {
  int _selectedTab = 0;
  final _logPanelKey = GlobalKey<LogPanelState>();
  bool? _onboardingComplete;

  /// Matches the hosts where main() initializes window_manager.
  static bool get _managesWindow => Platform.isWindows || Platform.isMacOS;

  @override
  void initState() {
    super.initState();
    _checkOnboarding();
    if (_managesWindow) {
      windowManager.addListener(this);
      windowManager.setPreventClose(true);
    }
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(notificationStateControllerProvider.notifier).checkPlatformSupport();
      _runCutsceneAutoDetect();
      ref.listenManual<AppState>(
        appStateControllerProvider,
        (prev, next) {
          if (prev?.selectedDirectory != next.selectedDirectory) {
            _runCutsceneAutoDetect();
          }
        },
      );
      ref.listenManual<int>(
        detectionRefreshProvider,
        (_, __) => _runCutsceneAutoDetect(),
      );
      ref.listenManual<int>(
        activeTabProvider,
        (prev, next) {
          if (next != _selectedTab) _switchTab(next);
        },
      );
    });
  }

  @override
  void dispose() {
    if (_managesWindow) {
      windowManager.removeListener(this);
    }
    super.dispose();
  }

  @override
  void onWindowClose() async {
    final busyCount = ref.read(busyOperationsProvider);
    final configUnsaved =
        ref.read(configStateControllerProvider).hasUnsavedChanges;
    final settingsUnsaved =
        ref.read(namsSettingsStateControllerProvider).hasUnsavedChanges;

    if (busyCount == 0 && !configUnsaved && !settingsUnsaved) {
      await windowManager.setPreventClose(false);
      await windowManager.close();
      return;
    }

    if (!mounted) return;
    final l10n = AppLocalizations.of(context)!;
    final shouldClose = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: AppColors.backgroundCard,
        title: Text(
          busyCount > 0 ? l10n.busyCloseTitle : l10n.unsavedChangesTitle,
          style: TextStyle(
            fontSize: AppSizes.fontXL(ctx),
            color: busyCount > 0 ? AppColors.warning : AppColors.textPrimary,
          ),
        ),
        content: Text(
          busyCount > 0 ? l10n.busyCloseBody : l10n.unsavedChangesMessage,
          style: TextStyle(
            fontSize: AppSizes.fontMD(ctx),
            color: AppColors.textSecondary,
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(false),
            child: Text(l10n.stay,
                style: const TextStyle(color: AppColors.textMuted)),
          ),
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(true),
            child: Text(
              busyCount > 0 ? l10n.busyCloseForce : l10n.discard,
              style: const TextStyle(
                color: AppColors.error,
                fontWeight: FontWeight.bold,
              ),
            ),
          ),
        ],
      ),
    );
    if (shouldClose == true) {
      await windowManager.setPreventClose(false);
      await windowManager.close();
    }
  }

  void _runCutsceneAutoDetect() {
    final dir = ref.read(appStateControllerProvider).selectedDirectory;
    if (dir.isEmpty) return;
    ref
        .read(configStateControllerProvider.notifier)
        .autoDetectCutscenes(dir);
  }

  Future<void> _checkOnboarding() async {
    final prefs = await SharedPreferences.getInstance();
    final complete = prefs.getBool('onboarding_complete') ?? false;
    if (mounted) {
      setState(() {
        _onboardingComplete = complete;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final appState = ref.watch(appStateControllerProvider);
    final hasDir = appState.isDirectorySelected;

    if (_onboardingComplete == null) {
      return const Scaffold(
        backgroundColor: AppColors.backgroundPrimary,
        body: SizedBox.shrink(),
      );
    }

    if (_onboardingComplete == false) {
      return Scaffold(
        body: Column(
          children: [
            if (PlatformGate.isWindows) const WindowsTitleBar(),
            Expanded(
              child: OnboardingWizard(
                onComplete: () => setState(() {
                  _onboardingComplete = true;
                }),
              ),
            ),
          ],
        ),
      );
    }

    return Scaffold(
      body: Column(
        children: [
          if (PlatformGate.isWindows) const WindowsTitleBar(),
          Expanded(
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                if (hasDir) _buildSidebar(),
                Expanded(
                  child: Stack(
                    children: [
                      _buildTabContent(),
                      Positioned(
                        bottom: AppSizes.paddingLG(context),
                        right: AppSizes.paddingLG(context),
                        child: NotificationBanners(),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildSidebar() {
    final l10n = AppLocalizations.of(context)!;
    return LauncherSidebar(
      sections: buildLauncherSections(l10n),
      activeIndex: _selectedTab,
      onSelect: _switchTab,
    );
  }

  bool get _hasUnsavedChanges {
    final configUnsaved = ref
        .read(configStateControllerProvider)
        .hasUnsavedChanges;
    final settingsUnsaved = ref
        .read(namsSettingsStateControllerProvider)
        .hasUnsavedChanges;
    return configUnsaved || settingsUnsaved;
  }

  Future<void> _switchTab(int index) async {
    if (_selectedTab == index) return;
    if (ref.read(texturesBusyProvider)) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(
              AppLocalizations.of(context)!.texturesBusyMessage,
              style: const TextStyle(color: Colors.white, fontWeight: FontWeight.bold),
            ),
            backgroundColor: AppColors.accentPrimary,
            behavior: SnackBarBehavior.floating,
            duration: const Duration(seconds: 2),
          ),
        );
      }
      return;
    }
    if (_hasUnsavedChanges) {
      final discard = await showDialog<bool>(
        context: context,
        builder: (ctx) {
          final cl10n = AppLocalizations.of(ctx)!;
          return AlertDialog(
            backgroundColor: AppColors.backgroundCard,
            title: Text(
              cl10n.unsavedChangesTitle,
              style: TextStyle(
                fontSize: AppSizes.fontXL(ctx),
                color: AppColors.textPrimary,
              ),
            ),
            content: Text(
              cl10n.unsavedChangesMessage,
              style: TextStyle(
                fontSize: AppSizes.fontMD(ctx),
                color: AppColors.textSecondary,
              ),
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.of(ctx).pop(false),
                child: Text(
                  cl10n.stay,
                  style: const TextStyle(color: AppColors.textMuted),
                ),
              ),
              TextButton(
                onPressed: () => Navigator.of(ctx).pop(true),
                child: Text(
                  cl10n.discard,
                  style: const TextStyle(
                    color: AppColors.error,
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ),
            ],
          );
        },
      );
      if (discard != true) return;
      final gameDir = ref.read(appStateControllerProvider).selectedDirectory;
      ref.read(configStateControllerProvider.notifier).discardChanges(gameDir);
      ref.read(namsSettingsStateControllerProvider.notifier).discardChanges();
    }
    setState(() => _selectedTab = index);
    ref.read(activeTabProvider.notifier).state = index;
  }

  Widget _buildTabContent() {
    return IndexedStack(
      index: _selectedTab,
      children: [
        _buildLauncherTab(),
        const SettingsView(),
        const LodmodView(),
        const TexturesView(),
        const YorhaProtocolView(),
        const ModsView(),
        const NaiomView(),
        const CutscenesView(),
      ],
    );
  }

  Widget _buildLauncherTab() {
    final l10n = AppLocalizations.of(context)!;
    return Stack(
      children: [
        AutomatoBackground(
          ref: ref,
          showBackgroundSVG: true,
          showMenuLines: true,
          backgroundColor: AppColors.backgroundPrimary,
          gradientColor: AppColors.backgroundSecondary,
          backgroundSvgConfig: const BackgroundSvgConfig(
            animateInner: true,
            animateOuter: true,
            showDual: true,
          ),
          linesConfig: LinesConfig(
            lineColor: AppColors.borderLight.withValues(alpha: 0.2),
            strokeWidth: 1.0,
            spacing: 10.0,
            drawVerticalLines: true,
            drawHorizontalLines: true,
            enableFlicker: false,
            flickerDuration: const Duration(milliseconds: 4000),
          ),
        ),
        Positioned.fill(
          child: IgnorePointer(
            child: DecoratedBox(
              decoration: BoxDecoration(
                gradient: RadialGradient(
                  center: Alignment.center,
                  radius: 0.9,
                  colors: [
                    Colors.transparent,
                    AppColors.backgroundPrimary.withValues(alpha: 0.55),
                  ],
                  stops: const [0.55, 1.0],
                ),
              ),
            ),
          ),
        ),
        if (!PlatformGate.canLaunchGame)
          Positioned.fill(
            child: IgnorePointer(
              child: DecoratedBox(
                decoration: BoxDecoration(
                  color: AppColors.error.withValues(alpha: 0.10),
                ),
              ),
            ),
          ),
        Center(
          child: Padding(
            padding: EdgeInsets.symmetric(
              horizontal: AppSizes.paddingXL(context),
            ),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                const PlayButton(),
                SizedBox(height: AppSizes.spacingSM(context)),
                const MinimizeOnLaunchToggle(),
                SizedBox(height: AppSizes.spacingSM(context)),
                _buildStatusLine(),
              ],
            ),
          ),
        ),
        Positioned(
          top: AppSizes.paddingMD(context),
          left: AppSizes.infoBarPaddingH(context),
          right: AppSizes.infoBarPaddingH(context) + 240,
          child: Align(
            alignment: Alignment.centerLeft,
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 720),
              child: const DirectorySelector(),
            ),
          ),
        ),
        Positioned(
          top: AppSizes.paddingMD(context),
          right: AppSizes.infoBarPaddingH(context),
          child: Container(
            padding: EdgeInsets.symmetric(
              horizontal: AppSizes.chipPaddingH(context),
              vertical: AppSizes.chipPaddingV(context),
            ),
            decoration: BoxDecoration(
              color: AppColors.backgroundCard,
              borderRadius: BorderRadius.circular(
                AppSizes.borderRadius(context),
              ),
              border: Border.all(color: AppColors.borderLight),
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                if (ref
                    .watch(appStateControllerProvider)
                    .isDirectorySelected) ...[
                  _buildIconAction(
                    Icons.terminal,
                    'Copy the NAMS command to clipboard so you can paste it into a terminal and start the game manually.',
                    () => _copyLaunchCommand(),
                  ),
                  SizedBox(width: AppSizes.spacingSM(context)),
                ],
                _buildLinkIcon(
                  Icons.code,
                  l10n.tooltipLauncherSource,
                  AppStrings.githubUrl,
                ),
                SizedBox(width: AppSizes.spacingSM(context)),
                _buildLinkIcon(
                  Icons.merge_type,
                  l10n.tooltipNamsSource,
                  AppStrings.namsGitlabUrl,
                ),
                SizedBox(width: AppSizes.spacingSM(context)),
                _buildLinkIcon(
                  Icons.menu_book,
                  l10n.tooltipGuide,
                  AppStrings.guideUrl,
                ),
                SizedBox(width: AppSizes.spacingSM(context)),
                _buildLinkIcon(
                  Icons.forum,
                  l10n.tooltipDiscord,
                  AppStrings.discordUrl,
                ),
              ],
            ),
          ),
        ),
        InfoBar(
          logPanelKey: _logPanelKey,
          onOpenLogs: () {
            ref.read(logPanelOpenProvider.notifier).state = true;
            ref.read(logStateControllerProvider.notifier).loadLogs();
          },
        ),
        Positioned(
          bottom: AppSizes.spacingMD(context),
          left: AppSizes.infoBarPaddingH(context),
          right: AppSizes.infoBarPaddingH(context),
          child: Container(
            padding: EdgeInsets.symmetric(
              horizontal: AppSizes.chipPaddingH(context),
              vertical: AppSizes.chipPaddingV(context) + 2,
            ),
            decoration: BoxDecoration(
              color: AppColors.backgroundCard.withValues(alpha: 0.7),
              borderRadius: BorderRadius.circular(
                AppSizes.borderRadius(context),
              ),
            ),
            child: Wrap(
              alignment: WrapAlignment.center,
              crossAxisAlignment: WrapCrossAlignment.center,
              children: [
                Text(
                  l10n.helpPrefix,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.textMuted,
                  ),
                ),
                _HoverTextLink(
                  label: l10n.helpNaoLauncher,
                  url: AppStrings.naoLauncherUrl,
                ),
                Text(
                  l10n.helpOr,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.textMuted,
                  ),
                ),
                _HoverTextLink(
                  label: l10n.helpCommandLine,
                  url: AppStrings.cliDocsUrl,
                ),
                Text(
                  l10n.helpJoinDiscord,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.textMuted,
                  ),
                ),
                _HoverTextLink(
                  label: l10n.helpDiscord,
                  url: AppStrings.discordUrl,
                ),
                Text(
                  l10n.helpSuffix,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.textMuted,
                  ),
                ),
              ],
            ),
          ),
        ),
        const PersistentWarningBanner(),
        if (ref.watch(logPanelOpenProvider)) ...[
          Positioned.fill(
            child: GestureDetector(
              onTap: () => _logPanelKey.currentState?.animateClose(),
              child: const ColoredBox(color: Colors.transparent),
            ),
          ),
          Positioned(
            top: 0,
            bottom: 0,
            right: 0,
            child: LogPanel(
              key: _logPanelKey,
              onClose: () {
                ref.read(logPanelOpenProvider.notifier).state = false;
                ref.read(logStateControllerProvider.notifier).stopStreaming();
              },
            ),
          ),
        ],
      ],
    );
  }

  Widget _buildStatusLine() {
    final l10n = AppLocalizations.of(context)!;
    final appState = ref.watch(appStateControllerProvider);

    final hasError = appState.errorMessage != null;
    final hasStatus = appState.statusMessage != null;
    final isReady =
        appState.isDirectorySelected &&
        appState.playButtonState == PlayButtonState.idle &&
        !hasError;

    String text;
    Color color;

    if (hasError) {
      text = appState.errorMessage!;
      color = AppColors.error;
    } else if (hasStatus) {
      text = appState.statusMessage!;
      color = AppColors.textSecondary;
    } else if (!PlatformGate.canLaunchGame) {
      text = PlatformGate.isMacOS
          ? l10n.platformUnsupportedMacos
          : l10n.platformUnsupportedLinux;
      color = AppColors.warning;
    } else if (isReady) {
      text = l10n.statusReady;
      color = AppColors.success;
    } else if (!appState.isDirectorySelected) {
      text = l10n.statusSelectGame;
      color = AppColors.textMuted;
    } else {
      text = '';
      color = AppColors.textMuted;
    }

    final isMultiLine = !PlatformGate.canLaunchGame && !hasError && !hasStatus;

    return AnimatedSwitcher(
      duration: const Duration(milliseconds: 200),
      child: ConstrainedBox(
        key: ValueKey(text),
        constraints: BoxConstraints(
          minHeight: 28,
          maxWidth: isMultiLine ? 520 : double.infinity,
        ),
        child: Text(
          text,
          style: TextStyle(
            fontSize: isMultiLine
                ? AppSizes.fontMD(context)
                : AppSizes.fontLG(context),
            color: color,
            height: 1.4,
          ),
          textAlign: TextAlign.center,
        ),
      ),
    );
  }

  Widget _buildLinkIcon(IconData icon, String tooltip, String url) {
    return HoverIconButton(
      tooltip: tooltip,
      bordered: false,
      padding: EdgeInsets.all(AppSizes.paddingXS(context)),
      radius: AppSizes.borderRadius(context),
      onTap: () async {
        final uri = Uri.parse(url);
        if (await canLaunchUrl(uri)) {
          await launchUrl(uri, mode: LaunchMode.externalApplication);
        }
      },
      icon: Icon(
        icon,
        size: AppSizes.iconMD(context),
        color: AppColors.accentPrimary,
      ),
    );
  }

  Widget _buildIconAction(IconData icon, String tooltip, VoidCallback onTap) {
    return HoverIconButton(
      tooltip: tooltip,
      bordered: false,
      padding: EdgeInsets.all(AppSizes.paddingXS(context)),
      radius: AppSizes.borderRadius(context),
      onTap: onTap,
      icon: Icon(
        icon,
        size: AppSizes.iconMD(context),
        color: AppColors.accentPrimary,
      ),
    );
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
          message:
              'Could not build launch command — launcher binaries are not ready yet.',
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
        message:
            'Launch command copied — paste it into a terminal to start the game manually.',
        icon: Icons.check_circle,
        color: AppColors.success,
        type: NotificationType.general,
      ),
    );
  }
}

class _HoverTextLink extends StatefulWidget {
  final String label;
  final String url;

  const _HoverTextLink({required this.label, required this.url});

  @override
  State<_HoverTextLink> createState() => _HoverTextLinkState();
}

class _HoverTextLinkState extends State<_HoverTextLink> {
  bool _hovering = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovering = true),
      onExit: (_) => setState(() => _hovering = false),
      child: GestureDetector(
        onTap: () async {
          final uri = Uri.parse(widget.url);
          if (await canLaunchUrl(uri)) {
            await launchUrl(uri, mode: LaunchMode.externalApplication);
          }
        },
        child: Text(
          widget.label,
          style: TextStyle(
            fontSize: AppSizes.fontXS(context),
            color: _hovering ? AppColors.textPrimary : AppColors.accentPrimary,
            decoration: TextDecoration.underline,
            decorationColor: _hovering
                ? AppColors.textPrimary
                : AppColors.accentPrimary,
          ),
        ),
      ),
    );
  }
}
