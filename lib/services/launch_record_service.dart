import 'dart:convert';
import 'dart:io';

import 'package:path/path.dart' as path;
import 'package:yp_launcher/services/log_service.dart';

class LaunchRecord {
  final DateTime startedAt;
  final DateTime? endedAt;
  final int? exitCode;
  final String label;
  final String outputTail;

  const LaunchRecord({
    required this.startedAt,
    this.endedAt,
    this.exitCode,
    this.label = '',
    this.outputTail = '',
  });

  Duration? get duration => endedAt?.difference(startedAt);

  bool get stillRunning => endedAt == null;

  LaunchRecord finished(int code, String tail) => LaunchRecord(
    startedAt: startedAt,
    endedAt: DateTime.now().toUtc(),
    exitCode: code,
    label: label,
    outputTail: tail,
  );

  Map<String, dynamic> toJson() => {
    'started_at': startedAt.toUtc().toIso8601String(),
    if (endedAt != null) 'ended_at': endedAt!.toUtc().toIso8601String(),
    if (exitCode != null) 'exit_code': exitCode,
    'label': label,
    'output_tail': outputTail,
  };

  static LaunchRecord? fromJson(Object? json) {
    if (json is! Map) return null;
    final started = DateTime.tryParse('${json['started_at']}');
    if (started == null) return null;
    final ended = json['ended_at'] == null
        ? null
        : DateTime.tryParse('${json['ended_at']}');
    final code = json['exit_code'];
    return LaunchRecord(
      startedAt: started,
      endedAt: ended,
      exitCode: code is int ? code : null,
      label: json['label'] is String ? json['label'] as String : '',
      outputTail: json['output_tail'] is String
          ? json['output_tail'] as String
          : '',
    );
  }
}

class LaunchRecordService {
  LaunchRecordService._();

  static const fileName = 'last_launch.json';
  static const tailChars = 6 * 1024;

  static String get filePath => path.join(LogService.logsDirectory, fileName);

  static String tailOf(String output) => output.length > tailChars
      ? output.substring(output.length - tailChars)
      : output;

  static Future<void> write(LaunchRecord record) async {
    try {
      final file = File(filePath);
      await file.parent.create(recursive: true);
      await file.writeAsString(jsonEncode(record.toJson()), flush: true);
    } catch (_) {}
  }

  static Future<LaunchRecord?> read() async {
    try {
      final file = File(filePath);
      if (!await file.exists()) return null;
      return LaunchRecord.fromJson(jsonDecode(await file.readAsString()));
    } catch (_) {
      return null;
    }
  }
}
