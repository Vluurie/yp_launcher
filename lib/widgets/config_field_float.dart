import 'package:flutter/material.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class ConfigFieldFloat extends StatefulWidget {
  final String label;
  final double value;
  final ValueChanged<double> onChanged;
  final String? tooltip;

  const ConfigFieldFloat({
    super.key,
    required this.label,
    required this.value,
    required this.onChanged,
    this.tooltip,
  });

  @override
  State<ConfigFieldFloat> createState() => _ConfigFieldFloatState();
}

class _ConfigFieldFloatState extends State<ConfigFieldFloat> {
  late TextEditingController _controller;
  bool _isValid = true;

  @override
  void initState() {
    super.initState();
    _controller = TextEditingController(text: _formatValue(widget.value));
  }

  @override
  void didUpdateWidget(ConfigFieldFloat oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.value != widget.value) {
      final cursorPos = _controller.selection;
      _controller.text = _formatValue(widget.value);
      try {
        _controller.selection = cursorPos;
      } catch (_) {}
    }
  }

  String _formatValue(double value) {
    if (value == value.roundToDouble() && !value.isInfinite) {
      return value.toStringAsFixed(1);
    }
    return value.toString();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final row = Padding(
      padding: const EdgeInsets.symmetric(vertical: 3),
      child: Row(
        children: [
          Expanded(
            child: Text(
              widget.label,
              style: const TextStyle(
                fontSize: AppSizes.fontMD,
                color: AppColors.textPrimary,
              ),
            ),
          ),
          const SizedBox(width: 8),
          SizedBox(
            width: AppSizes.configInputWidth,
            child: Container(
              decoration: BoxDecoration(
                color: AppColors.inputBackground,
                borderRadius: BorderRadius.circular(4),
                border: Border.all(
                  color: _isValid ? AppColors.borderMedium : AppColors.error,
                ),
              ),
              child: TextField(
                controller: _controller,
                keyboardType:
                    const TextInputType.numberWithOptions(decimal: true),
                style: const TextStyle(
                  fontSize: AppSizes.fontMD,
                  color: AppColors.textPrimary,
                ),
                decoration: const InputDecoration(
                  isDense: true,
                  contentPadding:
                      EdgeInsets.symmetric(horizontal: 8, vertical: 6),
                  border: InputBorder.none,
                ),
                onChanged: (text) {
                  final parsed = double.tryParse(text);
                  setState(() => _isValid = parsed != null);
                  if (parsed != null) {
                    widget.onChanged(parsed);
                  }
                },
              ),
            ),
          ),
        ],
      ),
    );

    if (widget.tooltip != null) {
      return Tooltip(
        richMessage: WidgetSpan(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 280),
            child: Text(
              widget.tooltip!,
              style: const TextStyle(fontSize: AppSizes.fontXS, color: Colors.white),
            ),
          ),
        ),
        decoration: BoxDecoration(
          color: const Color(0xE6303030),
          borderRadius: BorderRadius.circular(6),
        ),
        child: row,
      );
    }
    return row;
  }
}
