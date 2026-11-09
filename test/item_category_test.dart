import 'package:flutter_test/flutter_test.dart';
import 'package:yp_launcher/services/docs/item_builder_service.dart';
import 'package:yp_launcher/services/docs/outfit_builder_service.dart';

void main() {
  test('an outfit item carries the category NAMS uses for its own', () {
    final toml = ItemBuilderService.toToml(
      ItemDraft(
        displayName: 'Black Dress',
        description: 'A dress',
        id: 9100,
        category: OutfitBuilderService.outfitItemCategory,
      ),
    );

    expect(toml, contains('category = 6'));
    expect(toml, contains('id_type = 0'));
  });

  test('a plain item writes no category line', () {
    final toml = ItemBuilderService.toToml(
      const ItemDraft(displayName: 'Nut', description: 'A nut', id: 8000),
    );

    expect(toml, isNot(contains('category')));
  });
}
