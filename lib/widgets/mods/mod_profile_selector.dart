import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/mod_profile.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/mod_profiles_state.dart';
import 'package:yp_launcher/providers/notification_state.dart';
import 'package:yp_launcher/services/mod_profiles_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/app_dropdown.dart';
import 'package:yp_launcher/widgets/header_info_icon.dart';
import 'package:yp_launcher/widgets/hover_button.dart';

class ModProfileSelector extends ConsumerWidget {
  const ModProfileSelector({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final state = ref.watch(modProfilesStateControllerProvider);
    final appState = ref.watch(appStateControllerProvider);
    final gameDir = appState.selectedDirectory;
    final gameRunning = appState.playButtonState == PlayButtonState.running ||
        appState.playButtonState == PlayButtonState.loading;

    final profiles = [...state.profiles]
      ..sort((a, b) => a.name.toLowerCase().compareTo(b.name.toLowerCase()));
    final canDelete =
        profiles.length > 1 && state.activeName.isNotEmpty && !gameRunning;
    final canAct = !state.isLoading && gameDir.isNotEmpty && !gameRunning;

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
            l10n.headerMods,
            style: TextStyle(
              fontSize: AppSizes.fontXL(context),
              fontWeight: FontWeight.bold,
              color: AppColors.textPrimary,
              letterSpacing: 1.0,
            ),
          ),
          if (gameDir.isNotEmpty)
            HeaderInfoIcon(
              tooltip: l10n.modIntroBody,
              revealPath: p.join(gameDir, 'nams', 'mods'),
              isFile: false,
            ),
          SizedBox(width: AppSizes.spacingLG(context)),
          Text(
            l10n.modProfileLabel,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: AppColors.textMuted,
              letterSpacing: 0.6,
              fontWeight: FontWeight.bold,
            ),
          ),
          SizedBox(width: AppSizes.spacingSM(context)),
          Flexible(
            child: _buildDropdown(context, ref, profiles, state, gameDir, canAct),
          ),
          if (state.isLoading) ...[
            SizedBox(width: AppSizes.spacingSM(context)),
            const SizedBox(
              width: 14,
              height: 14,
              child: CircularProgressIndicator(
                strokeWidth: 2,
                color: AppColors.accentPrimary,
              ),
            ),
          ],
          SizedBox(width: AppSizes.spacingMD(context)),
          _iconAction(
            context,
            icon: Icons.add,
            tooltip: gameRunning
                ? l10n.modProfileLockedRunning
                : l10n.modProfileNewButton,
            color: AppColors.accentPrimary,
            enabled: canAct,
            onTap: () => _showNewDialog(context, ref, state, gameDir),
          ),
          SizedBox(width: AppSizes.paddingXS(context)),
          _iconAction(
            context,
            icon: Icons.edit_outlined,
            tooltip: gameRunning
                ? l10n.modProfileLockedRunning
                : l10n.modProfileRenameButton,
            color: AppColors.accentPrimary,
            enabled: canAct && profiles.isNotEmpty,
            onTap: () => _showRenameDialog(context, ref, state, gameDir),
          ),
          SizedBox(width: AppSizes.paddingXS(context)),
          _iconAction(
            context,
            icon: Icons.delete_outline,
            tooltip: gameRunning
                ? l10n.modProfileLockedRunning
                : (canDelete
                    ? l10n.modProfileDeleteButton
                    : l10n.modProfileErrorDeleteLast),
            color: AppColors.error,
            enabled: canAct && canDelete,
            onTap: () => _showDeleteDialog(context, ref, state, gameDir),
          ),
        ],
      ),
    );
  }

  Widget _iconAction(
    BuildContext context, {
    required IconData icon,
    required String tooltip,
    required Color color,
    required bool enabled,
    required VoidCallback onTap,
  }) {
    final effectiveColor = enabled ? color : AppColors.textMuted;
    return HoverIconButton(
      tooltip: tooltip,
      onTap: enabled ? onTap : null,
      borderColor: effectiveColor,
      icon: Icon(
        icon,
        size: AppSizes.iconSM(context),
        color: effectiveColor,
      ),
    );
  }

  Widget _buildDropdown(
    BuildContext context,
    WidgetRef ref,
    List<ModProfile> profiles,
    ModProfileState state,
    String gameDir,
    bool canAct,
  ) {
    final names = profiles.map((p) => p.name).toList();
    final value = names.contains(state.activeName)
        ? state.activeName
        : (names.isNotEmpty ? names.first : '');
    if (names.isEmpty) return const SizedBox.shrink();
    return AppDropdown<String>(
      value: value,
      items: names,
      itemLabel: (n) => n,
      maxWidth: 200,
      highlight: true,
      onChanged: !canAct
          ? null
          : (next) async {
              if (next == state.activeName) return;
              final ok = await ref
                  .read(modProfilesStateControllerProvider.notifier)
                  .switchTo(gameDir, next);
              if (!context.mounted) return;
              _toast(
                ref,
                ok
                    ? AppLocalizations.of(context)!
                        .modProfileSwitchedToast(next)
                    : _localizedError(
                        context,
                        ref.read(modProfilesStateControllerProvider).error,
                      ),
                ok ? AppColors.success : AppColors.error,
              );
            },
    );
  }

  Future<void> _showNewDialog(
    BuildContext context,
    WidgetRef ref,
    ModProfileState state,
    String gameDir,
  ) async {
    final result = await _showNameDialog(
      context,
      title: AppLocalizations.of(context)!.modProfileNewDialogTitle,
      initial: '',
      existingNames: state.profiles.map((p) => p.name).toSet(),
    );
    if (result == null || !context.mounted) return;
    final ok = await ref
        .read(modProfilesStateControllerProvider.notifier)
        .create(gameDir, result);
    if (!context.mounted) return;
    _toast(
      ref,
      ok
          ? AppLocalizations.of(context)!.modProfileCreatedToast(result)
          : _localizedError(
              context,
              ref.read(modProfilesStateControllerProvider).error,
            ),
      ok ? AppColors.success : AppColors.error,
    );
  }

  Future<void> _showRenameDialog(
    BuildContext context,
    WidgetRef ref,
    ModProfileState state,
    String gameDir,
  ) async {
    final activeName = state.activeName;
    final result = await _showNameDialog(
      context,
      title: AppLocalizations.of(context)!.modProfileRenameDialogTitle,
      initial: activeName,
      existingNames: state.profiles
          .map((p) => p.name)
          .where((n) => n != activeName)
          .toSet(),
    );
    if (result == null || result == activeName || !context.mounted) return;
    final ok = await ref
        .read(modProfilesStateControllerProvider.notifier)
        .rename(gameDir, activeName, result);
    if (!context.mounted) return;
    _toast(
      ref,
      ok
          ? AppLocalizations.of(context)!.modProfileRenamedToast(result)
          : _localizedError(
              context,
              ref.read(modProfilesStateControllerProvider).error,
            ),
      ok ? AppColors.success : AppColors.error,
    );
  }

  Future<void> _showDeleteDialog(
    BuildContext context,
    WidgetRef ref,
    ModProfileState state,
    String gameDir,
  ) async {
    final l10n = AppLocalizations.of(context)!;
    final activeName = state.activeName;
    final candidates = state.profiles
        .where((p) => p.name != activeName)
        .map((p) => p.name)
        .toList()
      ..sort();
    if (candidates.isEmpty) return;

    final picked = await showDialog<String>(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: AppColors.backgroundCard,
        title: Text(
          l10n.modProfileDeleteDialogTitle,
          style: TextStyle(
            color: AppColors.error,
            fontSize: AppSizes.fontLG(ctx),
          ),
        ),
        content: SizedBox(
          width: 360,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              for (final name in candidates)
                Padding(
                  padding: const EdgeInsets.only(bottom: 6),
                  child: SizedBox(
                    width: double.infinity,
                    child: ElevatedButton(
                      onPressed: () => Navigator.of(ctx).pop(name),
                      style: ElevatedButton.styleFrom(
                        backgroundColor: AppColors.surfaceLight,
                        foregroundColor: AppColors.textPrimary,
                        padding: const EdgeInsets.symmetric(
                          horizontal: 14,
                          vertical: 10,
                        ),
                        shape: RoundedRectangleBorder(
                          borderRadius: BorderRadius.circular(4),
                          side: BorderSide(
                            color: AppColors.error.withValues(alpha: 0.3),
                          ),
                        ),
                      ),
                      child: Row(
                        children: [
                          const Icon(
                            Icons.folder_outlined,
                            size: 16,
                            color: AppColors.error,
                          ),
                          const SizedBox(width: 8),
                          Expanded(
                            child: Text(
                              name,
                              style: TextStyle(
                                fontSize: AppSizes.fontSM(ctx),
                              ),
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
                ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: Text(
              l10n.buttonCancel,
              style: TextStyle(
                color: AppColors.textMuted,
                fontSize: AppSizes.fontSM(ctx),
              ),
            ),
          ),
        ],
      ),
    );
    if (picked == null || !context.mounted) return;

    final confirmed = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: AppColors.backgroundCard,
        title: Text(
          l10n.modProfileDeleteDialogTitle,
          style: TextStyle(
            color: AppColors.error,
            fontSize: AppSizes.fontLG(ctx),
          ),
        ),
        content: Text(
          l10n.modProfileDeleteDialogBody(picked),
          style: TextStyle(
            color: AppColors.textSecondary,
            fontSize: AppSizes.fontSM(ctx),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(false),
            child: Text(
              l10n.buttonCancel,
              style: TextStyle(
                color: AppColors.textMuted,
                fontSize: AppSizes.fontSM(ctx),
              ),
            ),
          ),
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(true),
            child: Text(
              l10n.modProfileDeleteConfirm,
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
    if (confirmed != true || !context.mounted) return;

    final ok = await ref
        .read(modProfilesStateControllerProvider.notifier)
        .delete(gameDir, picked);
    if (!context.mounted) return;
    _toast(
      ref,
      ok
          ? l10n.modProfileDeletedToast(picked)
          : _localizedError(
              context,
              ref.read(modProfilesStateControllerProvider).error,
            ),
      ok ? AppColors.success : AppColors.error,
    );
  }

  Future<String?> _showNameDialog(
    BuildContext context, {
    required String title,
    required String initial,
    required Set<String> existingNames,
  }) async {
    final controller = TextEditingController(text: initial);
    final l10n = AppLocalizations.of(context)!;
    return showDialog<String>(
      context: context,
      builder: (ctx) {
        String? errorText;
        return StatefulBuilder(
          builder: (ctx, setLocal) {
            String? validate(String raw) {
              final sanitized = ModProfilesService.sanitizeName(raw);
              if (sanitized.isEmpty) return l10n.modProfileErrorNameEmpty;
              if (!ModProfilesService.isValidName(sanitized)) {
                return l10n.modProfileErrorNameInvalid;
              }
              if (existingNames.contains(sanitized)) {
                return l10n.modProfileErrorNameCollision;
              }
              return null;
            }

            void submit() {
              final raw = controller.text;
              final err = validate(raw);
              if (err != null) {
                setLocal(() => errorText = err);
                return;
              }
              Navigator.of(ctx).pop(ModProfilesService.sanitizeName(raw));
            }

            return AlertDialog(
              backgroundColor: AppColors.backgroundCard,
              title: Text(
                title,
                style: TextStyle(
                  color: AppColors.accentPrimary,
                  fontSize: AppSizes.fontLG(ctx),
                ),
              ),
              content: SizedBox(
                width: 360,
                child: TextField(
                  controller: controller,
                  autofocus: true,
                  maxLength: 48,
                  inputFormatters: [
                    FilteringTextInputFormatter.allow(
                      RegExp(r'[a-zA-Z0-9 _-]'),
                    ),
                  ],
                  style: TextStyle(
                    color: AppColors.textPrimary,
                    fontSize: AppSizes.fontMD(ctx),
                  ),
                  decoration: InputDecoration(
                    isDense: true,
                    filled: true,
                    fillColor: AppColors.inputBackground,
                    hintText: l10n.modProfileNewDialogHint,
                    hintStyle: TextStyle(
                      color: AppColors.textMuted,
                      fontSize: AppSizes.fontSM(ctx),
                    ),
                    errorText: errorText,
                    counterText: '',
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(
                        AppSizes.borderRadius(ctx),
                      ),
                      borderSide: BorderSide.none,
                    ),
                  ),
                  onChanged: (_) {
                    if (errorText != null) setLocal(() => errorText = null);
                  },
                  onSubmitted: (_) => submit(),
                ),
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.of(ctx).pop(),
                  child: Text(
                    l10n.buttonCancel,
                    style: TextStyle(
                      color: AppColors.textMuted,
                      fontSize: AppSizes.fontSM(ctx),
                    ),
                  ),
                ),
                TextButton(
                  onPressed: submit,
                  child: Text(
                    'OK',
                    style: TextStyle(
                      color: AppColors.accentPrimary,
                      fontSize: AppSizes.fontSM(ctx),
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ),
              ],
            );
          },
        );
      },
    );
  }

  void _toast(WidgetRef ref, String message, Color color) {
    ref.read(notificationStateControllerProvider.notifier).addNotification(
          NotificationItem(
            id: 'profile_${DateTime.now().millisecondsSinceEpoch}',
            message: (l10n) => message,
            icon: color == AppColors.error
                ? Icons.error_outline
                : Icons.check_circle,
            color: color,
            type: NotificationType.general,
          ),
        );
  }

  String _localizedError(BuildContext context, String? raw) {
    final l10n = AppLocalizations.of(context)!;
    if (raw == null) return l10n.modProfileErrorFsBusy;
    if (raw.contains('cannot_delete_active')) {
      return l10n.modProfileErrorDeleteActive;
    }
    if (raw.contains('cannot_delete_last')) {
      return l10n.modProfileErrorDeleteLast;
    }
    if (raw.contains('name_collision')) {
      return l10n.modProfileErrorNameCollision;
    }
    if (raw.contains('invalid_name')) {
      return l10n.modProfileErrorNameInvalid;
    }
    if (raw.contains('target_missing')) {
      return l10n.modProfileErrorTargetMissing;
    }
    if (raw.contains('rename_failed') ||
        raw.contains('step_') ||
        raw.contains('disabled_collision') ||
        raw.contains('inactive_collision')) {
      return l10n.modProfileErrorFsBusy;
    }
    return raw;
  }
}
