import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class ConfigFieldKeybind extends StatefulWidget {
  final String label;
  final String value;
  final ValueChanged<String> onChanged;
  final String? tooltip;

  const ConfigFieldKeybind({
    super.key,
    required this.label,
    required this.value,
    required this.onChanged,
    this.tooltip,
  });

  @override
  State<ConfigFieldKeybind> createState() => _ConfigFieldKeybindState();
}

class _ConfigFieldKeybindState extends State<ConfigFieldKeybind> {
  bool _capturing = false;
  bool _hovered = false;
  final _focusNode = FocusNode();

  @override
  void dispose() {
    _focusNode.dispose();
    super.dispose();
  }

  String _logicalKeyToName(LogicalKeyboardKey key) {
    final label = key.keyLabel;
    if (label.isNotEmpty) return label;

    final knownKeys = <int, String>{
      LogicalKeyboardKey.f1.keyId: 'F1',
      LogicalKeyboardKey.f2.keyId: 'F2',
      LogicalKeyboardKey.f3.keyId: 'F3',
      LogicalKeyboardKey.f4.keyId: 'F4',
      LogicalKeyboardKey.f5.keyId: 'F5',
      LogicalKeyboardKey.f6.keyId: 'F6',
      LogicalKeyboardKey.f7.keyId: 'F7',
      LogicalKeyboardKey.f8.keyId: 'F8',
      LogicalKeyboardKey.f9.keyId: 'F9',
      LogicalKeyboardKey.f10.keyId: 'F10',
      LogicalKeyboardKey.f11.keyId: 'F11',
      LogicalKeyboardKey.f12.keyId: 'F12',
      LogicalKeyboardKey.insert.keyId: 'Insert',
      LogicalKeyboardKey.delete.keyId: 'Delete',
      LogicalKeyboardKey.home.keyId: 'Home',
      LogicalKeyboardKey.end.keyId: 'End',
      LogicalKeyboardKey.pageUp.keyId: 'PageUp',
      LogicalKeyboardKey.pageDown.keyId: 'PageDown',
      LogicalKeyboardKey.escape.keyId: 'Escape',
      LogicalKeyboardKey.space.keyId: 'Space',
      LogicalKeyboardKey.tab.keyId: 'Tab',
      LogicalKeyboardKey.capsLock.keyId: 'CapsLock',
      LogicalKeyboardKey.numLock.keyId: 'NumLock',
      LogicalKeyboardKey.scrollLock.keyId: 'ScrollLock',
      LogicalKeyboardKey.pause.keyId: 'Pause',
      LogicalKeyboardKey.printScreen.keyId: 'PrintScreen',
    };

    return knownKeys[key.keyId] ?? 'Key${key.keyId}';
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(
                  widget.label,
                  style: TextStyle(
                    fontSize: AppSizes.fontSM(context),
                    color: AppColors.textPrimary,
                  ),
                ),
                if (widget.tooltip != null)
                  Text(
                    widget.tooltip!,
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      color: AppColors.textMuted,
                    ),
                  ),
              ],
            ),
          ),
          const SizedBox(width: 8),
          MouseRegion(
            cursor: SystemMouseCursors.click,
            onEnter: (_) => setState(() => _hovered = true),
            onExit: (_) => setState(() => _hovered = false),
            child: KeyboardListener(
              focusNode: _focusNode,
              autofocus: false,
              onKeyEvent: _capturing
                  ? (event) {
                      if (event is KeyDownEvent) {
                        final name = _logicalKeyToName(event.logicalKey);
                        widget.onChanged(name);
                        setState(() => _capturing = false);
                      }
                    }
                  : null,
              child: GestureDetector(
                onTap: () {
                  setState(() => _capturing = true);
                  _focusNode.requestFocus();
                },
                child: AnimatedContainer(
                  duration: const Duration(milliseconds: 150),
                  constraints: const BoxConstraints(minWidth: 80),
                  padding: const EdgeInsets.symmetric(
                    horizontal: 10,
                    vertical: 5,
                  ),
                  decoration: BoxDecoration(
                    color: _capturing
                        ? AppColors.accentPrimary.withValues(alpha: 0.2)
                        : _hovered
                        ? AppColors.accentPrimary.withValues(alpha: 0.08)
                        : AppColors.inputBackground,
                    borderRadius: BorderRadius.circular(4),
                    border: Border.all(
                      color: _capturing
                          ? AppColors.accentPrimary
                          : _hovered
                          ? AppColors.accentPrimary.withValues(alpha: 0.6)
                          : AppColors.borderMedium,
                      width: _capturing ? 2.0 : 1.0,
                    ),
                  ),
                  child: Text(
                    _capturing
                        ? AppLocalizations.of(context)!.pressKey
                        : widget.value,
                    textAlign: TextAlign.center,
                    style: TextStyle(
                      fontSize: AppSizes.fontSM(context),
                      color: _capturing
                          ? AppColors.accentPrimary
                          : AppColors.textPrimary,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
