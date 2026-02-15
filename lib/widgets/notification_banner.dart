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
      crossAxisAlignment: CrossAxisAlignment.stretch,
      mainAxisSize: MainAxisSize.min,
      verticalDirection: VerticalDirection.up,
      children: notifications
          .map((item) => _NotificationBannerItem(
                item: item,
                onDismiss: () => ref
                    .read(notificationStateControllerProvider.notifier)
                    .dismiss(item.id),
              ))
          .toList(),
    );
  }
}

class _NotificationBannerItem extends StatefulWidget {
  final NotificationItem item;
  final VoidCallback onDismiss;

  const _NotificationBannerItem({
    required this.item,
    required this.onDismiss,
  });

  @override
  State<_NotificationBannerItem> createState() =>
      _NotificationBannerItemState();
}

class _NotificationBannerItemState extends State<_NotificationBannerItem>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _fadeAnimation;
  late Animation<Offset> _slideAnimation;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 300),
    );
    _fadeAnimation = CurvedAnimation(parent: _controller, curve: Curves.easeIn);
    _slideAnimation = Tween<Offset>(
      begin: const Offset(0, 0.5),
      end: Offset.zero,
    ).animate(CurvedAnimation(parent: _controller, curve: Curves.easeOut));
    _controller.forward();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  Future<void> _dismiss() async {
    await _controller.reverse();
    widget.onDismiss();
  }

  @override
  Widget build(BuildContext context) {
    return FadeTransition(
      opacity: _fadeAnimation,
      child: SlideTransition(
        position: _slideAnimation,
        child: Padding(
          padding: const EdgeInsets.only(top: 4),
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
            decoration: BoxDecoration(
              color: AppColors.backgroundCard,
              borderRadius: BorderRadius.circular(6),
              border: Border.all(color: AppColors.borderLight),
              boxShadow: const [
                BoxShadow(
                  color: AppColors.shadow,
                  blurRadius: 4,
                  offset: Offset(0, 1),
                ),
              ],
            ),
            child: Row(
              children: [
                Container(
                  width: 3,
                  height: 20,
                  margin: const EdgeInsets.only(right: 8),
                  decoration: BoxDecoration(
                    color: widget.item.color,
                    borderRadius: BorderRadius.circular(2),
                  ),
                ),
                Icon(
                  widget.item.icon,
                  size: AppSizes.iconMD,
                  color: widget.item.color,
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    widget.item.message,
                    style: TextStyle(
                      fontSize: AppSizes.fontSM,
                      color: AppColors.textSecondary,
                    ),
                  ),
                ),
                InkWell(
                  onTap: _dismiss,
                  borderRadius: BorderRadius.circular(4),
                  child: Padding(
                    padding: const EdgeInsets.all(4),
                    child: Icon(
                      Icons.close,
                      size: AppSizes.iconSM,
                      color: AppColors.textMuted,
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
