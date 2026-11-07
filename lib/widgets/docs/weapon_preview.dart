import 'dart:async';
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/services/three_d_inspector_service.dart';
import 'package:yp_launcher/src/rust/api/inspector.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class WeaponPreview extends StatefulWidget {
  final String datPath;
  final String dttPath;

  const WeaponPreview({
    super.key,
    required this.datPath,
    required this.dttPath,
  });

  @override
  State<WeaponPreview> createState() => _WeaponPreviewState();
}

class _WeaponPreviewState extends State<WeaponPreview> {
  ModelSession? _session;
  ui.Image? _image;
  Object? _error;
  bool _opening = true;
  bool _rendering = false;
  bool _renderQueued = false;
  int _renderWidth = 0;
  int _renderHeight = 0;
  double _yaw = math.pi;
  double _pitch = -0.18;
  double _distance = 2.6;

  @override
  void initState() {
    super.initState();
    unawaited(_open());
  }

  @override
  void dispose() {
    final session = _session;
    if (session != null) ThreeDInspectorService.close(session.id);
    _image?.dispose();
    super.dispose();
  }

  Future<void> _open() async {
    final path = widget.datPath;
    final pair = DataArchivePair(
      stem: p.basenameWithoutExtension(path),
      label: p.basename(path),
      configRoot: '',
      datPath: widget.datPath,
      dttPath: widget.dttPath,
    );

    try {
      final probe = await ThreeDInspectorService.probe(pair);
      if (!mounted || widget.datPath != path) return;
      if (probe.candidates.isEmpty) {
        setState(() {
          _opening = false;
          _error = probe.warnings.isEmpty ? '' : probe.warnings.join('\n');
        });
        return;
      }

      final session = await ThreeDInspectorService.open(
        pair,
        probe.candidates.first,
      );
      if (!mounted || widget.datPath != path) {
        ThreeDInspectorService.close(session.id);
        return;
      }

      setState(() {
        _session = session;
        _opening = false;
      });
      _requestRender();
    } catch (error) {
      if (!mounted || widget.datPath != path) return;
      setState(() {
        _opening = false;
        _error = error;
      });
    }
  }

  void _setViewport(BoxConstraints constraints) {
    final logicalWidth = constraints.maxWidth.clamp(16.0, 10000.0).toDouble();
    final logicalHeight = constraints.maxHeight.clamp(16.0, 10000.0).toDouble();
    final scale = math.min(2.0, 960.0 / logicalWidth);
    final width = math.max(16, (logicalWidth * scale).round());
    final height = math.max(16, (logicalHeight * scale).round());
    if (width == _renderWidth && height == _renderHeight) return;
    _renderWidth = width;
    _renderHeight = height;
    WidgetsBinding.instance.addPostFrameCallback((_) => _requestRender());
  }

  void _requestRender() {
    if (!mounted || _session == null || _renderWidth < 16) return;
    if (_rendering) {
      _renderQueued = true;
      return;
    }
    unawaited(_render());
  }

  Future<void> _render() async {
    final session = _session;
    if (session == null) return;
    _rendering = true;
    try {
      final frame = await ThreeDInspectorService.render(
        sessionId: session.id,
        width: _renderWidth,
        height: _renderHeight,
        yaw: _yaw,
        pitch: _pitch,
        distance: _distance,
        panX: 0,
        panY: 0,
      );
      final image = await _decode(frame);
      if (!mounted || _session?.id != session.id) {
        image.dispose();
        return;
      }
      final previous = _image;
      setState(() {
        _image = image;
        _error = null;
      });
      previous?.dispose();
    } catch (error) {
      if (mounted && _session?.id == session.id) {
        setState(() => _error = error);
      }
    } finally {
      _rendering = false;
      if (_renderQueued) {
        _renderQueued = false;
        _requestRender();
      }
    }
  }

  Future<ui.Image> _decode(RenderFrame frame) {
    final completer = Completer<ui.Image>();
    ui.decodeImageFromPixels(
      frame.rgba,
      frame.width,
      frame.height,
      ui.PixelFormat.rgba8888,
      completer.complete,
      rowBytes: frame.width * 4,
    );
    return completer.future;
  }

  void _move(PointerMoveEvent event) {
    if ((event.buttons & kPrimaryMouseButton) == 0) return;
    _yaw += event.delta.dx * 0.01;
    _pitch = (_pitch + event.delta.dy * 0.01).clamp(-1.45, 1.45).toDouble();
    _requestRender();
  }

  void _scroll(PointerSignalEvent event) {
    if (event is! PointerScrollEvent) return;
    GestureBinding.instance.pointerSignalResolver.register(event, (resolved) {
      final scrollEvent = resolved as PointerScrollEvent;
      _distance = (_distance * math.exp(scrollEvent.scrollDelta.dy * 0.001))
          .clamp(0.9, 12.0)
          .toDouble();
      _requestRender();
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    return AspectRatio(
      aspectRatio: 16 / 9,
      child: ClipRRect(
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        child: LayoutBuilder(
          builder: (context, constraints) {
            _setViewport(constraints);
            return Listener(
              behavior: HitTestBehavior.opaque,
              onPointerMove: _move,
              onPointerSignal: _scroll,
              child: ColoredBox(
                color: AppColors.backgroundPrimary,
                child: Stack(
                  fit: StackFit.expand,
                  children: [
                    if (_image != null)
                      RawImage(
                        image: _image,
                        fit: BoxFit.contain,
                        filterQuality: FilterQuality.high,
                      ),
                    if (_opening)
                      Center(
                        child: CircularProgressIndicator(
                          color: AppColors.accentPrimary,
                        ),
                      ),
                    if (_error != null && _image == null)
                      Center(
                        child: Padding(
                          padding: EdgeInsets.all(AppSizes.paddingSM(context)),
                          child: Text(
                            l10n.weaponPreviewFailed,
                            textAlign: TextAlign.center,
                            style: TextStyle(
                              color: AppColors.textMuted,
                              fontSize: AppSizes.fontXS(context),
                              height: 1.4,
                            ),
                          ),
                        ),
                      ),
                  ],
                ),
              ),
            );
          },
        ),
      ),
    );
  }
}
