import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:yp_launcher/src/rust/api/inspector.dart';
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/services/three_d_inspector_service.dart';

final threeDArchiveProbeProvider = FutureProvider.autoDispose
    .family<ArchiveProbe, DataArchivePair>((ref, pair) {
      return ThreeDInspectorService.probe(pair);
    });
