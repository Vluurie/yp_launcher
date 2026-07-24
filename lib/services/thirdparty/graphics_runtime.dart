import 'package:yp_launcher/services/thirdparty/thirdparty_flags.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_models.dart';

abstract class GraphicsRuntime {
  const GraphicsRuntime();

  ThirdPartyRuntime get runtime;

  String get disableFlagKey;

  String installDir(String gameDir);

  bool canInstall(ThirdPartyClassification c);

  Future<ThirdPartyInstallResult> install(
    String gameDir,
    ThirdPartyClassification c,
  );

  Future<ThirdPartyUpdateInfo?> wouldUpdate(
    String gameDir,
    ThirdPartyClassification c,
  ) async =>
      null;

  Future<void> repair(String gameDir);

  Future<bool> importFromGameRoot(String gameDir);

  Future<ThirdPartyRuntimeStatus> status(String gameDir);

  Future<void> remove(String gameDir);

  Future<void> setEnabled(String gameDir, bool enabled) =>
      ThirdPartyFlags.set(gameDir, disableFlagKey, !enabled);

  Future<bool> isEnabled(String gameDir) async {
    final flags = await ThirdPartyFlags.read(gameDir);
    return flags[disableFlagKey] != true;
  }
}
