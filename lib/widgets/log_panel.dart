import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/log_state.dart';
import 'package:yp_launcher/providers/notification_state.dart';
import 'package:yp_launcher/services/diagnostics_service.dart';
import 'package:yp_launcher/services/log_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class LogPanel extends ConsumerStatefulWidget {
  final VoidCallback onClose;

  const LogPanel({super.key, required this.onClose});

  @override
  LogPanelState createState() => LogPanelState();
}

class LogPanelState extends ConsumerState<LogPanel>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<Offset> _slideAnimation;
  final ScrollController _scrollController = ScrollController();
  final TextEditingController _searchController = TextEditingController();
  bool _autoScroll = true;
  int _lastEntryCount = 0;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 250),
    );
    _slideAnimation = Tween<Offset>(
      begin: const Offset(1, 0),
      end: Offset.zero,
    ).animate(CurvedAnimation(parent: _controller, curve: Curves.easeInOut));
    _controller.forward();
    _scrollController.addListener(_onScroll);
  }

  void _onScroll() {
    if (!_scrollController.hasClients) return;
    final atBottom = _scrollController.offset >=
        _scrollController.position.maxScrollExtent - 24;
    if (atBottom != _autoScroll) {
      setState(() => _autoScroll = atBottom);
    }
  }

  void _maybeAutoScroll(int newCount) {
    if (newCount == _lastEntryCount) return;
    _lastEntryCount = newCount;
    if (!_autoScroll) return;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!_scrollController.hasClients) return;
      _scrollController.jumpTo(_scrollController.position.maxScrollExtent);
    });
  }

  @override
  void dispose() {
    _searchController.dispose();
    _scrollController.removeListener(_onScroll);
    _scrollController.dispose();
    _controller.dispose();
    super.dispose();
  }

  Future<void> animateClose() async {
    await _controller.reverse();
    widget.onClose();
  }

  Future<void> _showDiagnostics(BuildContext context) async {
    final l10n = AppLocalizations.of(context)!;
    final gameDir = ref.read(appStateControllerProvider).selectedDirectory;

    showDialog(
      context: context,
      barrierDismissible: false,
      builder: (_) => const _DiagnosticsLoadingDialog(),
    );

    DiagnosticsReport report;
    try {
      report = await DiagnosticsService.collect(gameDir);
    } catch (e) {
      if (!mounted) return;
      Navigator.of(context, rootNavigator: true).pop();
      ref.read(notificationStateControllerProvider.notifier).addNotification(
            NotificationItem(
              id: 'diag_fail_${DateTime.now().millisecondsSinceEpoch}',
              message: (l10n) => '$e',
              icon: Icons.error_outline,
              color: AppColors.error,
              type: NotificationType.general,
            ),
          );
      return;
    }

    if (!mounted) return;
    Navigator.of(context, rootNavigator: true).pop();
    if (!mounted) return;

    showDialog(
      context: context,
      builder: (ctx) => _DiagnosticsDialog(report: report, l10n: l10n),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final logData = ref.watch(logStateControllerProvider);
    final filtered = logData.filteredEntries;
    _maybeAutoScroll(filtered.length);

    return SlideTransition(
      position: _slideAnimation,
      child: Container(
        width: AppSizes.logPanelWidth(context),
        decoration: const BoxDecoration(
          color: AppColors.logBackground,
          border: Border(
            left: BorderSide(color: AppColors.borderMedium, width: 1.5),
          ),
          boxShadow: [
            BoxShadow(
              color: AppColors.shadow,
              blurRadius: 8,
              offset: Offset(-2, 0),
            ),
          ],
        ),
        child: Column(
          children: [
            _buildHeader(context, logData, l10n),
            _buildTabBar(context, logData, l10n),
            _buildSearchBar(context, logData, l10n),
            Expanded(
              child: logData.isLoading && logData.activeEntries.isEmpty
                  ? const Center(
                      child: CircularProgressIndicator(
                        color: AppColors.accentPrimary,
                      ),
                    )
                  : filtered.isEmpty
                      ? Center(
                          child: Text(
                            logData.searchQuery.isEmpty
                                ? l10n.noLogEntries
                                : l10n.noLogMatches,
                            style: TextStyle(
                              fontSize: AppSizes.fontMD(context),
                              color: AppColors.textMuted,
                            ),
                          ),
                        )
                      : _buildLogList(filtered),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSearchBar(
    BuildContext context,
    LogData logData,
    AppLocalizations l10n,
  ) {
    final notifier = ref.read(logStateControllerProvider.notifier);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
      decoration: const BoxDecoration(
        color: AppColors.logHeaderBackground,
        border: Border(bottom: BorderSide(color: AppColors.borderLight)),
      ),
      child: Row(
        children: [
          Expanded(
            child: TextField(
              controller: _searchController,
              onChanged: notifier.setSearchQuery,
              style: TextStyle(
                fontSize: AppSizes.fontSM(context),
                color: AppColors.textPrimary,
              ),
              decoration: InputDecoration(
                isDense: true,
                contentPadding: const EdgeInsets.symmetric(
                  horizontal: 10,
                  vertical: 8,
                ),
                hintText: l10n.logSearchPlaceholder,
                hintStyle: TextStyle(
                  fontSize: AppSizes.fontSM(context),
                  color: AppColors.textMuted,
                ),
                filled: true,
                fillColor: AppColors.inputBackground,
                prefixIcon: Icon(
                  Icons.search,
                  size: AppSizes.iconSM(context),
                  color: AppColors.textMuted,
                ),
                suffixIcon: logData.searchQuery.isEmpty
                    ? null
                    : InkWell(
                        onTap: () {
                          _searchController.clear();
                          notifier.setSearchQuery('');
                        },
                        child: Icon(
                          Icons.close,
                          size: AppSizes.iconSM(context),
                          color: AppColors.textMuted,
                        ),
                      ),
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(4),
                  borderSide: BorderSide.none,
                ),
              ),
            ),
          ),
          if (logData.searchQuery.isNotEmpty) ...[
            const SizedBox(width: 8),
            Text(
              '${logData.filteredEntries.length}/${logData.activeEntries.length}',
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textMuted,
                fontFamily: 'monospace',
              ),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildHeader(
    BuildContext context,
    LogData logData,
    AppLocalizations l10n,
  ) {
    final notifier = ref.read(logStateControllerProvider.notifier);

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: const BoxDecoration(
        color: AppColors.logHeaderBackground,
        border: Border(bottom: BorderSide(color: AppColors.borderLight)),
      ),
      child: Row(
        children: [
          Text(
            l10n.logViewerTitle,
            style: TextStyle(
              fontSize: AppSizes.fontXL(context),
              fontWeight: FontWeight.bold,
              color: AppColors.textPrimary,
              letterSpacing: 1.0,
            ),
          ),
          const SizedBox(width: 8),
          Text(
            '${logData.activeEntries.length} ${l10n.entriesSuffix}',
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: AppColors.textMuted,
            ),
          ),
          const SizedBox(width: 8),
          _LiveBadge(label: l10n.logLiveBadge),
          const Spacer(),
          _buildHeaderAction(
            Icons.vertical_align_bottom,
            l10n.scrollToBottom,
            () {
              if (_scrollController.hasClients) {
                _scrollController.animateTo(
                  _scrollController.position.maxScrollExtent,
                  duration: const Duration(milliseconds: 300),
                  curve: Curves.easeOut,
                );
              }
            },
          ),
          const SizedBox(width: 8),
          _buildHeaderAction(Icons.folder_open, l10n.openLogsFolder, () async {
            final uri = Uri.directory(LogService.logsDirectory);
            if (await canLaunchUrl(uri)) {
              await launchUrl(uri, mode: LaunchMode.externalApplication);
            }
          }),
          const SizedBox(width: 8),
          _buildHeaderAction(
            Icons.medical_information_outlined,
            l10n.diagnosticsButton,
            () => _showDiagnostics(context),
          ),
          const SizedBox(width: 8),
          _buildHeaderAction(
            Icons.refresh,
            l10n.refreshButton,
            () => notifier.refresh(),
          ),
        ],
      ),
    );
  }

  Widget _buildHeaderAction(IconData icon, String tooltip, VoidCallback onTap) {
    return _HoverIcon(icon: icon, tooltip: tooltip, onTap: onTap);
  }

  Widget _buildTabBar(
    BuildContext context,
    LogData logData,
    AppLocalizations l10n,
  ) {
    final notifier = ref.read(logStateControllerProvider.notifier);

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
      decoration: const BoxDecoration(
        color: AppColors.logHeaderBackground,
        border: Border(bottom: BorderSide(color: AppColors.borderLight)),
      ),
      child: Row(
        children: [
          _buildTab(
            AppStrings.tabModloaderId,
            l10n.tabModloaderLabel,
            logData,
            notifier,
          ),
          const SizedBox(width: 6),
          _buildTab(
            AppStrings.tabYorhaId,
            l10n.tabYorhaLabel,
            logData,
            notifier,
          ),
        ],
      ),
    );
  }

  Widget _buildTab(
    String id,
    String label,
    LogData logData,
    LogStateController notifier,
  ) {
    final isActive = logData.activeTab == id;
    final count = id == AppStrings.tabModloaderId
        ? logData.modloaderEntries.length
        : logData.yorhaEntries.length;

    return _HoverTab(
      label: label,
      count: count,
      isActive: isActive,
      onTap: () => notifier.setActiveTab(id),
    );
  }

  Widget _buildLogList(List<LogEntry> entries) {
    return SelectionArea(
      child: Scrollbar(
        controller: _scrollController,
        thumbVisibility: true,
        child: ListView.builder(
          controller: _scrollController,
          padding: const EdgeInsets.symmetric(vertical: 2),
          itemCount: entries.length,
          itemBuilder: (context, index) =>
              _buildLogEntry(context, entries[index], index),
        ),
      ),
    );
  }

  Widget _buildLogEntry(BuildContext context, LogEntry entry, int index) {
    final isWarn = entry.level == 'WARN' || entry.level == 'ERROR';
    final bgColor = isWarn
        ? _levelColor(entry.level).withValues(alpha: 0.08)
        : index.isEven
        ? Colors.transparent
        : AppColors.surfaceLight.withValues(alpha: 0.6);

    final shortModule = entry.module.contains('::')
        ? entry.module.split('::').last
        : entry.module;

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
      decoration: BoxDecoration(
        color: bgColor,
        border: isWarn
            ? Border(
                left: BorderSide(color: _levelColor(entry.level), width: 3),
              )
            : null,
      ),
      child: Text.rich(
        TextSpan(
          children: [
            TextSpan(
              text: '${entry.timestamp}  ',
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textMuted,
                fontFamily: 'monospace',
              ),
            ),
            TextSpan(
              text: entry.level.padRight(5),
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                fontWeight: FontWeight.bold,
                color: _levelColor(entry.level),
                fontFamily: 'monospace',
              ),
            ),
            TextSpan(
              text: '  $shortModule  ',
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.accentSecondary,
                fontFamily: 'monospace',
              ),
            ),
            TextSpan(
              text: entry.message,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.logText,
                fontFamily: 'monospace',
              ),
            ),
            const TextSpan(text: '\n', style: TextStyle(fontSize: 0.1)),
          ],
        ),
      ),
    );
  }

  Color _levelColor(String level) {
    switch (level) {
      case 'INFO':
        return AppColors.success;
      case 'WARN':
        return AppColors.warning;
      case 'ERROR':
        return AppColors.error;
      case 'DEBUG':
        return const Color(0xFF64B5F6);
      default:
        return AppColors.textMuted;
    }
  }
}

class _HoverIcon extends StatefulWidget {
  final IconData icon;
  final String tooltip;
  final VoidCallback onTap;

  const _HoverIcon({
    required this.icon,
    required this.tooltip,
    required this.onTap,
  });

  @override
  State<_HoverIcon> createState() => _HoverIconState();
}

class _HoverIconState extends State<_HoverIcon> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return Tooltip(
      message: widget.tooltip,
      child: MouseRegion(
        cursor: SystemMouseCursors.click,
        onEnter: (_) => setState(() => _hovered = true),
        onExit: (_) => setState(() => _hovered = false),
        child: GestureDetector(
          onTap: widget.onTap,
          child: Container(
            padding: const EdgeInsets.all(6),
            decoration: BoxDecoration(
              color: _hovered ? AppColors.surfaceLight : Colors.transparent,
              borderRadius: BorderRadius.circular(4),
            ),
            child: Icon(
              widget.icon,
              size: AppSizes.iconMD(context),
              color: _hovered
                  ? AppColors.accentPrimary
                  : AppColors.textSecondary,
            ),
          ),
        ),
      ),
    );
  }
}

class _HoverTab extends StatefulWidget {
  final String label;
  final int count;
  final bool isActive;
  final VoidCallback onTap;

  const _HoverTab({
    required this.label,
    required this.count,
    required this.isActive,
    required this.onTap,
  });

  @override
  State<_HoverTab> createState() => _HoverTabState();
}

class _HoverTabState extends State<_HoverTab> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: widget.onTap,
      child: MouseRegion(
        cursor: SystemMouseCursors.click,
        onEnter: (_) => setState(() => _hovered = true),
        onExit: (_) => setState(() => _hovered = false),
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
          decoration: BoxDecoration(
            color: widget.isActive
                ? AppColors.accentPrimary
                : _hovered
                ? AppColors.surfaceLight
                : Colors.transparent,
            borderRadius: BorderRadius.circular(4),
            border: Border.all(
              color: widget.isActive
                  ? AppColors.accentPrimary
                  : _hovered
                  ? AppColors.borderMedium
                  : AppColors.borderLight,
            ),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                widget.label,
                style: TextStyle(
                  fontSize: AppSizes.fontSM(context),
                  fontWeight: widget.isActive
                      ? FontWeight.bold
                      : FontWeight.normal,
                  color: widget.isActive
                      ? AppColors.buttonText
                      : _hovered
                      ? AppColors.textPrimary
                      : AppColors.textSecondary,
                ),
              ),
              const SizedBox(width: 4),
              Text(
                '(${widget.count})',
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  color: widget.isActive
                      ? AppColors.buttonText.withValues(alpha: 0.7)
                      : AppColors.textMuted,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _LiveBadge extends StatefulWidget {
  final String label;
  const _LiveBadge({required this.label});

  @override
  State<_LiveBadge> createState() => _LiveBadgeState();
}

class _LiveBadgeState extends State<_LiveBadge>
    with SingleTickerProviderStateMixin {
  late final AnimationController _pulse;

  @override
  void initState() {
    super.initState();
    _pulse = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1100),
    )..repeat(reverse: true);
  }

  @override
  void dispose() {
    _pulse.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
      decoration: BoxDecoration(
        color: AppColors.success.withValues(alpha: 0.12),
        borderRadius: BorderRadius.circular(3),
        border: Border.all(
          color: AppColors.success.withValues(alpha: 0.4),
        ),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          AnimatedBuilder(
            animation: _pulse,
            builder: (_, __) => Container(
              width: 6,
              height: 6,
              decoration: BoxDecoration(
                color: AppColors.success
                    .withValues(alpha: 0.4 + 0.6 * _pulse.value),
                shape: BoxShape.circle,
              ),
            ),
          ),
          const SizedBox(width: 4),
          Text(
            widget.label,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              fontWeight: FontWeight.bold,
              color: AppColors.success,
              letterSpacing: 0.6,
            ),
          ),
        ],
      ),
    );
  }
}

class _DiagnosticsLoadingDialog extends StatelessWidget {
  const _DiagnosticsLoadingDialog();

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return Dialog(
      backgroundColor: Colors.transparent,
      child: Container(
        padding: EdgeInsets.symmetric(
          horizontal: AppSizes.paddingXL(context),
          vertical: AppSizes.paddingLG(context),
        ),
        decoration: BoxDecoration(
          color: AppColors.backgroundCard,
          borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
          border: Border.all(
            color: AppColors.accentPrimary.withValues(alpha: 0.4),
          ),
          boxShadow: const [
            BoxShadow(
              color: AppColors.shadow,
              blurRadius: 12,
              offset: Offset(0, 4),
            ),
          ],
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            const SizedBox(
              width: 22,
              height: 22,
              child: CircularProgressIndicator(
                strokeWidth: 2,
                color: AppColors.accentPrimary,
              ),
            ),
            SizedBox(width: AppSizes.spacingLG(context)),
            Text(
              l10n.diagnosticsCollecting,
              style: TextStyle(
                fontSize: AppSizes.fontMD(context),
                color: AppColors.textPrimary,
                letterSpacing: 0.4,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _DiagnosticsDialog extends ConsumerStatefulWidget {
  final DiagnosticsReport report;
  final AppLocalizations l10n;
  const _DiagnosticsDialog({required this.report, required this.l10n});

  @override
  ConsumerState<_DiagnosticsDialog> createState() => _DiagnosticsDialogState();
}

class _DiagnosticsDialogState extends ConsumerState<_DiagnosticsDialog> {
  bool _saving = false;
  final ScrollController _scrollController = ScrollController();

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  String _localTime(DateTime dt) {
    String two(int n) => n.toString().padLeft(2, '0');
    return '${dt.year}-${two(dt.month)}-${two(dt.day)} '
        '${two(dt.hour)}:${two(dt.minute)}:${two(dt.second)}';
  }

  @override
  Widget build(BuildContext context) {
    final l10n = widget.l10n;
    final r = widget.report;
    final summary = DiagnosticsService.formatSummary(r);

    return Dialog(
      backgroundColor: Colors.transparent,
      insetPadding: const EdgeInsets.all(24),
      child: Container(
        constraints: const BoxConstraints(maxWidth: 700, maxHeight: 640),
        decoration: BoxDecoration(
          color: AppColors.backgroundCard,
          borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
          border: Border.all(
            color: AppColors.accentPrimary.withValues(alpha: 0.45),
          ),
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
            _buildHeader(context, l10n, r),
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
                    children: _buildSections(context, r),
                  ),
                ),
              ),
            ),
            Container(
              height: 1,
              color: AppColors.borderLight,
            ),
            Padding(
              padding: EdgeInsets.symmetric(
                horizontal: AppSizes.paddingLG(context),
                vertical: AppSizes.paddingSM(context),
              ),
              child: Row(
                children: [
                  _diagButton(
                    context: context,
                    icon: Icons.copy,
                    label: l10n.diagnosticsCopy,
                    onTap: () async {
                      await Clipboard.setData(ClipboardData(text: summary));
                      ref
                          .read(notificationStateControllerProvider.notifier)
                          .addNotification(NotificationItem(
                            id: 'diag_copied_${DateTime.now().millisecondsSinceEpoch}',
                            message: (l10n) => l10n.diagnosticsCopied,
                            icon: Icons.check_circle,
                            color: AppColors.success,
                            type: NotificationType.general,
                          ));
                    },
                  ),
                  SizedBox(width: AppSizes.spacingMD(context)),
                  _diagButton(
                    context: context,
                    icon: Icons.save,
                    label: l10n.diagnosticsSaveFull,
                    primary: true,
                    busy: _saving,
                    onTap: _saving
                        ? null
                        : () async {
                            setState(() => _saving = true);
                            try {
                              final outPath =
                                  await DiagnosticsService.writeFullReport(r);
                              if (!mounted) return;
                              ref
                                  .read(notificationStateControllerProvider.notifier)
                                  .addNotification(NotificationItem(
                                    id: 'diag_saved_${DateTime.now().millisecondsSinceEpoch}',
                                    message: (l10n) => l10n.diagnosticsSavedAt(outPath),
                                    icon: Icons.save,
                                    color: AppColors.success,
                                    type: NotificationType.general,
                                  ));
                            } finally {
                              if (mounted) setState(() => _saving = false);
                            }
                          },
                  ),
                  const Spacer(),
                  TextButton(
                    onPressed: () => Navigator.of(context).pop(),
                    child: Text(
                      l10n.diagnosticsClose,
                      style: TextStyle(
                        color: AppColors.textMuted,
                        fontSize: AppSizes.fontSM(context),
                        letterSpacing: 0.4,
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildHeader(
    BuildContext context,
    AppLocalizations l10n,
    DiagnosticsReport r,
  ) {
    return Container(
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.paddingLG(context),
        vertical: AppSizes.paddingMD(context),
      ),
      decoration: BoxDecoration(
        color: AppColors.surfaceMedium,
        border: const Border(
          bottom: BorderSide(color: AppColors.borderLight),
        ),
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
              color: AppColors.accentPrimary.withValues(alpha: 0.12),
              borderRadius: BorderRadius.circular(6),
              border: Border.all(
                color: AppColors.accentPrimary.withValues(alpha: 0.5),
              ),
            ),
            child: const Icon(
              Icons.medical_information_outlined,
              size: 20,
              color: AppColors.accentPrimary,
            ),
          ),
          SizedBox(width: AppSizes.spacingMD(context)),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  l10n.diagnosticsTitle.toUpperCase(),
                  style: TextStyle(
                    fontSize: AppSizes.fontLG(context),
                    fontWeight: FontWeight.bold,
                    color: AppColors.textPrimary,
                    letterSpacing: 1.2,
                  ),
                ),
                SizedBox(height: AppSizes.spacingSM(context) / 2),
                Text(
                  '${r.launcherInfo['Launcher version'] ?? '?'}  ·  ${_localTime(r.generatedAt)}',
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.textMuted,
                    letterSpacing: 0.3,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  List<Widget> _buildSections(BuildContext context, DiagnosticsReport r) {
    final l10n = AppLocalizations.of(context)!;
    final modKinds = <ModKind, int>{};
    for (final m in r.mods) {
      modKinds[m.kind] = (modKinds[m.kind] ?? 0) + 1;
    }
    return [
      _sectionCard(
        context,
        icon: Icons.computer_outlined,
        title: l10n.logSectionSystem,
        children: [
          _kvRow(context, l10n.logDetailOs,
              '${r.systemInfo['OS'] ?? '?'} ${r.systemInfo['OS version'] ?? ''}'),
          _kvRow(context, l10n.logDetailLocale, r.systemInfo['Locale'] ?? '?'),
        ],
      ),
      SizedBox(height: AppSizes.spacingMD(context)),
      _sectionCard(
        context,
        icon: Icons.extension_outlined,
        title: l10n.logSectionModsNams,
        trailing: _countBadge(r.mods.length),
        children: [
          if (modKinds.isEmpty)
            _mutedLine(context, l10n.logNoModsInstalled)
          else
            Wrap(
              spacing: 6,
              runSpacing: 6,
              children: modKinds.entries
                  .map((e) => _chip(
                        context,
                        '${e.key.name}: ${e.value}',
                        AppColors.accentPrimary,
                      ))
                  .toList(),
            ),
          if (r.disabledModsEntries.isNotEmpty) ...[
            SizedBox(height: AppSizes.spacingSM(context)),
            _mutedLine(context,
                '${r.disabledModsEntries.length} disabled prefix(es) in disabled_mods.toml'),
          ],
        ],
      ),
      SizedBox(height: AppSizes.spacingMD(context)),
      _sectionCard(
        context,
        icon: Icons.movie_creation_outlined,
        title: l10n.logSectionCutscenes,
        trailing: _countBadge(r.cutsceneMods.length),
        children: [
          Wrap(
            spacing: 6,
            runSpacing: 6,
            children: [
              _flagChip(context, 'HD', r.cutsceneDetection.hasHdCutscenes),
              _flagChip(context, 'H264', r.cutsceneDetection.needsH264),
              if (r.directOverrides.isNotEmpty)
                _chip(
                  context,
                  'data/movie overrides: ${r.directOverrides.length}',
                  AppColors.warning,
                ),
            ],
          ),
        ],
      ),
      SizedBox(height: AppSizes.spacingMD(context)),
      _sectionCard(
        context,
        icon: Icons.texture,
        title: l10n.logSectionTextures,
        children: [
          Wrap(
            spacing: 6,
            runSpacing: 6,
            children: [
              _chip(context, 'NAMS: ${r.namsTextures.length}',
                  AppColors.accentPrimary),
              if (r.skResTextures.isNotEmpty)
                _chip(context, 'SK_Res: ${r.skResTextures.length}',
                    AppColors.warning),
              if (r.waxTextures.isNotEmpty)
                _chip(context, 'WAX: ${r.waxTextures.length}',
                    AppColors.warning),
            ],
          ),
        ],
      ),
      if (r.dataDirContents.isNotEmpty) ...[
        SizedBox(height: AppSizes.spacingMD(context)),
        _sectionCard(
          context,
          icon: Icons.warning_amber_rounded,
          title: l10n.diagnosticsSectionDataOverlay,
          accent: AppColors.warning,
          children: [
            for (final e in (r.dataDirContents.entries.toList()
                  ..sort((a, b) => b.value.compareTo(a.value)))
                .take(8))
              _kvRow(context, '${e.key}/', l10n.diagnosticsFileCount(e.value)),
            if (r.dataDirContents.length > 8)
              _mutedLine(
                context,
                l10n.diagnosticsMoreItems(r.dataDirContents.length - 8),
              ),
          ],
        ),
      ],
      if (r.waxModsRoot.isNotEmpty ||
          r.skResPacks.isNotEmpty ||
          r.reshadeContents.isNotEmpty) ...[
        SizedBox(height: AppSizes.spacingMD(context)),
        _sectionCard(
          context,
          icon: Icons.folder_outlined,
          title: l10n.diagnosticsSectionExternalLegacy,
          children: [
            if (r.waxModsRoot.isNotEmpty)
              _kvRow(context, 'wax/mods/', r.waxModsRoot.join(', ')),
            if (r.skResPacks.isNotEmpty)
              _kvRow(context, 'SK_Res packs', r.skResPacks.join(', ')),
            if (r.reshadeContents.isNotEmpty)
              _kvRow(context, 'reshade/', r.reshadeContents.join(', ')),
          ],
        ),
      ],
      if (r.gameRootExtras.isNotEmpty) ...[
        SizedBox(height: AppSizes.spacingMD(context)),
        _sectionCard(
          context,
          icon: Icons.travel_explore,
          title: l10n.diagnosticsSectionGameRootExtras,
          accent: AppColors.accentPrimary,
          children: [
            for (final name in r.gameRootExtras.take(20))
              _mutedLine(context, '• $name'),
            if (r.gameRootExtras.length > 20)
              _mutedLine(
                context,
                l10n.diagnosticsMoreItems(r.gameRootExtras.length - 20),
              ),
          ],
        ),
      ],
      SizedBox(height: AppSizes.spacingMD(context)),
    ];
  }

  Widget _sectionCard(
    BuildContext context, {
    required IconData icon,
    required String title,
    Color? accent,
    Widget? trailing,
    required List<Widget> children,
  }) {
    final color = accent ?? AppColors.accentPrimary;
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
              color: color.withValues(alpha: 0.08),
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
                Icon(icon, size: 14, color: color),
                SizedBox(width: AppSizes.spacingSM(context)),
                Expanded(
                  child: Text(
                    title,
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      fontWeight: FontWeight.bold,
                      color: color,
                      letterSpacing: 1.0,
                    ),
                  ),
                ),
                if (trailing != null) trailing,
              ],
            ),
          ),
          Padding(
            padding: EdgeInsets.symmetric(
              horizontal: AppSizes.paddingMD(context),
              vertical: AppSizes.paddingSM(context),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: children,
            ),
          ),
        ],
      ),
    );
  }

  Widget _countBadge(int n) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 1),
      decoration: BoxDecoration(
        color: AppColors.accentPrimary.withValues(alpha: 0.18),
        borderRadius: BorderRadius.circular(3),
        border: Border.all(
          color: AppColors.accentPrimary.withValues(alpha: 0.4),
        ),
      ),
      child: Text(
        '$n',
        style: const TextStyle(
          fontSize: 10,
          fontWeight: FontWeight.bold,
          color: AppColors.accentPrimary,
          fontFamily: 'monospace',
        ),
      ),
    );
  }

  Widget _chip(BuildContext context, String label, Color color) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.12),
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: color.withValues(alpha: 0.4)),
      ),
      child: Text(
        label,
        style: TextStyle(
          fontSize: AppSizes.fontXS(context),
          color: color,
          fontWeight: FontWeight.w600,
          letterSpacing: 0.4,
        ),
      ),
    );
  }

  Widget _flagChip(BuildContext context, String label, bool on) {
    final color = on ? AppColors.success : AppColors.textMuted;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.12),
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: color.withValues(alpha: 0.4)),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(
            on ? Icons.check_circle : Icons.remove_circle_outline,
            size: 12,
            color: color,
          ),
          const SizedBox(width: 4),
          Text(
            label,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: color,
              fontWeight: FontWeight.bold,
              letterSpacing: 0.4,
            ),
          ),
        ],
      ),
    );
  }

  Widget _kvRow(BuildContext context, String key, String value) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 3),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 110,
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
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _mutedLine(BuildContext context, String text) {
    return Text(
      text,
      style: TextStyle(
        fontSize: AppSizes.fontXS(context),
        color: AppColors.textMuted,
        fontStyle: FontStyle.italic,
      ),
    );
  }

  Widget _diagButton({
    required BuildContext context,
    required IconData icon,
    required String label,
    required VoidCallback? onTap,
    bool primary = false,
    bool busy = false,
  }) {
    final foreground = primary ? AppColors.buttonText : AppColors.accentPrimary;
    final background =
        primary ? AppColors.accentPrimary : Colors.transparent;
    final border = primary
        ? AppColors.accentPrimary
        : AppColors.accentPrimary.withValues(alpha: 0.5);
    return Material(
      color: Colors.transparent,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(4),
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          decoration: BoxDecoration(
            color: background,
            borderRadius: BorderRadius.circular(4),
            border: Border.all(color: border),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              busy
                  ? SizedBox(
                      width: 14,
                      height: 14,
                      child: CircularProgressIndicator(
                        strokeWidth: 2,
                        color: foreground,
                      ),
                    )
                  : Icon(icon, size: 14, color: foreground),
              const SizedBox(width: 6),
              Text(
                label,
                style: TextStyle(
                  fontSize: AppSizes.fontSM(context),
                  fontWeight: FontWeight.bold,
                  color: foreground,
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

