import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class AppDropdown<T> extends StatefulWidget {
  final T value;
  final List<T> items;
  final String Function(T) itemLabel;
  final ValueChanged<T>? onChanged;
  final double? maxWidth;
  final bool highlight;
  final IconData? icon;

  const AppDropdown({
    super.key,
    required this.value,
    required this.items,
    required this.itemLabel,
    required this.onChanged,
    this.maxWidth,
    this.highlight = false,
    this.icon,
  });

  @override
  State<AppDropdown<T>> createState() => _AppDropdownState<T>();
}

class _AppDropdownState<T> extends State<AppDropdown<T>> {
  bool _hovered = false;
  bool _open = false;
  final LayerLink _link = LayerLink();
  final GlobalKey _triggerKey = GlobalKey();
  OverlayEntry? _entry;

  @override
  void dispose() {
    _removeOverlay();
    super.dispose();
  }

  void _toggle() {
    if (_open) {
      _removeOverlay();
    } else {
      _showOverlay();
    }
  }

  void _showOverlay() {
    final renderBox =
        _triggerKey.currentContext?.findRenderObject() as RenderBox?;
    if (renderBox == null) return;
    final size = renderBox.size;

    _entry = OverlayEntry(
      builder: (context) {
        return Stack(
          children: [
            Positioned.fill(
              child: GestureDetector(
                behavior: HitTestBehavior.translucent,
                onTap: _removeOverlay,
              ),
            ),
            CompositedTransformFollower(
              link: _link,
              targetAnchor: Alignment.bottomLeft,
              followerAnchor: Alignment.topLeft,
              offset: const Offset(0, 2),
              showWhenUnlinked: false,
              child: Material(
                color: Colors.transparent,
                child: ConstrainedBox(
                  constraints: BoxConstraints(
                    minWidth: size.width,
                    maxWidth: size.width.clamp(120.0, 480.0),
                    maxHeight: 320,
                  ),
                  child: _MenuPanel<T>(
                    items: widget.items,
                    selectedValue: widget.value,
                    itemLabel: widget.itemLabel,
                    onPick: (it) {
                      _removeOverlay();
                      widget.onChanged?.call(it);
                    },
                    onDismiss: _removeOverlay,
                  ),
                ),
              ),
            ),
          ],
        );
      },
    );

    Overlay.of(context, rootOverlay: true).insert(_entry!);
    setState(() => _open = true);
  }

  void _removeOverlay() {
    _entry?.remove();
    _entry = null;
    if (_open && mounted) setState(() => _open = false);
  }

  @override
  Widget build(BuildContext context) {
    final enabled = widget.onChanged != null && widget.items.isNotEmpty;
    final highlight = widget.highlight && enabled;

    final borderColor = !enabled
        ? AppColors.borderLight
        : (_open || highlight)
            ? AppColors.accentPrimary.withValues(alpha: 0.55)
            : AppColors.borderMedium;

    final fillColor = !enabled
        ? AppColors.inputBackground.withValues(alpha: 0.6)
        : _hovered || _open
            ? AppColors.accentPrimary.withValues(alpha: 0.10)
            : AppColors.inputBackground;

    final iconColor = !enabled
        ? AppColors.textMuted
        : (highlight || _hovered || _open)
            ? AppColors.accentPrimary
            : AppColors.textSecondary;

    final textColor =
        !enabled ? AppColors.textMuted : AppColors.textPrimary;

    final selectedLabel = widget.items.contains(widget.value)
        ? widget.itemLabel(widget.value)
        : (widget.items.isNotEmpty
            ? widget.itemLabel(widget.items.first)
            : '');

    return CompositedTransformTarget(
      link: _link,
      child: MouseRegion(
        cursor:
            enabled ? SystemMouseCursors.click : SystemMouseCursors.basic,
        onEnter: (_) {
          if (!enabled || _hovered) return;
          setState(() => _hovered = true);
        },
        onExit: (_) {
          if (!_hovered) return;
          setState(() => _hovered = false);
        },
        child: GestureDetector(
          onTap: enabled ? _toggle : null,
          child: AnimatedContainer(
            key: _triggerKey,
            duration: const Duration(milliseconds: 100),
            constraints: widget.maxWidth != null
                ? BoxConstraints(maxWidth: widget.maxWidth!)
                : null,
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 7),
            decoration: BoxDecoration(
              color: fillColor,
              borderRadius: BorderRadius.circular(4),
              border: Border.all(
                color: borderColor,
                width: highlight || _open ? 1.2 : 1.0,
              ),
            ),
            child: Row(
              mainAxisSize: widget.maxWidth != null
                  ? MainAxisSize.max
                  : MainAxisSize.min,
              children: [
                Flexible(
                  child: Text(
                    selectedLabel,
                    overflow: TextOverflow.ellipsis,
                    maxLines: 1,
                    style: TextStyle(
                      fontSize: AppSizes.fontSM(context),
                      color: textColor,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                ),
                const SizedBox(width: 4),
                AnimatedRotation(
                  duration: const Duration(milliseconds: 120),
                  turns: _open ? 0.5 : 0.0,
                  child: Icon(
                    widget.icon ?? Icons.arrow_drop_down,
                    color: iconColor,
                    size: 18,
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

class _MenuPanel<T> extends StatefulWidget {
  final List<T> items;
  final T selectedValue;
  final String Function(T) itemLabel;
  final ValueChanged<T> onPick;
  final VoidCallback onDismiss;

  const _MenuPanel({
    required this.items,
    required this.selectedValue,
    required this.itemLabel,
    required this.onPick,
    required this.onDismiss,
  });

  @override
  State<_MenuPanel<T>> createState() => _MenuPanelState<T>();
}

class _MenuPanelState<T> extends State<_MenuPanel<T>> {
  late final FocusNode _focusNode = FocusNode();

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) _focusNode.requestFocus();
    });
  }

  @override
  void dispose() {
    _focusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Focus(
      focusNode: _focusNode,
      autofocus: true,
      onKeyEvent: (node, event) {
        if (event is KeyDownEvent &&
            event.logicalKey == LogicalKeyboardKey.escape) {
          widget.onDismiss();
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      child: Container(
        decoration: BoxDecoration(
          color: AppColors.backgroundCard,
          border: Border.all(color: AppColors.borderMedium),
          borderRadius: BorderRadius.circular(4),
          boxShadow: [
            BoxShadow(
              color: Colors.black.withValues(alpha: 0.55),
              blurRadius: 12,
              offset: const Offset(0, 4),
            ),
          ],
        ),
        padding: const EdgeInsets.symmetric(vertical: 4),
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: widget.items.map((it) {
              return _DesktopMenuItem<T>(
                value: it,
                label: widget.itemLabel(it),
                selected: it == widget.selectedValue,
                onTap: () => widget.onPick(it),
              );
            }).toList(),
          ),
        ),
      ),
    );
  }
}

class _DesktopMenuItem<T> extends StatefulWidget {
  final T value;
  final String label;
  final bool selected;
  final VoidCallback onTap;

  const _DesktopMenuItem({
    required this.value,
    required this.label,
    required this.selected,
    required this.onTap,
  });

  @override
  State<_DesktopMenuItem<T>> createState() => _DesktopMenuItemState<T>();
}

class _DesktopMenuItemState<T> extends State<_DesktopMenuItem<T>> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final selected = widget.selected;
    final bg = _hovered
        ? AppColors.accentPrimary.withValues(alpha: 0.18)
        : selected
            ? AppColors.accentPrimary.withValues(alpha: 0.08)
            : Colors.transparent;
    final fg = selected || _hovered
        ? AppColors.accentPrimary
        : AppColors.textPrimary;
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: Container(
          width: double.infinity,
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 7),
          color: bg,
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              SizedBox(
                width: 14,
                child: selected
                    ? const Icon(
                        Icons.check,
                        size: 12,
                        color: AppColors.accentPrimary,
                      )
                    : null,
              ),
              const SizedBox(width: 6),
              Flexible(
                child: Text(
                  widget.label,
                  overflow: TextOverflow.ellipsis,
                  maxLines: 1,
                  style: TextStyle(
                    fontSize: AppSizes.fontSM(context),
                    color: fg,
                    fontWeight:
                        selected ? FontWeight.w700 : FontWeight.w500,
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
