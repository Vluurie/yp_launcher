import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/log_state.dart';
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
