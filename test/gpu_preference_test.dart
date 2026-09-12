import 'package:flutter_test/flutter_test.dart';
import 'package:yp_launcher/services/gpu_preference_service.dart';

void main() {
  group('linuxGpuEnv', () {
    test('a single GPU gets no offload variables at all', () {
      const probe = GpuProbe(nvidiaDriverLoaded: false, renderNodes: 1);
      expect(GpuPreferenceService.linuxGpuEnv(probe), isEmpty);
    });

    test('a single NVIDIA GPU has nothing to offload to either', () {
      const probe = GpuProbe(nvidiaDriverLoaded: true, renderNodes: 1);
      expect(GpuPreferenceService.linuxGpuEnv(probe), isEmpty);
    });

    test('no render nodes at all yields nothing', () {
      const probe = GpuProbe(nvidiaDriverLoaded: false, renderNodes: 0);
      expect(GpuPreferenceService.linuxGpuEnv(probe), isEmpty);
    });

    test('an NVIDIA hybrid gets the PRIME offload set and no DRI_PRIME', () {
      const probe = GpuProbe(nvidiaDriverLoaded: true, renderNodes: 2);
      final env = GpuPreferenceService.linuxGpuEnv(probe);

      expect(env, GpuPreferenceService.nvidiaOffloadEnv);
      expect(env.containsKey('DRI_PRIME'), isFalse);
      expect(env['__GLX_VENDOR_LIBRARY_NAME'], 'nvidia');
      expect(env['__VK_LAYER_NV_optimus'], 'NVIDIA_only');
    });

    test('a Mesa hybrid gets DRI_PRIME only, never the NVIDIA vendor', () {
      const probe = GpuProbe(nvidiaDriverLoaded: false, renderNodes: 2);
      final env = GpuPreferenceService.linuxGpuEnv(probe);

      expect(env, {'DRI_PRIME': '1'});
      expect(env.containsKey('__GLX_VENDOR_LIBRARY_NAME'), isFalse);
      expect(env.containsKey('__VK_LAYER_NV_optimus'), isFalse);
    });
  });

  group('mergedLaunchEnv', () {
    test('disabled leaves the base environment untouched', () {
      final base = {'HOME': '/home/u'};
      const probe = GpuProbe(nvidiaDriverLoaded: true, renderNodes: 2);

      expect(
        GpuPreferenceService.mergedLaunchEnv(
          base,
          enabled: false,
          probe: probe,
        ),
        same(base),
      );
    });
  });

  group('GpuProbe.detect', () {
    test('never throws and reports no hybrid setup off Linux', () {
      final probe = GpuProbe.detect();
      expect(probe.renderNodes, greaterThanOrEqualTo(0));
      expect(probe.isHybrid, probe.renderNodes > 1);
    });
  });
}
