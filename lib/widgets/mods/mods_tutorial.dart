import 'package:flutter/material.dart';
import 'package:flutter_markdown_plus/flutter_markdown_plus.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class TutorialStep {
  final GlobalKey? anchor;
  final String title;
  final String body;

  const TutorialStep({
    this.anchor,
    required this.title,
    required this.body,
  });
}

enum TutorialKind { ecosystem, installMod, profiles, supporting }

class ModsTutorialAnchors {
  final GlobalKey dropZoneKey;
  final GlobalKey listKey;
  final GlobalKey detailKey;
  final GlobalKey profileSelectorKey;
  final GlobalKey helpIconKey;
  final GlobalKey texturesTabKey;
  final GlobalKey cutscenesTabKey;

  const ModsTutorialAnchors({
    required this.dropZoneKey,
    required this.listKey,
    required this.detailKey,
    required this.profileSelectorKey,
    required this.helpIconKey,
    required this.texturesTabKey,
    required this.cutscenesTabKey,
  });
}

List<TutorialStep> _stepsForEcosystem(
  AppLocalizations l10n,
  ModsTutorialAnchors a,
) =>
    [
      TutorialStep(
        title: l10n.modsTutorialEcosystemStep1Title,
        body: l10n.modsTutorialEcosystemStep1Body,
      ),
      TutorialStep(
        title: l10n.modsTutorialEcosystemStep2Title,
        body: l10n.modsTutorialEcosystemStep2Body,
      ),
      TutorialStep(
        title: l10n.modsTutorialEcosystemStep3Title,
        body: l10n.modsTutorialEcosystemStep3Body,
      ),
      TutorialStep(
        title: l10n.modsTutorialEcosystemStep4Title,
        body: l10n.modsTutorialEcosystemStep4Body,
      ),
    ];

List<TutorialStep> _stepsForSupporting(
  AppLocalizations l10n,
  ModsTutorialAnchors a,
) =>
    [
      TutorialStep(
        title: l10n.modsTutorialSupportingStep1Title,
        body: l10n.modsTutorialSupportingStep1Body,
      ),
      TutorialStep(
        title: l10n.modsTutorialSupportingStep2Title,
        body: l10n.modsTutorialSupportingStep2Body,
      ),
      TutorialStep(
        title: l10n.modsTutorialSupportingStep3Title,
        body: l10n.modsTutorialSupportingStep3Body,
      ),
      TutorialStep(
        title: l10n.modsTutorialSupportingStep4Title,
        body: l10n.modsTutorialSupportingStep4Body,
      ),
      TutorialStep(
        title: l10n.modsTutorialSupportingStep5Title,
        body: l10n.modsTutorialSupportingStep5Body,
      ),
    ];

List<TutorialStep> _stepsForInstall(
  AppLocalizations l10n,
  ModsTutorialAnchors a,
) =>
    [
      TutorialStep(
        anchor: a.dropZoneKey,
        title: l10n.modsTutorialInstallStep1Title,
        body: l10n.modsTutorialInstallStep1Body,
      ),
      TutorialStep(
        title: l10n.modsTutorialInstallStep2Title,
        body: l10n.modsTutorialInstallStep2Body,
      ),
      TutorialStep(
        anchor: a.listKey,
        title: l10n.modsTutorialInstallStep3Title,
        body: l10n.modsTutorialInstallStep3Body,
      ),
      TutorialStep(
        anchor: a.detailKey,
        title: l10n.modsTutorialInstallStep4Title,
        body: l10n.modsTutorialInstallStep4Body,
      ),
      TutorialStep(
        anchor: a.texturesTabKey,
        title: l10n.modsTutorialInstallStep5Title,
        body: l10n.modsTutorialInstallStep5Body,
      ),
      TutorialStep(
        anchor: a.cutscenesTabKey,
        title: l10n.modsTutorialInstallStep6Title,
        body: l10n.modsTutorialInstallStep6Body,
      ),
      TutorialStep(
        title: l10n.modsTutorialInstallStep7Title,
        body: l10n.modsTutorialInstallStep7Body,
      ),
    ];

List<TutorialStep> _stepsForProfiles(
  AppLocalizations l10n,
  ModsTutorialAnchors a,
) =>
    [
      TutorialStep(
        anchor: a.profileSelectorKey,
        title: l10n.modsTutorialProfilesStep1Title,
        body: l10n.modsTutorialProfilesStep1Body,
      ),
      TutorialStep(
        anchor: a.profileSelectorKey,
        title: l10n.modsTutorialProfilesStep2Title,
        body: l10n.modsTutorialProfilesStep2Body,
      ),
      TutorialStep(
        anchor: a.profileSelectorKey,
        title: l10n.modsTutorialProfilesStep3Title,
        body: l10n.modsTutorialProfilesStep3Body,
      ),
      TutorialStep(
        title: l10n.modsTutorialProfilesStep4Title,
        body: l10n.modsTutorialProfilesStep4Body,
      ),
    ];

Future<void> showModsTutorial(
  BuildContext context, {
  required TutorialKind kind,
  required ModsTutorialAnchors anchors,
}) async {
  final l10n = AppLocalizations.of(context)!;
  final steps = switch (kind) {
    TutorialKind.ecosystem => _stepsForEcosystem(l10n, anchors),
    TutorialKind.installMod => _stepsForInstall(l10n, anchors),
    TutorialKind.profiles => _stepsForProfiles(l10n, anchors),
    TutorialKind.supporting => _stepsForSupporting(l10n, anchors),
  };

  await Navigator.of(context).push(
    PageRouteBuilder(
      opaque: false,
      barrierColor: Colors.transparent,
      pageBuilder: (_, __, ___) => _TutorialOverlay(steps: steps),
      transitionsBuilder: (_, animation, __, child) {
        return FadeTransition(opacity: animation, child: child);
      },
      transitionDuration: const Duration(milliseconds: 180),
    ),
  );
}

class _TutorialOverlay extends StatefulWidget {
  final List<TutorialStep> steps;
  const _TutorialOverlay({required this.steps});

  @override
  State<_TutorialOverlay> createState() => _TutorialOverlayState();
}

class _TutorialOverlayState extends State<_TutorialOverlay>
    with SingleTickerProviderStateMixin {
  int _index = 0;
  late final AnimationController _pulseController;
  late final Animation<double> _pulse;

  @override
  void initState() {
    super.initState();
    _pulseController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1500),
    )..repeat(reverse: true);
    _pulse = Tween<double>(begin: 0.45, end: 1.0).animate(
      CurvedAnimation(parent: _pulseController, curve: Curves.easeInOut),
    );
  }

  @override
  void dispose() {
    _pulseController.dispose();
    super.dispose();
  }

  TutorialStep get _step => widget.steps[_index];

  Rect? _anchorRect() {
    final key = _step.anchor;
    if (key == null) return null;
    final ctx = key.currentContext;
    if (ctx == null) return null;
    final box = ctx.findRenderObject() as RenderBox?;
    if (box == null || !box.hasSize) return null;
    final offset = box.localToGlobal(Offset.zero);
    return offset & box.size;
  }

  void _next() {
    if (_index < widget.steps.length - 1) {
      setState(() => _index++);
    } else {
      Navigator.of(context).pop();
    }
  }

  void _back() {
    if (_index > 0) setState(() => _index--);
  }

  void _skip() => Navigator.of(context).pop();

  @override
  Widget build(BuildContext context) {
    final mediaSize = MediaQuery.of(context).size;
    final rect = _anchorRect();
    final l10n = AppLocalizations.of(context)!;
    final isLast = _index == widget.steps.length - 1;

    return Material(
      color: Colors.transparent,
      child: Stack(
        children: [
          Positioned.fill(
            child: GestureDetector(
              behavior: HitTestBehavior.opaque,
              onTap: _next,
              child: CustomPaint(
                painter: _SpotlightPainter(
                  cutout: rect,
                  screen: Offset.zero & mediaSize,
                ),
              ),
            ),
          ),
          if (rect != null)
            Positioned(
              left: rect.left - 4,
              top: rect.top - 4,
              width: rect.width + 8,
              height: rect.height + 8,
              child: IgnorePointer(
                child: AnimatedBuilder(
                  animation: _pulse,
                  builder: (_, __) => Container(
                    decoration: BoxDecoration(
                      border: Border.all(
                        color: AppColors.accentPrimary
                            .withValues(alpha: _pulse.value),
                        width: 2,
                      ),
                      borderRadius: BorderRadius.circular(6),
                      boxShadow: [
                        BoxShadow(
                          color: AppColors.accentPrimary
                              .withValues(alpha: _pulse.value * 0.35),
                          blurRadius: 14,
                          spreadRadius: 1,
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            ),
          _positionCard(rect, mediaSize, l10n, isLast),
        ],
      ),
    );
  }

  Widget _positionCard(
    Rect? rect,
    Size screen,
    AppLocalizations l10n,
    bool isLast,
  ) {
    const margin = 24.0;
    final isReadingStep = rect == null;
    final cardWidth = isReadingStep
        ? (screen.width - margin * 2).clamp(420.0, 720.0)
        : 360.0;
    final maxHeight = isReadingStep
        ? (screen.height - margin * 2).clamp(360.0, 760.0)
        : (screen.height - margin * 2).clamp(200.0, 560.0);
    double left;
    double top;

    if (rect == null) {
      left = (screen.width - cardWidth) / 2;
      top = ((screen.height - maxHeight) / 2).clamp(margin, double.infinity);
    } else {
      final spaceRight = screen.width - rect.right - margin;
      final spaceLeft = rect.left - margin;
      final spaceBelow = screen.height - rect.bottom - margin;

      if (spaceRight >= cardWidth) {
        left = rect.right + margin;
        top = rect.top;
      } else if (spaceLeft >= cardWidth) {
        left = rect.left - margin - cardWidth;
        top = rect.top;
      } else if (spaceBelow >= 180) {
        left = (rect.center.dx - cardWidth / 2)
            .clamp(margin, screen.width - cardWidth - margin);
        top = rect.bottom + margin;
      } else {
        left = (rect.center.dx - cardWidth / 2)
            .clamp(margin, screen.width - cardWidth - margin);
        top = margin;
      }

      left = left.clamp(margin, screen.width - cardWidth - margin);
      top = top.clamp(margin, screen.height - maxHeight - margin);
    }

    return Positioned(
      left: left,
      top: top,
      width: cardWidth,
      child: ConstrainedBox(
        constraints: BoxConstraints(maxHeight: maxHeight),
        child: _TutorialCard(
          step: _step,
          index: _index,
          total: widget.steps.length,
          isLast: isLast,
          isReadingStep: isReadingStep,
          l10n: l10n,
          onNext: _next,
          onBack: _back,
          onSkip: _skip,
        ),
      ),
    );
  }
}

class _TutorialCard extends StatelessWidget {
  final TutorialStep step;
  final int index;
  final int total;
  final bool isLast;
  final bool isReadingStep;
  final AppLocalizations l10n;
  final VoidCallback onNext;
  final VoidCallback onBack;
  final VoidCallback onSkip;

  const _TutorialCard({
    required this.step,
    required this.index,
    required this.total,
    required this.isLast,
    required this.isReadingStep,
    required this.l10n,
    required this.onNext,
    required this.onBack,
    required this.onSkip,
  });

  @override
  Widget build(BuildContext context) {
    final titleSize = isReadingStep
        ? AppSizes.fontXXL(context)
        : AppSizes.fontLG(context);
    final bodySize = isReadingStep
        ? AppSizes.fontMD(context)
        : AppSizes.fontSM(context);
    final codeSize = isReadingStep
        ? AppSizes.fontSM(context)
        : AppSizes.fontXS(context);
    final h3Size = isReadingStep
        ? AppSizes.fontLG(context)
        : AppSizes.fontMD(context);
    final cardPadding = isReadingStep ? 28.0 : 18.0;
    final blockSpacing = isReadingStep ? 14.0 : 10.0;

    return Container(
      decoration: BoxDecoration(
        color: AppColors.backgroundCard,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: AppColors.accentPrimary.withValues(alpha: 0.5),
        ),
        boxShadow: const [
          BoxShadow(
            color: AppColors.shadow,
            blurRadius: 18,
            offset: Offset(0, 6),
          ),
        ],
      ),
      padding: EdgeInsets.all(cardPadding),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Container(
                padding: const EdgeInsets.symmetric(
                  horizontal: 8,
                  vertical: 3,
                ),
                decoration: BoxDecoration(
                  color: AppColors.accentPrimary.withValues(alpha: 0.15),
                  borderRadius: BorderRadius.circular(4),
                ),
                child: Text(
                  '${index + 1} / $total',
                  style: const TextStyle(
                    color: AppColors.accentPrimary,
                    fontSize: 11,
                    fontWeight: FontWeight.w600,
                    letterSpacing: 0.5,
                  ),
                ),
              ),
              const Spacer(),
              InkWell(
                onTap: onSkip,
                borderRadius: BorderRadius.circular(4),
                child: Padding(
                  padding: const EdgeInsets.all(4),
                  child: Icon(
                    Icons.close,
                    size: AppSizes.iconSM(context),
                    color: AppColors.textMuted,
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: 10),
          Flexible(
            child: _CardScroll(
              stepIndex: index,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text(
                    step.title,
                    style: TextStyle(
                      fontSize: titleSize,
                      fontWeight: FontWeight.w600,
                      color: AppColors.textPrimary,
                    ),
                  ),
                  SizedBox(height: isReadingStep ? 14 : 8),
                  MarkdownBody(
                    data: step.body,
                    styleSheet: MarkdownStyleSheet(
                      p: TextStyle(
                        fontSize: bodySize,
                        color: AppColors.textSecondary,
                        height: 1.5,
                      ),
                      strong: TextStyle(
                        fontSize: bodySize,
                        color: AppColors.textPrimary,
                        fontWeight: FontWeight.w700,
                      ),
                      em: TextStyle(
                        fontSize: bodySize,
                        color: AppColors.textSecondary,
                        fontStyle: FontStyle.italic,
                      ),
                      code: TextStyle(
                        fontSize: codeSize,
                        color: AppColors.accentPrimary,
                        fontFamily: 'monospace',
                        backgroundColor:
                            AppColors.accentPrimary.withValues(alpha: 0.08),
                      ),
                      codeblockDecoration: BoxDecoration(
                        color: AppColors.surfaceLight,
                        borderRadius: BorderRadius.circular(4),
                        border: Border.all(color: AppColors.borderLight),
                      ),
                      listBullet: TextStyle(
                        fontSize: bodySize,
                        color: AppColors.accentPrimary,
                      ),
                      h3: TextStyle(
                        fontSize: h3Size,
                        color: AppColors.textPrimary,
                        fontWeight: FontWeight.w600,
                      ),
                      blockSpacing: blockSpacing,
                    ),
                    softLineBreak: true,
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 16),
          Row(
            children: [
              if (index > 0)
                TextButton(
                  onPressed: onBack,
                  child: Text(
                    l10n.modsTutorialBack,
                    style: const TextStyle(color: AppColors.textMuted),
                  ),
                ),
              const Spacer(),
              TextButton(
                onPressed: onSkip,
                child: Text(
                  l10n.modsTutorialSkip,
                  style: const TextStyle(color: AppColors.textMuted),
                ),
              ),
              const SizedBox(width: 4),
              FilledButton(
                onPressed: onNext,
                style: FilledButton.styleFrom(
                  backgroundColor: AppColors.accentPrimary,
                  foregroundColor: AppColors.backgroundPrimary,
                ),
                child: Text(
                  isLast ? l10n.modsTutorialFinish : l10n.modsTutorialNext,
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class _SpotlightPainter extends CustomPainter {
  final Rect? cutout;
  final Rect screen;

  _SpotlightPainter({required this.cutout, required this.screen});

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()..color = Colors.black.withValues(alpha: 0.72);
    if (cutout == null) {
      canvas.drawRect(Offset.zero & size, paint);
      return;
    }
    final padded = cutout!.inflate(6);
    final rrect = RRect.fromRectAndRadius(padded, const Radius.circular(8));
    final path = Path()
      ..addRect(Offset.zero & size)
      ..addRRect(rrect)
      ..fillType = PathFillType.evenOdd;
    canvas.drawPath(path, paint);
  }

  @override
  bool shouldRepaint(covariant _SpotlightPainter old) =>
      old.cutout != cutout || old.screen != screen;
}

class _CardScroll extends StatefulWidget {
  final int stepIndex;
  final Widget child;
  const _CardScroll({required this.stepIndex, required this.child});

  @override
  State<_CardScroll> createState() => _CardScrollState();
}

class _CardScrollState extends State<_CardScroll> {
  final _controller = ScrollController();

  @override
  void didUpdateWidget(covariant _CardScroll old) {
    super.didUpdateWidget(old);
    if (old.stepIndex != widget.stepIndex && _controller.hasClients) {
      _controller.jumpTo(0);
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scrollbar(
      controller: _controller,
      thumbVisibility: true,
      thickness: 6,
      radius: const Radius.circular(3),
      child: SingleChildScrollView(
        controller: _controller,
        padding: const EdgeInsets.only(right: 16),
        child: widget.child,
      ),
    );
  }
}
