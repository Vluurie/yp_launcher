import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:yp_launcher/services/mod_load_order_service.dart';

part 'mod_load_order_state.g.dart';

class ModLoadOrderData {
  final List<String> order;
  final bool isLoading;

  const ModLoadOrderData({this.order = const [], this.isLoading = false});

  /// Position in the list, or null when the mod has no defined order.
  int? indexOf(String relPath) {
    final rel = ModLoadOrderService.normalize(relPath);
    final i = order.indexOf(rel);
    return i < 0 ? null : i;
  }
}

@Riverpod(keepAlive: true)
class ModLoadOrderStateController extends _$ModLoadOrderStateController {
  @override
  ModLoadOrderData build() => const ModLoadOrderData();

  /// State is highest-priority-first, matching the texture load-order UI.
  /// The file stores lowest-first because NAMS treats the last entry as winner,
  /// so both directions are reversed at the file boundary.
  Future<void> load(String gameDir) async {
    if (gameDir.isEmpty) return;
    state = ModLoadOrderData(order: state.order, isLoading: true);
    final stored = await ModLoadOrderService.list(gameDir);
    state = ModLoadOrderData(order: stored.reversed.toList());
  }

  Future<void> save(String gameDir, List<String> order) async {
    await ModLoadOrderService.save(gameDir, order.reversed.toList());
    await load(gameDir);
  }
}
