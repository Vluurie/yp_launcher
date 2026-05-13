import 'package:flutter/widgets.dart';

class AppSizes {
  AppSizes._();

  static double fontXS(BuildContext context) => _scale(context, 13, 15, 17);
  static double fontSM(BuildContext context) => _scale(context, 14, 16, 19);
  static double fontMD(BuildContext context) => _scale(context, 15, 17, 20);
  static double fontLG(BuildContext context) => _scale(context, 17, 19, 22);
  static double fontXL(BuildContext context) => _scale(context, 18, 21, 24);
  static double fontXXL(BuildContext context) => _scale(context, 24, 30, 36);

  static double iconSM(BuildContext context) => _scale(context, 14, 16, 18);
  static double iconMD(BuildContext context) => _scale(context, 16, 19, 22);
  static double iconLG(BuildContext context) => _scale(context, 18, 22, 26);

  static const double titleBarHeight = 38.0;
  static const double titleBarButtonWidth = 50.0;

  static double playButtonWidth(BuildContext context) =>
      _scale(context, 160, 200, 250);
  static double playButtonHeight(BuildContext context) =>
      _scale(context, 44, 56, 68);
  static double playButtonLoadingSize(BuildContext context) =>
      _scale(context, 22, 28, 34);

  static double configInputWidth(BuildContext context) =>
      _scale(context, 70, 90, 110);
  static double checkboxSize(BuildContext context) =>
      _scale(context, 20, 23, 26);

  static double logPanelWidth(BuildContext context) =>
      _scale(context, 350, 480, 620);

  static double paddingXS(BuildContext context) => _scale(context, 2, 4, 6);
  static double paddingSM(BuildContext context) => _scale(context, 4, 6, 8);
  static double paddingMD(BuildContext context) => _scale(context, 6, 10, 14);
  static double paddingLG(BuildContext context) => _scale(context, 8, 14, 20);
  static double paddingXL(BuildContext context) => _scale(context, 12, 20, 32);

  static double spacingSM(BuildContext context) => _scale(context, 2, 4, 6);
  static double spacingMD(BuildContext context) => _scale(context, 4, 6, 8);
  static double spacingLG(BuildContext context) => _scale(context, 6, 10, 16);

  static double borderRadius(BuildContext context) => _scale(context, 4, 6, 8);

  static double cardPaddingH(BuildContext context) =>
      _scale(context, 8, 12, 16);
  static double cardPaddingV(BuildContext context) => _scale(context, 6, 8, 10);

  static double infoBarBottom(BuildContext context) =>
      _scale(context, 28, 34, 40);
  static double infoBarPaddingH(BuildContext context) =>
      _scale(context, 8, 10, 14);
  static double infoBarPaddingV(BuildContext context) =>
      _scale(context, 6, 8, 10);

  static double chipPaddingH(BuildContext context) => _scale(context, 4, 6, 8);
  static double chipPaddingV(BuildContext context) => _scale(context, 1, 2, 3);

  static double dropZonePaddingV(BuildContext context) =>
      _scale(context, 14, 18, 24);

  static double contentPadding(BuildContext context) =>
      _scale(context, 8, 14, 20);

  // Left navigation sidebar.
  static double sidebarWidth(BuildContext context) =>
      _scale(context, 64, 180, 200);
  static bool sidebarLabelsVisible(BuildContext context) =>
      MediaQuery.of(context).size.width >= 800;
  static double sidebarRowHeight(BuildContext context) =>
      _scale(context, 44, 38, 40);
  static double sidebarSectionGap(BuildContext context) =>
      _scale(context, 10, 14, 18);

  static double _scale(
    BuildContext context,
    double compact,
    double medium,
    double wide,
  ) {
    final w = MediaQuery.of(context).size.width;
    if (w < 800) return compact;
    if (w < 1100) return medium;
    return wide;
  }
}
