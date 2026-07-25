import 'dart:async';
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:automato_theme/automato_theme.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:window_manager/window_manager.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/l10n/app_localizations_en.dart';
import 'package:yp_launcher/widgets/directory_selector.dart';
import 'package:yp_launcher/widgets/hover_button.dart';
import 'package:yp_launcher/widgets/play_button.dart';
import 'package:yp_launcher/widgets/textures/textures_view.dart';
import 'package:yp_launcher/services/platform_gate.dart';
import 'package:yp_launcher/widgets/windows_title_bar.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/config_state.dart';
import 'package:yp_launcher/providers/nams_settings_state.dart';
import 'package:yp_launcher/providers/locale_state.dart';
import 'package:yp_launcher/providers/log_state.dart';
import 'package:yp_launcher/providers/notification_state.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/log_panel.dart';
import 'package:yp_launcher/widgets/notification_banner.dart';
import 'package:yp_launcher/widgets/settings_view.dart';
import 'package:yp_launcher/widgets/launcher_settings_view.dart';
import 'package:yp_launcher/widgets/lodmod_view.dart';
import 'package:yp_launcher/widgets/yorha_protocol_view.dart';
import 'package:yp_launcher/widgets/mods/mods_view.dart';
import 'package:yp_launcher/widgets/cutscenes_view.dart';
import 'package:yp_launcher/widgets/thirdparty/thirdparty_view.dart';
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
  final _languageMenuKey = GlobalKey<PopupMenuButtonState<Locale>>();
  bool? _onboardingComplete;
  bool _launchOptionsOpen = false;
  final Set<int> _visitedTabs = {0};
  Timer? _warmupTimer;

  /// Matches the hosts where main() initializes window_manager.
  static bool get _managesWindow =>
      Platform.isWindows || Platform.isMacOS || Platform.isLinux;

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
      _startWarmup();
    });
  }

  @override
  void dispose() {
    _warmupTimer?.cancel();
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

  Widget _buildSplash(BuildContext context) {
    return Scaffold(
      backgroundColor: AppColors.backgroundPrimary,
      body: Column(
        children: [
          if (PlatformGate.isWindows) const WindowsTitleBar(),
          Expanded(
            child: Stack(
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
                Center(
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      const SizedBox(
                        width: 40,
                        height: 40,
                        child: CircularProgressIndicator(
                          strokeWidth: 2.5,
                          color: AppColors.accentPrimary,
                        ),
                      ),
                      SizedBox(height: AppSizes.spacingLG(context)),
                      Text(
                        AppLocalizations.of(context)!.appTitle,
                        style: TextStyle(
                          fontSize: AppSizes.fontLG(context),
                          fontWeight: FontWeight.bold,
                          color: AppColors.textSecondary,
                          letterSpacing: 1.5,
                        ),
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

  @override
  Widget build(BuildContext context) {
    final appState = ref.watch(appStateControllerProvider);
    final hasDir = appState.isDirectorySelected;

    if (_onboardingComplete == null) {
      return _buildSplash(context);
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

  void _startWarmup() {
    var next = 1;
    _warmupTimer = Timer.periodic(const Duration(milliseconds: 350), (timer) {
      while (next < 10 && _visitedTabs.contains(next)) {
        next++;
      }
      if (next >= 10) {
        timer.cancel();
        return;
      }
      if (mounted) setState(() => _visitedTabs.add(next));
      next++;
    });
  }

  Widget _tabAt(int index) {
    switch (index) {
      case 0:
        return _buildLauncherTab();
      case 1:
        return const SettingsView();
      case 2:
        return const LodmodView();
      case 3:
        return const TexturesView();
      case 4:
        return const YorhaProtocolView();
      case 5:
        return const ModsView();
      case 6:
        return const NaiomView();
      case 7:
        return const CutscenesView();
      case 8:
        return const LauncherSettingsView();
      case 9:
        return const ThirdPartyView();
      default:
        return const SizedBox.shrink();
    }
  }

  Widget _buildTabContent() {
    _visitedTabs.add(_selectedTab);
    return IndexedStack(
      index: _selectedTab,
      children: [
        for (var i = 0; i < 10; i++)
          _visitedTabs.contains(i)
              ? _tabAt(i)
              : const SizedBox.shrink(),
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
        Positioned(
          top: AppSizes.paddingMD(context) + 56,
          left: 0,
          right: 56,
          bottom: _footerReserve(context),
          child: Center(
            child: SingleChildScrollView(
              padding: EdgeInsets.symmetric(
                horizontal: AppSizes.paddingXL(context),
                vertical: AppSizes.paddingMD(context),
              ),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  const PlayButton(),
                  SizedBox(height: AppSizes.spacingSM(context)),
                  _buildStatusLine(),
                ],
              ),
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
                SizedBox(width: AppSizes.spacingSM(context)),
                _buildLanguageSelector(l10n),
              ],
            ),
          ),
        ),
        Positioned(
          top: AppSizes.paddingMD(context) + 56,
          bottom: _footerReserve(context),
          right: AppSizes.infoBarPaddingH(context),
          child: const Center(
            child: SingleChildScrollView(
              child: DetectionStatusStrip(),
            ),
          ),
        ),
        Positioned(
          bottom: AppSizes.spacingMD(context),
          left: AppSizes.infoBarPaddingH(context),
          right: AppSizes.infoBarPaddingH(context),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              InfoBar(
                logPanelKey: _logPanelKey,
                leading: _buildLaunchOptionsHeader(context),
                onOpenLogs: () {
                  ref.read(logPanelOpenProvider.notifier).state = true;
                  ref.read(logStateControllerProvider.notifier).loadLogs();
                },
              ),
              SizedBox(height: AppSizes.spacingSM(context)),
              SizedBox(
                width: double.infinity,
                child: _buildHelpText(l10n),
              ),
            ],
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
        if (_launchOptionsOpen) ...[
          Positioned.fill(
            child: GestureDetector(
              behavior: HitTestBehavior.translucent,
              onTap: () => setState(() => _launchOptionsOpen = false),
            ),
          ),
          Positioned(
            left: AppSizes.infoBarPaddingH(context),
            bottom: _footerReserve(context) + 40,
            child: _buildLaunchOptionsExpanded(context),
          ),
        ],
      ],
    );
  }

  double _footerReserve(BuildContext context) {
    return AppSizes.spacingMD(context) + 88;
  }

  Widget _buildHelpText(AppLocalizations l10n) {
    return Container(
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.chipPaddingH(context),
        vertical: AppSizes.chipPaddingV(context) + 2,
      ),
      decoration: BoxDecoration(
        color: AppColors.backgroundCard.withValues(alpha: 0.7),
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
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
    );
  }

  List<Widget> _launchOptionItems() {
    return const [
      MinimizeOnLaunchToggle(),
      PreferDedicatedGpuToggle(),
    ];
  }

  Widget _buildLaunchOptionsHeader(BuildContext context) {
    if (!Platform.isWindows && !Platform.isLinux) {
      return const SizedBox.shrink();
    }
    final l10n = AppLocalizations.of(context)!;
    return Material(
      color: Colors.transparent,
      child: InkWell(
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        onTap: () => setState(() => _launchOptionsOpen = !_launchOptionsOpen),
        child: Container(
          padding: EdgeInsets.symmetric(
            horizontal: AppSizes.paddingMD(context),
            vertical: AppSizes.paddingSM(context),
          ),
          decoration: BoxDecoration(
            color: AppColors.backgroundCard.withValues(alpha: 0.7),
            borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
            border: Border.all(
              color: AppColors.borderLight.withValues(alpha: 0.7),
            ),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(
                Icons.tune,
                size: AppSizes.iconSM(context),
                color: AppColors.accentPrimary,
              ),
              const SizedBox(width: 6),
              Text(
                l10n.launchOptionsTitle,
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  fontWeight: FontWeight.bold,
                  color: AppColors.accentPrimary,
                  letterSpacing: 1.0,
                ),
              ),
              const SizedBox(width: 6),
              AnimatedRotation(
                turns: _launchOptionsOpen ? 0.5 : 0,
                duration: const Duration(milliseconds: 150),
                child: Icon(
                  Icons.keyboard_arrow_up,
                  size: AppSizes.iconSM(context),
                  color: AppColors.accentPrimary,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildLaunchOptionsExpanded(BuildContext context) {
    return ConstrainedBox(
      constraints: BoxConstraints(
        maxWidth: 300,
        maxHeight: MediaQuery.sizeOf(context).height * 0.5,
      ),
      child: Container(
        padding: EdgeInsets.symmetric(
          horizontal: AppSizes.paddingMD(context),
          vertical: AppSizes.paddingSM(context),
        ),
        decoration: BoxDecoration(
          color: AppColors.backgroundCard.withValues(alpha: 0.95),
          borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
          border: Border.all(
            color: AppColors.borderLight.withValues(alpha: 0.7),
          ),
        ),
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: _launchOptionItems(),
          ),
        ),
      ),
    );
  }

  Widget _buildStatusLine() {
    final l10n = AppLocalizations.of(context)!;
    final appState = ref.watch(appStateControllerProvider);

    final hasError = appState.errorMessage != null;
    final hasStatus = appState.status != null;
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
      text = switch (appState.status!) {
        LaunchStatus.launching => l10n.statusLaunching,
        LaunchStatus.running => l10n.statusRunning,
        LaunchStatus.stopped => l10n.statusStopped,
        LaunchStatus.stopping => l10n.statusStopping,
      };
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

  Widget _buildLanguageSelector(AppLocalizations l10n) {
    final current =
        ref.watch(localeControllerProvider) ??
        Localizations.localeOf(context);
    return PopupMenuButton<Locale>(
      key: _languageMenuKey,
      tooltip: '',
      color: AppColors.backgroundCard,
      position: PopupMenuPosition.under,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        side: const BorderSide(color: AppColors.borderLight),
      ),
      onSelected: (locale) =>
          ref.read(localeControllerProvider.notifier).setLocale(locale),
      itemBuilder: (context) => [
        for (final locale in kSupportedLocales)
          PopupMenuItem<Locale>(
            value: locale,
            child: Row(
              children: [
                Icon(
                  locale.languageCode == current.languageCode
                      ? Icons.check
                      : Icons.language,
                  size: AppSizes.iconSM(context),
                  color: locale.languageCode == current.languageCode
                      ? AppColors.accentPrimary
                      : AppColors.textSecondary,
                ),
                SizedBox(width: AppSizes.spacingSM(context)),
                Text(
                  localeDisplayName(locale),
                  style: TextStyle(
                    color: locale.languageCode == current.languageCode
                        ? AppColors.accentPrimary
                        : AppColors.textPrimary,
                  ),
                ),
              ],
            ),
          ),
        const PopupMenuDivider(),
        PopupMenuItem<Locale>(
          enabled: false,
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 240),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Icon(
                      Icons.info_outline,
                      size: AppSizes.iconSM(context),
                      color: AppColors.warning,
                    ),
                    SizedBox(width: AppSizes.spacingSM(context)),
                    Expanded(
                      child: Text(
                        l10n.languageSupportNotice,
                        style: TextStyle(
                          fontSize: AppSizes.fontXS(context),
                          color: AppColors.textMuted,
                          height: 1.35,
                        ),
                      ),
                    ),
                  ],
                ),
                if (current.languageCode != 'en') ...[
                  SizedBox(height: AppSizes.spacingSM(context) / 2),
                  Padding(
                    padding: EdgeInsets.only(
                      left: AppSizes.iconSM(context) + AppSizes.spacingSM(context),
                    ),
                    child: Text(
                      AppLocalizationsEn().languageSupportNotice,
                      style: TextStyle(
                        fontSize: AppSizes.fontXS(context),
                        color: AppColors.textMuted.withValues(alpha: 0.7),
                        height: 1.35,
                        fontStyle: FontStyle.italic,
                      ),
                    ),
                  ),
                ],
              ],
            ),
          ),
        ),
      ],
      child: HoverIconButton(
        tooltip: l10n.tooltipLanguage,
        bordered: false,
        padding: EdgeInsets.all(AppSizes.paddingXS(context)),
        radius: AppSizes.borderRadius(context),
        onTap: () => _languageMenuKey.currentState?.showButtonMenu(),
        icon: Icon(
          Icons.language,
          size: AppSizes.iconMD(context),
          color: AppColors.accentPrimary,
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
