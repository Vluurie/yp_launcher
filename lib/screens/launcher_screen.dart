import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:automato_theme/automato_theme.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:window_manager/window_manager.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/widgets/directory_selector.dart';
import 'package:yp_launcher/widgets/play_button.dart';
import 'package:yp_launcher/widgets/windows_title_bar.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/theme/app_colors.dart';

class LauncherScreen extends ConsumerStatefulWidget {
  const LauncherScreen({super.key});

  @override
  ConsumerState<LauncherScreen> createState() => _LauncherScreenState();
}

class _LauncherScreenState extends ConsumerState<LauncherScreen> with WindowListener {
  bool _isClosing = false;

  @override
  void initState() {
    super.initState();
    if (Platform.isWindows) {
      windowManager.addListener(this);
      windowManager.setPreventClose(true);
    }
  }

  @override
  void dispose() {
    if (Platform.isWindows && !_isClosing) {
      windowManager.removeListener(this);
    }
    super.dispose();
  }

  @override
  void onWindowClose() {
    if (_isClosing) return;
    _isClosing = true;
    windowManager.destroy();
  }

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
                    child: const Text(
                      AppStrings.infoText,
                      style: TextStyle(fontSize: 10, color: AppColors.textSecondary),
                    ),
                  ),
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
                        _buildLinkIcon(FontAwesomeIcons.github, AppStrings.tooltipSourceCode, AppStrings.githubUrl),
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
                        const Text(
                          AppStrings.helpPrefix,
                          style: TextStyle(fontSize: 9, color: AppColors.textMuted),
                        ),
                        _buildTextLink(AppStrings.helpNaoLauncher, AppStrings.naoLauncherUrl),
                        const Text(
                          AppStrings.helpOr,
                          style: TextStyle(fontSize: 9, color: AppColors.textMuted),
                        ),
                        _buildTextLink(AppStrings.helpCommandLine, AppStrings.cliDocsUrl),
                        const Text(
                          AppStrings.helpJoinDiscord,
                          style: TextStyle(fontSize: 9, color: AppColors.textMuted),
                        ),
                        _buildTextLink(AppStrings.helpDiscord, AppStrings.discordUrl),
                        const Text(
                          AppStrings.helpSuffix,
                          style: TextStyle(fontSize: 9, color: AppColors.textMuted),
                        ),
                      ],
                    ),
                  ),
                ),
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
      height: 16,
      child: Text(
        text,
        style: TextStyle(fontSize: 10, color: color),
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
          child: FaIcon(icon, size: 14, color: AppColors.accentPrimary),
        ),
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
            fontSize: 9,
            color: _hovering ? AppColors.textPrimary : AppColors.accentPrimary,
            decoration: TextDecoration.underline,
            decorationColor: _hovering ? AppColors.textPrimary : AppColors.accentPrimary,
          ),
        ),
      ),
    );
  }
}
