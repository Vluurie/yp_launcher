import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:automato_theme/automato_theme.dart';
import 'package:window_manager/window_manager.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/screens/launcher_screen.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  if (Platform.isWindows) {
    await windowManager.ensureInitialized();

    const windowOptions = WindowOptions(
      size: Size(700, 450),
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
