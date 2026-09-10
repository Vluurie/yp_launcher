import 'package:flutter/material.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class TwoColumnLayout extends StatelessWidget {
  static const defaultBreakpoint = 760.0;

  final Widget left;
  final Widget right;
  final double breakpoint;
  final int leftFlex;
  final int rightFlex;

  final bool fillHeight;

  const TwoColumnLayout({
    super.key,
    required this.left,
    required this.right,
    this.breakpoint = defaultBreakpoint,
    this.leftFlex = 1,
    this.rightFlex = 1,
    this.fillHeight = false,
  });

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final stack = constraints.maxWidth < breakpoint;
        final gap = stack
            ? SizedBox(height: AppSizes.spacingLG(context))
            : SizedBox(width: AppSizes.spacingLG(context));

        final children = <Widget>[
          if (stack && !fillHeight)
            left
          else
            Expanded(flex: leftFlex, child: left),
          gap,
          if (stack && !fillHeight)
            right
          else
            Expanded(flex: rightFlex, child: right),
        ];

        if (stack) {
          return Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: children,
          );
        }
        return Row(
          crossAxisAlignment: fillHeight
              ? CrossAxisAlignment.stretch
              : CrossAxisAlignment.start,
          children: children,
        );
      },
    );
  }
}
