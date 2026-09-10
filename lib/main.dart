import 'dart:async';
import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:automato_theme/automato_theme.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:window_manager/window_manager.dart';
import 'package:yp_launcher/services/window_state_service.dart';
import 'package:yp_launcher/providers/app_theme_state.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/theme/app_theme.dart';
import 'package:yp_launcher/widgets/notification_banner.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/locale_state.dart';
import 'package:yp_launcher/screens/launcher_screen.dart';
import 'package:yp_launcher/services/launcher_setup_service.dart';
import 'package:yp_launcher/services/platform/platform_adapter.dart';
import 'package:yp_launcher/services/platform_gate.dart';

void main(List<String> args) async {
  WidgetsFlutterBinding.ensureInitialized();

  LicenseRegistry.addLicense(() async* {
    final license = await rootBundle.loadString('assets/fonts/OFL.txt');
    yield LicenseEntryWithLineBreaks(const ['Rajdhani'], license);
  });

  if (args.contains('--simulate-linux')) {
    PlatformGate.overrideAs = SimulatedOs.linux;
  } else if (args.contains('--simulate-macos')) {
    PlatformGate.overrideAs = SimulatedOs.macos;
  } else if (args.contains('--simulate-windows')) {
    PlatformGate.overrideAs = SimulatedOs.windows;
  }

  if (Platform.isWindows || Platform.isMacOS || Platform.isLinux) {
    await windowManager.ensureInitialized();

    final restoredSize = await WindowStateService.savedSize();

    final windowOptions = WindowOptions(
      size: restoredSize,
      minimumSize: WindowStateService.minimumSize,
      center: false,
      backgroundColor: PlatformAdapter.current.usesNativeTitleBar
          ? null
          : Colors.transparent,
      skipTaskbar: false,
      titleBarStyle: PlatformAdapter.current.usesNativeTitleBar
          ? TitleBarStyle.normal
          : TitleBarStyle.hidden,
      title: 'YoRHa Protocol Launcher',
    );

    await windowManager.waitUntilReadyToShow(windowOptions, () async {
      await WindowStateService.instance.restore();
      await windowManager.show();
      await windowManager.focus();
    });

    WindowStateService.instance.start();
  }

  unawaited(LauncherSetupService.ensureReady());

  await _loadAppTheme();

  final themeNotifier = await AutomatoThemeNotifier.loadFromPreferences();

  runApp(
    ProviderScope(
      overrides: [
        automatoThemeNotifierProvider.overrideWith((ref) => themeNotifier),
      ],
      child: const YoRHaProtocolLauncher(),
    ),
  );
}

Future<void> _loadAppTheme() async {
  try {
    final prefs = await SharedPreferences.getInstance();
    final saved = prefs.getString('app_theme');
    AppColors.active = AppTheme.byId(
      saved == AppThemeId.nier.name ? AppThemeId.nier : AppThemeId.dark,
    );
  } catch (_) {}
}

class YoRHaProtocolLauncher extends ConsumerWidget {
  const YoRHaProtocolLauncher({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeId = ref.watch(appThemeControllerProvider);
    final appTheme = AppColors.active;
    final themeState = ref.watch(automatoThemeNotifierProvider);
    final baseTheme = themeState.theme;
    final gameFont = baseTheme.textTheme.apply(fontFamily: 'Rajdhani');
    return MaterialApp(
      onGenerateTitle: (context) => AppLocalizations.of(context)!.appTitle,
      debugShowCheckedModeBanner: false,
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      locale: ref.watch(localeControllerProvider),
      theme: baseTheme.copyWith(
        brightness: appTheme.brightness,
        scaffoldBackgroundColor: AppColors.backgroundPrimary,
        canvasColor: AppColors.backgroundPrimary,
        colorScheme: ColorScheme(
          brightness: appTheme.brightness,
          primary: AppColors.accentPrimary,
          onPrimary: AppColors.buttonText,
          secondary: AppColors.accentSecondary,
          onSecondary: AppColors.buttonText,
          surface: AppColors.backgroundCard,
          onSurface: AppColors.textPrimary,
          error: AppColors.error,
          onError: AppColors.buttonText,
        ),
        dialogTheme: DialogThemeData(backgroundColor: AppColors.backgroundCard),
        snackBarTheme: SnackBarThemeData(
          backgroundColor: AppColors.backgroundCard,
          contentTextStyle: gameFont.bodyMedium?.copyWith(
            color: AppColors.textPrimary,
          ),
        ),
        checkboxTheme: CheckboxThemeData(
          fillColor: WidgetStateProperty.resolveWith(
            (states) => states.contains(WidgetState.selected)
                ? AppColors.accentPrimary
                : Colors.transparent,
          ),
          checkColor: WidgetStateProperty.all(AppColors.buttonText),
          side: BorderSide(color: AppColors.borderMedium, width: 1.2),
        ),
        textTheme: gameFont.apply(
          bodyColor: AppColors.textPrimary,
          displayColor: AppColors.textPrimary,
        ),
        tooltipTheme: TooltipThemeData(
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
          margin: const EdgeInsets.all(4),
          constraints: const BoxConstraints(maxWidth: 320),
          decoration: BoxDecoration(
            color: AppColors.backgroundCard.withValues(alpha: 0.96),
            borderRadius: BorderRadius.circular(6),
            border: Border.all(
              color: AppColors.accentPrimary.withValues(alpha: 0.35),
              width: 1,
            ),
            boxShadow: [
              BoxShadow(
                color: AppColors.shadow,
                blurRadius: 12,
                offset: const Offset(0, 3),
              ),
            ],
          ),
          textStyle: gameFont.bodySmall?.copyWith(
            color: AppColors.textSecondary,
            fontSize: 12.5,
            height: 1.35,
            letterSpacing: 0.2,
          ),
          waitDuration: const Duration(milliseconds: 350),
        ),
      ),
      builder: (context, child) {
        return Stack(
          children: [
            if (child != null) child,
            Positioned(
              bottom: AppSizes.paddingLG(context),
              right: AppSizes.paddingLG(context),
              child: Material(
                type: MaterialType.transparency,
                child: const NotificationBanners(),
              ),
            ),
          ],
        );
      },
      home: LauncherScreen(key: ValueKey(themeId)),
    );
  }
}