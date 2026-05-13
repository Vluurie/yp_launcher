import 'dart:async';
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:automato_theme/automato_theme.dart';
import 'package:path/path.dart' as path;
import 'package:shared_preferences/shared_preferences.dart';
import 'package:window_manager/window_manager.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/screens/launcher_screen.dart';
import 'package:yp_launcher/services/launcher_setup_service.dart';
import 'package:yp_launcher/services/platform_gate.dart';
import 'package:yp_launcher/services/plugins_service.dart';
import 'package:yp_launcher/services/steam_service.dart';
import 'package:google_fonts/google_fonts.dart';

void main(List<String> args) async {
  WidgetsFlutterBinding.ensureInitialized();

  // PlatformGate.overrideAs = SimulatedOs.linux;

  if (args.contains('--simulate-linux')) {
    PlatformGate.overrideAs = SimulatedOs.linux;
  } else if (args.contains('--simulate-macos')) {
    PlatformGate.overrideAs = SimulatedOs.macos;
  } else if (args.contains('--simulate-windows')) {
    PlatformGate.overrideAs = SimulatedOs.windows;
  }

  if (args.contains('--launch')) {
    await _launchDirectAndExit();
    return;
  }

  if (Platform.isWindows) {
    await windowManager.ensureInitialized();

    const windowOptions = WindowOptions(
      size: Size(1300, 750),
      minimumSize: Size(600, 420),
      center: true,
      backgroundColor: Colors.transparent,
      skipTaskbar: false,
      titleBarStyle: TitleBarStyle.hidden,
      title: AppStrings.appTitle,
    );

    await windowManager.waitUntilReadyToShow(windowOptions, () async {
      await windowManager.show();
      await windowManager.focus();
    });
  }

  unawaited(LauncherSetupService.ensureReady());

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

Future<void> _launchDirectAndExit() async {
  try {
    final prefs = await SharedPreferences.getInstance();
    final gameDir = prefs.getString(AppStrings.prefKeyDirectory);
    if (gameDir == null || gameDir.isEmpty) {
      exit(1);
    }

    await LauncherSetupService.ensureReady();

    final paths = await LauncherSetupService.getLauncherPaths();
    final nierExe = path.join(gameDir, AppStrings.gameExeName);
    if (!await File(nierExe).exists()) {
      exit(1);
    }

    // Best-effort: try to start Steam if it isn't running. Ignore failure —
    // NAMS will surface its own Steam error at launch if needed.
    await SteamService.ensureRunning();

    final modloaderDll = paths['modloaderDll']!.replaceAll('/', '\\');
    final yorhaDll = paths['yorhaDll']!.replaceAll('/', '\\');
    final pluginPaths = await PluginsService.enabledPaths();

    final args = <String>[
      AppStrings.argModloaderDll,
      modloaderDll,
      AppStrings.argModDll,
      yorhaDll,
    ];
    for (final extra in pluginPaths) {
      args.add(AppStrings.argModDll);
      args.add(extra.replaceAll('/', '\\'));
    }

    await Process.start(
      paths['launcherExe']!,
      args,
      workingDirectory: gameDir,
      mode: ProcessStartMode.detached,
    );
  } catch (_) {}

  exit(0);
}

class YoRHaProtocolLauncher extends ConsumerWidget {
  const YoRHaProtocolLauncher({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeState = ref.watch(automatoThemeNotifierProvider);
    final baseTheme = themeState.theme;
    final gameFont = GoogleFonts.rajdhaniTextTheme(baseTheme.textTheme);
    return MaterialApp(
      title: AppStrings.appTitle,
      debugShowCheckedModeBanner: false,
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      theme: baseTheme.copyWith(
        textTheme: gameFont,
        tooltipTheme: TooltipThemeData(
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
          margin: const EdgeInsets.all(4),
          constraints: const BoxConstraints(maxWidth: 320),
          decoration: BoxDecoration(
            color: const Color(0xFF222228).withValues(alpha: 0.96),
            borderRadius: BorderRadius.circular(6),
            border: Border.all(
              color: const Color(0xFFD4A86A).withValues(alpha: 0.35),
              width: 1,
            ),
            boxShadow: const [
              BoxShadow(
                color: Color(0x66000000),
                blurRadius: 12,
                offset: Offset(0, 3),
              ),
            ],
          ),
          textStyle: gameFont.bodySmall?.copyWith(
            color: const Color(0xFFEDE6D8),
            fontSize: 12.5,
            height: 1.35,
            letterSpacing: 0.2,
          ),
          waitDuration: const Duration(milliseconds: 350),
        ),
      ),
      home: const LauncherScreen(),
    );
  }
}
