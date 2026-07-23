import 'dart:convert';
import 'dart:io';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/isolate_service.dart';

class CheatTableResult {
  final int replacements;
  final String? outputPath;
  final String? error;

  const CheatTableResult({
    this.replacements = 0,
    this.outputPath,
    this.error,
  });

  bool get success => error == null && replacements > 0;
}

CheatTableResult _convertSync(String inputPath) {
  const oldName = 'NieRAutomata.exe';
  const newName = 'game.bin';

  final file = File(inputPath);
  if (!file.existsSync()) {
    return const CheatTableResult(error: 'file_not_found');
  }

  String content;
  Encoding encoding = utf8;
  try {
    final bytes = file.readAsBytesSync();
    try {
      content = utf8.decode(bytes);
    } on FormatException {
      encoding = latin1;
      content = latin1.decode(bytes);
    }
  } catch (_) {
    return const CheatTableResult(error: 'unreadable');
  }

  final pattern = RegExp(RegExp.escape(oldName), caseSensitive: false);
  final count = pattern.allMatches(content).length;
  if (count == 0) {
    return const CheatTableResult();
  }

  final dir = p.dirname(inputPath);
  final stem = p.basenameWithoutExtension(inputPath);
  final ext = p.extension(inputPath).isEmpty ? '.CT' : p.extension(inputPath);
  final outputPath = p.join(dir, '$stem-NAMS$ext');

  try {
    File(outputPath).writeAsBytesSync(
      encoding.encode(content.replaceAll(pattern, newName)),
    );
  } catch (_) {
    return const CheatTableResult(error: 'write_failed');
  }

  return CheatTableResult(replacements: count, outputPath: outputPath);
}

class CheatTableService {
  CheatTableService._();

  static Future<CheatTableResult> convert(String inputPath) {
    return IsolateService.run(_convertSync, inputPath);
  }
}
