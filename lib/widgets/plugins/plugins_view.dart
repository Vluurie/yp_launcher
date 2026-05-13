import 'dart:io';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/notification_state.dart';
import 'package:yp_launcher/providers/plugins_state.dart';
import 'package:yp_launcher/services/plugins_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/busy_guard.dart';
import 'package:yp_launcher/widgets/busy_overlay.dart';
import 'package:yp_launcher/widgets/mods/mod_drop_zone.dart';

class PluginsView extends ConsumerStatefulWidget {
  const PluginsView({super.key});

  @override
  ConsumerState<PluginsView> createState() => _PluginsViewState();
}

class _PluginsViewState extends ConsumerState<PluginsView> {
  bool _loaded = false;
  bool _busy = false;
  String _busyMessage = '';

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!_loaded) {
        _loaded = true;
        ref.read(pluginsStateControllerProvider.notifier).load();
      }
    });
  }

  Future<void> _handleDrop(List<String> paths) async {
    if (ref.read(activeTabProvider) != 8) return;
    for (final p in paths) {
      if (!p.toLowerCase().endsWith('.dll')) continue;
      await _install(p);
    }
  }

  Future<void> _handleBrowse() async {
    final result = await FilePicker.platform.pickFiles(
      type: FileType.custom,
      allowedExtensions: ['dll'],
    );
    final picked = result?.files.singleOrNull?.path;
    if (picked == null) return;
    await _install(picked);
  }

  Future<void> _install(String dllPath) async {
    final l10n = AppLocalizations.of(context)!;
    final notifier = ref.read(pluginsStateControllerProvider.notifier);
    final notif = ref.read(notificationStateControllerProvider.notifier);
    setState(() {
      _busy = true;
      _busyMessage = l10n.pluginValidatingInstalling;
    });
    final PluginInstallResult result;
    try {
      result = await withBusyGuard(ref, () => notifier.install(dllPath));
    } finally {
      if (mounted) setState(() => _busy = false);
    }
    if (!mounted) return;
    final fileName = dllPath.split(Platform.pathSeparator).last;
    notif.addNotification(
      NotificationItem(
        id: 'plugin_install_${DateTime.now().millisecondsSinceEpoch}',
        message: result.success
            ? l10n.pluginInstalled(fileName)
            : '${l10n.pluginInstallFailed}: ${_localizeReason(l10n, result.failureReason ?? 'unknown')}',
        icon: result.success ? Icons.check_circle : Icons.error_outline,
        color: result.success ? AppColors.success : AppColors.error,
        type: NotificationType.general,
      ),
    );
  }

  String _localizeReason(AppLocalizations l10n, String code) {
    if (code.startsWith('incompatible:')) {
      return l10n.pluginReasonIncompatible(code.substring('incompatible:'.length));
    }
    switch (code) {
      case 'missing_nams_export':
        return l10n.pluginReasonMissingExport;
      case 'not_a_dll':
        return l10n.pluginReasonNotADll;
      case 'not_64bit':
        return l10n.pluginReasonNot64bit;
      case 'corrupt_pe_header':
      case 'corrupt_exports':
        return l10n.pluginReasonCorrupt;
      case 'no_exports':
        return l10n.pluginReasonNoExports;
      case 'reserved_name':
        return l10n.pluginReasonReservedName;
      case 'file_not_found':
        return l10n.pluginReasonFileNotFound;
      case 'read_failed':
        return l10n.pluginReasonReadFailed;
      case 'copy_failed':
        return l10n.pluginReasonCopyFailed;
      default:
        return code;
    }
  }

  Future<void> _confirmDelete(LauncherPlugin plugin) async {
    final l10n = AppLocalizations.of(context)!;
    final ok = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: AppColors.backgroundCard,
        title: Text(
          l10n.pluginDeleteConfirmTitle,
          style: TextStyle(
            color: AppColors.error,
            fontSize: AppSizes.fontLG(ctx),
          ),
        ),
        content: Text(
          l10n.pluginDeleteConfirmBody(plugin.name),
          style: TextStyle(
            color: AppColors.textSecondary,
            fontSize: AppSizes.fontSM(ctx),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(false),
            child: Text(
              'Cancel',
              style: TextStyle(
                color: AppColors.textMuted,
                fontSize: AppSizes.fontSM(ctx),
              ),
            ),
          ),
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(true),
            child: Text(
              l10n.pluginDelete,
              style: TextStyle(
                color: AppColors.error,
                fontSize: AppSizes.fontSM(ctx),
                fontWeight: FontWeight.bold,
              ),
            ),
          ),
        ],
      ),
    );
    if (ok != true) return;
    if (!mounted) return;
    setState(() {
      _busy = true;
      _busyMessage = l10n.busyDeletingPlugin;
    });
    try {
      await withBusyGuard(
        ref,
        () => ref
            .read(pluginsStateControllerProvider.notifier)
            .delete(plugin.name),
      );
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final data = ref.watch(pluginsStateControllerProvider);

    return Stack(
      children: [
        Container(
          color: AppColors.backgroundPrimary,
          child: Padding(
            padding: EdgeInsets.all(AppSizes.contentPadding(context)),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                _intro(l10n),
                SizedBox(height: AppSizes.spacingMD(context)),
                _dropZone(l10n),
                SizedBox(height: AppSizes.spacingMD(context)),
                Expanded(child: _list(l10n, data)),
              ],
            ),
          ),
        ),
        if (_busy) BusyOverlay(message: _busyMessage),
      ],
    );
  }

  Widget _intro(AppLocalizations l10n) {
    return Tooltip(
      message: l10n.pluginIntroBody,
      child: Container(
        padding: EdgeInsets.symmetric(
          horizontal: AppSizes.cardPaddingH(context),
          vertical: AppSizes.chipPaddingV(context),
        ),
        decoration: BoxDecoration(
          color: AppColors.accentPrimary.withValues(alpha: 0.06),
          borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
          border: Border.all(
            color: AppColors.accentPrimary.withValues(alpha: 0.25),
          ),
        ),
        child: Row(
          children: [
            Icon(
              Icons.extension_outlined,
              size: AppSizes.iconSM(context),
              color: AppColors.accentPrimary,
            ),
            SizedBox(width: AppSizes.spacingMD(context)),
            Expanded(
              child: Text(
                l10n.pluginIntroTitle,
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  fontWeight: FontWeight.bold,
                  color: AppColors.accentPrimary,
                  letterSpacing: 0.4,
                ),
                overflow: TextOverflow.ellipsis,
              ),
            ),
            Icon(
              Icons.info_outline,
              size: AppSizes.iconSM(context),
              color: AppColors.accentPrimary.withValues(alpha: 0.6),
            ),
          ],
        ),
      ),
    );
  }

  Widget _dropZone(AppLocalizations l10n) {
    return ModDropZone(
      onDrop: _handleDrop,
      onBrowse: _handleBrowse,
      title: l10n.pluginDropHere,
      hint: l10n.pluginDropHereHint,
    );
  }

  Widget _list(AppLocalizations l10n, PluginsData data) {
    if (data.isLoading && data.plugins.isEmpty) {
      return Center(
        child: SizedBox(
          width: AppSizes.iconLG(context),
          height: AppSizes.iconLG(context),
          child: const CircularProgressIndicator(
            strokeWidth: 2,
            color: AppColors.accentPrimary,
          ),
        ),
      );
    }
    if (data.plugins.isEmpty) {
      return Center(
        child: Padding(
          padding: EdgeInsets.all(AppSizes.paddingLG(context)),
          child: Text(
            l10n.pluginListEmpty,
            style: TextStyle(
              color: AppColors.textMuted,
              fontStyle: FontStyle.italic,
              fontSize: AppSizes.fontMD(context),
            ),
          ),
        ),
      );
    }
    return Container(
      decoration: BoxDecoration(
        color: AppColors.backgroundCard,
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(color: AppColors.borderLight),
      ),
      child: ListView.separated(
        padding: EdgeInsets.zero,
        itemCount: data.plugins.length,
        separatorBuilder: (_, __) => Divider(
          height: 1,
          thickness: 1,
          color: AppColors.borderLight.withValues(alpha: 0.4),
        ),
        itemBuilder: (_, i) {
          final plugin = data.plugins[i];
          return _PluginRow(
            plugin: plugin,
            onToggle: (enabled) => ref
                .read(pluginsStateControllerProvider.notifier)
                .setEnabled(plugin.name, enabled),
            onDelete: () => _confirmDelete(plugin),
          );
        },
      ),
    );
  }
}

class _PluginRow extends StatelessWidget {
  final LauncherPlugin plugin;
  final void Function(bool enabled) onToggle;
  final VoidCallback onDelete;

  const _PluginRow({
    required this.plugin,
    required this.onToggle,
    required this.onDelete,
  });

  String get _sizeLabel {
    final mb = plugin.sizeBytes / (1024 * 1024);
    if (mb >= 1) return '${mb.toStringAsFixed(1)} MB';
    final kb = plugin.sizeBytes / 1024;
    return '${kb.toStringAsFixed(0)} KB';
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return Padding(
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.cardPaddingH(context),
        vertical: AppSizes.cardPaddingV(context),
      ),
      child: Opacity(
        opacity: plugin.enabled ? 1.0 : 0.45,
        child: Row(
          children: [
            Tooltip(
              message: plugin.filePath,
              child: Expanded(
                child: Text(
                  plugin.name,
                  style: TextStyle(
                    fontSize: AppSizes.fontMD(context),
                    color: AppColors.textPrimary,
                    decoration: plugin.enabled
                        ? null
                        : TextDecoration.lineThrough,
                  ),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
            ),
            SizedBox(width: AppSizes.spacingMD(context)),
            Text(
              _sizeLabel,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textMuted,
              ),
            ),
            SizedBox(width: AppSizes.spacingMD(context)),
            Tooltip(
              message: plugin.enabled
                  ? l10n.modEnableTooltip
                  : l10n.modDisabledTooltip,
              child: InkWell(
                onTap: () => onToggle(!plugin.enabled),
                borderRadius: BorderRadius.circular(
                  AppSizes.borderRadius(context) / 2,
                ),
                child: Padding(
                  padding: EdgeInsets.symmetric(
                    horizontal: AppSizes.chipPaddingH(context),
                    vertical: AppSizes.chipPaddingV(context),
                  ),
                  child: Icon(
                    plugin.enabled
                        ? Icons.toggle_on
                        : Icons.toggle_off_outlined,
                    size: AppSizes.iconLG(context),
                    color: plugin.enabled
                        ? AppColors.success
                        : AppColors.textMuted,
                  ),
                ),
              ),
            ),
            InkWell(
              onTap: onDelete,
              borderRadius: BorderRadius.circular(
                AppSizes.borderRadius(context) / 2,
              ),
              hoverColor: AppColors.error.withValues(alpha: 0.15),
              child: Padding(
                padding: EdgeInsets.all(AppSizes.paddingSM(context)),
                child: Icon(
                  Icons.delete_outline,
                  size: AppSizes.iconMD(context),
                  color: AppColors.error,
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
