import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:yp_launcher/constants/naiom_keys.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class ConfigFieldKeyName extends StatefulWidget {
  final String label;
  final String bindingKey;
  final String value;
  final ValueChanged<String> onChanged;
  final String? tooltip;
  final String? conflictWith;

  const ConfigFieldKeyName({
    super.key,
    required this.label,
    required this.bindingKey,
    required this.value,
    required this.onChanged,
    this.tooltip,
    this.conflictWith,
  });

  @override
  State<ConfigFieldKeyName> createState() => _ConfigFieldKeyNameState();
}

class _ConfigFieldKeyNameState extends State<ConfigFieldKeyName> {
  bool _capturing = false;
  bool _hovered = false;
  final _focusNode = FocusNode();
  final List<({String generic, String? side})> _heldModifiers = [];

  static final _physicalToModifier =
      <PhysicalKeyboardKey, ({String generic, String? side})>{
        PhysicalKeyboardKey.shiftLeft: (generic: 'SHIFT', side: 'LSHIFT'),
        PhysicalKeyboardKey.shiftRight: (generic: 'SHIFT', side: 'RSHIFT'),
        PhysicalKeyboardKey.controlLeft: (generic: 'CTRL', side: 'LCTRL'),
        PhysicalKeyboardKey.controlRight: (generic: 'CTRL', side: 'RCTRL'),
        PhysicalKeyboardKey.altLeft: (generic: 'ALT', side: null),
        PhysicalKeyboardKey.altRight: (generic: 'ALT', side: null),
      };

  static final _physicalToName = <PhysicalKeyboardKey, String>{
    PhysicalKeyboardKey.keyA: 'A',
    PhysicalKeyboardKey.keyB: 'B',
    PhysicalKeyboardKey.keyC: 'C',
    PhysicalKeyboardKey.keyD: 'D',
    PhysicalKeyboardKey.keyE: 'E',
    PhysicalKeyboardKey.keyF: 'F',
    PhysicalKeyboardKey.keyG: 'G',
    PhysicalKeyboardKey.keyH: 'H',
    PhysicalKeyboardKey.keyI: 'I',
    PhysicalKeyboardKey.keyJ: 'J',
    PhysicalKeyboardKey.keyK: 'K',
    PhysicalKeyboardKey.keyL: 'L',
    PhysicalKeyboardKey.keyM: 'M',
    PhysicalKeyboardKey.keyN: 'N',
    PhysicalKeyboardKey.keyO: 'O',
    PhysicalKeyboardKey.keyP: 'P',
    PhysicalKeyboardKey.keyQ: 'Q',
    PhysicalKeyboardKey.keyR: 'R',
    PhysicalKeyboardKey.keyS: 'S',
    PhysicalKeyboardKey.keyT: 'T',
    PhysicalKeyboardKey.keyU: 'U',
    PhysicalKeyboardKey.keyV: 'V',
    PhysicalKeyboardKey.keyW: 'W',
    PhysicalKeyboardKey.keyX: 'X',
    PhysicalKeyboardKey.keyY: 'Y',
    PhysicalKeyboardKey.keyZ: 'Z',
    PhysicalKeyboardKey.digit0: '0',
    PhysicalKeyboardKey.digit1: '1',
    PhysicalKeyboardKey.digit2: '2',
    PhysicalKeyboardKey.digit3: '3',
    PhysicalKeyboardKey.digit4: '4',
    PhysicalKeyboardKey.digit5: '5',
    PhysicalKeyboardKey.digit6: '6',
    PhysicalKeyboardKey.digit7: '7',
    PhysicalKeyboardKey.digit8: '8',
    PhysicalKeyboardKey.digit9: '9',
    PhysicalKeyboardKey.space: 'SPACE',
    PhysicalKeyboardKey.end: 'END',
    PhysicalKeyboardKey.arrowUp: 'UPARROW',
    PhysicalKeyboardKey.arrowDown: 'DOWNARROW',
    PhysicalKeyboardKey.arrowLeft: 'LEFTARROW',
    PhysicalKeyboardKey.arrowRight: 'RIGHTARROW',
  };

  List<String> get _heldGenerics =>
      _heldModifiers.map((m) => m.generic).toList();

  @override
  void dispose() {
    _focusNode.dispose();
    super.dispose();
  }

  void _startCapture() {
    setState(() {
      _capturing = true;
      _heldModifiers.clear();
    });
    _focusNode.requestFocus();
  }

  void _finish(String binding) {
    widget.onChanged(binding);
    setState(() {
      _capturing = false;
      _heldModifiers.clear();
    });
  }

  void _cancelCapture() {
    if (!_capturing) return;
    setState(() {
      _capturing = false;
      _heldModifiers.clear();
    });
  }

  void _onKeyEvent(KeyEvent event) {
    if (!_capturing) return;

    if (event is KeyDownEvent) {
      if (event.logicalKey == LogicalKeyboardKey.escape) {
        _finish('');
        return;
      }
      final modifier = _physicalToModifier[event.physicalKey];
      if (modifier != null) {
        if (!_heldGenerics.contains(modifier.generic)) {
          setState(() => _heldModifiers.add(modifier));
        }
        return;
      }
      final name = _physicalToName[event.physicalKey];
      if (name == null) return;
      _finish([..._heldGenerics, name].join('+'));
      return;
    }

    if (event is KeyUpEvent) {
      final modifier = _physicalToModifier[event.physicalKey];
      if (modifier == null) return;
      final index = _heldModifiers.indexWhere(
        (m) => m.generic == modifier.generic,
      );
      if (index == -1) return;
      if (_heldModifiers.length == 1) {
        final side = _heldModifiers.first.side;
        if (side != null) {
          _finish(side);
        } else {
          setState(_heldModifiers.clear);
        }
        return;
      }
      setState(() => _heldModifiers.removeAt(index));
    }
  }

  void _onPointerDown(PointerDownEvent event) {
    if (!_capturing) return;
    if (!NaiomKeys.allowsMouse(widget.bindingKey)) return;
    final name = switch (event.buttons) {
      kPrimaryMouseButton => 'MOUSE1',
      kSecondaryMouseButton => 'MOUSE2',
      kMiddleMouseButton => 'MOUSE3',
      kBackMouseButton => 'MOUSE4',
      kForwardMouseButton => 'MOUSE5',
      _ => null,
    };
    if (name == null) return;
    _finish([..._heldGenerics, name].join('+'));
  }

  String _captureText(AppLocalizations l10n) {
    if (_heldModifiers.isEmpty) return l10n.pressKey;
    return '${_heldGenerics.join('+')}+…';
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final unbound = widget.value.isEmpty;
    final hasConflict = widget.conflictWith != null;
    final displayText = _capturing
        ? _captureText(l10n)
        : unbound
        ? l10n.keybindUnbound
        : widget.value;

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
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
                child: Focus(
                  focusNode: _focusNode,
                  onFocusChange: (focused) {
                    if (!focused) _cancelCapture();
                  },
                  onKeyEvent: (node, event) {
                    if (!_capturing) return KeyEventResult.ignored;
                    _onKeyEvent(event);
                    return KeyEventResult.handled;
                  },
                  child: Listener(
                    onPointerDown: _capturing ? _onPointerDown : null,
                    child: GestureDetector(
                      onTap: _capturing ? null : _startCapture,
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
                                : hasConflict
                                ? AppColors.warning
                                : _hovered
                                ? AppColors.accentPrimary.withValues(alpha: 0.6)
                                : AppColors.borderMedium,
                            width: _capturing
                                ? 2.0
                                : hasConflict || _hovered
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
                                : unbound
                                ? AppColors.textMuted
                                : AppColors.textPrimary,
                            fontWeight: FontWeight.bold,
                          ),
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            ],
          ),
          if (_capturing && !NaiomKeys.allowsMouse(widget.bindingKey))
            Padding(
              padding: const EdgeInsets.only(top: 2),
              child: Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  Icon(
                    Icons.keyboard_outlined,
                    size: 13,
                    color: AppColors.textMuted,
                  ),
                  const SizedBox(width: 4),
                  Text(
                    l10n.keybindMouseNotSupported,
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      color: AppColors.textMuted,
                    ),
                  ),
                ],
              ),
            ),
          if (hasConflict)
            Padding(
              padding: const EdgeInsets.only(top: 2),
              child: Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  Icon(
                    Icons.warning_amber_rounded,
                    size: 13,
                    color: AppColors.warning,
                  ),
                  const SizedBox(width: 4),
                  Text(
                    l10n.keybindConflict(widget.conflictWith!),
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      color: AppColors.warning,
                    ),
                  ),
                ],
              ),
            ),
        ],
      ),
    );
  }
}
