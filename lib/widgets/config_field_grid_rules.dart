import 'package:flutter/material.dart';
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/app_dropdown.dart';
import 'package:yp_launcher/widgets/config_apply_marker.dart';

class GridRuleColumn {
  final String key;
  final String label;
  final bool hex;
  final bool text;
  final List<int>? options;
  final dynamic fallback;

  const GridRuleColumn({
    required this.key,
    required this.label,
    required this.fallback,
    this.hex = false,
    this.text = false,
    this.options,
  });
}

class ConfigFieldGridRules extends StatelessWidget {
  final String label;
  final String? tooltip;
  final bool restartRequired;
  final List<dynamic> rules;
  final List<GridRuleColumn> columns;
  final String addLabel;
  final String emptyLabel;
  final ValueChanged<List<dynamic>> onChanged;

  const ConfigFieldGridRules({
    super.key,
    required this.label,
    required this.rules,
    required this.columns,
    required this.addLabel,
    required this.emptyLabel,
    required this.onChanged,
    this.tooltip,
    this.restartRequired = false,
  });

  Map<String, dynamic> _rowAt(int index) {
    final raw = rules[index];
    return raw is Map
        ? raw.map((k, v) => MapEntry(k.toString(), v))
        : <String, dynamic>{};
  }

  void _update(int index, String key, dynamic value) {
    final next = [for (var i = 0; i < rules.length; i++) _rowAt(i)];
    next[index][key] = value;
    onChanged(next);
  }

  void _add() {
    final next = [
      for (var i = 0; i < rules.length; i++) _rowAt(i),
      {for (final c in columns) c.key: c.fallback},
    ];
    onChanged(next);
  }

  void _remove(int index) {
    final next = [
      for (var i = 0; i < rules.length; i++)
        if (i != index) _rowAt(i),
    ];
    onChanged(next);
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Flexible(
                child: Text(
                  label,
                  style: TextStyle(
                    fontSize: AppSizes.fontMD(context),
                    color: AppColors.textPrimary,
                  ),
                ),
              ),
              const SizedBox(width: 6),
              ConfigApplyMarker(restartRequired: restartRequired),
            ],
          ),
          if (tooltip != null)
            Padding(
              padding: const EdgeInsets.only(top: 2),
              child: Text(
                tooltip!,
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  color: AppColors.textMuted,
                  height: 1.3,
                ),
              ),
            ),
          SizedBox(height: AppSizes.spacingSM(context)),
          if (rules.isEmpty)
            Padding(
              padding: const EdgeInsets.only(bottom: 6),
              child: Text(
                emptyLabel,
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  color: AppColors.textMuted.withValues(alpha: 0.7),
                  fontStyle: FontStyle.italic,
                ),
              ),
            )
          else
            for (var i = 0; i < rules.length; i++) _row(context, i, _rowAt(i)),
          Align(
            alignment: Alignment.centerLeft,
            child: TextButton.icon(
              onPressed: _add,
              icon: Icon(
                Icons.add,
                size: AppSizes.iconSM(context),
                color: AppColors.accentPrimary,
              ),
              label: Text(
                addLabel,
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  color: AppColors.accentPrimary,
                ),
              ),
              style: TextButton.styleFrom(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                minimumSize: Size.zero,
                tapTargetSize: MaterialTapTargetSize.shrinkWrap,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _row(BuildContext context, int index, Map<String, dynamic> row) {
    return Container(
      margin: const EdgeInsets.only(bottom: 6),
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
      decoration: BoxDecoration(
        color: AppColors.surfaceLight,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: AppColors.borderLight),
      ),
      child: Row(
        children: [
          for (final column in columns) ...[
            Expanded(
              flex: column.text ? 2 : 1,
              child: column.text
                  ? _textInput(context, index, column, row)
                  : column.options != null
                  ? _dropdown(context, index, column, row)
                  : _hexInput(context, index, column, row),
            ),
            const SizedBox(width: 8),
          ],
          InkWell(
            onTap: () => _remove(index),
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
    );
  }

  Widget _dropdown(
    BuildContext context,
    int index,
    GridRuleColumn column,
    Map<String, dynamic> row,
  ) {
    final value = _intFrom(row[column.key], column.fallback);
    return _labelled(
      context,
      column.label,
      AppDropdown<int>(
        value: column.options!.contains(value) ? value : column.options!.first,
        items: column.options!,
        itemLabel: (v) => _optionLabel(column, v),
        onChanged: (v) => _update(index, column.key, v),
        highlight: true,
      ),
    );
  }

  int _intFrom(Object? raw, int fallback) {
    if (raw is int) return raw;
    if (raw is num) return raw.toInt();
    if (raw is String) return int.tryParse(raw) ?? fallback;
    return fallback;
  }

  String _optionLabel(GridRuleColumn column, int option) {
    if (!column.hex) return option.toString();
    final name = LodModFields.highGridsRooms[option];
    final hex = '0x${option.toRadixString(16).toUpperCase()}';
    return name == null ? hex : '$hex  $name';
  }

  Widget _hexInput(
    BuildContext context,
    int index,
    GridRuleColumn column,
    Map<String, dynamic> row,
  ) {
    final value = _intFrom(row[column.key], column.fallback);
    return _labelled(
      context,
      column.label,
      _HexField(value: value, onChanged: (v) => _update(index, column.key, v)),
    );
  }

  Widget _textInput(
    BuildContext context,
    int index,
    GridRuleColumn column,
    Map<String, dynamic> row,
  ) {
    final raw = row[column.key];
    final value = raw is String ? raw : '${column.fallback}';
    return _labelled(
      context,
      column.label,
      _TextRuleField(
        value: value,
        onChanged: (v) => _update(index, column.key, v),
      ),
    );
  }

  Widget _labelled(BuildContext context, String label, Widget child) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          label,
          style: TextStyle(
            fontSize: AppSizes.fontXS(context),
            color: AppColors.textMuted,
          ),
        ),
        const SizedBox(height: 2),
        child,
      ],
    );
  }
}

class _TextRuleField extends StatefulWidget {
  final String value;
  final ValueChanged<String> onChanged;

  const _TextRuleField({required this.value, required this.onChanged});

  @override
  State<_TextRuleField> createState() => _TextRuleFieldState();
}

class _TextRuleFieldState extends State<_TextRuleField> {
  late final TextEditingController _controller = TextEditingController(
    text: widget.value,
  );

  @override
  void didUpdateWidget(_TextRuleField oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.value != widget.value && !_controller.selection.isValid) {
      _controller.text = widget.value;
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: AppColors.inputBackground,
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: AppColors.borderMedium),
      ),
      child: TextField(
        controller: _controller,
        style: TextStyle(
          fontSize: AppSizes.fontSM(context),
          color: AppColors.textPrimary,
          fontFamily: AppSizes.monoFamily,
          fontFamilyFallback: AppSizes.monoFallback,
        ),
        decoration: const InputDecoration(
          isDense: true,
          contentPadding: EdgeInsets.symmetric(horizontal: 8, vertical: 6),
          border: InputBorder.none,
        ),
        onChanged: (text) => widget.onChanged(text.trim()),
      ),
    );
  }
}

class _HexField extends StatefulWidget {
  final int value;
  final ValueChanged<int> onChanged;

  const _HexField({required this.value, required this.onChanged});

  @override
  State<_HexField> createState() => _HexFieldState();
}

class _HexFieldState extends State<_HexField> {
  late TextEditingController _controller;
  bool _isValid = true;

  @override
  void initState() {
    super.initState();
    _controller = TextEditingController(text: _format(widget.value));
  }

  @override
  void didUpdateWidget(_HexField oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.value != widget.value && !_controller.selection.isValid) {
      _controller.text = _format(widget.value);
    }
  }

  static String _format(int value) =>
      '0x${value.toRadixString(16).toUpperCase()}';

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: AppColors.inputBackground,
        borderRadius: BorderRadius.circular(4),
        border: Border.all(
          color: _isValid ? AppColors.borderMedium : AppColors.error,
        ),
      ),
      child: TextField(
        controller: _controller,
        style: TextStyle(
          fontSize: AppSizes.fontSM(context),
          color: AppColors.textPrimary,
          fontFamily: AppSizes.monoFamily,
          fontFamilyFallback: AppSizes.monoFallback,
        ),
        decoration: const InputDecoration(
          isDense: true,
          contentPadding: EdgeInsets.symmetric(horizontal: 8, vertical: 6),
          border: InputBorder.none,
        ),
        onChanged: (text) {
          final cleaned = text.trim();
          final parsed = cleaned.toLowerCase().startsWith('0x')
              ? int.tryParse(cleaned.substring(2), radix: 16)
              : int.tryParse(cleaned, radix: 16);
          setState(() => _isValid = parsed != null);
          if (parsed != null) widget.onChanged(parsed);
        },
      ),
    );
  }
}
