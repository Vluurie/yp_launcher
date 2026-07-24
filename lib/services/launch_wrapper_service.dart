import 'dart:io';

import 'package:shared_preferences/shared_preferences.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/services/wine/launch_command.dart';

class LaunchWrapperService {
  LaunchWrapperService._();

  static Future<String> read() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      return prefs.getString(AppStrings.prefKeyLaunchWrapper)?.trim() ?? '';
    } catch (_) {
      return '';
    }
  }

  static Future<void> save(String wrapper) async {
    try {
      final prefs = await SharedPreferences.getInstance();
      await prefs.setString(AppStrings.prefKeyLaunchWrapper, wrapper.trim());
    } catch (_) {}
  }

  static List<String> tokenize(String input) {
    final tokens = <String>[];
    final buffer = StringBuffer();
    var inSingle = false;
    var inDouble = false;
    var hasToken = false;
    for (var i = 0; i < input.length; i++) {
      final ch = input[i];
      if (ch == "'" && !inDouble) {
        inSingle = !inSingle;
        hasToken = true;
      } else if (ch == '"' && !inSingle) {
        inDouble = !inDouble;
        hasToken = true;
      } else if (ch == ' ' && !inSingle && !inDouble) {
        if (hasToken) {
          tokens.add(buffer.toString());
          buffer.clear();
          hasToken = false;
        }
      } else {
        buffer.write(ch);
        hasToken = true;
      }
    }
    if (hasToken) tokens.add(buffer.toString());
    return tokens;
  }

  static LaunchCommand wrap(LaunchCommand base, String wrapper) {
    if (!Platform.isLinux) return base;
    return applyWrapper(base, wrapper);
  }

  static LaunchCommand applyWrapper(LaunchCommand base, String wrapper) {
    final trimmed = wrapper.trim();
    if (trimmed.isEmpty) return base;

    final tokens = tokenize(trimmed);
    if (tokens.isEmpty) return base;

    final withoutTrailingSeparator =
        tokens.last == '--' ? tokens.sublist(0, tokens.length - 1) : tokens;
    if (withoutTrailingSeparator.isEmpty) return base;

    final wrapperCmd = withoutTrailingSeparator.first;
    final wrapperArgs = withoutTrailingSeparator.sublist(1);

    return LaunchCommand(
      command: wrapperCmd,
      args: [...wrapperArgs, '--', base.command, ...base.args],
      cwd: base.cwd,
      env: base.env,
      label: base.label,
    );
  }
}
