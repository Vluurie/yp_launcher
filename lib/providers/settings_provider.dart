import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:yp_launcher/services/settings_service.dart';

final settingsServiceProvider = FutureProvider<SettingsService>((ref) async {
  return await SettingsService.initialize();
});
