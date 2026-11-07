import 'dart:async';
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/services/docs/outfit_builder_service.dart';
import 'package:yp_launcher/services/three_d_inspector_service.dart';
import 'package:yp_launcher/src/rust/api/inspector.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/docs/outfit_mesh_row.dart';

class OutfitMeshTuner extends StatefulWidget {
  final String modelPath;
  final OutfitCharacter character;
  final Map<String, MeshRule> rules;
  final ValueChanged<Map<String, MeshRule>> onChanged;

  const OutfitMeshTuner({
    super.key,
    required this.modelPath,
    required this.character,
    required this.rules,
    required this.onChanged,
  });

  @override
  State<OutfitMeshTuner> createState() => _OutfitMeshTunerState();
}

class _OutfitMeshTunerState extends State<OutfitMeshTuner> {
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

  final Set<String> _shown = {};
  List<String> _meshNames = const [];
  late Map<String, MeshRule> _rules;

  @override
  void initState() {
    super.initState();
    _rules = {...widget.rules};
    unawaited(_open());
  }

  @override
  void didUpdateWidget(covariant OutfitMeshTuner oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.modelPath != widget.modelPath) {
      _rules = {};
      unawaited(_open());
    }
  }

  @override
  void dispose() {
    final session = _session;
    if (session != null) ThreeDInspectorService.close(session.id);
    _image?.dispose();
    super.dispose();
  }

  Future<void> _open() async {
    final previous = _session;
    _session = null;
    if (previous != null) ThreeDInspectorService.close(previous.id);

    setState(() {
      _opening = true;
      _error = null;
    });

    final path = widget.modelPath;
    final isDat = path.toLowerCase().endsWith('.dat');

    final datPath = isDat
        ? path
        : await OutfitBuilderService.existingSibling(path, '.dat');
    final dttPath = isDat
        ? await OutfitBuilderService.existingSibling(path, '.dtt')
        : path;

    if (!mounted || widget.modelPath != path) return;

    final pair = DataArchivePair(
      stem: p.basenameWithoutExtension(path),
      label: p.basename(path),
      configRoot: '',
      datPath: datPath,
      dttPath: dttPath,
    );

    try {
      final probe = await ThreeDInspectorService.probe(pair);
      if (!mounted || widget.modelPath != path) return;
      if (probe.candidates.isEmpty) {
        setState(() {
          _opening = false;
          _error = probe.warnings.isEmpty
              ? const _NoModelFound()
              : probe.warnings.join('\n');
        });
        return;
      }

      final session = await ThreeDInspectorService.open(
        pair,
        probe.candidates.first,
      );
      if (!mounted || widget.modelPath != path) {
        ThreeDInspectorService.close(session.id);
        return;
      }

      final names = <String>[];
      for (final mesh in session.meshes) {
        if (mesh.lodIndex != 0) continue;
        if (!names.contains(mesh.name)) names.add(mesh.name);
      }

      _meshNames = names;
      _shown
        ..clear()
        ..addAll(
          session.meshes
              .where((m) => m.lodIndex == 0 && m.visible)
              .map((m) => m.name),
        );

      final rules = {..._rules};
      for (final name in names) {
        rules.putIfAbsent(name, () => const MeshRule());
      }
      rules.removeWhere((name, _) => !names.contains(name));
      _rules = rules;
      widget.onChanged({..._rules});

      setState(() {
        _session = session;
        _opening = false;
      });
      _requestRender();
    } catch (error) {
      if (!mounted || widget.modelPath != path) return;
      setState(() {
        _opening = false;
        _error = error;
      });
    }
  }

  void _setRule(String name, MeshRule rule) {
    setState(() => _rules[name] = rule);
    widget.onChanged({..._rules});
  }

  Future<void> _toggleShown(String name) async {
    final session = _session;
    if (session == null) return;

    final next = !_shown.contains(name);
    setState(() {
      if (next) {
        _shown.add(name);
      } else {
        _shown.remove(name);
      }
    });

    try {
      for (final mesh in session.meshes) {
        if (mesh.name != name || mesh.lodIndex != 0) continue;
        await ThreeDInspectorService.setMeshVisible(
          sessionId: session.id,
          meshId: mesh.id,
          visible: next,
        );
      }
      if (!mounted || _session?.id != session.id) return;
      _requestRender();
    } catch (error) {
      if (mounted && _session?.id == session.id) {
        setState(() => _error = error);
      }
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

    return LayoutBuilder(
      builder: (context, constraints) {
        final narrow = constraints.maxWidth < 720;
        if (narrow) {
          return Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              AspectRatio(aspectRatio: 4 / 3, child: _viewport(context)),
              SizedBox(height: AppSizes.spacingMD(context)),
              _meshList(context, l10n),
            ],
          );
        }
        return Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Expanded(
              flex: 3,
              child: AspectRatio(
                aspectRatio: 3 / 4,
                child: _viewport(context),
              ),
            ),
            SizedBox(width: AppSizes.spacingMD(context)),
            Expanded(flex: 4, child: _meshList(context, l10n)),
          ],
        );
      },
    );
  }

  Widget _viewport(BuildContext context) {
    return ClipRRect(
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
                          _errorText(context),
                          textAlign: TextAlign.center,
                          style: TextStyle(
                            color: AppColors.error,
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
    );
  }

  String _errorText(BuildContext context) {
    final error = _error;
    if (error is _NoModelFound) {
      return AppLocalizations.of(context)!.outfitBuilderNoModelInArchive;
    }
    return '$error';
  }

  Widget _meshList(BuildContext context, AppLocalizations l10n) {
    if (_session == null) return const SizedBox.shrink();

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        Text(
          l10n.outfitBuilderMeshesInModel(_meshNames.length),
          style: TextStyle(
            fontSize: AppSizes.fontXS(context),
            color: AppColors.textMuted,
          ),
        ),
        SizedBox(height: AppSizes.spacingSM(context)),
        for (final name in _meshNames)
          OutfitMeshRow(
            key: ValueKey(name),
            name: name,
            rule: _rules[name] ?? const MeshRule(),
            character: widget.character,
            shown: _shown.contains(name),
            onChanged: (rule) => _setRule(name, rule),
            onToggleShown: () => unawaited(_toggleShown(name)),
          ),
      ],
    );
  }
}

class _NoModelFound {
  const _NoModelFound();
}
