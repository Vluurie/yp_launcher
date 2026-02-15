import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:automato_theme/automato_theme.dart';
import 'package:path/path.dart' as path;
import 'package:shared_preferences/shared_preferences.dart';
import 'package:window_manager/window_manager.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/screens/launcher_screen.dart';
import 'package:yp_launcher/services/launcher_setup_service.dart';

void main(List<String> args) async {
  WidgetsFlutterBinding.ensureInitialized();

  if (args.contains('--launch')) {
    await _launchDirectAndExit();
    return;
  }

  if (Platform.isWindows) {
    await windowManager.ensureInitialized();

    const windowOptions = WindowOptions(
      size: Size(1200, 700),
      minimumSize: Size(550, 380),
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

    final isSetup = await LauncherSetupService.isLauncherSetup();
    if (!isSetup) {
      await LauncherSetupService.setupLauncher();
    }

    final paths = await LauncherSetupService.getLauncherPaths();
    final nierExe = path.join(gameDir, AppStrings.gameExeName);
    if (!await File(nierExe).exists()) {
      exit(1);
    }

    final modloaderDll = paths['modloaderDll']!.replaceAll('/', '\\');
    final yorhaDll = paths['yorhaDll']!.replaceAll('/', '\\');

    await Process.start(
      paths['launcherExe']!,
      [
        AppStrings.argModloaderDll,
        modloaderDll,
        AppStrings.argModDll,
        yorhaDll,
      ],
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
    return MaterialApp(
      title: AppStrings.appTitle,
      debugShowCheckedModeBanner: false,
      theme: themeState.theme,
      home: const LauncherScreen(),
    );
  }
}
