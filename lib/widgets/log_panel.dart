import 'package:flutter/material.dart';
import 'package:yp_launcher/theme/nier_curves.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/log_state.dart';
import 'package:yp_launcher/services/log_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/autoscroll_selection.dart';

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
  final ScrollController _hScrollController = ScrollController();
  final TextEditingController _searchController = TextEditingController();
  bool _autoScroll = true;
  int _lastEntryCount = 0;
  double? _width;
  bool _wrap = true;

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
    ).animate(CurvedAnimation(parent: _controller, curve: NierCurves.smooth));
    _controller.forward();
    _scrollController.addListener(_onScroll);
  }

  void _onScroll() {
    if (!_scrollController.hasClients) return;
    final atBottom =
        _scrollController.offset >=
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
    _hScrollController.dispose();
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

    final screenWidth = MediaQuery.of(context).size.width;
    final maxWidth = screenWidth - 120;
    final width = (_width ?? screenWidth / 2).clamp(
      320.0,
      maxWidth < 320 ? 320.0 : maxWidth,
    );

    return SlideTransition(
      position: _slideAnimation,
      child: Row(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _ResizeHandle(
            onDrag: (delta) => setState(() => _width = width - delta),
          ),
          _panel(context, logData, filtered, l10n, width),
        ],
      ),
    );
  }

  Widget _panel(
    BuildContext context,
    LogData logData,
    List<LogEntry> filtered,
    AppLocalizations l10n,
    double width,
  ) {
    return Container(
      width: width,
      decoration: BoxDecoration(
        color: AppColors.logBackground,
        border: Border(
          left: BorderSide(color: AppColors.borderMedium, width: 1.5),
        ),
        boxShadow: [
          BoxShadow(
            color: AppColors.shadow,
            blurRadius: 8,
            offset: const Offset(-2, 0),
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
                ? Center(
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
      decoration: BoxDecoration(
        color: AppColors.logHeaderBackground,
        border: Border(bottom: BorderSide(color: AppColors.borderLight)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
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
                    fontFamily: AppSizes.monoFamily,
                    fontFamilyFallback: AppSizes.monoFallback,
                  ),
                ),
              ],
            ],
          ),
          const SizedBox(height: 6),
          Builder(
            builder: (context) {
              final counts = logData.levelCounts;
              return Row(
                children: [
                  for (final level in logLevels) ...[
                    _LevelChip(
                      level: level,
                      count: counts[level] ?? 0,
                      shown: logData.isLevelShown(level),
                      color: _levelColor(level),
                      onTap: () => notifier.toggleLevel(level),
                    ),
                    const SizedBox(width: 4),
                  ],
                ],
              );
            },
          ),
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
      decoration: BoxDecoration(
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
            _wrap ? Icons.wrap_text : Icons.short_text,
            _wrap ? l10n.logWrapOff : l10n.logWrapOn,
            () => setState(() => _wrap = !_wrap),
          ),
          const SizedBox(width: 8),
          _buildHeaderAction(
            Icons.vertical_align_bottom,
            l10n.scrollToBottom,
            () {
              if (_scrollController.hasClients) {
                _scrollController.animateTo(
                  _scrollController.position.maxScrollExtent,
                  duration: const Duration(milliseconds: 300),
                  curve: NierCurves.fill,
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
      decoration: BoxDecoration(
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
    final lineHeight = AppSizes.fontXS(context) * 1.35;

    return AutoScrollSelection(
      controller: _scrollController,
      child: SelectionArea(
        child: Scrollbar(
          controller: _scrollController,
          thumbVisibility: true,
          child: ListView.builder(
            controller: _scrollController,
            padding: const EdgeInsets.symmetric(vertical: 2),
            itemCount: entries.length,
            itemExtent: _wrap ? null : lineHeight * 2 + 12,
            addAutomaticKeepAlives: false,
            addRepaintBoundaries: false,
            itemBuilder: (context, index) =>
                _buildLogEntry(context, entries[index], index),
          ),
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
      alignment: Alignment.centerLeft,
      decoration: BoxDecoration(
        color: bgColor,
        border: isWarn
            ? Border(
                left: BorderSide(color: _levelColor(entry.level), width: 3),
              )
            : null,
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          Row(
            children: [
              Text(
                entry.timestamp,
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  color: AppColors.textMuted,
                  fontFamily: AppSizes.monoFamily,
                  fontFamilyFallback: AppSizes.monoFallback,
                ),
              ),
              const SizedBox(width: 8),
              Text(
                entry.level,
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  fontWeight: FontWeight.bold,
                  color: _levelColor(entry.level),
                  fontFamily: AppSizes.monoFamily,
                  fontFamilyFallback: AppSizes.monoFallback,
                ),
              ),
              const SizedBox(width: 8),
              Flexible(
                child: Text(
                  shortModule,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.accentSecondary,
                    fontFamily: AppSizes.monoFamily,
                    fontFamilyFallback: AppSizes.monoFallback,
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: 2),
          Text(
            entry.message,
            maxLines: _wrap ? null : 1,
            softWrap: _wrap,
            overflow: _wrap ? null : TextOverflow.ellipsis,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: AppColors.logText,
              height: 1.35,
              fontFamily: AppSizes.monoFamily,
              fontFamilyFallback: AppSizes.monoFallback,
            ),
          ),
        ],
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

class _LevelChip extends StatefulWidget {
  final String level;
  final int count;
  final bool shown;
  final Color color;
  final VoidCallback onTap;

  const _LevelChip({
    required this.level,
    required this.count,
    required this.shown,
    required this.color,
    required this.onTap,
  });

  @override
  State<_LevelChip> createState() => _LevelChipState();
}

class _LevelChipState extends State<_LevelChip> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final shown = widget.shown;
    final color = widget.color;

    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          padding: const EdgeInsets.symmetric(horizontal: 7, vertical: 3),
          decoration: BoxDecoration(
            color: shown
                ? color.withValues(alpha: _hovered ? 0.28 : 0.18)
                : _hovered
                ? AppColors.surfaceLight
                : Colors.transparent,
            borderRadius: BorderRadius.circular(3),
            border: Border.all(
              color: shown
                  ? color.withValues(alpha: 0.6)
                  : AppColors.borderLight,
            ),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                widget.level,
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  fontWeight: FontWeight.bold,
                  letterSpacing: 0.4,
                  color: shown ? color : AppColors.textMuted,
                  fontFamily: AppSizes.monoFamily,
                  fontFamilyFallback: AppSizes.monoFallback,
                ),
              ),
              if (widget.count > 0) ...[
                const SizedBox(width: 5),
                Text(
                  '${widget.count}',
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: shown
                        ? color.withValues(alpha: 0.8)
                        : AppColors.textMuted,
                    fontFamily: AppSizes.monoFamily,
                    fontFamilyFallback: AppSizes.monoFallback,
                  ),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }
}

class _ResizeHandle extends StatefulWidget {
  final ValueChanged<double> onDrag;

  const _ResizeHandle({required this.onDrag});

  @override
  State<_ResizeHandle> createState() => _ResizeHandleState();
}

class _ResizeHandleState extends State<_ResizeHandle> {
  bool _hovered = false;
  bool _dragging = false;

  @override
  Widget build(BuildContext context) {
    final active = _hovered || _dragging;

    return MouseRegion(
      cursor: SystemMouseCursors.resizeLeftRight,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onHorizontalDragStart: (_) => setState(() => _dragging = true),
        onHorizontalDragEnd: (_) => setState(() => _dragging = false),
        onHorizontalDragCancel: () => setState(() => _dragging = false),
        onHorizontalDragUpdate: (details) => widget.onDrag(details.delta.dx),
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          width: 6,
          color: active
              ? AppColors.accentPrimary.withValues(alpha: 0.5)
              : Colors.transparent,
        ),
      ),
    );
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
        border: Border.all(color: AppColors.success.withValues(alpha: 0.4)),
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
                color: AppColors.success.withValues(
                  alpha: 0.4 + 0.6 * _pulse.value,
                ),
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
