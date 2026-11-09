class FeatureFlags {
  FeatureFlags._();

  static const bool docs = bool.fromEnvironment('YP_DOCS');
  static const bool tools = bool.fromEnvironment('YP_TOOLS');
}
