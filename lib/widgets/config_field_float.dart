import 'package:flutter/material.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
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
  bool _hovered = false;

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
    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: Container(
        padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 8),
        margin: const EdgeInsets.symmetric(vertical: 2),
        decoration: BoxDecoration(
          color: _hovered ? AppColors.surfaceLight : Colors.transparent,
          borderRadius: BorderRadius.circular(6),
        ),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    widget.label,
                    style: TextStyle(
                      fontSize: AppSizes.fontMD(context),
                      color: AppColors.textPrimary,
                    ),
                  ),
                  if (widget.tooltip != null)
                    Padding(
                      padding: const EdgeInsets.only(top: 3),
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
            Column(
              crossAxisAlignment: CrossAxisAlignment.end,
              children: [
                SizedBox(
                  width: AppSizes.configInputWidth(context),
                  child: Container(
                    decoration: BoxDecoration(
                      color: AppColors.inputBackground,
                      borderRadius: BorderRadius.circular(6),
                      border: Border.all(
                        color: _isValid
                            ? AppColors.borderMedium
                            : AppColors.error,
                      ),
                    ),
                    child: TextField(
                      controller: _controller,
                      keyboardType: const TextInputType.numberWithOptions(
                        decimal: true,
                      ),
                      style: TextStyle(
                        fontSize: AppSizes.fontMD(context),
                        color: AppColors.textPrimary,
                      ),
                      decoration: const InputDecoration(
                        isDense: true,
                        contentPadding: EdgeInsets.symmetric(
                          horizontal: 10,
                          vertical: 10,
                        ),
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
                if (!_isValid)
                  Padding(
                    padding: const EdgeInsets.only(top: 3),
                    child: Text(
                      AppLocalizations.of(context)!.enterValidNumber,
                      style: TextStyle(
                        fontSize: AppSizes.fontXS(context),
                        color: AppColors.error,
                      ),
                    ),
                  ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
