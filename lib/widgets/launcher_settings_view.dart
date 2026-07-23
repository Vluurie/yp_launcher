import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/verify_result.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/services/launch_wrapper_service.dart';
import 'package:yp_launcher/services/nams_cli_service.dart';
import 'package:yp_launcher/services/platform_gate.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/hover_button.dart';

class LauncherSettingsView extends ConsumerStatefulWidget {
  const LauncherSettingsView({super.key});

  @override
  ConsumerState<LauncherSettingsView> createState() =>
      _LauncherSettingsViewState();
}

class _LauncherSettingsViewState extends ConsumerState<LauncherSettingsView> {
  final _scrollController = ScrollController();
  final _wrapperController = TextEditingController();
  bool _wrapperLoaded = false;
  bool _verifying = false;
  VerifyOutcome? _verifyOutcome;

  @override
  void dispose() {
    _scrollController.dispose();
    _wrapperController.dispose();
    super.dispose();
  }

  Future<void> _loadWrapper() async {
    if (_wrapperLoaded) return;
    _wrapperLoaded = true;
    final value = await LaunchWrapperService.read();
    if (!mounted) return;
    setState(() => _wrapperController.text = value);
  }

  Future<void> _runVerify() async {
    final l10n = AppLocalizations.of(context)!;
    final gameDir = ref.read(appStateControllerProvider).selectedDirectory;
    if (gameDir.isEmpty) {
      setState(() => _verifyOutcome =
          const VerifyOutcome(status: VerifyStatus.error));
      return;
    }
    setState(() {
      _verifying = true;
      _verifyOutcome = null;
    });
    final outcome = await NamsCliService.verify(gameDir, l10n);
    if (!mounted) return;
    setState(() {
      _verifying = false;
      _verifyOutcome = outcome;
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return Container(
      color: AppColors.backgroundPrimary,
      child: Column(
        children: [
          _header(context, l10n),
          Expanded(
            child: Scrollbar(
              controller: _scrollController,
              thumbVisibility: true,
              child: SingleChildScrollView(
                controller: _scrollController,
                padding: EdgeInsets.all(AppSizes.contentPadding(context)),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    _verifyCard(context, l10n),
                    if (PlatformGate.isLinux) _wrapperCard(context, l10n),
                  ],
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _header(BuildContext context, AppLocalizations l10n) {
    return Container(
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.cardPaddingH(context),
        vertical: AppSizes.cardPaddingV(context),
      ),
      decoration: const BoxDecoration(
        color: AppColors.surfaceMedium,
        border: Border(bottom: BorderSide(color: AppColors.borderLight)),
      ),
      child: Row(
        children: [
          Text(
            l10n.tabLauncherSettings,
            style: TextStyle(
              fontSize: AppSizes.fontXL(context),
              fontWeight: FontWeight.bold,
              color: AppColors.textPrimary,
              letterSpacing: 1.0,
            ),
          ),
        ],
      ),
    );
  }

  Widget _verifyCard(BuildContext context, AppLocalizations l10n) {
    return _card(context, l10n.verifyInstallTitle, [
      Text(
        l10n.verifyInstallDesc,
        style: TextStyle(
          fontSize: AppSizes.fontXS(context),
          color: AppColors.textMuted,
          height: 1.4,
        ),
      ),
      SizedBox(height: AppSizes.spacingMD(context)),
      Row(
        children: [
          if (_verifying) ...[
            SizedBox(
              width: AppSizes.iconSM(context),
              height: AppSizes.iconSM(context),
              child: const CircularProgressIndicator(
                strokeWidth: 2,
                color: AppColors.accentPrimary,
              ),
            ),
            SizedBox(width: AppSizes.spacingSM(context)),
            Text(
              l10n.verifyInstallRunning,
              style: TextStyle(
                fontSize: AppSizes.fontSM(context),
                color: AppColors.textMuted,
              ),
            ),
          ] else
            HoverButton(
              label: l10n.verifyInstallButton,
              color: AppColors.accentPrimary,
              onTap: _runVerify,
            ),
        ],
      ),
      if (_verifyOutcome != null && !_verifying) ...[
        SizedBox(height: AppSizes.spacingMD(context)),
        ..._verifyResultWidgets(context, l10n, _verifyOutcome!),
      ],
    ]);
  }

  List<Widget> _verifyResultWidgets(
    BuildContext context,
    AppLocalizations l10n,
    VerifyOutcome outcome,
  ) {
    switch (outcome.status) {
      case VerifyStatus.ok:
        return [_statusLine(context, true, l10n.verifyInstallOk)];
      case VerifyStatus.failed:
        return [
          _statusLine(context, false, l10n.verifyInstallFailed),
          SizedBox(height: AppSizes.spacingSM(context)),
          ...?outcome.result?.checks.map((c) => _checkRow(context, l10n, c)),
        ];
      case VerifyStatus.noRuntime:
        return [_statusLine(context, false, l10n.verifyNoRuntime)];
      case VerifyStatus.steamNotRunning:
        return [_statusLine(context, false, l10n.verifySteamNotRunning)];
      case VerifyStatus.cannotParse:
      case VerifyStatus.error:
        return [_statusLine(context, false, l10n.verifyInstallError)];
    }
  }

  Widget _statusLine(BuildContext context, bool ok, String text) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Icon(
          ok ? Icons.check_circle : Icons.error_outline,
          size: AppSizes.iconSM(context),
          color: ok ? AppColors.success : AppColors.warning,
        ),
        SizedBox(width: AppSizes.spacingSM(context)),
        Expanded(
          child: Text(
            text,
            style: TextStyle(
              fontSize: AppSizes.fontSM(context),
              color: ok ? AppColors.success : AppColors.warning,
              height: 1.35,
            ),
          ),
        ),
      ],
    );
  }

  Widget _checkRow(
    BuildContext context,
    AppLocalizations l10n,
    VerifyCheck check,
  ) {
    return Padding(
      padding: EdgeInsets.only(bottom: AppSizes.paddingXS(context)),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(
            check.ok ? Icons.check_circle_outline : Icons.error_outline,
            size: AppSizes.iconSM(context),
            color: check.ok ? AppColors.success : AppColors.error,
          ),
          SizedBox(width: AppSizes.spacingSM(context)),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  _checkLabel(l10n, check.name),
                  style: TextStyle(
                    fontSize: AppSizes.fontSM(context),
                    color: AppColors.textSecondary,
                  ),
                ),
                if (check.detail != null && check.detail!.isNotEmpty)
                  Text(
                    check.detail!,
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      color: AppColors.textMuted,
                      height: 1.35,
                    ),
                  ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  String _checkLabel(AppLocalizations l10n, String name) {
    switch (name) {
      case 'steam_install':
        return l10n.verifyCheckSteamInstall;
      case 'nier_exe':
        return l10n.verifyCheckNierExe;
      case 'steam_api64_source':
        return l10n.verifyCheckSteamApi64;
      case 'runtime_writable':
        return l10n.verifyCheckRuntimeWritable;
      case 'runtime_library_cached':
        return l10n.verifyCheckRuntimeCached;
      default:
        return name;
    }
  }

  Widget _wrapperCard(BuildContext context, AppLocalizations l10n) {
    if (!_wrapperLoaded) {
      WidgetsBinding.instance.addPostFrameCallback((_) => _loadWrapper());
    }
    return _card(context, l10n.launchWrapperTitle, [
      Text(
        l10n.launchWrapperDesc,
        style: TextStyle(
          fontSize: AppSizes.fontXS(context),
          color: AppColors.textMuted,
          height: 1.4,
        ),
      ),
      SizedBox(height: AppSizes.spacingMD(context)),
      TextField(
        controller: _wrapperController,
        onChanged: (v) => LaunchWrapperService.save(v),
        style: TextStyle(
          fontSize: AppSizes.fontSM(context),
          color: AppColors.textPrimary,
          fontFamily: 'monospace',
        ),
        decoration: InputDecoration(
          isDense: true,
          hintText: l10n.launchWrapperHint,
          hintStyle: TextStyle(
            fontSize: AppSizes.fontSM(context),
            color: AppColors.textMuted.withValues(alpha: 0.6),
            fontFamily: 'monospace',
          ),
          filled: true,
          fillColor: AppColors.backgroundPrimary,
          contentPadding: EdgeInsets.symmetric(
            horizontal: AppSizes.paddingSM(context),
            vertical: AppSizes.paddingSM(context),
          ),
          enabledBorder: OutlineInputBorder(
            borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
            borderSide: const BorderSide(color: AppColors.borderLight),
          ),
          focusedBorder: OutlineInputBorder(
            borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
            borderSide: const BorderSide(color: AppColors.accentPrimary),
          ),
        ),
      ),
      SizedBox(height: AppSizes.spacingSM(context)),
      Text(
        l10n.launchWrapperExample,
        style: TextStyle(
          fontSize: AppSizes.fontXS(context),
          color: AppColors.textMuted.withValues(alpha: 0.8),
          fontFamily: 'monospace',
          height: 1.4,
        ),
      ),
    ]);
  }

  Widget _card(BuildContext context, String title, List<Widget> children) {
    return Container(
      width: double.infinity,
      margin: EdgeInsets.only(bottom: AppSizes.paddingLG(context)),
      constraints: const BoxConstraints(maxWidth: 640),
      decoration: BoxDecoration(
        color: AppColors.backgroundCard,
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(color: AppColors.borderLight),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            width: double.infinity,
            padding: EdgeInsets.symmetric(
              horizontal: AppSizes.cardPaddingH(context),
              vertical: AppSizes.cardPaddingV(context),
            ),
            decoration: BoxDecoration(
              color: AppColors.surfaceMedium,
              borderRadius: BorderRadius.only(
                topLeft: Radius.circular(AppSizes.borderRadius(context)),
                topRight: Radius.circular(AppSizes.borderRadius(context)),
              ),
            ),
            child: Text(
              title,
              style: TextStyle(
                fontSize: AppSizes.fontSM(context),
                fontWeight: FontWeight.bold,
                color: AppColors.accentPrimary,
                letterSpacing: 0.5,
              ),
            ),
          ),
          Padding(
            padding: EdgeInsets.all(AppSizes.cardPaddingH(context)),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: children,
            ),
          ),
        ],
      ),
    );
  }
}
