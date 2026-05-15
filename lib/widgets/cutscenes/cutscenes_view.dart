import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/config_state.dart';
import 'package:yp_launcher/providers/disabled_mods_state.dart';
import 'package:yp_launcher/services/cutscene_install_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/cutscenes/cutscene_drop_zone.dart';
import 'package:yp_launcher/widgets/cutscenes/cutscene_info.dart';
import 'package:yp_launcher/widgets/cutscenes/cutscene_installed_list.dart';
import 'package:yp_launcher/widgets/cutscenes/cutscene_isolates.dart';

class CutscenesView extends ConsumerStatefulWidget {
  const CutscenesView({super.key});

  @override
  ConsumerState<CutscenesView> createState() => _CutscenesViewState();
}

class _CutscenesViewState extends ConsumerState<CutscenesView> {
  final _scrollController = ScrollController();
  bool _loaded = false;
  bool _installing = false;
  bool _loadingMods = true;
  List<CutsceneMod> _mods = [];
  List<String> _directOverrides = [];
  String _progressText = '';
  double _progressPercent = 0;
  late CutsceneInstallService _installService;

  String get _gameDir => ref.read(appStateControllerProvider).selectedDirectory;
  String get _cutscenesDir => path.join(_gameDir, 'nams', 'cutscenes');
  String get _originalMovieDir => path.join(_gameDir, 'data', 'movie');
  String get _modsDir => path.join(_gameDir, 'nams', 'mods');

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _installService = CutsceneInstallService(
        ref: ref,
        context: context,
        cutscenesDir: _cutscenesDir,
        originalMovieDir: _originalMovieDir,
        onProgress: (installing, text, percent) {
          if (mounted) {
            setState(() {
              _installing = installing;
              _progressText = text;
              _progressPercent = percent;
            });
          }
        },
        onReload: _loadMods,
        getMods: () => _mods,
      );
      if (!_loaded) {
        _loaded = true;
        _loadMods();
        ref.read(disabledModsStateControllerProvider.notifier).load(_gameDir);
      }
      ref.listenManual<int>(detectionRefreshProvider, (_, __) {
        if (_gameDir.isNotEmpty && mounted) _loadMods();
      });
      ref.listenManual<AppState>(appStateControllerProvider, (prev, next) {
        if (prev?.selectedDirectory != next.selectedDirectory &&
            next.selectedDirectory.isNotEmpty &&
            mounted) {
          _loadMods();
        }
      });
    });
  }

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  Future<void> _loadMods() async {
    final isFirstLoad = _mods.isEmpty && _directOverrides.isEmpty;
    setState(() {
      if (isFirstLoad) _loadingMods = true;
      _progressText = '';
    });
    final result = await scanCutscenes(
      _cutscenesDir,
      _originalMovieDir,
      _modsDir,
      onProgress: (scanned, total) {
        if (mounted) {
          setState(() => _progressText = '$scanned / $total USM');
        }
      },
    );
    setState(() {
      _mods = result.mods;
      _directOverrides = result.directOverrides;
      _loadingMods = false;
      _progressText = '';
    });
    if (_gameDir.isNotEmpty) {
      await ref
          .read(configStateControllerProvider.notifier)
          .autoDetectCutscenes(_gameDir);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      color: AppColors.backgroundPrimary,
      child: Column(
        children: [
          CutsceneHeader(installing: _installing),
          Expanded(
            child: _loadingMods
                ? Center(
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        const CircularProgressIndicator(color: AppColors.accentPrimary),
                        if (_progressText.isNotEmpty) ...[
                          SizedBox(height: AppSizes.spacingLG(context)),
                          Text(_progressText, style: TextStyle(fontSize: AppSizes.fontSM(context), color: AppColors.textMuted)),
                        ],
                      ],
                    ),
                  )
                : Scrollbar(
                    controller: _scrollController,
                    thumbVisibility: true,
                    child: SingleChildScrollView(
                      controller: _scrollController,
                      padding: EdgeInsets.all(AppSizes.contentPadding(context)),
                      child: Column(
                        children: [
                          if (_directOverrides.isNotEmpty) ...[
                            CutsceneMigrationBanner(directOverrides: _directOverrides),
                            const SizedBox(height: 16),
                          ],
                          Row(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Expanded(
                                child: Column(
                                  children: [
                                    CutsceneDropZone(
                                      installing: _installing,
                                      progressText: _progressText,
                                      progressPercent: _progressPercent,
                                      onDrop: (paths) {
                                        if (ref.read(activeTabProvider) != 7) return;
                                        _installService.handleDrop(paths);
                                      },
                                      onBrowse: _installService.handleBrowse,
                                      onBrowseFolder:
                                          _installService.handleBrowseFolder,
                                    ),
                                    const SizedBox(height: 16),
                                    const CutsceneInfoCard(),
                                  ],
                                ),
                              ),
                              const SizedBox(width: 16),
                              Expanded(
                                child: CutsceneInstalledList(
                                  mods: _mods,
                                  onDelete: _installService.deleteMod,
                                ),
                              ),
                            ],
                          ),
                        ],
                      ),
                    ),
                  ),
          ),
        ],
      ),
    );
  }
}
