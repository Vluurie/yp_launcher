import 'package:flutter_test/flutter_test.dart';
import 'package:yp_launcher/providers/log_state.dart';
import 'package:yp_launcher/services/log_service.dart';

LogEntry entry(String message) => LogEntry(
      timestamp: '00:00:00',
      level: 'INFO',
      module: 'test',
      message: message,
    );

List<LogEntry> entries(int from, int count) =>
    List.generate(count, (i) => entry('${from + i}'));

void main() {
  group('appendCapped', () {
    test('keeps everything while below the cap', () {
      final result = appendCapped(entries(0, 10), entries(10, 5));

      expect(result.length, 15);
      expect(result.first.message, '0');
      expect(result.last.message, '14');
    });

    test('drops the oldest entries once the cap is passed', () {
      final result = appendCapped(entries(0, maxEntriesPerTab), entries(0, 3));

      expect(result.length, maxEntriesPerTab);
      expect(result.first.message, '3');
      expect(result.last.message, '2');
    });

    test('keeps only the newest when a single batch exceeds the cap', () {
      final result = appendCapped(
        entries(0, 10),
        entries(0, maxEntriesPerTab + 500),
      );

      expect(result.length, maxEntriesPerTab);
      expect(result.last.message, '${maxEntriesPerTab + 499}');
    });

    test('returns the current list untouched for an empty batch', () {
      final current = entries(0, 5);
      expect(identical(appendCapped(current, const []), current), isTrue);
    });

    test('never exceeds the cap across repeated appends', () {
      var list = <LogEntry>[];
      for (var i = 0; i < 20; i++) {
        list = appendCapped(list, entries(i * 400, 400));
        expect(list.length, lessThanOrEqualTo(maxEntriesPerTab));
      }
      expect(list.last.message, '${20 * 400 - 1}');
    });
  });
}
