import 'package:flutter/widgets.dart';

abstract interface class ConfigChangeSource {
  List<String> get changedLabels;
}

bool configValueEquals(Object? a, Object? b) {
  if (a is num && b is num) return (a - b).abs() < 1e-6;
  if (a is List && b is List) {
    if (a.length != b.length) return false;
    for (var i = 0; i < a.length; i++) {
      if (!configValueEquals(a[i], b[i])) return false;
    }
    return true;
  }
  if (a is Map && b is Map) {
    if (a.length != b.length) return false;
    for (final key in a.keys) {
      if (!b.containsKey(key) || !configValueEquals(a[key], b[key])) {
        return false;
      }
    }
    return true;
  }
  return a == b;
}

List<String> changedLabelOf(String label, Object? value, Object? defaultValue) {
  if (defaultValue == null || configValueEquals(value, defaultValue)) {
    return const [];
  }
  return [label];
}

List<String> collectChangedLabels(Iterable<Widget> widgets) {
  final out = <String>[];
  void visit(Widget w) {
    if (w is ConfigChangeSource) {
      out.addAll((w as ConfigChangeSource).changedLabels);
    } else if (w is MultiChildRenderObjectWidget) {
      w.children.forEach(visit);
    } else if (w is SingleChildRenderObjectWidget) {
      final child = w.child;
      if (child != null) visit(child);
    } else if (w is ProxyWidget) {
      visit(w.child);
    }
  }

  widgets.forEach(visit);
  return out;
}

class ConfigChangeGroup extends StatelessWidget implements ConfigChangeSource {
  @override
  final List<String> changedLabels;
  final Widget child;

  const ConfigChangeGroup({
    super.key,
    required this.changedLabels,
    required this.child,
  });

  @override
  Widget build(BuildContext context) => child;
}
