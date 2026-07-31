import 'dart:async';
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/services/three_d_inspector_config_service.dart';
import 'package:yp_launcher/services/three_d_inspector_service.dart';
import 'package:yp_launcher/src/rust/api/inspector.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/app_dropdown.dart';
import 'package:yp_launcher/widgets/config_field_bool.dart';
import 'package:yp_launcher/widgets/hover_button.dart';

Future<void> showThreeDInspector(
  BuildContext context, {
  required DataArchivePair archive,
  required ArchiveProbe probe,
  int? initialOutfitId,
}) {
  return Navigator.of(context).push<void>(
    MaterialPageRoute(
      fullscreenDialog: true,
      builder: (_) => ThreeDInspectorPage(
        archive: archive,
        probe: probe,
        initialOutfitId: initialOutfitId,
      ),
    ),
  );
}

class ThreeDInspectorPage extends StatefulWidget {
  final DataArchivePair archive;
  final ArchiveProbe probe;
  final int? initialOutfitId;

  const ThreeDInspectorPage({
    super.key,
    required this.archive,
    required this.probe,
    this.initialOutfitId,
  });

  @override
  State<ThreeDInspectorPage> createState() => _ThreeDInspectorPageState();
}

class _ThreeDInspectorPageState extends State<ThreeDInspectorPage> {
  late final List<InspectorOutfitPreset> _presets;
  ModelCandidate? _candidate;
  ModelSession? _session;
  InspectorOutfitPreset? _selectedPreset;
  ui.Image? _image;
  Object? _error;
  bool _opening = true;
  bool _rendering = false;
  bool _renderQueued = false;
  int _renderWidth = 0;
  int _renderHeight = 0;
  double _viewportWidth = 1;
  double _viewportHeight = 1;
  double _yaw = math.pi;
  double _pitch = -0.18;
  double _distance = 3.2;
  double _panX = 0;
  double _panY = 0;
  int _selectedLod = 0;
  Set<int> _visibleMeshes = {};
  Set<int> _defaultVisibleMeshes = {};
  Map<String, bool> _stateValues = {};

  @override
  void initState() {
    super.initState();
    _presets = [...ThreeDInspectorConfigService.load(widget.archive)];
    if (_presets.isNotEmpty) {
      for (final preset in _presets) {
        if (preset.outfitId == widget.initialOutfitId) {
          _selectedPreset = preset;
          break;
        }
      }
      _selectedPreset ??= _presets.first;
      _stateValues = _initialStateValues(_selectedPreset!);
    }
    _candidate = widget.probe.candidates.first;
    unawaited(_openCandidate(_candidate!));
  }

  @override
  void dispose() {
    final session = _session;
    if (session != null) {
      ThreeDInspectorService.close(session.id);
    }
    _image?.dispose();
    super.dispose();
  }

  Future<void> _openCandidate(ModelCandidate candidate) async {
    final previous = _session;
    final previousImage = _image;
    _session = null;
    if (previous != null) {
      ThreeDInspectorService.close(previous.id);
    }
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
      setState(() {
        _session = session;
        _selectedLod = 0;
        _defaultVisibleMeshes = {
          for (final mesh in session.meshes)
            if (mesh.visible) mesh.id,
        };
        _visibleMeshes = {..._defaultVisibleMeshes};
        _presets.removeWhere((preset) => preset.id.startsWith('vanilla:'));
        if (_presets.isEmpty) {
          _presets.addAll(
            ThreeDInspectorConfigService.inferVanillaVariants(
              archiveStem: widget.archive.stem,
              meshNames: session.meshes.map((mesh) => mesh.name),
            ),
          );
          _selectedPreset = _presets.isEmpty ? null : _presets.first;
          if (_selectedPreset != null) {
            _stateValues = _initialStateValues(_selectedPreset!);
          }
        }
        _opening = false;
      });
      if (_selectedPreset != null) {
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

  void _setViewport(BoxConstraints constraints) {
    final logicalWidth = constraints.maxWidth.clamp(16.0, 100000.0).toDouble();
    final logicalHeight = constraints.maxHeight
        .clamp(16.0, 100000.0)
        .toDouble();
    var scale = math.min(1.0, 1920.0 / logicalWidth);
    scale = math.min(scale, 1920.0 / logicalHeight);
    scale = math.min(
      scale,
      math.sqrt(2073600.0 / (logicalWidth * logicalHeight)),
    );
    final width = math.max(16, (logicalWidth * scale).round());
    final height = math.max(16, (logicalHeight * scale).round());
    _viewportWidth = logicalWidth;
    _viewportHeight = logicalHeight;
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
    unawaited(_renderFrame());
  }

  Future<void> _renderFrame() async {
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
        panX: _panX,
        panY: _panY,
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
    if ((event.buttons & kPrimaryMouseButton) != 0) {
      _yaw += event.delta.dx * 0.01;
      _pitch = (_pitch + event.delta.dy * 0.01).clamp(-1.45, 1.45).toDouble();
      _requestRender();
    } else if ((event.buttons & kSecondaryMouseButton) != 0) {
      _panX += event.delta.dx * _renderWidth / _viewportWidth;
      _panY += event.delta.dy * _renderHeight / _viewportHeight;
      _requestRender();
    }
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

  void _resetCamera() {
    _yaw = math.pi;
    _pitch = -0.18;
    _distance = 3.2;
    _panX = 0;
    _panY = 0;
    _requestRender();
  }

  Future<void> _changeLod(int lodIndex) async {
    final session = _session;
    if (session == null || lodIndex == _selectedLod) return;
    setState(() => _selectedLod = lodIndex);
    try {
      await ThreeDInspectorService.setLod(
        sessionId: session.id,
        lodIndex: lodIndex,
      );
      _requestRender();
    } catch (error) {
      if (mounted && _session?.id == session.id) {
        setState(() => _error = error);
      }
    }
  }

  Future<void> _setMeshes(List<ModelMesh> meshes, bool visible) async {
    final session = _session;
    if (session == null) return;
    setState(() {
      for (final mesh in meshes) {
        if (visible) {
          _visibleMeshes.add(mesh.id);
        } else {
          _visibleMeshes.remove(mesh.id);
        }
      }
    });
    try {
      await Future.wait(
        meshes.map(
          (mesh) => ThreeDInspectorService.setMeshVisible(
            sessionId: session.id,
            meshId: mesh.id,
            visible: visible,
          ),
        ),
      );
      _requestRender();
    } catch (error) {
      if (mounted && _session?.id == session.id) {
        setState(() => _error = error);
      }
    }
  }

  Map<String, bool> _initialStateValues(InspectorOutfitPreset preset) {
    return {
      for (final state in preset.states)
        state.id: state.id == inspectorStateEyemask,
    };
  }

  Future<void> _selectPreset(InspectorOutfitPreset preset) async {
    setState(() {
      _selectedPreset = preset;
      _stateValues = _initialStateValues(preset);
    });
    await _applyPreset();
  }

  Future<void> _setConfiguredState(String stateId, bool value) async {
    const storyStates = {
      inspectorStatePrologue,
      inspectorStateHoly,
      inspectorStateNoRight,
      inspectorStateTower,
    };
    setState(() {
      if (value && storyStates.contains(stateId)) {
        for (final storyState in storyStates) {
          _stateValues[storyState] = false;
        }
      }
      _stateValues[stateId] = value;
    });
    await _applyPreset();
  }

  Future<void> _applyPreset() async {
    final session = _session;
    final preset = _selectedPreset;
    if (session == null || preset == null) return;
    try {
      final target = await ThreeDInspectorService.applyPreset(
        session: session,
        preset: preset,
        stateValues: _stateValues,
        defaultVisibleMeshes: _defaultVisibleMeshes,
        currentVisibleMeshes: _visibleMeshes,
      );
      if (!mounted || _session?.id != session.id) return;
      setState(() => _visibleMeshes = target);
      _requestRender();
    } catch (error) {
      if (mounted && _session?.id == session.id) {
        setState(() => _error = error);
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return CallbackShortcuts(
      bindings: {
        const SingleActivator(LogicalKeyboardKey.escape): () =>
            Navigator.of(context).pop(),
      },
      child: Focus(
        autofocus: true,
        child: Scaffold(
          backgroundColor: AppColors.backgroundPrimary,
          body: Stack(
            children: [
              Positioned.fill(
                child: LayoutBuilder(
                  builder: (context, constraints) {
                    _setViewport(constraints);
                    return Listener(
                      behavior: HitTestBehavior.opaque,
                      onPointerMove: _move,
                      onPointerSignal: _scroll,
                      child: _viewport(),
                    );
                  },
                ),
              ),
              Positioned(left: 16, top: 16, right: 16, child: _toolbar()),
              if (_session != null)
                Positioned(
                  right: 16,
                  top: 80,
                  bottom: 58,
                  width: 320,
                  child: _meshPanel(),
                ),
              Positioned(left: 16, bottom: 16, child: _help()),
            ],
          ),
        ),
      ),
    );
  }

  Widget _viewport() {
    if (_opening) {
      return Center(
        child: CircularProgressIndicator(color: AppColors.accentPrimary),
      );
    }
    if (_error != null && _image == null) {
      return Center(
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 560),
          child: Text(
            _error.toString(),
            textAlign: TextAlign.center,
            style: TextStyle(color: AppColors.error),
          ),
        ),
      );
    }
    return RawImage(
      image: _image,
      fit: BoxFit.contain,
      filterQuality: FilterQuality.high,
    );
  }

  Widget _toolbar() {
    final session = _session;
    final l10n = AppLocalizations.of(context)!;
    return Container(
      decoration: BoxDecoration(
        color: AppColors.backgroundCard.withValues(alpha: 0.96),
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(color: AppColors.borderMedium),
        boxShadow: [
          BoxShadow(
            color: AppColors.shadow,
            blurRadius: 8,
            offset: const Offset(0, 3),
          ),
        ],
      ),
      child: Padding(
        padding: EdgeInsets.symmetric(
          horizontal: AppSizes.paddingSM(context),
          vertical: AppSizes.paddingXS(context),
        ),
        child: Row(
          children: [
            HoverIconButton(
              tooltip: l10n.threeDInspectorClose,
              onTap: () => Navigator.of(context).pop(),
              icon: Icon(
                Icons.close,
                size: AppSizes.iconSM(context),
                color: AppColors.textSecondary,
              ),
            ),
            SizedBox(width: AppSizes.spacingSM(context)),
            Expanded(
              child: widget.probe.candidates.length > 1
                  ? Align(
                      alignment: Alignment.centerLeft,
                      child: AppDropdown<ModelCandidate>(
                        value: _candidate!,
                        items: widget.probe.candidates,
                        itemLabel: (candidate) => candidate.name,
                        maxWidth: 360,
                        onChanged: (candidate) {
                          if (candidate.id != _candidate?.id) {
                            unawaited(_openCandidate(candidate));
                          }
                        },
                      ),
                    )
                  : Text(
                      _candidate?.name ?? widget.archive.label,
                      overflow: TextOverflow.ellipsis,
                      style: TextStyle(
                        color: AppColors.textPrimary,
                        fontWeight: FontWeight.w600,
                      ),
                    ),
            ),
            if (session != null) ...[
              SizedBox(width: AppSizes.spacingMD(context)),
              if (session.lodNames.length > 1) ...[
                AppDropdown<int>(
                  value: _selectedLod,
                  items: [
                    for (
                      var index = 0;
                      index < session.lodNames.length;
                      index++
                    )
                      index,
                  ],
                  itemLabel: (index) =>
                      '${l10n.threeDInspectorLod}: ${session.lodNames[index]}',
                  onChanged: (value) => unawaited(_changeLod(value)),
                  maxWidth: 180,
                ),
                SizedBox(width: AppSizes.spacingMD(context)),
              ],
              Text(
                '${session.vertexCount} ${l10n.threeDInspectorVertices} · '
                '${session.triangleCount} ${l10n.threeDInspectorTriangles}',
                style: TextStyle(
                  color: AppColors.textSecondary,
                  fontSize: AppSizes.fontXS(context),
                ),
              ),
            ],
            SizedBox(width: AppSizes.spacingSM(context)),
            HoverIconButton(
              tooltip: l10n.threeDInspectorResetCamera,
              onTap: _resetCamera,
              icon: Icon(
                Icons.center_focus_strong,
                size: AppSizes.iconSM(context),
                color: AppColors.accentPrimary,
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _help() {
    final l10n = AppLocalizations.of(context)!;
    return Container(
      decoration: BoxDecoration(
        color: AppColors.backgroundCard.withValues(alpha: 0.92),
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(color: AppColors.borderLight),
      ),
      child: Padding(
        padding: EdgeInsets.symmetric(
          horizontal: AppSizes.paddingSM(context),
          vertical: AppSizes.paddingXS(context),
        ),
        child: Text(
          l10n.threeDInspectorControls,
          style: TextStyle(
            color: AppColors.textSecondary,
            fontSize: AppSizes.fontXS(context),
          ),
        ),
      ),
    );
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

  String _stateLabel(AppLocalizations l10n, String stateId) {
    return switch (stateId) {
      inspectorStateSelfDestructed => l10n.threeDInspectorStateSelfDestructed,
      inspectorStateEyemask => l10n.threeDInspectorStateEyemask,
      inspectorStateCombat => l10n.threeDInspectorStateCombat,
      inspectorStatePrologue => l10n.threeDInspectorStatePrologue,
      inspectorStateHoly => l10n.threeDInspectorStateHoly,
      inspectorStateNoRight => l10n.threeDInspectorStateNoRight,
      inspectorStateTower => l10n.threeDInspectorStateTower,
      inspectorStateWig => l10n.threeDInspectorStateWig,
      _ => stateId,
    };
  }

  Widget _meshPanel() {
    final l10n = AppLocalizations.of(context)!;
    final session = _session!;
    final groups = <String, List<ModelMesh>>{};
    for (final mesh in session.meshes) {
      if (mesh.lodIndex != _selectedLod) continue;
      groups.putIfAbsent(mesh.name, () => []).add(mesh);
    }
    final entries = groups.entries.toList()
      ..sort((a, b) => a.key.toLowerCase().compareTo(b.key.toLowerCase()));
    final preset = _selectedPreset;
    final availableNames = session.meshes.map((mesh) => mesh.name).toSet();
    final missingNames = preset == null
        ? <String>[]
        : (preset.configuredMeshNames
              .where((name) => !availableNames.contains(name))
              .toList()
            ..sort());
    return Container(
      decoration: BoxDecoration(
        color: AppColors.backgroundCard.withValues(alpha: 0.96),
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(color: AppColors.borderMedium),
        boxShadow: [
          BoxShadow(
            color: AppColors.shadow,
            blurRadius: 8,
            offset: const Offset(0, 3),
          ),
        ],
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 8),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            if (_presets.isNotEmpty && preset != null) ...[
              Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: 12,
                  vertical: 4,
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    Text(
                      preset.id.startsWith('vanilla:')
                          ? l10n.threeDInspectorVariant
                          : l10n.threeDInspectorOutfit,
                      style: const TextStyle(fontWeight: FontWeight.bold),
                    ),
                    const SizedBox(height: 4),
                    AppDropdown<InspectorOutfitPreset>(
                      value: preset,
                      items: _presets,
                      itemLabel: (item) => _presetLabel(l10n, item),
                      onChanged: (value) {
                        if (value.id != preset.id) {
                          unawaited(_selectPreset(value));
                        }
                      },
                      maxWidth: 294,
                      minWidth: 294,
                      highlight: true,
                      menuMatchesTriggerWidth: true,
                    ),
                    if (preset.states.isNotEmpty) ...[
                      const SizedBox(height: 6),
                      Text(
                        l10n.threeDInspectorStates,
                        style: const TextStyle(fontWeight: FontWeight.bold),
                      ),
                      SizedBox(
                        height: (preset.states.length * 60.0)
                            .clamp(60.0, 280.0)
                            .toDouble(),
                        child: ListView(
                          children: [
                            for (final state in preset.states)
                              ConfigFieldBool(
                                label: _stateLabel(l10n, state.id),
                                value: _stateValues[state.id] ?? false,
                                onChanged: (value) => unawaited(
                                  _setConfiguredState(state.id, value),
                                ),
                              ),
                          ],
                        ),
                      ),
                    ],
                    if (missingNames.isNotEmpty)
                      Padding(
                        padding: const EdgeInsets.only(top: 4),
                        child: Tooltip(
                          message: missingNames.join('\n'),
                          child: Text(
                            l10n.threeDInspectorMissingMeshes(
                              missingNames.length,
                            ),
                            style: TextStyle(
                              color: AppColors.warning,
                              fontSize: 12,
                            ),
                          ),
                        ),
                      ),
                  ],
                ),
              ),
              Divider(height: 12, color: AppColors.borderLight),
            ],
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
              child: Text(
                l10n.threeDInspectorMeshes,
                style: const TextStyle(fontWeight: FontWeight.bold),
              ),
            ),
            Divider(height: 12, color: AppColors.borderLight),
            Expanded(
              child: ListView.builder(
                itemCount: entries.length,
                itemBuilder: (context, index) {
                  final entry = entries[index];
                  final visibleCount = entry.value
                      .where((mesh) => _visibleMeshes.contains(mesh.id))
                      .length;
                  final visible = visibleCount == entry.value.length;
                  return ConfigFieldBool(
                    label: entry.key,
                    value: visible,
                    onChanged: (next) =>
                        unawaited(_setMeshes(entry.value, next)),
                  );
                },
              ),
            ),
          ],
        ),
      ),
    );
  }
}
