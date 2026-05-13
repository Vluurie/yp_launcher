class ModProfile {
  final String name;
  final DateTime createdAt;
  const ModProfile({required this.name, required this.createdAt});
}

class ModProfileState {
  final String activeName;
  final List<ModProfile> profiles;
  final bool isLoading;
  final String? error;

  const ModProfileState({
    this.activeName = 'default',
    this.profiles = const [],
    this.isLoading = false,
    this.error,
  });

  ModProfile? get active {
    for (final p in profiles) {
      if (p.name == activeName) return p;
    }
    return null;
  }

  ModProfileState copyWith({
    String? activeName,
    List<ModProfile>? profiles,
    bool? isLoading,
    Object? error = const _Sentinel(),
  }) {
    return ModProfileState(
      activeName: activeName ?? this.activeName,
      profiles: profiles ?? this.profiles,
      isLoading: isLoading ?? this.isLoading,
      error: error is _Sentinel ? this.error : error as String?,
    );
  }
}

class _Sentinel {
  const _Sentinel();
}
