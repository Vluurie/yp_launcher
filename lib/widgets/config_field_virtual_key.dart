import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class ConfigFieldVirtualKey extends StatefulWidget {
  final String label;
  final int value;
  final ValueChanged<int> onChanged;
  final String? tooltip;

  const ConfigFieldVirtualKey({
    super.key,
    required this.label,
    required this.value,
    required this.onChanged,
    this.tooltip,
  });

  @override
  State<ConfigFieldVirtualKey> createState() => _ConfigFieldVirtualKeyState();
}

class _ConfigFieldVirtualKeyState extends State<ConfigFieldVirtualKey> {
  bool _capturing = false;
  bool _hovered = false;
  final _focusNode = FocusNode();

  static const _vkNames = <int, String>{
    0: 'Disabled',
    0x08: 'Backspace',
    0x09: 'Tab',
    0x0D: 'Enter',
    0x10: 'Shift',
    0x11: 'Ctrl',
    0x12: 'Alt',
    0x13: 'Pause',
    0x14: 'CapsLock',
    0x1B: 'Escape',
    0x20: 'Space',
    0x21: 'PageUp',
    0x22: 'PageDown',
    0x23: 'End',
    0x24: 'Home',
    0x25: 'Left',
    0x26: 'Up',
    0x27: 'Right',
    0x28: 'Down',
    0x2D: 'Insert',
    0x2E: 'Delete',
    0x30: '0',
    0x31: '1',
    0x32: '2',
    0x33: '3',
    0x34: '4',
    0x35: '5',
    0x36: '6',
    0x37: '7',
    0x38: '8',
    0x39: '9',
    0x41: 'A',
    0x42: 'B',
    0x43: 'C',
    0x44: 'D',
    0x45: 'E',
    0x46: 'F',
    0x47: 'G',
    0x48: 'H',
    0x49: 'I',
    0x4A: 'J',
    0x4B: 'K',
    0x4C: 'L',
    0x4D: 'M',
    0x4E: 'N',
    0x4F: 'O',
    0x50: 'P',
    0x51: 'Q',
    0x52: 'R',
    0x53: 'S',
    0x54: 'T',
    0x55: 'U',
    0x56: 'V',
    0x57: 'W',
    0x58: 'X',
    0x59: 'Y',
    0x5A: 'Z',
    0x60: 'Num0',
    0x61: 'Num1',
    0x62: 'Num2',
    0x63: 'Num3',
    0x64: 'Num4',
    0x65: 'Num5',
    0x66: 'Num6',
    0x67: 'Num7',
    0x68: 'Num8',
    0x69: 'Num9',
    0x6A: 'Num*',
    0x6B: 'Num+',
    0x6D: 'Num-',
    0x6E: 'Num.',
    0x6F: 'Num/',
    0x70: 'F1',
    0x71: 'F2',
    0x72: 'F3',
    0x73: 'F4',
    0x74: 'F5',
    0x75: 'F6',
    0x76: 'F7',
    0x77: 'F8',
    0x78: 'F9',
    0x79: 'F10',
    0x7A: 'F11',
    0x7B: 'F12',
    0x90: 'NumLock',
    0x91: 'ScrollLock',
    0xBA: ';',
    0xBB: '=',
    0xBC: ',',
    0xBD: '-',
    0xBE: '.',
    0xBF: '/',
    0xC0: '`',
    0xDB: '[',
    0xDC: '\\',
    0xDD: ']',
    0xDE: "'",
  };

  static int? _keyEventToVk(KeyEvent event) {
    final physical = event.physicalKey;

    final physicalVk = <PhysicalKeyboardKey, int>{
      PhysicalKeyboardKey.backspace: 0x08,
      PhysicalKeyboardKey.tab: 0x09,
      PhysicalKeyboardKey.enter: 0x0D,
      PhysicalKeyboardKey.shiftLeft: 0x10,
      PhysicalKeyboardKey.shiftRight: 0x10,
      PhysicalKeyboardKey.controlLeft: 0x11,
      PhysicalKeyboardKey.controlRight: 0x11,
      PhysicalKeyboardKey.altLeft: 0x12,
      PhysicalKeyboardKey.altRight: 0x12,
      PhysicalKeyboardKey.pause: 0x13,
      PhysicalKeyboardKey.capsLock: 0x14,
      PhysicalKeyboardKey.escape: 0x1B,
      PhysicalKeyboardKey.space: 0x20,
      PhysicalKeyboardKey.pageUp: 0x21,
      PhysicalKeyboardKey.pageDown: 0x22,
      PhysicalKeyboardKey.end: 0x23,
      PhysicalKeyboardKey.home: 0x24,
      PhysicalKeyboardKey.arrowLeft: 0x25,
      PhysicalKeyboardKey.arrowUp: 0x26,
      PhysicalKeyboardKey.arrowRight: 0x27,
      PhysicalKeyboardKey.arrowDown: 0x28,
      PhysicalKeyboardKey.insert: 0x2D,
      PhysicalKeyboardKey.delete: 0x2E,
      PhysicalKeyboardKey.digit0: 0x30,
      PhysicalKeyboardKey.digit1: 0x31,
      PhysicalKeyboardKey.digit2: 0x32,
      PhysicalKeyboardKey.digit3: 0x33,
      PhysicalKeyboardKey.digit4: 0x34,
      PhysicalKeyboardKey.digit5: 0x35,
      PhysicalKeyboardKey.digit6: 0x36,
      PhysicalKeyboardKey.digit7: 0x37,
      PhysicalKeyboardKey.digit8: 0x38,
      PhysicalKeyboardKey.digit9: 0x39,
      PhysicalKeyboardKey.keyA: 0x41,
      PhysicalKeyboardKey.keyB: 0x42,
      PhysicalKeyboardKey.keyC: 0x43,
      PhysicalKeyboardKey.keyD: 0x44,
      PhysicalKeyboardKey.keyE: 0x45,
      PhysicalKeyboardKey.keyF: 0x46,
      PhysicalKeyboardKey.keyG: 0x47,
      PhysicalKeyboardKey.keyH: 0x48,
      PhysicalKeyboardKey.keyI: 0x49,
      PhysicalKeyboardKey.keyJ: 0x4A,
      PhysicalKeyboardKey.keyK: 0x4B,
      PhysicalKeyboardKey.keyL: 0x4C,
      PhysicalKeyboardKey.keyM: 0x4D,
      PhysicalKeyboardKey.keyN: 0x4E,
      PhysicalKeyboardKey.keyO: 0x4F,
      PhysicalKeyboardKey.keyP: 0x50,
      PhysicalKeyboardKey.keyQ: 0x51,
      PhysicalKeyboardKey.keyR: 0x52,
      PhysicalKeyboardKey.keyS: 0x53,
      PhysicalKeyboardKey.keyT: 0x54,
      PhysicalKeyboardKey.keyU: 0x55,
      PhysicalKeyboardKey.keyV: 0x56,
      PhysicalKeyboardKey.keyW: 0x57,
      PhysicalKeyboardKey.keyX: 0x58,
      PhysicalKeyboardKey.keyY: 0x59,
      PhysicalKeyboardKey.keyZ: 0x5A,
      PhysicalKeyboardKey.numpad0: 0x60,
      PhysicalKeyboardKey.numpad1: 0x61,
      PhysicalKeyboardKey.numpad2: 0x62,
      PhysicalKeyboardKey.numpad3: 0x63,
      PhysicalKeyboardKey.numpad4: 0x64,
      PhysicalKeyboardKey.numpad5: 0x65,
      PhysicalKeyboardKey.numpad6: 0x66,
      PhysicalKeyboardKey.numpad7: 0x67,
      PhysicalKeyboardKey.numpad8: 0x68,
      PhysicalKeyboardKey.numpad9: 0x69,
      PhysicalKeyboardKey.numpadMultiply: 0x6A,
      PhysicalKeyboardKey.numpadAdd: 0x6B,
      PhysicalKeyboardKey.numpadSubtract: 0x6D,
      PhysicalKeyboardKey.numpadDecimal: 0x6E,
      PhysicalKeyboardKey.numpadDivide: 0x6F,
      PhysicalKeyboardKey.f1: 0x70,
      PhysicalKeyboardKey.f2: 0x71,
      PhysicalKeyboardKey.f3: 0x72,
      PhysicalKeyboardKey.f4: 0x73,
      PhysicalKeyboardKey.f5: 0x74,
      PhysicalKeyboardKey.f6: 0x75,
      PhysicalKeyboardKey.f7: 0x76,
      PhysicalKeyboardKey.f8: 0x77,
      PhysicalKeyboardKey.f9: 0x78,
      PhysicalKeyboardKey.f10: 0x79,
      PhysicalKeyboardKey.f11: 0x7A,
      PhysicalKeyboardKey.f12: 0x7B,
      PhysicalKeyboardKey.numLock: 0x90,
      PhysicalKeyboardKey.scrollLock: 0x91,
      PhysicalKeyboardKey.semicolon: 0xBA,
      PhysicalKeyboardKey.equal: 0xBB,
      PhysicalKeyboardKey.comma: 0xBC,
      PhysicalKeyboardKey.minus: 0xBD,
      PhysicalKeyboardKey.period: 0xBE,
      PhysicalKeyboardKey.slash: 0xBF,
      PhysicalKeyboardKey.backquote: 0xC0,
      PhysicalKeyboardKey.bracketLeft: 0xDB,
      PhysicalKeyboardKey.backslash: 0xDC,
      PhysicalKeyboardKey.bracketRight: 0xDD,
      PhysicalKeyboardKey.quote: 0xDE,
    };

    return physicalVk[physical];
  }

  String _displayName(int vk) {
    if (vk == 0) return 'Disabled';
    return _vkNames[vk] ?? '0x${vk.toRadixString(16).toUpperCase()}';
  }

  @override
  void dispose() {
    _focusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final displayText = _capturing
        ? AppLocalizations.of(context)!.pressKey
        : _displayName(widget.value);

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
                        if (event.logicalKey == LogicalKeyboardKey.escape) {
                          widget.onChanged(0);
                          setState(() => _capturing = false);
                          return;
                        }
                        final vk = _keyEventToVk(event);
                        if (vk != null) {
                          widget.onChanged(vk);
                          setState(() => _capturing = false);
                        }
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
                      width: _capturing
                          ? 2.0
                          : _hovered
                          ? 1.5
                          : 1.0,
                    ),
                  ),
                  child: Text(
                    displayText,
                    textAlign: TextAlign.center,
                    style: TextStyle(
                      fontSize: AppSizes.fontSM(context),
                      color: _capturing
                          ? AppColors.accentPrimary
                          : widget.value == 0
                          ? AppColors.textMuted
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
