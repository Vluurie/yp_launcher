import 'dart:async';

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';

class AutoScrollSelection extends StatefulWidget {
  final ScrollController controller;
  final Widget child;

  const AutoScrollSelection({
    super.key,
    required this.controller,
    required this.child,
  });

  @override
  State<AutoScrollSelection> createState() => _AutoScrollSelectionState();
}

class _AutoScrollSelectionState extends State<AutoScrollSelection> {
  static const _edge = 60.0;
  static const _maxStep = 24.0;

  Timer? _ticker;
  double _step = 0;
  bool _pressed = false;
  Offset? _origin;

  @override
  void dispose() {
    _ticker?.cancel();
    super.dispose();
  }

  void _onPointerDown(PointerDownEvent event) {
    if (event.buttons & kPrimaryButton == 0) return;
    _pressed = true;
    _origin = event.position;
  }

  void _onPointerUp(PointerEvent event) {
    _pressed = false;
    _origin = null;
    _stop();
  }

  void _onPointerMove(PointerMoveEvent event) {
    if (!_pressed) return;

    final origin = _origin;
    if (origin != null) {
      if ((event.position - origin).distance < kTouchSlop) return;
      _origin = null;
    }

    final box = context.findRenderObject() as RenderBox?;
    if (box == null) return;

    final local = box.globalToLocal(event.position);
    final height = box.size.height;

    if (local.dy < _edge) {
      _start(-_speedFor(_edge - local.dy));
    } else if (local.dy > height - _edge) {
      _start(_speedFor(local.dy - (height - _edge)));
    } else {
      _stop();
    }
  }

  double _speedFor(double overshoot) {
    final ratio = (overshoot / _edge).clamp(0.0, 1.0);
    return 4 + ratio * (_maxStep - 4);
  }

  void _start(double step) {
    _step = step;
    _ticker ??= Timer.periodic(
      const Duration(milliseconds: 16),
      (_) => _scroll(),
    );
  }

  void _stop() {
    _ticker?.cancel();
    _ticker = null;
    _step = 0;
  }

  void _scroll() {
    final controller = widget.controller;
    if (!controller.hasClients) return;

    final position = controller.position;
    final target = (position.pixels + _step).clamp(
      position.minScrollExtent,
      position.maxScrollExtent,
    );
    if (target == position.pixels) return;
    controller.jumpTo(target);
  }

  @override
  Widget build(BuildContext context) {
    return Listener(
      onPointerDown: _onPointerDown,
      onPointerMove: _onPointerMove,
      onPointerUp: _onPointerUp,
      onPointerCancel: _onPointerUp,
      child: widget.child,
    );
  }
}
