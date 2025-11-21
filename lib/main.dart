import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:automato_theme/automato_theme.dart';
import 'package:yp_launcher/screens/launcher_screen.dart';
import 'package:yp_launcher/services/logging_service.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // Initialize logging service
  await LoggingService.initialize();

  final themeNotifier = await AutomatoThemeNotifier.loadFromPreferences();
  runApp(
    ProviderScope(
      overrides: [
        automatoThemeNotifierProvider.overrideWith((ref) => themeNotifier),
      ],
      child: const YorHaProtocolLauncher(),
    ),
  );
}

class YorHaProtocolLauncher extends ConsumerWidget {
  const YorHaProtocolLauncher({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeState = ref.watch(automatoThemeNotifierProvider);
    return MaterialApp(
      title: 'YorHa Protocol Launcher',
      debugShowCheckedModeBanner: false,
      theme: themeState.theme,
      home: const LauncherScreen(),
    );
  }
}
