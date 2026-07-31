import 'dart:async';
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/services/three_d_inspector_config_service.dart';
import 'package:yp_launcher/services/three_d_inspector_service.dart';
import 'package:yp_launcher/src/rust/api/inspector.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/app_dropdown.dart';
import 'package:yp_launcher/widgets/hover_button.dart';
import 'package:yp_launcher/widgets/three_d_inspector/three_d_inspector_page.dart';

class ThreeDInspectorPreview extends StatefulWidget {
  final DataArchivePair archive;
  final ArchiveProbe probe;
  final int? initialOutfitId;

  const ThreeDInspectorPreview({
    super.key,
    required this.archive,
    required this.probe,
    this.initialOutfitId,
  });

  @override
  State<ThreeDInspectorPreview> createState() => _ThreeDInspectorPreviewState();
}

class _ThreeDInspectorPreviewState extends State<ThreeDInspectorPreview> {
  late final List<InspectorOutfitPreset> _presets;
  ModelCandidate? _candidate;
  ModelSession? _session;
  InspectorOutfitPreset? _preset;
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
  Set<int> _defaultVisibleMeshes = {};
  Set<int> _visibleMeshes = {};

  @override
  void initState() {
    super.initState();
    _presets = [...ThreeDInspectorConfigService.load(widget.archive)];
    _preset = _presetFor(widget.initialOutfitId);
    _candidate = widget.probe.candidates.first;
    unawaited(_openCandidate(_candidate!));
  }

  @override
  void didUpdateWidget(covariant ThreeDInspectorPreview oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.initialOutfitId == widget.initialOutfitId) return;
    final next = _presetFor(widget.initialOutfitId);
    if (next?.id == _preset?.id) return;
    setState(() => _preset = next);
    unawaited(_applyPreset());
  }

  @override
  void dispose() {
    final session = _session;
    if (session != null) ThreeDInspectorService.close(session.id);
    _image?.dispose();
    super.dispose();
  }

  InspectorOutfitPreset? _presetFor(int? outfitId) {
    for (final preset in _presets) {
      if (preset.outfitId == outfitId) return preset;
    }
    return _presets.isEmpty ? null : _presets.first;
  }

  Map<String, bool> _stateValues(InspectorOutfitPreset preset) {
    return {
      for (final state in preset.states)
        state.id: state.id == inspectorStateEyemask,
    };
  }

  Future<void> _openCandidate(ModelCandidate candidate) async {
    final previous = _session;
    _session = null;
    if (previous != null) ThreeDInspectorService.close(previous.id);
    final previousImage = _image;
    setState(() {
      _candidate = candidate;
      _image = null;
      _opening = true;
      _error = null;
    });
    previousImage?.dispose();
    try {
      final session = await ThreeDInspectorService.open(
        widget.archive,
        candidate,
      );
      if (!mounted || _candidate?.id != candidate.id) {
        ThreeDInspectorService.close(session.id);
        return;
      }
      _defaultVisibleMeshes = {
        for (final mesh in session.meshes)
          if (mesh.visible) mesh.id,
      };
      _visibleMeshes = {..._defaultVisibleMeshes};
      if (_presets.isEmpty) {
        _presets.addAll(
          ThreeDInspectorConfigService.inferVanillaVariants(
            archiveStem: widget.archive.stem,
            meshNames: session.meshes.map((mesh) => mesh.name),
          ),
        );
        _preset = _presets.isEmpty ? null : _presets.first;
      }
      setState(() {
        _session = session;
        _opening = false;
      });
      if (_preset != null) {
        await _applyPreset();
      } else {
        _requestRender();
      }
    } catch (error) {
      if (!mounted || _candidate?.id != candidate.id) return;
      setState(() {
        _opening = false;
        _error = error;
      });
    }
  }

  Future<void> _selectPreset(InspectorOutfitPreset preset) async {
    setState(() => _preset = preset);
    await _applyPreset();
  }

  Future<void> _applyPreset() async {
    final session = _session;
    final preset = _preset;
    if (session == null || preset == null) return;
    try {
      final target = await ThreeDInspectorService.applyPreset(
        session: session,
        preset: preset,
        stateValues: _stateValues(preset),
        defaultVisibleMeshes: _defaultVisibleMeshes,
        currentVisibleMeshes: _visibleMeshes,
      );
      if (!mounted || _session?.id != session.id) return;
      _visibleMeshes = target;
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
    GestureBinding.instance.pointerSignalResolver.register(event, (
      resolvedEvent,
    ) {
      final scrollEvent = resolvedEvent as PointerScrollEvent;
      _distance = (_distance * math.exp(scrollEvent.scrollDelta.dy * 0.001))
          .clamp(0.9, 12.0)
          .toDouble();
      _requestRender();
    });
  }

  String _presetLabel(AppLocalizations l10n, InspectorOutfitPreset preset) {
    final inferredLabel = switch (preset.id) {
      'vanilla:normal' => l10n.threeDInspectorVariantNormal,
      'vanilla:damaged' => l10n.threeDInspectorVariantDamaged,
      'vanilla:base' => l10n.threeDInspectorVariantBase,
      'vanilla:self_destruct' => l10n.threeDInspectorVariantSelfDestruct,
      'vanilla:damaged_left' => l10n.threeDInspectorVariantDamagedLeft,
      'vanilla:damaged_right' => l10n.threeDInspectorVariantDamagedRight,
      'vanilla:damaged_holes' => l10n.threeDInspectorVariantDamagedHoles,
      'vanilla:damaged_2b_hand' => l10n.threeDInspectorVariantDamaged2bHand,
      'vanilla:armor' => l10n.threeDInspectorVariantArmor,
      'vanilla:dlc' => l10n.threeDInspectorVariantDlc,
      'vanilla:dlc_damaged' => l10n.threeDInspectorVariantDlcDamaged,
      'vanilla:dlc_damaged_left' => l10n.threeDInspectorVariantDlcDamagedLeft,
      'vanilla:dlc_damaged_right' => l10n.threeDInspectorVariantDlcDamagedRight,
      'vanilla:dlc_damaged_holes' => l10n.threeDInspectorVariantDlcDamagedHoles,
      'vanilla:dlc_damaged_2b_hand' =>
        l10n.threeDInspectorVariantDlcDamaged2bHand,
      'vanilla:berserk' => l10n.threeDInspectorVariantBerserk,
      _ => null,
    };
    if (inferredLabel != null) return inferredLabel;
    return preset.name.isNotEmpty
        ? preset.name
        : l10n.threeDInspectorOutfitNumber(preset.outfitId);
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return Container(
      margin: EdgeInsets.only(bottom: AppSizes.spacingMD(context)),
      decoration: BoxDecoration(
        color: AppColors.backgroundCard,
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(color: AppColors.borderLight),
        boxShadow: [
          BoxShadow(
            color: AppColors.shadow,
            blurRadius: 4,
            offset: const Offset(0, 2),
          ),
        ],
      ),
      clipBehavior: Clip.antiAlias,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Container(
            color: AppColors.surfaceMedium,
            padding: EdgeInsets.symmetric(
              horizontal: AppSizes.paddingSM(context),
              vertical: AppSizes.paddingXS(context),
            ),
            child: Row(
              children: [
                Expanded(
                  child: widget.probe.candidates.length > 1
                      ? AppDropdown<ModelCandidate>(
                          value: _candidate!,
                          items: widget.probe.candidates,
                          itemLabel: (candidate) => candidate.name,
                          onChanged: (candidate) {
                            if (candidate.id != _candidate?.id) {
                              unawaited(_openCandidate(candidate));
                            }
                          },
                          maxWidth: 240,
                        )
                      : Text(
                          widget.archive.label,
                          overflow: TextOverflow.ellipsis,
                          style: TextStyle(
                            color: AppColors.textPrimary,
                            fontSize: AppSizes.fontSM(context),
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                ),
                if (_presets.length > 1 &&
                    _preset != null &&
                    !_preset!.id.startsWith('vanilla:')) ...[
                  SizedBox(width: AppSizes.spacingSM(context)),
                  AppDropdown<InspectorOutfitPreset>(
                    value: _preset!,
                    items: _presets,
                    itemLabel: (preset) => _presetLabel(l10n, preset),
                    onChanged: (preset) => unawaited(_selectPreset(preset)),
                    maxWidth: 160,
                    minWidth: 160,
                    highlight: true,
                    menuMatchesTriggerWidth: true,
                  ),
                ],
                SizedBox(width: AppSizes.spacingSM(context)),
                HoverIconButton(
                  tooltip: l10n.threeDInspectorExpand,
                  onTap: () => showThreeDInspector(
                    context,
                    archive: widget.archive,
                    probe: widget.probe,
                    initialOutfitId: _preset?.outfitId,
                  ),
                  icon: Icon(
                    Icons.open_in_full,
                    size: AppSizes.iconSM(context),
                    color: AppColors.accentPrimary,
                  ),
                ),
              ],
            ),
          ),
          AspectRatio(
            aspectRatio: 16 / 9,
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
                        if (_opening || (_image == null && _error == null))
                          Center(
                            child: CircularProgressIndicator(
                              color: AppColors.accentPrimary,
                            ),
                          ),
                        if (_error != null && _image == null)
                          Center(
                            child: Padding(
                              padding: EdgeInsets.all(
                                AppSizes.paddingSM(context),
                              ),
                              child: Text(
                                _error.toString(),
                                textAlign: TextAlign.center,
                                style: TextStyle(
                                  color: AppColors.error,
                                  fontSize: AppSizes.fontXS(context),
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
        ],
      ),
    );
  }
}
