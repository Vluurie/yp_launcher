import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:url_launcher/url_launcher.dart';
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
  }

  @override
  void dispose() {
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
    final logData = ref.watch(logStateControllerProvider);

    return SlideTransition(
      position: _slideAnimation,
      child: Container(
        width: AppSizes.logPanelWidth,
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
            _buildHeader(logData),
            _buildTabBar(logData),
            Expanded(
              child: logData.isLoading
                  ? const Center(
                      child: CircularProgressIndicator(
                        color: AppColors.accentPrimary,
                      ),
                    )
                  : logData.activeEntries.isEmpty
                      ? Center(
                          child: Text(
                            'No log entries found',
                            style: TextStyle(
                              fontSize: AppSizes.fontMD,
                              color: AppColors.textMuted,
                            ),
                          ),
                        )
                      : _buildLogList(logData.activeEntries),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildHeader(LogData logData) {
    final notifier = ref.read(logStateControllerProvider.notifier);

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: const BoxDecoration(
        color: AppColors.logHeaderBackground,
        border: Border(
          bottom: BorderSide(color: AppColors.borderLight),
        ),
      ),
      child: Row(
        children: [
          Text(
            'LOG VIEWER',
            style: TextStyle(
              fontSize: AppSizes.fontXL,
              fontWeight: FontWeight.bold,
              color: AppColors.textPrimary,
              letterSpacing: 1.0,
            ),
          ),
          const SizedBox(width: 8),
          Text(
            '${logData.activeEntries.length} entries',
            style: TextStyle(
              fontSize: AppSizes.fontXS,
              color: AppColors.textMuted,
            ),
          ),
          const Spacer(),
          _buildHeaderAction(Icons.vertical_align_bottom, 'Scroll to bottom', () {
            if (_scrollController.hasClients) {
              _scrollController.animateTo(
                _scrollController.position.maxScrollExtent,
                duration: const Duration(milliseconds: 300),
                curve: Curves.easeOut,
              );
            }
          }),
          const SizedBox(width: 2),
          _buildHeaderAction(Icons.folder_open, 'Open logs folder', () async {
            final uri = Uri.directory(LogService.logsDirectory);
            if (await canLaunchUrl(uri)) {
              await launchUrl(uri, mode: LaunchMode.externalApplication);
            }
          }),
          const SizedBox(width: 2),
          _buildHeaderAction(Icons.refresh, 'Refresh', () => notifier.refresh()),
        ],
      ),
    );
  }

  Widget _buildHeaderAction(IconData icon, String tooltip, VoidCallback onTap) {
    return Tooltip(
      message: tooltip,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(4),
        child: Padding(
          padding: const EdgeInsets.all(4),
          child: Icon(
            icon,
            size: AppSizes.iconMD,
            color: AppColors.textSecondary,
          ),
        ),
      ),
    );
  }

  Widget _buildTabBar(LogData logData) {
    final notifier = ref.read(logStateControllerProvider.notifier);

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: const BoxDecoration(
        color: AppColors.logHeaderBackground,
        border: Border(
          bottom: BorderSide(color: AppColors.borderLight),
        ),
      ),
      child: Row(
        children: [
          _buildTab('modloader', 'Modloader', logData, notifier),
          const SizedBox(width: 4),
          _buildTab('yorha', 'YoRHa Protocol', logData, notifier),
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
    final count = id == 'modloader'
        ? logData.modloaderEntries.length
        : logData.yorhaEntries.length;

    return GestureDetector(
      onTap: () => notifier.setActiveTab(id),
      child: MouseRegion(
        cursor: SystemMouseCursors.click,
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
          decoration: BoxDecoration(
            color: isActive ? AppColors.accentPrimary : Colors.transparent,
            borderRadius: BorderRadius.circular(4),
            border: Border.all(
              color: isActive ? AppColors.accentPrimary : AppColors.borderLight,
            ),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                label,
                style: TextStyle(
                  fontSize: AppSizes.fontSM,
                  fontWeight: isActive ? FontWeight.bold : FontWeight.normal,
                  color: isActive ? AppColors.buttonText : AppColors.textSecondary,
                ),
              ),
              const SizedBox(width: 4),
              Text(
                '($count)',
                style: TextStyle(
                  fontSize: AppSizes.fontXS,
                  color: isActive
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
              _buildLogEntry(entries[index], index),
        ),
      ),
    );
  }

  Widget _buildLogEntry(LogEntry entry, int index) {
    final isWarn = entry.level == 'WARN' || entry.level == 'ERROR';
    final bgColor = isWarn
        ? _levelColor(entry.level).withValues(alpha: 0.08)
        : index.isEven
            ? Colors.transparent
            : AppColors.surfaceLight.withValues(alpha: 0.4);

    // Show only the last segment of the module path
    final shortModule = entry.module.contains('::')
        ? entry.module.split('::').last
        : entry.module;

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 3),
      decoration: BoxDecoration(
        color: bgColor,
        border: isWarn
            ? Border(
                left: BorderSide(
                  color: _levelColor(entry.level),
                  width: 3,
                ),
              )
            : null,
      ),
      child: Text.rich(
        TextSpan(
          children: [
            // Timestamp
            TextSpan(
              text: '${entry.timestamp}  ',
              style: TextStyle(
                fontSize: AppSizes.fontXS,
                color: AppColors.textMuted,
                fontFamily: 'monospace',
              ),
            ),
            // Level
            TextSpan(
              text: entry.level.padRight(5),
              style: TextStyle(
                fontSize: AppSizes.fontXS,
                fontWeight: FontWeight.bold,
                color: _levelColor(entry.level),
                fontFamily: 'monospace',
              ),
            ),
            // Module (short)
            TextSpan(
              text: '  $shortModule  ',
              style: TextStyle(
                fontSize: AppSizes.fontXS,
                color: AppColors.accentSecondary,
                fontFamily: 'monospace',
              ),
            ),
            // Message
            TextSpan(
              text: entry.message,
              style: TextStyle(
                fontSize: AppSizes.fontXS,
                color: AppColors.logText,
                fontFamily: 'monospace',
              ),
            ),
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
