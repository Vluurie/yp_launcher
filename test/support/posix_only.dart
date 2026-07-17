import 'dart:io';

final Object? skipOnWindows = Platform.isWindows
    ? 'POSIX-only: exercises Wine/CrossOver/Steam paths that only run on macOS/Linux'
    : null;
