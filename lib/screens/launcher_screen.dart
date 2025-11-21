import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:automato_theme/automato_theme.dart';
import 'package:yp_launcher/widgets/directory_selector.dart';
import 'package:yp_launcher/widgets/play_button.dart';
import 'package:yp_launcher/widgets/config_dialog.dart';

class LauncherScreen extends ConsumerStatefulWidget {
  const LauncherScreen({super.key});

  @override
  ConsumerState<LauncherScreen> createState() => _LauncherScreenState();
}

class _LauncherScreenState extends ConsumerState<LauncherScreen>
    with SingleTickerProviderStateMixin {
  late AnimationController _animationController;
  late Animation<double> _pulseAnimation;

  @override
  void initState() {
    super.initState();
    _animationController = AnimationController(
      duration: const Duration(seconds: 4),
      vsync: this,
    )..repeat(reverse: true);

    _pulseAnimation = Tween<double>(begin: 0.6, end: 1.0).animate(
      CurvedAnimation(
        parent: _animationController,
        curve: Curves.easeInOut,
      ),
    );
  }

  @override
  void dispose() {
    _animationController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Stack(
        children: [
          AutomatoBackground(
            ref: ref,
            showBackgroundSVG: true,
            showMenuLines: true,
            showRepeatingBorders: true,
            backgroundColor: Colors.black,
            gradientColor: AutomatoThemeColors.darkBrown(ref),
            backgroundSvgConfig: const BackgroundSvgConfig(
              animateInner: true,
              animateOuter: true,
              showDual: true,
            ),
            linesConfig: LinesConfig(
              lineColor: AutomatoThemeColors.primaryColor(ref).withValues(alpha: 0.12),
              strokeWidth: 1.0,
              spacing: 10.0,
              drawVerticalLines: true,
              drawHorizontalLines: true,
              enableFlicker: true,
              flickerDuration: const Duration(milliseconds: 4000),
            ),
          ),
          SafeArea(
            child: LayoutBuilder(
              builder: (context, constraints) {
                return SingleChildScrollView(
                  child: ConstrainedBox(
                    constraints: BoxConstraints(
                      minHeight: constraints.maxHeight,
                    ),
                    child: IntrinsicHeight(
                      child: Column(
                        children: [
                          Flexible(
                            flex: 3,
                            child: _buildHeader(ref),
                          ),
                          Flexible(
                            flex: 2,
                            child: _buildContent(ref),
                          ),
                        ],
                      ),
                    ),
                  ),
                );
              },
            ),
          ),
          // Config button OUTSIDE and ON TOP of everything
          SafeArea(
            child: Align(
              alignment: Alignment.topRight,
              child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: IconButton(
                  icon: Icon(
                    Icons.settings,
                    color: AutomatoThemeColors.primaryColor(ref),
                    size: 28,
                  ),
                  onPressed: () {
                    showDialog(
                      context: context,
                      builder: (context) => const ConfigDialog(),
                    );
                  },
                  tooltip: 'Configuration',
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildHeader(WidgetRef ref) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 32.0, vertical: 20.0),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        mainAxisSize: MainAxisSize.min,
        children: [
          AnimatedBuilder(
            animation: _animationController,
            builder: (context, child) {
              return Opacity(
                opacity: 0.92 + (0.08 * _pulseAnimation.value),
                child: ClipRRect(
                  borderRadius: BorderRadius.circular(20.0),
                  child: Image.asset(
                    'assets/images/yp_logo.jpg',
                    height: 180,
                    width: 180,
                    fit: BoxFit.contain,
                  ),
                ),
              );
            },
          ),
          const SizedBox(height: 24),
          Text(
            'YorHa Protocol',
            style: TextStyle(
              fontSize: 40,
              fontWeight: FontWeight.bold,
              color: AutomatoThemeColors.bright(ref),
              letterSpacing: 4,
              shadows: [
                Shadow(
                  color: AutomatoThemeColors.primaryColor(ref),
                  blurRadius: 15,
                ),
              ],
            ),
          ),
          const SizedBox(height: 8),
          Text(
            'LAUNCHER',
            style: TextStyle(
              fontSize: 18,
              fontWeight: FontWeight.w300,
              color: AutomatoThemeColors.primaryColor(ref),
              letterSpacing: 8,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildContent(WidgetRef ref) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 48.0, vertical: 16.0),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        mainAxisSize: MainAxisSize.min,
        children: [
          const DirectorySelector(),
          const SizedBox(height: 24),
          const PlayButton(),
          const SizedBox(height: 12),
          Text(
            'Glory to mankind.',
            style: TextStyle(
              fontSize: 14,
              color: AutomatoThemeColors.primaryColor(ref).withValues(alpha: 0.6),
              fontStyle: FontStyle.italic,
            ),
          ),
        ],
      ),
    );
  }
}
