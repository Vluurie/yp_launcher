import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/services/rust_bridge.dart';
import 'package:yp_launcher/services/three_d_inspector_config_service.dart';
import 'package:yp_launcher/src/rust/api/inspector.dart';

class ThreeDInspectorService {
  static Future<void> initialize() => RustBridge.initialize();

  static Future<ArchiveProbe> probe(DataArchivePair pair) async {
    await initialize();
    return probeArchivePair(datPath: pair.datPath, dttPath: pair.dttPath);
  }

  static Future<ModelSession> open(
    DataArchivePair pair,
    ModelCandidate candidate,
  ) async {
    await initialize();
    return openModel(
      datPath: pair.datPath,
      dttPath: pair.dttPath,
      candidateId: candidate.id,
    );
  }

  static Future<RenderFrame> render({
    required BigInt sessionId,
    required int width,
    required int height,
    required double yaw,
    required double pitch,
    required double distance,
    required double panX,
    required double panY,
  }) {
    return renderModel(
      sessionId: sessionId,
      width: width,
      height: height,
      yaw: yaw,
      pitch: pitch,
      distance: distance,
      panX: panX,
      panY: panY,
    );
  }

  static void close(BigInt sessionId) {
    closeModel(sessionId: sessionId);
  }

  static Future<void> setMeshVisible({
    required BigInt sessionId,
    required int meshId,
    required bool visible,
  }) {
    return setMeshVisibility(
      sessionId: sessionId,
      meshId: meshId,
      visible: visible,
    );
  }

  static Future<void> setLod({
    required BigInt sessionId,
    required int lodIndex,
  }) {
    return setModelLod(sessionId: sessionId, lodIndex: lodIndex);
  }

  static Future<Set<int>> applyPreset({
    required ModelSession session,
    required InspectorOutfitPreset preset,
    required Map<String, bool> stateValues,
    required Set<int> defaultVisibleMeshes,
    required Set<int> currentVisibleMeshes,
  }) async {
    final target = preset.customAssembly
        ? <int>{}
        : <int>{...defaultVisibleMeshes};
    final visibleNames = preset.visibleMeshNames(stateValues);
    if (preset.customAssembly) {
      for (final mesh in session.meshes) {
        if (visibleNames.contains(mesh.name)) target.add(mesh.id);
      }
    } else {
      for (final mesh in session.meshes) {
        if (preset.forceHideMeshes.contains(mesh.name)) {
          target.remove(mesh.id);
        }
        if (preset.forceShowMeshes.contains(mesh.name)) {
          target.add(mesh.id);
        }
      }
    }
    await Future.wait(
      session.meshes
          .where(
            (mesh) =>
                currentVisibleMeshes.contains(mesh.id) !=
                target.contains(mesh.id),
          )
          .map(
            (mesh) => setMeshVisible(
              sessionId: session.id,
              meshId: mesh.id,
              visible: target.contains(mesh.id),
            ),
          ),
    );
    return target;
  }

}
