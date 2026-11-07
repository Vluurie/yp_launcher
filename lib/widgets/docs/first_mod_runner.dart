import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/verify_result.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/services/docs/first_mod_service.dart';
import 'package:yp_launcher/services/nams_cli_service.dart';
import 'package:yp_launcher/services/platform/platform_adapter.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/hover_button.dart';
import 'package:yp_launcher/widgets/docs/toml_editor_dialog.dart';

class FirstModRunner extends ConsumerStatefulWidget {
  const FirstModRunner({super.key});

  @override
  ConsumerState<FirstModRunner> createState() => _FirstModRunnerState();
}

class _FirstModRunnerState extends ConsumerState<FirstModRunner> {
  final _done = <int>{};
  bool _busy = false;
  String? _error;
  VerifyOutcome? _verify;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => _refresh());
  }

  String get _gameDir => ref.read(appStateControllerProvider).selectedDirectory;

  Future<void> _refresh() async {
    final dir = _gameDir;
    if (dir.isEmpty) return;
    final folder = await FirstModService.hasFolder(dir);
    final manifest = await FirstModService.hasManifest(dir);
    final item = await FirstModService.hasItem(dir);
    if (!mounted) return;
    setState(() {
      _done.clear();
      if (folder) _done.add(0);
      if (manifest) _done.add(1);
      if (item) _done.add(2);
    });
  }

  Future<void> _run(int step, Future<void> Function() action) async {
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      await action();
      if (!mounted) return;
      setState(() => _done.add(step));
    } catch (e) {
      if (!mounted) return;
      setState(() => _error = '$e');
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _editManifest() async {
    final l10n = AppLocalizations.of(context)!;
    final current = await FirstModService.readManifest(_gameDir);
    if (!mounted) return;

    final edited = await TomlEditorDialog.show(
      context,
      title: l10n.firstModStep2,
      fileName: 'mod.toml',
      explanation: l10n.firstModExplainManifest,
      initialContent: current,
      resetContent: FirstModFiles.manifest,
    );
    if (edited == null) return;
    await _run(1, () => FirstModService.writeManifest(_gameDir, edited));
  }

  Future<void> _editItem() async {
    final l10n = AppLocalizations.of(context)!;
    final current = await FirstModService.readItem(_gameDir);
    if (!mounted) return;

    final edited = await TomlEditorDialog.show(
      context,
      title: l10n.firstModStep3,
      fileName: 'entities/${FirstModFiles.itemFile}',
      explanation: l10n.firstModExplainItem,
      initialContent: current,
      resetContent: FirstModFiles.item,
    );
    if (edited == null) return;
    await _run(2, () => FirstModService.writeItem(_gameDir, edited));
  }

  Future<void> _runVerify() async {
    final l10n = AppLocalizations.of(context)!;
    setState(() {
      _busy = true;
      _error = null;
      _verify = null;
    });
    final outcome = await NamsCliService.verify(_gameDir, l10n);
    if (!mounted) return;
    setState(() {
      _verify = outcome;
      _busy = false;
      if (outcome.status == VerifyStatus.ok) _done.add(3);
    });
  }

  Future<void> _reset() async {
    await FirstModService.removeAll(_gameDir);
    if (!mounted) return;
    setState(() {
      _done.clear();
      _verify = null;
    });
  }

  Future<void> _openFolder() async {
    try {
      await launchUrl(Uri.file(FirstModService.modRoot(_gameDir)));
    } catch (_) {}
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final gameDir = ref.watch(appStateControllerProvider).selectedDirectory;
    final ready = gameDir.isNotEmpty;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
          if (!ready)
            Text(
              l10n.firstModNoGameDir,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.warning,
              ),
            )
          else ...[
            _step(
              context,
              index: 0,
              title: l10n.firstModStep1,
              detail: l10n.firstModStep1Detail,
              action: l10n.firstModDo,
              onRun: () =>
                  _run(0, () => FirstModService.createFolder(_gameDir)),
            ),
            _step(
              context,
              index: 1,
              title: l10n.firstModStep2,
              detail: l10n.firstModStep2Detail,
              action: l10n.firstModEdit,
              onRun: _editManifest,
            ),
            _step(
              context,
              index: 2,
              title: l10n.firstModStep3,
              detail: l10n.firstModStep3Detail,
              action: l10n.firstModEdit,
              onRun: _editItem,
            ),
            _step(
              context,
              index: 3,
              title: l10n.firstModStep4,
              detail: l10n.firstModStep4Detail,
              action: l10n.firstModVerify,
              onRun: _runVerify,
            ),
            if (_verify != null) _verifyLine(context, l10n),
            if (_error != null) ...[
              SizedBox(height: AppSizes.spacingMD(context)),
              Text(
                l10n.firstModFailed(_error!),
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  color: AppColors.error,
                ),
              ),
            ],
            if (_done.length == 4) ...[
              SizedBox(height: AppSizes.spacingMD(context)),
              Row(
                children: [
                  Icon(
                    Icons.celebration_outlined,
                    size: AppSizes.iconSM(context),
                    color: AppColors.success,
                  ),
                  SizedBox(width: AppSizes.spacingSM(context)),
                  Expanded(
                    child: Text(
                      l10n.firstModFinished,
                      style: TextStyle(
                        fontSize: AppSizes.fontXS(context),
                        color: AppColors.success,
                        height: 1.4,
                      ),
                    ),
                  ),
                ],
              ),
            ],
            if (_done.isNotEmpty) ...[
              SizedBox(height: AppSizes.paddingMD(context)),
              Row(
                children: [
                  HoverButton(
                    label: l10n.firstModOpenFolder,
                    color: AppColors.accentPrimary,
                    onTap: _openFolder,
                  ),
                  SizedBox(width: AppSizes.spacingMD(context)),
                  HoverButton(
                    label: l10n.firstModReset,
                    color: AppColors.textMuted,
                    onTap: _reset,
                  ),
                ],
              ),
            ],
          ],
      ],
    );
  }

  Widget _verifyLine(BuildContext context, AppLocalizations l10n) {
    final status = _verify!.status;
    final ok = status == VerifyStatus.ok;
    final message = switch (status) {
      VerifyStatus.ok => l10n.verifyInstallOk,
      VerifyStatus.failed => l10n.verifyInstallFailed,
      VerifyStatus.noRuntime => l10n.verifyNoRuntime,
      VerifyStatus.steamNotRunning => l10n.verifySteamNotRunning,
      VerifyStatus.timedOut => PlatformAdapter.current.verifyTimedOut(l10n),
      _ => l10n.verifyInstallError,
    };

    return Padding(
      padding: EdgeInsets.only(top: AppSizes.spacingSM(context)),
      child: Row(
        children: [
          Icon(
            ok ? Icons.check_circle_outline : Icons.error_outline,
            size: AppSizes.iconSM(context),
            color: ok ? AppColors.success : AppColors.warning,
          ),
          SizedBox(width: AppSizes.spacingSM(context)),
          Expanded(
            child: Text(
              message,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: ok ? AppColors.success : AppColors.warning,
                height: 1.4,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _step(
    BuildContext context, {
    required int index,
    required String title,
    required String detail,
    required String action,
    required VoidCallback onRun,
  }) {
    final l10n = AppLocalizations.of(context)!;
    final done = _done.contains(index);

    return Padding(
      padding: EdgeInsets.only(bottom: AppSizes.spacingMD(context)),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          Container(
            width: AppSizes.iconLG(context),
            height: AppSizes.iconLG(context),
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              color: done
                  ? AppColors.success.withValues(alpha: 0.18)
                  : AppColors.surfaceLight,
              border: Border.all(
                color: done ? AppColors.success : AppColors.borderLight,
              ),
            ),
            alignment: Alignment.center,
            child: done
                ? Icon(
                    Icons.check,
                    size: AppSizes.fontXS(context),
                    color: AppColors.success,
                  )
                : Text(
                    '${index + 1}',
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      fontWeight: FontWeight.w700,
                      color: AppColors.textMuted,
                    ),
                  ),
          ),
          SizedBox(width: AppSizes.spacingMD(context)),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  title,
                  style: TextStyle(
                    fontSize: AppSizes.fontSM(context),
                    color: AppColors.textPrimary,
                    fontWeight: done ? FontWeight.w400 : FontWeight.w600,
                  ),
                ),
                Text(
                  detail,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.textMuted,
                    fontFamily: AppSizes.monoFamily,
                    fontFamilyFallback: AppSizes.monoFallback,
                  ),
                ),
              ],
            ),
          ),
          SizedBox(width: AppSizes.spacingMD(context)),
          if (_busy)
            SizedBox(
              width: AppSizes.iconSM(context),
              height: AppSizes.iconSM(context),
              child: CircularProgressIndicator(
                strokeWidth: 2,
                color: AppColors.accentPrimary,
              ),
            )
          else
            HoverButton(
              label: done ? l10n.firstModRedo : action,
              color: done ? AppColors.textMuted : AppColors.accentPrimary,
              onTap: onRun,
            ),
        ],
      ),
    );
  }
}
