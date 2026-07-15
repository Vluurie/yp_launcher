import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:yp_launcher/services/launch_failure.dart';
import 'package:yp_launcher/services/process_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

Future<void> showLaunchFailureDialog(
  BuildContext context,
  LaunchFailure failure, {
  String? installDirectory,
}) {
  return showDialog(
    context: context,
    builder: (_) => _LaunchFailureDialog(
      failure: failure,
      installDirectory: installDirectory,
    ),
  );
}

class _LaunchFailureDialog extends StatefulWidget {
  final LaunchFailure failure;
  final String? installDirectory;
  const _LaunchFailureDialog({
    required this.failure,
    required this.installDirectory,
  });

  @override
  State<_LaunchFailureDialog> createState() => _LaunchFailureDialogState();
}

class _LaunchFailureDialogState extends State<_LaunchFailureDialog> {
  String? _command;
  final ScrollController _scrollController = ScrollController();

  @override
  void initState() {
    super.initState();
    final dir = widget.installDirectory;
    if (dir != null && dir.isNotEmpty) {
      ProcessService.buildLaunchCommandPreview(installDirectory: dir).then(
        (cmd) {
          if (!mounted) return;
          setState(() => _command = cmd);
        },
      );
    }
  }

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  LaunchFailure get failure => widget.failure;

  @override
  Widget build(BuildContext context) {
    final accent = failure.panic ? AppColors.error : AppColors.warning;
    return Dialog(
      backgroundColor: Colors.transparent,
      insetPadding: const EdgeInsets.all(24),
      child: Container(
        constraints: const BoxConstraints(maxWidth: 720, maxHeight: 640),
        decoration: BoxDecoration(
          color: AppColors.backgroundCard,
          borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
          border: Border.all(color: accent.withValues(alpha: 0.55)),
          boxShadow: const [
            BoxShadow(
              color: AppColors.shadow,
              blurRadius: 18,
              offset: Offset(0, 6),
            ),
          ],
        ),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            _header(context, accent),
            Flexible(
              child: Scrollbar(
                controller: _scrollController,
                thumbVisibility: true,
                child: SingleChildScrollView(
                  controller: _scrollController,
                  padding: EdgeInsets.symmetric(
                    horizontal: AppSizes.paddingLG(context),
                    vertical: AppSizes.paddingMD(context),
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: _buildSections(context, accent),
                  ),
                ),
              ),
            ),
            Container(height: 1, color: AppColors.borderLight),
            _footer(context),
          ],
        ),
      ),
    );
  }

  Widget _header(BuildContext context, Color accent) {
    return Container(
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.paddingLG(context),
        vertical: AppSizes.paddingMD(context),
      ),
      decoration: BoxDecoration(
        color: AppColors.surfaceMedium,
        border: const Border(bottom: BorderSide(color: AppColors.borderLight)),
        borderRadius: BorderRadius.only(
          topLeft: Radius.circular(AppSizes.borderRadius(context)),
          topRight: Radius.circular(AppSizes.borderRadius(context)),
        ),
      ),
      child: Row(
        children: [
          Container(
            width: 36,
            height: 36,
            alignment: Alignment.center,
            decoration: BoxDecoration(
              color: accent.withValues(alpha: 0.14),
              borderRadius: BorderRadius.circular(6),
              border: Border.all(color: accent.withValues(alpha: 0.55)),
            ),
            child: Icon(
              failure.panic
                  ? Icons.bug_report_outlined
                  : Icons.error_outline,
              size: 20,
              color: accent,
            ),
          ),
          SizedBox(width: AppSizes.spacingMD(context)),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  failure.friendlyTitle().toUpperCase(),
                  style: TextStyle(
                    fontSize: AppSizes.fontLG(context),
                    fontWeight: FontWeight.bold,
                    color: AppColors.textPrimary,
                    letterSpacing: 1.1,
                  ),
                ),
                SizedBox(height: AppSizes.spacingSM(context) / 2),
                Text(
                  _subtitle(),
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.textMuted,
                    letterSpacing: 0.3,
                    fontFamily: 'monospace',
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  String _subtitle() {
    final parts = <String>[];
    if (failure.code != null) {
      parts.add(failure.panic
          ? 'PANIC[${failure.code}]'
          : 'ERROR[${failure.code}]');
    }
    if (failure.phase != null) parts.add(failure.phase!);
    return parts.join('  ·  ');
  }

  List<Widget> _buildSections(BuildContext context, Color accent) {
    final hints = failure.hints();
    return [
      _card(
        context,
        accent: accent,
        icon: Icons.info_outline,
        title: 'What happened',
        body: Text(
          failure.friendlyExplanation(),
          style: TextStyle(
            fontSize: AppSizes.fontSM(context),
            color: AppColors.textSecondary,
            height: 1.4,
          ),
        ),
      ),
      if (failure.headline.isNotEmpty &&
          failure.headline != failure.friendlyTitle()) ...[
        SizedBox(height: AppSizes.spacingMD(context)),
        _card(
          context,
          accent: accent,
          icon: Icons.report_outlined,
          title: 'Reported by NAMS',
          body: SelectableText(
            failure.headline,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: AppColors.textPrimary,
              fontFamily: 'monospace',
            ),
          ),
        ),
      ],
      if (hints.isNotEmpty) ...[
        SizedBox(height: AppSizes.spacingMD(context)),
        _card(
          context,
          accent: AppColors.accentPrimary,
          icon: Icons.lightbulb_outline,
          title: 'Try this',
          body: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              for (final h in hints)
                Padding(
                  padding: EdgeInsets.only(bottom: AppSizes.spacingSM(context)),
                  child: Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Icon(
                        Icons.chevron_right,
                        size: 14,
                        color: AppColors.accentPrimary,
                      ),
                      SizedBox(width: AppSizes.spacingSM(context)),
                      Expanded(
                        child: Text(
                          h,
                          style: TextStyle(
                            fontSize: AppSizes.fontSM(context),
                            color: AppColors.textSecondary,
                            height: 1.4,
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
            ],
          ),
        ),
      ],
      if (failure.cause != null || failure.osError != null) ...[
        SizedBox(height: AppSizes.spacingMD(context)),
        _card(
          context,
          accent: AppColors.textMuted,
          icon: Icons.terminal,
          title: 'Diagnostic detail',
          body: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              if (failure.osError != null)
                _kv(context, 'OS', failure.osError!),
              if (failure.cause != null)
                _kv(context, 'Cause', failure.cause!),
              if (failure.fix != null) _kv(context, 'Suggested', failure.fix!),
            ],
          ),
        ),
      ],
      if (_command != null) ...[
        SizedBox(height: AppSizes.spacingMD(context)),
        _card(
          context,
          accent: AppColors.accentPrimary,
          icon: Icons.code,
          title: 'Launch manually from a terminal',
          body: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Padding(
                padding: EdgeInsets.only(bottom: AppSizes.spacingSM(context)),
                child: Text(
                  'If the launcher UI keeps failing for you, paste this into a terminal to start the game manually. It is the exact same command the Play button runs.',
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.textMuted,
                    fontStyle: FontStyle.italic,
                    height: 1.35,
                  ),
                ),
              ),
              Container(
                padding: const EdgeInsets.all(8),
                decoration: BoxDecoration(
                  color: AppColors.backgroundPrimary,
                  borderRadius: BorderRadius.circular(4),
                  border: Border.all(color: AppColors.borderLight),
                ),
                child: SelectableText(
                  _command!,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    fontFamily: 'monospace',
                    color: AppColors.textSecondary,
                    height: 1.35,
                  ),
                ),
              ),
              SizedBox(height: AppSizes.spacingSM(context)),
              Align(
                alignment: Alignment.centerLeft,
                child: TextButton.icon(
                  onPressed: () async {
                    await Clipboard.setData(
                      ClipboardData(text: _command!),
                    );
                  },
                  icon: const Icon(Icons.copy, size: 14),
                  label: Text(
                    'Copy command',
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      letterSpacing: 0.4,
                    ),
                  ),
                  style: TextButton.styleFrom(
                    foregroundColor: AppColors.accentPrimary,
                    padding: const EdgeInsets.symmetric(
                      horizontal: 10,
                      vertical: 4,
                    ),
                  ),
                ),
              ),
            ],
          ),
        ),
      ],
      SizedBox(height: AppSizes.spacingMD(context)),
      _card(
        context,
        accent: AppColors.borderMedium,
        icon: Icons.article_outlined,
        title: 'Raw output',
        body: Container(
          padding: const EdgeInsets.all(8),
          decoration: BoxDecoration(
            color: AppColors.backgroundPrimary,
            borderRadius: BorderRadius.circular(4),
            border: Border.all(color: AppColors.borderLight),
          ),
          child: SelectableText(
            failure.rawOutput,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              fontFamily: 'monospace',
              color: AppColors.textSecondary,
              height: 1.35,
            ),
          ),
        ),
      ),
      SizedBox(height: AppSizes.spacingMD(context)),
    ];
  }

  Widget _card(
    BuildContext context, {
    required Color accent,
    required IconData icon,
    required String title,
    required Widget body,
  }) {
    return Container(
      decoration: BoxDecoration(
        color: AppColors.surfaceLight,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: AppColors.borderLight),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Container(
            padding: EdgeInsets.symmetric(
              horizontal: AppSizes.paddingMD(context),
              vertical: AppSizes.paddingSM(context),
            ),
            decoration: BoxDecoration(
              color: accent.withValues(alpha: 0.10),
              border: const Border(
                bottom: BorderSide(color: AppColors.borderLight),
              ),
              borderRadius: const BorderRadius.only(
                topLeft: Radius.circular(6),
                topRight: Radius.circular(6),
              ),
            ),
            child: Row(
              children: [
                Icon(icon, size: 14, color: accent),
                SizedBox(width: AppSizes.spacingSM(context)),
                Text(
                  title,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    fontWeight: FontWeight.bold,
                    color: accent,
                    letterSpacing: 1.0,
                  ),
                ),
              ],
            ),
          ),
          Padding(
            padding: EdgeInsets.symmetric(
              horizontal: AppSizes.paddingMD(context),
              vertical: AppSizes.paddingSM(context),
            ),
            child: body,
          ),
        ],
      ),
    );
  }

  Widget _kv(BuildContext context, String key, String value) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 90,
            child: Text(
              key,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textMuted,
                fontWeight: FontWeight.w600,
              ),
            ),
          ),
          Expanded(
            child: SelectableText(
              value,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textSecondary,
                fontFamily: 'monospace',
                height: 1.35,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _footer(BuildContext context) {
    return Padding(
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.paddingLG(context),
        vertical: AppSizes.paddingSM(context),
      ),
      child: Row(
        children: [
          _action(
            context,
            icon: Icons.copy,
            label: 'Copy report',
            onTap: () async {
              await Clipboard.setData(ClipboardData(text: failure.rawOutput));
            },
          ),
          if (failure.capturedLogPath != null) ...[
            SizedBox(width: AppSizes.spacingMD(context)),
            _action(
              context,
              icon: Icons.folder_open,
              label: 'Open log file',
              onTap: () async {
                final logPath = failure.capturedLogPath!;
                final uri = Uri.file(File(logPath).parent.path);
                if (await canLaunchUrl(uri)) {
                  await launchUrl(uri,
                      mode: LaunchMode.externalApplication);
                }
              },
            ),
          ],
          const Spacer(),
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: Text(
              'Close',
              style: TextStyle(
                color: AppColors.textMuted,
                fontSize: AppSizes.fontSM(context),
                letterSpacing: 0.4,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _action(
    BuildContext context, {
    required IconData icon,
    required String label,
    required VoidCallback onTap,
  }) {
    return Material(
      color: Colors.transparent,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(4),
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(4),
            border: Border.all(
              color: AppColors.accentPrimary.withValues(alpha: 0.5),
            ),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(icon, size: 14, color: AppColors.accentPrimary),
              const SizedBox(width: 6),
              Text(
                label,
                style: TextStyle(
                  fontSize: AppSizes.fontSM(context),
                  fontWeight: FontWeight.bold,
                  color: AppColors.accentPrimary,
                  letterSpacing: 0.4,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
