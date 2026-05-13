import 'package:flutter/widgets.dart';

extension ResponsiveContext on BuildContext {
  double get screenWidth => MediaQuery.of(this).size.width;
  double get screenHeight => MediaQuery.of(this).size.height;

  bool get isCompact => screenWidth < 800;
  bool get isMedium => screenWidth >= 800 && screenWidth < 1100;
  bool get isWide => screenWidth >= 1100;

  bool get isShort => screenHeight < 500;

  double get tabPaddingH => isCompact
      ? 6
      : isMedium
      ? 10
      : 16;
  double get tabFontSize => isCompact
      ? 10
      : isMedium
      ? 14
      : 15;
  double get contentPadding => isCompact ? 10 : 20;

  bool get useOneColumn => screenWidth < 900;
}
