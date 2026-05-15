import 'package:flutter/material.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/config_apply_marker.dart';

class ConfigFieldInt extends StatefulWidget {
  final String label;
  final int value;
  final ValueChanged<int> onChanged;
  final String? tooltip;
  final bool hex;
  final int? min;
  final int? max;
  final bool restartRequired;
  final bool showApplyMarker;

  const ConfigFieldInt({
    super.key,
    required this.label,
    required this.value,
    required this.onChanged,
    this.tooltip,
    this.hex = false,
    this.min,
    this.max,
    this.restartRequired = false,
    this.showApplyMarker = false,
  });

  @override
  State<ConfigFieldInt> createState() => _ConfigFieldIntState();
}

class _ConfigFieldIntState extends State<ConfigFieldInt> {
  late TextEditingController _controller;
  bool _isValid = true;

  @override
  void initState() {
    super.initState();
    _controller = TextEditingController(text: _formatValue(widget.value));
  }

  @override
  void didUpdateWidget(ConfigFieldInt oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.value != widget.value) {
      final cursorPos = _controller.selection;
      _controller.text = _formatValue(widget.value);
      try {
        _controller.selection = cursorPos;
      } catch (_) {}
    }
  }

  String _formatValue(int value) {
    if (widget.hex && value != 0) {
      return '0x${value.toRadixString(16).toUpperCase()}';
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
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Flexible(
                      child: Text(
                        widget.label,
                        style: TextStyle(
                          fontSize: AppSizes.fontMD(context),
                          color: AppColors.textPrimary,
                        ),
                      ),
                    ),
                    if (widget.showApplyMarker) ...[
                      const SizedBox(width: 6),
                      ConfigApplyMarker(
                        restartRequired: widget.restartRequired,
                      ),
                    ],
                  ],
                ),
                if (widget.tooltip != null)
                  Padding(
                    padding: const EdgeInsets.only(top: 2),
                    child: Text(
                      widget.tooltip!,
                      style: TextStyle(
                        fontSize: AppSizes.fontXS(context),
                        color: AppColors.textMuted,
                        height: 1.3,
                      ),
                    ),
                  ),
              ],
            ),
          ),
          const SizedBox(width: 12),
          SizedBox(
            width: AppSizes.configInputWidth(context),
            child: Container(
              decoration: BoxDecoration(
                color: AppColors.inputBackground,
                borderRadius: BorderRadius.circular(6),
                border: Border.all(
                  color: _isValid ? AppColors.borderMedium : AppColors.error,
                ),
              ),
              child: TextField(
                controller: _controller,
                keyboardType: TextInputType.text,
                style: TextStyle(
                  fontSize: AppSizes.fontMD(context),
                  color: AppColors.textPrimary,
                ),
                decoration: const InputDecoration(
                  isDense: true,
                  contentPadding: EdgeInsets.symmetric(
                    horizontal: 10,
                    vertical: 8,
                  ),
                  border: InputBorder.none,
                ),
                onChanged: (text) {
                  var parsed = int.tryParse(text);
                  setState(() => _isValid = parsed != null);
                  if (parsed != null) {
                    if (widget.min != null && parsed < widget.min!) {
                      parsed = widget.min;
                    }
                    if (widget.max != null && parsed! > widget.max!) {
                      parsed = widget.max;
                    }
                    widget.onChanged(parsed!);
                  }
                },
              ),
            ),
          ),
        ],
      ),
    );
  }
}
