enum VerifyStatus { ok, failed, noRuntime, steamNotRunning, cannotParse, error }

class VerifyCheck {
  final String name;
  final String status;
  final String? detail;

  const VerifyCheck({
    required this.name,
    required this.status,
    this.detail,
  });

  bool get ok => status == 'ok';

  factory VerifyCheck.fromJson(Map<String, dynamic> json) {
    return VerifyCheck(
      name: json['name'] as String? ?? '',
      status: json['status'] as String? ?? 'fail',
      detail: json['detail'] as String?,
    );
  }
}

class VerifyResult {
  final bool ok;
  final List<VerifyCheck> checks;

  const VerifyResult({required this.ok, required this.checks});

  factory VerifyResult.fromJson(Map<String, dynamic> json) {
    final rawChecks = json['checks'];
    return VerifyResult(
      ok: json['ok'] as bool? ?? false,
      checks: rawChecks is List
          ? rawChecks
              .whereType<Map>()
              .map((m) => VerifyCheck.fromJson(Map<String, dynamic>.from(m)))
              .toList()
          : const [],
    );
  }
}

class VerifyOutcome {
  final VerifyStatus status;
  final VerifyResult? result;
  final int? exitCode;
  final String? raw;

  const VerifyOutcome({
    required this.status,
    this.result,
    this.exitCode,
    this.raw,
  });
}
