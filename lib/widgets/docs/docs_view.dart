import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_markdown_plus/flutter_markdown_plus.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:highlight/highlight.dart' show highlight, Node;
import 'package:markdown/markdown.dart' as md;
import 'package:path/path.dart' as p;
import 'package:url_launcher/url_launcher.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/notification_state.dart';
import 'package:yp_launcher/services/docs/docs_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/theme/nier_curves.dart';
import 'package:yp_launcher/widgets/autoscroll_selection.dart';
import 'package:yp_launcher/widgets/docs/docs_home.dart';
import 'package:yp_launcher/widgets/docs/first_mod_runner.dart';
import 'package:yp_launcher/widgets/docs/item_builder_card.dart';
import 'package:yp_launcher/widgets/docs/outfit_builder_card.dart';
import 'package:yp_launcher/widgets/docs/weapon_builder_card.dart';
import 'package:yp_launcher/widgets/docs/docs_tool_panel.dart';
import 'package:yp_launcher/widgets/docs/docs_figure.dart';
import 'package:yp_launcher/services/docs/docs_marker.dart';
import 'package:yp_launcher/widgets/hover_button.dart';
import 'package:yp_launcher/widgets/docs/markdown_editor_dialog.dart';
import 'package:yp_launcher/services/platform_gate.dart';
import 'package:yp_launcher/widgets/windows_title_bar.dart';

class _PageTransition extends StatefulWidget {
  final String pageKey;
  final Widget child;

  const _PageTransition({required this.pageKey, required this.child});

  @override
  State<_PageTransition> createState() => _PageTransitionState();
}

class _PageTransitionState extends State<_PageTransition>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 520),
  )..forward();

  @override
  void didUpdateWidget(_PageTransition oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.pageKey != widget.pageKey) _controller.forward(from: 0);
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: _controller,
      builder: (context, child) {
        final fade = NierCurves.fade.transform(_controller.value);
        final rise = NierCurves.fill.transform(_controller.value);

        return Opacity(
          opacity: fade,
          child: Transform.translate(
            offset: Offset(0, (1 - rise) * 14),
            child: child,
          ),
        );
      },
      child: widget.child,
    );
  }
}

class DocsScreen extends StatelessWidget {
  final String? initialPath;

  const DocsScreen({super.key, this.initialPath});

  static Future<void> open(BuildContext context, {String? initialPath}) {
    return Navigator.of(context).push(
      PageRouteBuilder(
        transitionDuration: const Duration(milliseconds: 420),
        reverseTransitionDuration: const Duration(milliseconds: 320),
        pageBuilder: (_, __, ___) => DocsScreen(initialPath: initialPath),
        transitionsBuilder: (_, animation, __, child) {
          return FadeTransition(
            opacity: CurvedAnimation(
              parent: animation,
              curve: NierCurves.fade,
            ),
            child: SlideTransition(
              position:
                  Tween<Offset>(
                    begin: const Offset(0, 0.02),
                    end: Offset.zero,
                  ).animate(
                    CurvedAnimation(
                      parent: animation,
                      curve: NierCurves.fill,
                    ),
                  ),
              child: child,
            ),
          );
        },
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    return Scaffold(
      backgroundColor: AppColors.backgroundPrimary,
      body: Column(
        children: [
          if (PlatformGate.isWindows) const WindowsTitleBar(),
          Container(
            padding: EdgeInsets.symmetric(
              horizontal: AppSizes.cardPaddingH(context),
              vertical: AppSizes.spacingSM(context),
            ),
            decoration: BoxDecoration(
              color: AppColors.surfaceMedium,
              border: Border(bottom: BorderSide(color: AppColors.borderLight)),
            ),
            child: Row(
              children: [
                _BackToLauncher(label: l10n.docsBackToLauncher),
                const Spacer(),
                Icon(
                  Icons.menu_book_outlined,
                  size: AppSizes.iconSM(context),
                  color: AppColors.textMuted,
                ),
                SizedBox(width: AppSizes.spacingSM(context)),
                Text(
                  l10n.tabDocs,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    fontWeight: FontWeight.bold,
                    color: AppColors.textMuted,
                    letterSpacing: 1.0,
                  ),
                ),
              ],
            ),
          ),
          Expanded(child: DocsView(initialPath: initialPath)),
        ],
      ),
    );
  }
}

class _BackToLauncher extends StatefulWidget {
  final String label;

  const _BackToLauncher({required this.label});

  @override
  State<_BackToLauncher> createState() => _BackToLauncherState();
}

class _BackToLauncherState extends State<_BackToLauncher> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final color = _hovered ? AppColors.accentPrimary : AppColors.textSecondary;
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: () => Navigator.of(context).maybePop(),
        child: Padding(
          padding: EdgeInsets.symmetric(
            horizontal: AppSizes.spacingSM(context),
            vertical: AppSizes.spacingSM(context),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(
                Icons.arrow_back,
                size: AppSizes.iconMD(context),
                color: color,
              ),
              SizedBox(width: AppSizes.spacingSM(context)),
              Text(
                widget.label,
                style: TextStyle(
                  fontSize: AppSizes.fontSM(context),
                  color: color,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class DocsView extends ConsumerStatefulWidget {
  final String? initialPath;

  const DocsView({super.key, this.initialPath});

  @override
  ConsumerState<DocsView> createState() => _DocsViewState();
}

class _DocsViewState extends ConsumerState<DocsView> {
  final _searchController = TextEditingController();
  final _searchFocus = FocusNode();
  final _contentScroll = ScrollController();
  final _navScroll = ScrollController();
  final Map<String, GlobalKey> _anchorKeys = {};

  List<DocPage> _pages = const [];
  List<DocSearchHit> _hits = const [];
  final List<String> _history = [];
  String _activeCategory = 'native';

  DocPage? _current;
  String _markdown = '';
  List<DocHeading> _outline = const [];
  String _query = '';
  bool _loading = true;
  bool _showHome = true;
  String? _loadError;

  @override
  void initState() {
    super.initState();
    _load();
  }

  @override
  void dispose() {
    _searchController.dispose();
    _searchFocus.dispose();
    _contentScroll.dispose();
    _navScroll.dispose();
    super.dispose();
  }

  Future<void> _load() async {
    List<DocPage> pages;
    String? error;
    try {
      pages = await DocsService.loadIndex();
    } catch (e) {
      pages = const [];
      error = '$e';
    }
    if (!mounted) return;
    setState(() {
      _pages = pages;
      _loadError = error;
      _loading = false;
    });

    final target = widget.initialPath;
    if (target == null) return;
    final match = pages.where((p) => p.path == target).firstOrNull;
    if (match != null) await _open(match, pushHistory: false);
  }

  void _openFileName(String fileName) {
    for (final page in _pages) {
      if (page.path.endsWith('/$fileName') && !page.isResource) {
        _open(page);
        return;
      }
    }
    for (final page in _pages) {
      if (page.fileName == fileName) {
        _open(page);
        return;
      }
    }
  }

  void _goHome() {
    setState(() {
      _showHome = true;
      _current = null;
      _outline = const [];
      _history.clear();
    });
  }

  Future<void> _open(
    DocPage page, {
    String? anchor,
    bool pushHistory = true,
  }) async {
    final markdown = await DocsService.content(page.path);
    if (!mounted) return;

    if (pushHistory && _current != null) {
      _history.add(_current!.path);
    }

    setState(() {
      _showHome = false;
      _activeCategory = page.category;
      _current = page;
      _markdown = markdown;
      _outline = DocsService.outline(markdown);
      _anchorKeys.clear();
      for (final heading in _outline) {
        _anchorKeys[heading.anchor] = GlobalKey();
      }
    });

    if (anchor != null) {
      WidgetsBinding.instance.addPostFrameCallback((_) => _scrollTo(anchor));
    } else if (_contentScroll.hasClients) {
      _contentScroll.jumpTo(0);
    }
  }

  void _scrollTo(String anchor) {
    final key = _anchorKeys[anchor];
    final context = key?.currentContext;
    if (context == null) return;
    Scrollable.ensureVisible(
      context,
      duration: const Duration(milliseconds: 480),
      curve: NierCurves.fill,
      alignment: 0.05,
    );
  }

  Future<void> _openExternal(String href) async {
    final uri = Uri.parse(href);

    for (final mode in const [
      LaunchMode.externalApplication,
      LaunchMode.platformDefault,
    ]) {
      try {
        if (await launchUrl(uri, mode: mode)) return;
      } catch (_) {}
    }

    await Clipboard.setData(ClipboardData(text: href));
    if (!mounted) return;

    ref.read(notificationStateControllerProvider.notifier).addNotification(
          NotificationItem(
            id: 'docs_link_copied',
            message: (l10n) => l10n.docsLinkCopied(href),
            icon: Icons.link_off,
            color: AppColors.warning,
            type: NotificationType.general,
          ),
        );
  }

  void _goBack() {
    if (_history.isEmpty) return;
    final previous = _history.removeLast();
    final page = _pages.firstWhere(
      (p) => p.path == previous,
      orElse: () => _pages.first,
    );
    _open(page, pushHistory: false);
  }

  Future<void> _onLinkTapped(String href) async {
    if (href.startsWith('http://') || href.startsWith('https://')) {
      await _openExternal(href);
      return;
    }

    if (href.startsWith('#')) {
      _scrollTo(href.substring(1));
      return;
    }

    final current = _current;
    if (current == null) return;

    final resolved = DocsService.resolveLink(current.path, href);
    if (resolved == null) return;

    final match = _pages.where((p) => p.path == resolved).firstOrNull;
    if (match == null) return;

    await _open(match, anchor: DocsService.anchorIn(href));
  }

  Future<void> _reloadCurrent() async {
    final page = _current;
    if (page == null) return;
    DocsService.invalidate();
    final markdown = await DocsService.content(page.path);
    if (!mounted) return;
    setState(() {
      _markdown = markdown;
      _outline = DocsService.outline(markdown);
      _anchorKeys.clear();
      for (final heading in _outline) {
        _anchorKeys[heading.anchor] = GlobalKey();
      }
    });
  }

  Future<void> _editCurrent() async {
    final page = _current;
    if (page == null) return;
    final l10n = AppLocalizations.of(context)!;

    final edited = await MarkdownEditorDialog.show(
      context,
      fileName: page.fileName,
      filePath: DocsService.sourceFileFor(page.path)?.path ?? page.path,
      initialContent: _markdown,
      saveLabel: l10n.docsSaveFile,
    );
    if (edited == null) return;

    await DocsService.saveSource(page.path, edited);
    if (!mounted) return;
    await _reloadCurrent();
    if (!mounted) return;

    final refreshed = await DocsService.loadIndex();
    if (!mounted) return;
    setState(() => _pages = refreshed);
  }

  Future<void> _search(String query) async {
    setState(() => _query = query);
    if (query.trim().isEmpty) {
      setState(() => _hits = const []);
      return;
    }
    final hits = await DocsService.search(query);
    if (!mounted) return;
    setState(() => _hits = hits);
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    return Container(
      color: AppColors.backgroundPrimary,
      child: _loading
          ? Center(
              child: CircularProgressIndicator(color: AppColors.accentPrimary),
            )
          : _pages.isEmpty
          ? _emptyState(context)
          : Row(
              children: [
                _navPanel(context, l10n),
                Expanded(
                  child: _showHome
                      ? DocsHome(
                          pages: _pages,
                          onOpenFile: _openFileName,
                          onBrowseAll: () => _searchFocus.requestFocus(),
                        )
                      : _contentPanel(context, l10n),
                ),
                if (!_showHome && _outline.length > 2)
                  _outlinePanel(context, l10n),
              ],
            ),
    );
  }

  Widget _emptyState(BuildContext context) {
    return Center(
      child: Padding(
        padding: EdgeInsets.all(AppSizes.contentPadding(context)),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              Icons.menu_book_outlined,
              size: AppSizes.iconLG(context) * 2,
              color: AppColors.textMuted,
            ),
            SizedBox(height: AppSizes.spacingLG(context)),
            SelectableText(
              _loadError ?? 'assets/docs is not bundled',
              textAlign: TextAlign.center,
              style: TextStyle(
                fontSize: AppSizes.fontSM(context),
                color: AppColors.textMuted,
                height: 1.5,
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _navPanel(BuildContext context, AppLocalizations l10n) {
    final searching = _query.trim().isNotEmpty;

    return Container(
      width: AppSizes.docsNavWidth(context),
      decoration: BoxDecoration(
        color: AppColors.surfaceMedium,
        border: Border(right: BorderSide(color: AppColors.borderLight)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: EdgeInsets.all(AppSizes.paddingMD(context)),
            child: _searchField(context, l10n),
          ),
          if (!searching) _categorySwitch(context, l10n),
          Expanded(
            child: Scrollbar(
              controller: _navScroll,
              child: ListView(
                controller: _navScroll,
                padding: EdgeInsets.only(bottom: AppSizes.paddingLG(context)),
                children: searching
                    ? _searchResults(context, l10n)
                    : _categoryTree(context, l10n),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _categorySwitch(BuildContext context, AppLocalizations l10n) {
    return Padding(
      padding: EdgeInsets.fromLTRB(
        AppSizes.paddingMD(context),
        0,
        AppSizes.paddingMD(context),
        AppSizes.paddingMD(context),
      ),
      child: Container(
        padding: const EdgeInsets.all(3),
        decoration: BoxDecoration(
          color: AppColors.inputBackground,
          borderRadius: BorderRadius.circular(6),
          border: Border.all(color: AppColors.borderMedium),
        ),
        child: Row(
          children: [
            Expanded(
              child: _CategoryTab(
                label: l10n.docsCategoryNative,
                active: _activeCategory == 'native',
                onTap: () => setState(() => _activeCategory = 'native'),
              ),
            ),
            Expanded(
              child: _CategoryTab(
                label: l10n.docsCategoryScripting,
                active: _activeCategory == 'scripting',
                onTap: () => setState(() => _activeCategory = 'scripting'),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _searchField(BuildContext context, AppLocalizations l10n) {
    return Container(
      decoration: BoxDecoration(
        color: AppColors.inputBackground,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: AppColors.borderMedium),
      ),
      child: Row(
        children: [
          Padding(
            padding: const EdgeInsets.only(left: 8),
            child: Icon(
              Icons.search,
              size: AppSizes.iconSM(context),
              color: AppColors.textMuted,
            ),
          ),
          Expanded(
            child: TextField(
              controller: _searchController,
              focusNode: _searchFocus,
              onChanged: _search,
              style: TextStyle(
                fontSize: AppSizes.fontSM(context),
                color: AppColors.textPrimary,
              ),
              decoration: InputDecoration(
                isDense: true,
                hintText: l10n.docsSearchHint,
                hintStyle: TextStyle(
                  fontSize: AppSizes.fontSM(context),
                  color: AppColors.textMuted.withValues(alpha: 0.6),
                ),
                contentPadding: const EdgeInsets.symmetric(
                  horizontal: 8,
                  vertical: 10,
                ),
                border: InputBorder.none,
              ),
            ),
          ),
          if (_query.isNotEmpty)
            HoverIconButton(
              tooltip: MaterialLocalizations.of(context).cancelButtonLabel,
              bordered: false,
              onTap: () {
                _searchController.clear();
                _search('');
              },
              icon: Icon(
                Icons.close,
                size: AppSizes.iconSM(context),
                color: AppColors.textMuted,
              ),
            ),
        ],
      ),
    );
  }

  List<Widget> _searchResults(BuildContext context, AppLocalizations l10n) {
    if (_hits.isEmpty) {
      return [
        Padding(
          padding: EdgeInsets.symmetric(
            horizontal: AppSizes.cardPaddingH(context),
            vertical: AppSizes.spacingMD(context),
          ),
          child: Text(
            l10n.docsNoResults,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: AppColors.textMuted,
              fontStyle: FontStyle.italic,
            ),
          ),
        ),
      ];
    }

    return [
      Padding(
        padding: EdgeInsets.fromLTRB(
          AppSizes.cardPaddingH(context),
          AppSizes.spacingSM(context),
          AppSizes.cardPaddingH(context),
          AppSizes.spacingSM(context),
        ),
        child: Text(
          l10n.docsResultCount(_hits.length),
          style: TextStyle(
            fontSize: AppSizes.fontXS(context),
            fontWeight: FontWeight.bold,
            color: AppColors.textMuted,
            letterSpacing: 0.8,
          ),
        ),
      ),
      for (final hit in _hits)
        _NavRow(
          title: hit.page.title,
          subtitle: hit.excerpt,
          active: hit.page.path == _current?.path,
          onTap: () => _open(hit.page),
        ),
    ];
  }

  List<Widget> _categoryTree(BuildContext context, AppLocalizations l10n) {
    final widgets = <Widget>[
      _NavRow(title: l10n.docsHome, active: _showHome, onTap: _goHome),
      SizedBox(height: AppSizes.spacingMD(context)),
    ];

    final pages = _pages.where((p) => p.category == _activeCategory).toList();

    var resourcesStarted = false;
    var authoringStarted = false;
    for (final page in pages) {
      if (page.isAuthoring && !authoringStarted) {
        authoringStarted = true;
        widgets.add(_NavHeader(title: l10n.docsCategoryAuthoring));
      } else if (page.isResource && !resourcesStarted) {
        resourcesStarted = true;
        widgets.add(_NavHeader(title: l10n.docsCategoryResources));
      }
      widgets.add(
        _NavRow(
          title: page.title,
          active: page.path == _current?.path,
          onTap: () => _open(page),
        ),
      );
    }
    return widgets;
  }

  String _categoryLabel(String category, AppLocalizations l10n) {
    switch (category) {
      case 'native':
        return l10n.docsCategoryNative;
      case 'scripting':
        return l10n.docsCategoryScripting;
      default:
        return DocsService.prettyName(category);
    }
  }

  Widget _contentPanel(BuildContext context, AppLocalizations l10n) {
    final page = _current;
    if (page == null) return const SizedBox.shrink();

    return Column(
      children: [
        _contentHeader(context, l10n, page),
        Expanded(
          child: AutoScrollSelection(
            controller: _contentScroll,
            child: Scrollbar(
              controller: _contentScroll,
              thumbVisibility: true,
              child: SingleChildScrollView(
                controller: _contentScroll,
                padding: EdgeInsets.all(AppSizes.contentPadding(context)),
                child: Align(
                  alignment: Alignment.topCenter,
                  child: ConstrainedBox(
                    constraints: const BoxConstraints(maxWidth: 900),
                    child: SelectionArea(
                      child: _PageTransition(
                        pageKey: page.path,
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.stretch,
                          mainAxisSize: MainAxisSize.min,
                          children: _bodyWithTools(context),
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      ],
    );
  }

  Widget? _toolFor(String id) {
    final l10n = AppLocalizations.of(context)!;
    switch (id) {
      case 'item-builder':
        return DocsToolPanel(
          key: const ValueKey('panel:item-builder'),
          title: l10n.itemBuilderTitle,
          subtitle: l10n.itemBuilderBody,
          icon: Icons.add_box_outlined,
          child: const ItemBuilderCard(),
        );
      case 'outfit-builder':
        return DocsToolPanel(
          key: const ValueKey('panel:outfit-builder'),
          title: l10n.outfitBuilderTitle,
          subtitle: l10n.outfitBuilderDesc,
          icon: Icons.checkroom_outlined,
          child: const OutfitBuilderCard(),
        );
      case 'weapon-builder':
        return DocsToolPanel(
          key: const ValueKey('panel:weapon-builder'),
          title: l10n.weaponBuilderTitle,
          subtitle: l10n.weaponBuilderDesc,
          icon: Icons.hardware_outlined,
          child: const WeaponBuilderCard(),
        );
      case 'first-mod':
        return DocsToolPanel(
          key: const ValueKey('panel:first-mod'),
          title: l10n.firstModTitle,
          subtitle: l10n.firstModBody,
          icon: Icons.play_lesson_outlined,
          initiallyOpen: true,
          child: const FirstModRunner(),
        );
      default:
        return null;
    }
  }

  final Map<String, int> _anchorCounts = {};

  List<Widget> _bodyWithTools(BuildContext context) {
    _anchorCounts.clear();

    final widgets = <Widget>[];
    var rest = _markdown;

    while (true) {
      final marker = DocsMarker.firstIn(rest);
      if (marker == null) break;

      final before = rest.substring(0, marker.start);
      rest = rest.substring(marker.end);

      if (before.trim().isNotEmpty) {
        widgets.add(_markdownBody(context, before));
      }
      final tool = _toolFor(marker.id);
      if (tool != null) {
        widgets.add(
          KeyedSubtree(
            key: ValueKey('tool:${marker.id}'),
            child: SelectionContainer.disabled(child: tool),
          ),
        );
      }
    }

    if (rest.trim().isNotEmpty) widgets.add(_markdownBody(context, rest));
    return widgets.isEmpty ? [_markdownBody(context, _markdown)] : widgets;
  }

  Widget _contentHeader(
    BuildContext context,
    AppLocalizations l10n,
    DocPage page,
  ) {
    return Container(
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.cardPaddingH(context),
        vertical: AppSizes.cardPaddingV(context),
      ),
      decoration: BoxDecoration(
        color: AppColors.surfaceMedium,
        border: Border(bottom: BorderSide(color: AppColors.borderLight)),
      ),
      child: Row(
        children: [
          Padding(
            padding: const EdgeInsets.only(right: 8),
            child: HoverIconButton(
              tooltip: MaterialLocalizations.of(context).backButtonTooltip,
              bordered: false,
              onTap: _history.isNotEmpty ? _goBack : _goHome,
              icon: Icon(
                Icons.arrow_back,
                size: AppSizes.iconMD(context),
                color: AppColors.textSecondary,
              ),
            ),
          ),
          Expanded(
            child: Text(
              page.title,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: TextStyle(
                fontSize: AppSizes.fontLG(context),
                fontWeight: FontWeight.bold,
                color: AppColors.textPrimary,
                letterSpacing: 1.0,
              ),
            ),
          ),
          if (DocsService.canEdit) ...[
            HoverIconButton(
              tooltip: l10n.docsReload,
              bordered: false,
              onTap: _reloadCurrent,
              icon: Icon(
                Icons.refresh,
                size: AppSizes.iconSM(context),
                color: AppColors.textMuted,
              ),
            ),
            SizedBox(width: AppSizes.spacingSM(context)),
            HoverIconButton(
              tooltip: l10n.docsEdit,
              bordered: false,
              onTap: _editCurrent,
              icon: Icon(
                Icons.edit_outlined,
                size: AppSizes.iconSM(context),
                color: AppColors.accentPrimary,
              ),
            ),
            SizedBox(width: AppSizes.spacingMD(context)),
          ],
          Container(
            padding: EdgeInsets.symmetric(
              horizontal: AppSizes.chipPaddingH(context),
              vertical: AppSizes.chipPaddingV(context),
            ),
            decoration: BoxDecoration(
              color: AppColors.accentPrimary.withValues(alpha: 0.10),
              borderRadius: BorderRadius.circular(
                AppSizes.borderRadius(context),
              ),
              border: Border.all(
                color: AppColors.accentPrimary.withValues(alpha: 0.3),
              ),
            ),
            child: Text(
              _categoryLabel(page.category, l10n),
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                fontWeight: FontWeight.bold,
                color: AppColors.accentPrimary,
                letterSpacing: 0.5,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _markdownBody(BuildContext context, String data) {
    final body = AppSizes.fontSM(context);
    final code = AppSizes.fontXS(context);
    final seen = _anchorCounts;

    return MarkdownBody(
      data: data,
      onTapLink: (text, href, title) {
        if (href != null) _onLinkTapped(href);
      },
      imageBuilder: (uri, title, alt) {
        final page = _current;
        final src = uri.toString();
        if (page == null || src.startsWith('http')) {
          return const SizedBox.shrink();
        }
        return DocsImage(
          image: p.url.normalize(p.url.join(p.url.dirname(page.path), src)),
          caption: alt == null || alt.isEmpty ? null : alt,
        );
      },
      builders: {
        'h2': _AnchorBuilder(this, seen),
        'h3': _AnchorBuilder(this, seen),
        'pre': _CodeBlockBuilder(code),
      },
      styleSheet: MarkdownStyleSheet(
        p: TextStyle(
          fontSize: body,
          color: AppColors.textSecondary,
          height: 1.6,
        ),
        h1: TextStyle(
          fontSize: body * 1.9,
          color: AppColors.textPrimary,
          fontWeight: FontWeight.bold,
          height: 1.25,
          letterSpacing: 0.2,
        ),
        h2: TextStyle(
          fontSize: body * 1.5,
          color: AppColors.textPrimary,
          fontWeight: FontWeight.w700,
          height: 1.3,
        ),
        h3: TextStyle(
          fontSize: body * 1.22,
          color: AppColors.textPrimary,
          fontWeight: FontWeight.w700,
          height: 1.35,
        ),
        h4: TextStyle(
          fontSize: body * 1.08,
          color: AppColors.textPrimary,
          fontWeight: FontWeight.w700,
          height: 1.4,
        ),
        h5: TextStyle(
          fontSize: body,
          color: AppColors.textPrimary,
          fontWeight: FontWeight.w700,
          height: 1.4,
        ),
        h6: TextStyle(
          fontSize: body,
          color: AppColors.textSecondary,
          fontWeight: FontWeight.w700,
          height: 1.4,
        ),
        h1Padding: EdgeInsets.only(top: AppSizes.paddingLG(context)),
        h2Padding: EdgeInsets.only(top: AppSizes.paddingLG(context)),
        h3Padding: EdgeInsets.only(top: AppSizes.paddingMD(context)),
        h4Padding: EdgeInsets.only(top: AppSizes.paddingMD(context)),
        strong: TextStyle(
          fontSize: body,
          color: AppColors.textPrimary,
          fontWeight: FontWeight.w700,
        ),
        em: TextStyle(
          fontSize: body,
          color: AppColors.textSecondary,
          fontStyle: FontStyle.italic,
        ),
        a: TextStyle(
          fontSize: body,
          color: AppColors.accentPrimary,
          decoration: TextDecoration.underline,
          decorationColor: AppColors.accentPrimary.withValues(alpha: 0.4),
        ),
        code: TextStyle(
          fontSize: code,
          color: AppColors.accentPrimary,
          fontFamily: AppSizes.monoFamily,
          fontFamilyFallback: AppSizes.monoFallback,
          backgroundColor: AppColors.accentPrimary.withValues(alpha: 0.08),
        ),
        codeblockDecoration: const BoxDecoration(),
        codeblockPadding: EdgeInsets.zero,
        blockquoteDecoration: BoxDecoration(
          color: AppColors.surfaceLight,
          borderRadius: BorderRadius.circular(4),
          border: Border(
            left: BorderSide(color: AppColors.accentPrimary, width: 3),
          ),
        ),
        listBullet: TextStyle(fontSize: body, color: AppColors.accentPrimary),
        tableHead: TextStyle(
          fontSize: body,
          color: AppColors.textPrimary,
          fontWeight: FontWeight.w700,
        ),
        tableBody: TextStyle(fontSize: body, color: AppColors.textSecondary),
        tableBorder: TableBorder.all(color: AppColors.borderLight, width: 1),
        horizontalRuleDecoration: BoxDecoration(
          border: Border(top: BorderSide(color: AppColors.borderLight)),
        ),
        blockSpacing: AppSizes.spacingMD(context),
      ),
    );
  }

  Widget _outlinePanel(BuildContext context, AppLocalizations l10n) {
    return Container(
      width: AppSizes.docsOutlineWidth(context),
      decoration: BoxDecoration(
        color: AppColors.surfaceMedium,
        border: Border(left: BorderSide(color: AppColors.borderLight)),
      ),
      child: ListView(
        padding: EdgeInsets.symmetric(vertical: AppSizes.paddingMD(context)),
        children: [
          _NavHeader(title: l10n.docsOnThisPage),
          for (final heading in _outline)
            _OutlineRow(
              heading: heading,
              onTap: () => _scrollTo(heading.anchor),
            ),
        ],
      ),
    );
  }

  GlobalKey? anchorKey(String anchor) => _anchorKeys[anchor];
}

class _CodeBlock extends StatefulWidget {
  final String code;
  final double fontSize;
  final String? language;

  const _CodeBlock({required this.code, required this.fontSize, this.language});

  @override
  State<_CodeBlock> createState() => _CodeBlockState();
}

class _CodeBlockState extends State<_CodeBlock> {
  bool _hovered = false;
  bool _copied = false;
  Timer? _resetTimer;
  final _codeScroll = ScrollController();

  @override
  void dispose() {
    _resetTimer?.cancel();
    _codeScroll.dispose();
    super.dispose();
  }

  TextSpan _span(BuildContext context) {
    final language = widget.language;
    if (language == null) {
      return TextSpan(text: widget.code, style: _plainStyle);
    }

    try {
      final result = highlight.parse(widget.code, language: language);
      final nodes = result.nodes;
      if (nodes == null || nodes.isEmpty) {
        return TextSpan(text: widget.code, style: _plainStyle);
      }
      return TextSpan(children: nodes.map(_nodeToSpan).toList());
    } catch (_) {
      return TextSpan(text: widget.code, style: _plainStyle);
    }
  }

  TextSpan _nodeToSpan(Node node) {
    final children = node.children;
    if (children != null && children.isNotEmpty) {
      return TextSpan(
        style: _styleFor(node.className),
        children: children.map(_nodeToSpan).toList(),
      );
    }
    return TextSpan(text: node.value, style: _styleFor(node.className));
  }

  TextStyle? _styleFor(String? className) {
    if (className == null) return null;
    final color = _syntaxColors[className];
    if (color == null) return null;
    return TextStyle(color: color);
  }

  static TextStyle get _plainStyle =>
      TextStyle(color: AppColors.accentSecondary);

  static final _syntaxColors = <String, Color>{
    'comment': AppColors.textMuted,
    'quote': AppColors.textMuted,
    'string': AppColors.success,
    'attr': AppColors.accentPrimary,
    'attribute': AppColors.accentPrimary,
    'section': AppColors.warning,
    'title': AppColors.warning,
    'name': AppColors.warning,
    'tag': AppColors.warning,
    'built_in': AppColors.accentPrimary,
    'keyword': AppColors.error,
    'literal': AppColors.error,
    'symbol': AppColors.error,
    'number': AppColors.accentPrimary,
    'variable': AppColors.textPrimary,
    'meta': AppColors.textMuted,
  };

  void _copy() {
    Clipboard.setData(ClipboardData(text: widget.code));
    setState(() => _copied = true);
    _resetTimer?.cancel();
    _resetTimer = Timer(const Duration(seconds: 2), () {
      if (mounted) setState(() => _copied = false);
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: Container(
        width: double.infinity,
        margin: EdgeInsets.symmetric(vertical: AppSizes.spacingSM(context)),
        decoration: BoxDecoration(
          color: AppColors.surfaceLight,
          borderRadius: BorderRadius.circular(6),
          border: Border.all(color: AppColors.borderLight),
        ),
        child: Stack(
          children: [
            Padding(
              padding: EdgeInsets.all(AppSizes.paddingMD(context)),
              child: SingleChildScrollView(
                controller: _codeScroll,
                scrollDirection: Axis.horizontal,
                child: Text.rich(
                  _span(context),
                  style: TextStyle(
                    fontSize: widget.fontSize,
                    fontFamily: AppSizes.monoFamily,
                    fontFamilyFallback: AppSizes.monoFallback,
                    color: AppColors.textSecondary,
                    height: 1.4,
                  ),
                ),
              ),
            ),
            Positioned(
              top: 4,
              right: 4,
              child: SelectionContainer.disabled(
                child: AnimatedOpacity(
                  duration: const Duration(milliseconds: 120),
                  opacity: _hovered || _copied ? 1 : 0,
                  child: HoverIconButton(
                    tooltip: _copied ? l10n.docsCopied : l10n.docsCopy,
                    onTap: _copy,
                    borderColor: _copied
                        ? AppColors.success
                        : AppColors.accentPrimary,
                    icon: Icon(
                      _copied ? Icons.check : Icons.content_copy,
                      size: AppSizes.iconSM(context),
                      color: _copied ? AppColors.success : AppColors.textMuted,
                    ),
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _CodeBlockBuilder extends MarkdownElementBuilder {
  final double fontSize;

  _CodeBlockBuilder(this.fontSize);

  @override
  Widget? visitElementAfter(md.Element element, TextStyle? preferredStyle) {
    var code = element.textContent;
    if (code.endsWith('\n')) {
      code = code.substring(0, code.length - 1);
    }
    return _CodeBlock(
      code: code,
      fontSize: fontSize,
      language: _languageOf(element),
    );
  }

  static String? _languageOf(md.Element element) {
    for (final child in element.children ?? const <md.Node>[]) {
      if (child is! md.Element) continue;
      final className = child.attributes['class'];
      if (className == null || !className.startsWith('language-')) continue;
      final name = className.substring('language-'.length).toLowerCase();
      return _supported[name];
    }
    return null;
  }

  static const _supported = <String, String>{
    'toml': 'ini',
    'ini': 'ini',
    'ruby': 'ruby',
    'rb': 'ruby',
    'xml': 'xml',
    'html': 'xml',
    'json': 'json',
  };
}

class _AnchorBuilder extends MarkdownElementBuilder {
  final _DocsViewState state;
  final Map<String, int> _seen;

  _AnchorBuilder(this.state, this._seen);

  @override
  Widget? visitElementAfter(md.Element element, TextStyle? preferredStyle) {
    final content = element.textContent;
    final base = DocsService.anchorFor(content);
    final count = _seen.update(base, (v) => v + 1, ifAbsent: () => 0);
    final anchor = count == 0 ? base : '$base-$count';

    return Padding(
      key: state.anchorKey(anchor),
      padding: const EdgeInsets.only(top: 4),
      child: Text(content, style: preferredStyle),
    );
  }
}

class _CategoryTab extends StatefulWidget {
  final String label;
  final bool active;
  final VoidCallback onTap;

  const _CategoryTab({
    required this.label,
    required this.active,
    required this.onTap,
  });

  @override
  State<_CategoryTab> createState() => _CategoryTabState();
}

class _CategoryTabState extends State<_CategoryTab> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 320),
          curve: NierCurves.fill,
          padding: EdgeInsets.symmetric(
            vertical: AppSizes.spacingSM(context) + 2,
          ),
          decoration: BoxDecoration(
            color: widget.active
                ? AppColors.accentPrimary
                : _hovered
                ? AppColors.surfaceLight
                : Colors.transparent,
            borderRadius: BorderRadius.circular(4),
          ),
          alignment: Alignment.center,
          child: Text(
            widget.label,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              fontWeight: widget.active ? FontWeight.w700 : FontWeight.w500,
              color: widget.active
                  ? AppColors.backgroundPrimary
                  : AppColors.textSecondary,
            ),
          ),
        ),
      ),
    );
  }
}

class _NavHeader extends StatelessWidget {
  final String title;

  const _NavHeader({required this.title});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: EdgeInsets.fromLTRB(
        AppSizes.cardPaddingH(context),
        AppSizes.spacingMD(context),
        AppSizes.cardPaddingH(context),
        AppSizes.spacingSM(context),
      ),
      child: Text(
        title.toUpperCase(),
        style: TextStyle(
          fontSize: AppSizes.fontXS(context),
          fontWeight: FontWeight.bold,
          color: AppColors.textMuted,
          letterSpacing: 1.0,
        ),
      ),
    );
  }
}

class _NavRow extends StatefulWidget {
  final String title;
  final String? subtitle;
  final bool active;
  final VoidCallback onTap;

  const _NavRow({
    required this.title,
    required this.active,
    required this.onTap,
    this.subtitle,
  });

  @override
  State<_NavRow> createState() => _NavRowState();
}

class _NavRowState extends State<_NavRow> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final color = widget.active
        ? AppColors.accentPrimary
        : _hovered
        ? AppColors.textPrimary
        : AppColors.textSecondary;

    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 320),
          curve: NierCurves.fill,
          padding: EdgeInsets.symmetric(
            horizontal: AppSizes.cardPaddingH(context),
            vertical: AppSizes.spacingSM(context),
          ),
          decoration: BoxDecoration(
            color: widget.active
                ? AppColors.accentPrimary.withValues(alpha: 0.10)
                : _hovered
                ? AppColors.surfaceLight
                : Colors.transparent,
            border: Border(
              left: BorderSide(
                color: widget.active
                    ? AppColors.accentPrimary
                    : Colors.transparent,
                width: 3,
              ),
            ),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                widget.title,
                overflow: TextOverflow.ellipsis,
                style: TextStyle(
                  fontSize: AppSizes.fontSM(context),
                  color: color,
                  fontWeight: widget.active
                      ? FontWeight.w600
                      : FontWeight.normal,
                ),
              ),
              if (widget.subtitle != null)
                Padding(
                  padding: const EdgeInsets.only(top: 2),
                  child: Text(
                    widget.subtitle!,
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      color: AppColors.textMuted,
                      height: 1.3,
                    ),
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }
}

class _OutlineRow extends StatefulWidget {
  final DocHeading heading;
  final VoidCallback onTap;

  const _OutlineRow({required this.heading, required this.onTap});

  @override
  State<_OutlineRow> createState() => _OutlineRowState();
}

class _OutlineRowState extends State<_OutlineRow> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: Padding(
          padding: EdgeInsets.fromLTRB(
            widget.heading.level == 3
                ? AppSizes.cardPaddingH(context) * 1.7
                : AppSizes.cardPaddingH(context),
            AppSizes.spacingXS(context),
            AppSizes.spacingSM(context),
            AppSizes.spacingXS(context),
          ),
          child: Text(
            widget.heading.text,
            maxLines: 2,
            overflow: TextOverflow.ellipsis,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: _hovered ? AppColors.accentPrimary : AppColors.textMuted,
              height: 1.4,
            ),
          ),
        ),
      ),
    );
  }
}
