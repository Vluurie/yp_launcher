import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:yp_launcher/services/mod_names_service.dart';

part 'mod_names_state.g.dart';

class ModNamesData {
  final Map<String, String> names;

  const ModNamesData({this.names = const {}});

  String? customNameOf(String modId) => names[modId];
}

@Riverpod(keepAlive: true)
class ModNamesStateController extends _$ModNamesStateController {
  @override
  ModNamesData build() => const ModNamesData();

  Future<void> load(String gameDir) async {
    if (gameDir.isEmpty) return;
    state = ModNamesData(names: await ModNamesService.load(gameDir));
  }

  Future<void> rename(String gameDir, String modId, String? name) async {
    await ModNamesService.setName(gameDir, modId, name);
    await load(gameDir);
  }
}
