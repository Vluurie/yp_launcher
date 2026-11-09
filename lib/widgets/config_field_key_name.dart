import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:yp_launcher/constants/naiom_keys.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/app_dropdown.dart';

String controllerButtonLabel(AppLocalizations l10n, String name) =>
    switch (name) {
      'CONTROLLER_A' => l10n.controllerA,
      'CONTROLLER_B' => l10n.controllerB,
      'CONTROLLER_X' => l10n.controllerX,
      'CONTROLLER_Y' => l10n.controllerY,
      'CONTROLLER_LB' => l10n.controllerLB,
      'CONTROLLER_RB' => l10n.controllerRB,
      'CONTROLLER_LT' => l10n.controllerLT,
      'CONTROLLER_RT' => l10n.controllerRT,
      'CONTROLLER_LTHUMB' => l10n.controllerLThumb,
      'CONTROLLER_RTHUMB' => l10n.controllerRThumb,
      'CONTROLLER_DUP' => l10n.controllerDUp,
      'CONTROLLER_DDOWN' => l10n.controllerDDown,
      'CONTROLLER_DLEFT' => l10n.controllerDLeft,
      'CONTROLLER_DRIGHT' => l10n.controllerDRight,
      'CONTROLLER_BACK' => l10n.controllerBack,
      'CONTROLLER_START' => l10n.controllerStart,
      _ => name,
    };

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
        PhysicalKeyboardKey.altLeft: (generic: 'ALT', side: 'LALT'),
        PhysicalKeyboardKey.altRight: (generic: 'ALT', side: 'RALT'),
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
    PhysicalKeyboardKey.tab: 'TAB',
    PhysicalKeyboardKey.enter: 'ENTER',
    PhysicalKeyboardKey.backspace: 'BACKSPACE',
    PhysicalKeyboardKey.insert: 'INS',
    PhysicalKeyboardKey.delete: 'DEL',
    PhysicalKeyboardKey.home: 'HOME',
    PhysicalKeyboardKey.pageUp: 'PGUP',
    PhysicalKeyboardKey.pageDown: 'PGDN',
    PhysicalKeyboardKey.f1: 'F1',
    PhysicalKeyboardKey.f2: 'F2',
    PhysicalKeyboardKey.f3: 'F3',
    PhysicalKeyboardKey.f4: 'F4',
    PhysicalKeyboardKey.f5: 'F5',
    PhysicalKeyboardKey.f6: 'F6',
    PhysicalKeyboardKey.f7: 'F7',
    PhysicalKeyboardKey.f8: 'F8',
    PhysicalKeyboardKey.f9: 'F9',
    PhysicalKeyboardKey.f10: 'F10',
    PhysicalKeyboardKey.f11: 'F11',
    PhysicalKeyboardKey.f12: 'F12',
    PhysicalKeyboardKey.numpad0: 'KP_0',
    PhysicalKeyboardKey.numpad1: 'KP_1',
    PhysicalKeyboardKey.numpad2: 'KP_2',
    PhysicalKeyboardKey.numpad3: 'KP_3',
    PhysicalKeyboardKey.numpad4: 'KP_4',
    PhysicalKeyboardKey.numpad5: 'KP_5',
    PhysicalKeyboardKey.numpad6: 'KP_6',
    PhysicalKeyboardKey.numpad7: 'KP_7',
    PhysicalKeyboardKey.numpad8: 'KP_8',
    PhysicalKeyboardKey.numpad9: 'KP_9',
    PhysicalKeyboardKey.numpadAdd: 'KP_PLUS',
    PhysicalKeyboardKey.numpadSubtract: 'KP_MINUS',
    PhysicalKeyboardKey.numpadDivide: 'KP_SLASH',
    PhysicalKeyboardKey.numpadMultiply: 'KP_STAR',
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

  void _onPointerSignal(PointerSignalEvent event) {
    if (!_capturing || event is! PointerScrollEvent) return;
    if (!NaiomKeys.allowsMouse(widget.bindingKey)) return;
    GestureBinding.instance.pointerSignalResolver.register(event, (resolved) {
      if (!_capturing || resolved is! PointerScrollEvent) return;
      final dy = resolved.scrollDelta.dy;
      if (dy == 0) return;
      final name = dy < 0 ? 'MOUSE_WHEEL_UP' : 'MOUSE_WHEEL_DOWN';
      _finish([..._heldGenerics, name].join('+'));
    });
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
    final parts = NaiomKeys.comboParts(widget.value);
    final isController =
        !_capturing && parts.any(NaiomKeys.controllerButtons.contains);
    final displayText = _capturing
        ? _captureText(l10n)
        : unbound
        ? l10n.keybindUnbound
        : isController
        ? parts.map((part) => controllerButtonLabel(l10n, part)).join('+')
        : widget.value;
    final keyboardOnlyHint =
        _capturing && !NaiomKeys.allowsMouse(widget.bindingKey);
    final textColor = _capturing
        ? AppColors.accentPrimary
        : unbound
        ? AppColors.textMuted
        : AppColors.textPrimary;

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
                    if (keyboardOnlyHint)
                      Row(
                        children: [
                          Icon(
                            Icons.keyboard_outlined,
                            size: 13,
                            color: AppColors.accentPrimary,
                          ),
                          const SizedBox(width: 4),
                          Flexible(
                            child: Text(
                              l10n.keybindMouseNotSupported,
                              style: TextStyle(
                                fontSize: AppSizes.fontXS(context),
                                color: AppColors.accentPrimary,
                              ),
                            ),
                          ),
                        ],
                      )
                    else if (widget.tooltip != null)
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
                    onPointerSignal: _capturing ? _onPointerSignal : null,
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
                        child: Row(
                          mainAxisSize: MainAxisSize.min,
                          mainAxisAlignment: MainAxisAlignment.center,
                          children: [
                            if (isController) ...[
                              Icon(
                                Icons.sports_esports_outlined,
                                size: 14,
                                color: AppColors.accentPrimary,
                              ),
                              const SizedBox(width: 6),
                            ],
                            Text(
                              displayText,
                              textAlign: TextAlign.center,
                              style: TextStyle(
                                fontSize: AppSizes.fontSM(context),
                                color: textColor,
                                fontWeight: FontWeight.bold,
                              ),
                            ),
                          ],
                        ),
                      ),
                    ),
                  ),
                ),
              ),
              if (NaiomKeys.allowsMouse(widget.bindingKey)) ...[
                const SizedBox(width: 4),
                AppDropdown<String>(
                  value: widget.value,
                  items: NaiomKeys.controllerButtons,
                  itemLabel: (name) => controllerButtonLabel(l10n, name),
                  onChanged: widget.onChanged,
                  triggerIcon: Icons.sports_esports_outlined,
                  tooltip: l10n.keybindPickController,
                ),
              ],
            ],
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
                  Flexible(
                    child: Text(
                      l10n.keybindConflict(widget.conflictWith!),
                      textAlign: TextAlign.end,
                      style: TextStyle(
                        fontSize: AppSizes.fontXS(context),
                        color: AppColors.warning,
                      ),
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
