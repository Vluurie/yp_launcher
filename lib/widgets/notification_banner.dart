import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:yp_launcher/providers/notification_state.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class NotificationBanners extends ConsumerWidget {
  const NotificationBanners({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final notifications = ref.watch(notificationStateControllerProvider);
    if (notifications.isEmpty) return const SizedBox.shrink();

    return Column(
      crossAxisAlignment: CrossAxisAlignment.end,
      mainAxisSize: MainAxisSize.min,
      children: notifications
          .map(
            (item) => _ToastItem(
              item: item,
              onDismiss: () => ref
                  .read(notificationStateControllerProvider.notifier)
                  .dismiss(item.id),
            ),
          )
          .toList(),
    );
  }
}

class _ToastItem extends StatefulWidget {
  final NotificationItem item;
  final VoidCallback onDismiss;

  const _ToastItem({required this.item, required this.onDismiss});

  @override
  State<_ToastItem> createState() => _ToastItemState();
}

class _ToastItemState extends State<_ToastItem>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<Offset> _slideAnimation;
  late Animation<double> _fadeAnimation;
  bool _hovered = false;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 250),
    );
    _slideAnimation = Tween<Offset>(
      begin: const Offset(1.0, 0),
      end: Offset.zero,
    ).animate(CurvedAnimation(parent: _controller, curve: Curves.easeOutCubic));
    _fadeAnimation = Tween<double>(
      begin: 0.0,
      end: 1.0,
    ).animate(CurvedAnimation(parent: _controller, curve: Curves.easeIn));
    _controller.forward();
    _startAutoDismiss();
  }

  Duration get _readDuration {
    final ms = (widget.item.message.length * 55).clamp(4000, 12000);
    return Duration(milliseconds: ms);
  }

  void _startAutoDismiss() {
    Future.delayed(_readDuration, () {
      if (mounted && !_hovered) _dismiss();
    });
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  Future<void> _dismiss() async {
    if (!mounted) return;
    await _controller.reverse();
    if (mounted) widget.onDismiss();
  }

  @override
  Widget build(BuildContext context) {
    return SlideTransition(
      position: _slideAnimation,
      child: FadeTransition(
        opacity: _fadeAnimation,
        child: MouseRegion(
          onEnter: (_) => setState(() => _hovered = true),
          onExit: (_) {
            setState(() => _hovered = false);
            Future.delayed(const Duration(seconds: 2), () {
              if (mounted && !_hovered) _dismiss();
            });
          },
          child: GestureDetector(
            onTap: _dismiss,
            child: Container(
              margin: const EdgeInsets.only(bottom: 6),
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              constraints: BoxConstraints(
                maxWidth: (MediaQuery.of(context).size.width - 80)
                    .clamp(280.0, 720.0),
              ),
              decoration: BoxDecoration(
                color: AppColors.backgroundCard.withValues(alpha: 0.95),
                borderRadius: BorderRadius.circular(6),
                border: Border.all(
                  color: widget.item.color.withValues(alpha: 0.3),
                ),
                boxShadow: [
                  BoxShadow(
                    color: Colors.black.withValues(alpha: 0.3),
                    blurRadius: 8,
                    offset: const Offset(0, 2),
                  ),
                ],
              ),
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  Padding(
                    padding: const EdgeInsets.only(top: 2),
                    child: Icon(
                      widget.item.icon,
                      size: 16,
                      color: widget.item.color,
                    ),
                  ),
                  const SizedBox(width: 8),
                  Flexible(
                    child: Text(
                      widget.item.message,
                      style: TextStyle(
                        fontSize: AppSizes.fontSM(context),
                        color: AppColors.textPrimary,
                        height: 1.35,
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}
