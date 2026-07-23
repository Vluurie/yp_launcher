import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:yp_launcher/services/wine/launch_command.dart';
import 'package:yp_launcher/services/launch_wrapper_service.dart';

void main() {
  group('tokenize', () {
    test('splits a plain command on spaces', () {
      expect(
        LaunchWrapperService.tokenize('gamescope -w 2560 -h 1440 -f --'),
        ['gamescope', '-w', '2560', '-h', '1440', '-f', '--'],
      );
    });

    test('keeps double-quoted segments together', () {
      expect(
        LaunchWrapperService.tokenize('sh -c "echo hi there"'),
        ['sh', '-c', 'echo hi there'],
      );
    });

    test('keeps single-quoted segments together', () {
      expect(
        LaunchWrapperService.tokenize("run 'a b' c"),
        ['run', 'a b', 'c'],
      );
    });

    test('collapses repeated spaces', () {
      expect(
        LaunchWrapperService.tokenize('a    b'),
        ['a', 'b'],
      );
    });

    test('empty input yields no tokens', () {
      expect(LaunchWrapperService.tokenize('   '), isEmpty);
    });
  });

  group('wrap', () {
    final base = const LaunchCommand(
      command: '/opt/proton/proton',
      args: ['run', 'NAMS.exe', 'run', '--nier-path', 'Z:\\game'],
      cwd: '/launcher',
      label: 'Proton',
    );

    test('empty wrapper returns the base command unchanged', () {
      expect(identical(LaunchWrapperService.wrap(base, ''), base), isTrue);
      expect(identical(LaunchWrapperService.wrap(base, '   '), base), isTrue);
    });

    test('wrap is a no-op off Linux (host guard)', () {
      final wrapped = LaunchWrapperService.wrap(base, 'gamescope --');
      if (Platform.isLinux) {
        expect(wrapped.command, 'gamescope');
      } else {
        expect(identical(wrapped, base), isTrue);
      }
    });

    test('applyWrapper prepends the wrapper and inserts a separator', () {
      final wrapped = LaunchWrapperService.applyWrapper(
        base,
        'gamescope -w 2560 -h 1440 -f --',
      );
      expect(wrapped.command, 'gamescope');
      expect(wrapped.args, [
        '-w', '2560', '-h', '1440', '-f',
        '--',
        '/opt/proton/proton', 'run', 'NAMS.exe', 'run', '--nier-path', 'Z:\\game',
      ]);
      expect(wrapped.cwd, base.cwd);
      expect(wrapped.env, base.env);
      expect(wrapped.label, base.label);
    });

    test('trailing -- in wrapper is not duplicated', () {
      final wrapped = LaunchWrapperService.applyWrapper(base, 'gamescope --');
      expect(wrapped.command, 'gamescope');
      expect(wrapped.args.where((a) => a == '--').length, 1);
      expect(wrapped.args[0], '--');
    });

    test('single-word wrapper (mangohud) prepends correctly', () {
      final wrapped = LaunchWrapperService.applyWrapper(base, 'mangohud');
      expect(wrapped.command, 'mangohud');
      expect(wrapped.args[0], '--');
      expect(wrapped.args[1], '/opt/proton/proton');
    });
  });
}
