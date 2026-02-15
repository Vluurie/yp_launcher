import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:automato_theme/automato_theme.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/widgets/directory_selector.dart';
import 'package:yp_launcher/widgets/play_button.dart';
import 'package:yp_launcher/widgets/windows_title_bar.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/config_state.dart';
import 'package:yp_launcher/providers/log_state.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/config_panel.dart';
import 'package:yp_launcher/widgets/log_panel.dart';
import 'package:yp_launcher/providers/notification_state.dart';
import 'package:yp_launcher/services/shortcut_service.dart';
import 'package:yp_launcher/widgets/notification_banner.dart';

class LauncherScreen extends ConsumerStatefulWidget {
  const LauncherScreen({super.key});

  @override
  ConsumerState<LauncherScreen> createState() => _LauncherScreenState();
}

class _LauncherScreenState extends ConsumerState<LauncherScreen> {
  final _configPanelKey = GlobalKey<ConfigPanelState>();
  final _logPanelKey = GlobalKey<LogPanelState>();

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Column(
        children: [
          if (Platform.isWindows) const WindowsTitleBar(),
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
                    lineColor: AppColors.borderLight.withValues(alpha: 0.4),
                    strokeWidth: 1.0,
                    spacing: 10.0,
                    drawVerticalLines: true,
                    drawHorizontalLines: true,
                    enableFlicker: false,
                    flickerDuration: const Duration(milliseconds: 4000),
                  ),
                ),
                // Main content centered
                Center(
                  child: Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 32.0),
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        const DirectorySelector(),
                        const SizedBox(height: 16),
                        const PlayButton(),
                        const SizedBox(height: 8),
                        _buildStatusLine(),
                        _buildFeatureInfo(),
                      ],
                    ),
                  ),
                ),
                // Top-left: info text
                Positioned(
                  top: 10,
                  left: 12,
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 5),
                    decoration: BoxDecoration(
                      color: AppColors.backgroundCard,
                      borderRadius: BorderRadius.circular(6),
                      border: Border.all(color: AppColors.borderLight),
                    ),
                    child: Text(
                      AppStrings.infoText,
                      style: TextStyle(fontSize: AppSizes.fontMD, color: AppColors.textSecondary),
                    ),
                  ),
                ),
                // Notification banners
                Positioned(
                  bottom: 40,
                  left: 12,
                  right: 12,
                  child: NotificationBanners(),
                ),
                // Top-right: social links
                Positioned(
                  top: 10,
                  right: 12,
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 3),
                    decoration: BoxDecoration(
                      color: AppColors.backgroundCard,
                      borderRadius: BorderRadius.circular(6),
                      border: Border.all(color: AppColors.borderLight),
                    ),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        _buildLinkIcon(FontAwesomeIcons.github, AppStrings.tooltipLauncherSource, AppStrings.githubUrl),
                        const SizedBox(width: 2),
                        _buildLinkIcon(FontAwesomeIcons.gitlab, AppStrings.tooltipNamsSource, AppStrings.namsGitlabUrl),
                        const SizedBox(width: 2),
                        _buildLinkIcon(FontAwesomeIcons.book, AppStrings.tooltipGuide, AppStrings.guideUrl),
                        const SizedBox(width: 2),
                        _buildLinkIcon(FontAwesomeIcons.discord, AppStrings.tooltipDiscord, AppStrings.discordUrl),
                      ],
                    ),
                  ),
                ),
                // Bottom: troubleshooting help
                Positioned(
                  bottom: 8,
                  left: 12,
                  right: 12,
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 5),
                    decoration: BoxDecoration(
                      color: AppColors.backgroundCard,
                      borderRadius: BorderRadius.circular(6),
                      border: Border.all(color: AppColors.borderLight),
                    ),
                    child: Row(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        Text(
                          AppStrings.helpPrefix,
                          style: TextStyle(fontSize: AppSizes.fontSM, color: AppColors.textMuted),
                        ),
                        _buildTextLink(AppStrings.helpNaoLauncher, AppStrings.naoLauncherUrl),
                        Text(
                          AppStrings.helpOr,
                          style: TextStyle(fontSize: AppSizes.fontSM, color: AppColors.textMuted),
                        ),
                        _buildTextLink(AppStrings.helpCommandLine, AppStrings.cliDocsUrl),
                        Text(
                          AppStrings.helpJoinDiscord,
                          style: TextStyle(fontSize: AppSizes.fontSM, color: AppColors.textMuted),
                        ),
                        _buildTextLink(AppStrings.helpDiscord, AppStrings.discordUrl),
                        Text(
                          AppStrings.helpSuffix,
                          style: TextStyle(fontSize: AppSizes.fontSM, color: AppColors.textMuted),
                        ),
                      ],
                    ),
                  ),
                ),
                // Config panel overlay
                if (ref.watch(configPanelOpenProvider)) ...[
                  Positioned.fill(
                    child: GestureDetector(
                      onTap: () => _configPanelKey.currentState?.animateClose(),
                      child: const ColoredBox(color: Colors.transparent),
                    ),
                  ),
                  Positioned(
                    top: 0,
                    bottom: 0,
                    right: 0,
                    child: ConfigPanel(
                      key: _configPanelKey,
                      onClose: () => ref.read(configPanelOpenProvider.notifier).state = false,
                    ),
                  ),
                ],
                // Log panel overlay
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
                      onClose: () => ref.read(logPanelOpenProvider.notifier).state = false,
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

  Widget _buildStatusLine() {
    final appState = ref.watch(appStateControllerProvider);

    final hasError = appState.errorMessage != null;
    final hasStatus = appState.statusMessage != null;
    final isReady = appState.isDirectorySelected &&
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
    } else if (isReady) {
      text = AppStrings.statusReady;
      color = AppColors.success;
    } else if (!appState.isDirectorySelected) {
      text = AppStrings.statusSelectGame;
      color = AppColors.textMuted;
    } else {
      text = '';
      color = AppColors.textMuted;
    }

    return SizedBox(
      height: 20,
      child: Text(
        text,
        style: TextStyle(fontSize: AppSizes.fontMD, color: color),
        textAlign: TextAlign.center,
      ),
    );
  }

  Widget _buildFeatureInfo() {
    final appState = ref.watch(appStateControllerProvider);
    if (!appState.isDirectorySelected) return const SizedBox.shrink();

    final gameDir = appState.selectedDirectory;

    return Padding(
      padding: const EdgeInsets.only(top: 8),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          _buildFeatureLine(AppStrings.featureReshade),
          _buildFeatureLine(AppStrings.featureTextures),
          _buildFeatureLine(AppStrings.featureLodMod),
          const SizedBox(height: 4),
          Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                'Configs: ',
                style: TextStyle(fontSize: AppSizes.fontSM, color: AppColors.textMuted),
              ),
              _HoverTextLink(
                label: '$gameDir\\nams\\',
                url: Uri.directory('$gameDir\\nams').toString(),
                isLocalPath: true,
              ),
              const SizedBox(width: 8),
              _HoverActionButton(
                label: 'EDIT',
                tooltip: AppStrings.tooltipEditConfigs,
                onTap: () {
                  ref.read(configPanelOpenProvider.notifier).state = true;
                  ref.read(configStateControllerProvider.notifier).loadConfigs(gameDir);
                },
              ),
              const SizedBox(width: 6),
              _HoverActionButton(
                label: 'LOGS',
                tooltip: AppStrings.tooltipOpenLogs,
                onTap: () {
                  ref.read(logPanelOpenProvider.notifier).state = true;
                  ref.read(logStateControllerProvider.notifier).loadLogs();
                },
              ),
              if (Platform.isWindows) ...[
                const SizedBox(width: 6),
                _HoverActionButton(
                  label: 'SHORTCUT',
                  tooltip: AppStrings.tooltipCreateShortcut,
                  onTap: () async {
                    final success = await ShortcutService.createDesktopShortcut(
                      gameDirectory: gameDir,
                    );
                    final notifier = ref.read(
                      notificationStateControllerProvider.notifier,
                    );
                    notifier.addNotification(NotificationItem(
                      id: 'shortcut_${DateTime.now().millisecondsSinceEpoch}',
                      message: success
                          ? AppStrings.notifyShortcutCreated
                          : AppStrings.notifyShortcutFailed,
                      icon: success ? Icons.check_circle : Icons.error_outline,
                      color: success ? AppColors.success : AppColors.error,
                      type: NotificationType.shortcut,
                    ));
                  },
                ),
              ],
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildFeatureLine(String text) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 1),
      child: Text(
        text,
        style: TextStyle(fontSize: AppSizes.fontSM, color: AppColors.textMuted),
        textAlign: TextAlign.center,
      ),
    );
  }

  Widget _buildTextLink(String label, String url) {
    return _HoverTextLink(label: label, url: url);
  }

  Widget _buildLinkIcon(IconData icon, String tooltip, String url) {

    return Tooltip(
      message: tooltip,
      child: InkWell(
        onTap: () async {
          final uri = Uri.parse(url);
          if (await canLaunchUrl(uri)) {
            await launchUrl(uri, mode: LaunchMode.externalApplication);
          }
        },
        borderRadius: BorderRadius.circular(4),
        child: Padding(
          padding: const EdgeInsets.all(4),
          child: FaIcon(icon, size: AppSizes.iconMD, color: AppColors.accentPrimary),
        ),
      ),
    );
  }
}

class _HoverActionButton extends StatefulWidget {
  final String label;
  final String tooltip;
  final VoidCallback onTap;

  const _HoverActionButton({
    required this.label,
    required this.tooltip,
    required this.onTap,
  });

  @override
  State<_HoverActionButton> createState() => _HoverActionButtonState();
}

class _HoverActionButtonState extends State<_HoverActionButton> {
  bool _hovering = false;

  @override
  Widget build(BuildContext context) {
    return Tooltip(
      message: widget.tooltip,
      child: MouseRegion(
        cursor: SystemMouseCursors.click,
        onEnter: (_) => setState(() => _hovering = true),
        onExit: (_) => setState(() => _hovering = false),
        child: GestureDetector(
          onTap: widget.onTap,
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
            decoration: BoxDecoration(
              color: _hovering ? AppColors.accentPrimary : Colors.transparent,
              border: Border.all(color: AppColors.accentPrimary),
              borderRadius: BorderRadius.circular(4),
            ),
            child: Text(
              widget.label,
              style: TextStyle(
                fontSize: AppSizes.fontMD,
                color: _hovering ? AppColors.buttonText : AppColors.accentPrimary,
                fontWeight: FontWeight.bold,
              ),
            ),
          ),
        ),
      ),
    );
  }
}

class _HoverTextLink extends StatefulWidget {
  final String label;
  final String url;
  final bool isLocalPath;

  const _HoverTextLink({required this.label, required this.url, this.isLocalPath = false});

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
            fontSize: AppSizes.fontSM,
            color: _hovering ? AppColors.textPrimary : AppColors.accentPrimary,
            decoration: TextDecoration.underline,
            decorationColor: _hovering ? AppColors.textPrimary : AppColors.accentPrimary,
          ),
        ),
      ),
    );
  }
}
