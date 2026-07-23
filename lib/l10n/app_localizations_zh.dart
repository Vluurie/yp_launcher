// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Chinese (`zh`).
class AppLocalizationsZh extends AppLocalizations {
  AppLocalizationsZh([String locale = 'zh']) : super(locale);

  @override
  String get appTitle => 'YoRHa Protocol 启动器';

  @override
  String get tooltipLauncherSource => '启动器源代码';

  @override
  String get tooltipNamsSource => 'NAMS 项目源代码';

  @override
  String get tooltipGuide => '指南';

  @override
  String get tooltipDiscord => 'Discord';

  @override
  String get tooltipLanguage => '语言';

  @override
  String get languageSupportNotice =>
      '翻译由社区提供或自动生成，可能存在不准确之处。维护者仅使用英语 — 寻求帮助时请使用英语提问。';

  @override
  String get tooltipCopyCommand => '将 NAMS 命令复制到剪贴板，以便粘贴到终端中手动启动游戏。';

  @override
  String get notificationCommandCopied => '启动命令已复制 — 将其粘贴到终端中即可手动启动游戏。';

  @override
  String get notificationCommandNotReady => '无法生成启动命令 — 启动器二进制文件尚未就绪。';

  @override
  String get textureAutoRecommended => '自动（推荐）';

  @override
  String get detectionDlcPresent => 'DLC：已存在';

  @override
  String get detectionDlcNotDetected => 'DLC：未检测到';

  @override
  String get detectionDlcPresentTooltip =>
      '已找到 DLC data100.cpk。附带仅限 DLC 的服装文件（pl000d、pl010d、pl020d）的模组将按原样安装。';

  @override
  String get detectionDlcNotDetectedTooltip =>
      '未检测到 DLC。附带仅限 DLC 的服装文件（pl000d、pl010d、pl020d）的模组将以非 DLC 名称（pl0000、pl0100、pl0200）安装，以便在游戏中显示。';

  @override
  String get detectionExeWolfLimitBreak => 'EXE：Wolf Limit Break';

  @override
  String get detectionExeOriginal => 'EXE：原版';

  @override
  String get detectionExeMissing => 'EXE：缺失';

  @override
  String get detectionExeUnrecognised => 'EXE：无法识别';

  @override
  String get detectionExeUnrecognisedTooltip =>
      'NieRAutomata.exe 存在，但其哈希值不在我们的已知列表中。NAMS 仍可运行；这只是提醒你我们尚未见过这个确切的版本。';

  @override
  String get detectionExeWolfLimitBreakTooltip =>
      '检测到 Wolf Limit Break 版本的 NieRAutomata.exe。NAMS 不需要此补丁，且从未针对它进行测试。游戏或许仍能启动，但可能出现性能问题、内存崩溃或模组不兼容。若需完整支持，请还原原版 Steam 可执行文件（在 Steam 中验证游戏文件）。';

  @override
  String get detectionExeLegacyWin7 => 'EXE：Windows 7 版本';

  @override
  String get detectionExeLegacyWin7Tooltip =>
      '这是旧版 Windows 7/8 的 NieRAutomata.exe。NAMS 需要 Windows 10/11 的 Steam 版本，无法启动此版本。这在 Proton/Linux 上很常见，Steam 有时会下载 Windows 7 可执行文件。\n\n解决方法：\n1. 删除 NieRAutomata 游戏文件夹中的每个 .exe 文件。\n2. 在 Steam 中将 Proton 设置为“Proton Experimental”（右键点击游戏 > 属性 > 兼容性）。\n3. 右键点击游戏 > 属性 > 已安装文件 > 验证游戏文件完整性。\n4. 通过 Steam 启动一次游戏以保留正确的可执行文件，然后再次使用启动器。';

  @override
  String get launchOptionsTitle => '启动选项';

  @override
  String get launchWrapperTitle => '启动包装命令 (LINUX)';

  @override
  String get launchWrapperDesc =>
      '在游戏启动前添加一个命令，例如 gamescope 或 mangohud。启动器通过 Proton 运行游戏，因此 Steam 启动选项在此无效。留空则正常启动。将在下次启动时生效。';

  @override
  String get launchWrapperHint => 'gamescope -w 2560 -h 1440 -f --';

  @override
  String get launchWrapperExample =>
      '示例：\ngamescope -w 2560 -h 1440 -f --\nmangohud\ngamemoderun';

  @override
  String get tabLauncherSettings => '启动器设置';

  @override
  String get verifyInstallTitle => '安装诊断';

  @override
  String get verifyInstallDesc =>
      '运行 NAMS 内置检查以诊断游戏无法启动的原因（错误的 Windows 版本、缺失的 Steam 文件、权限）。';

  @override
  String get verifyInstallButton => '验证安装';

  @override
  String get verifyInstallRunning => '检查中...';

  @override
  String get verifyInstallOk => '所有检查均已通过。';

  @override
  String get verifyInstallFailed => '部分检查失败。详见下方。';

  @override
  String get verifyNoRuntime => '无法验证：未找到此安装的 Proton/Wine 运行时。';

  @override
  String get verifySteamNotRunning => '无法验证：Steam 必须正在运行并拥有该游戏。';

  @override
  String get verifyInstallError => '无法运行检查。请确保已选择游戏文件夹。';

  @override
  String get verifyInstallNoGameDir => '请先选择你的游戏文件夹。';

  @override
  String get verifyCheckSteamInstall => 'Steam 安装';

  @override
  String get verifyCheckNierExe => '游戏可执行文件';

  @override
  String get verifyCheckSteamApi64 => 'Steam API 库';

  @override
  String get verifyCheckRuntimeWritable => '运行时可写';

  @override
  String get verifyCheckRuntimeCached => '运行时库已缓存';

  @override
  String get launchOptionMinimizeOnLaunch => '游戏时最小化启动器';

  @override
  String get launchOptionPreferDedicatedGpu => '优先使用独立显卡';

  @override
  String get launchOptionPreferDedicatedGpuTooltip =>
      '让系统使用独立显卡而非节能显卡运行游戏。仅在配备双显卡的电脑上有效（例如游戏笔记本）。';

  @override
  String get failTitlePanic => 'NAMS 已崩溃';

  @override
  String get failTitleUnknown => '游戏启动失败';

  @override
  String get failExplanationPanic =>
      'NAMS 在游戏启动前遇到了无法恢复的错误。这几乎总是一个程序缺陷 — 请将下面的报告分享给维护者。';

  @override
  String get failExplanationUnknown => '游戏在 60 秒内未能启动，且没有报告任何错误。';

  @override
  String get failHintPanicShare => '复制下面的完整报告并发送给维护者。';

  @override
  String get failHintPanicReboot => '重启电脑后再试一次 — 有时残留的句柄会自行清除。';

  @override
  String get failHintUnknownSpawned => 'NAMS 似乎已启动，但游戏窗口从未出现。';

  @override
  String get failHintUnknownTaskManager =>
      '请检查任务管理器 — NieRAutomata.exe 是否正在运行但不可见？结束该进程后重试。';

  @override
  String get failHintUnknownOtherLauncher =>
      '请确认没有其他启动器或 DRM 工具占用该程序（如 FAR、Special K 等）。';

  @override
  String get failTitleNamsFailure => 'NAMS 报告了一个故障';

  @override
  String get failExplanationNamsFailure => '游戏运行前 NAMS 的一项检查失败。详情请参阅下面的报告。';

  @override
  String get failHintShareReport => '复制下面的完整报告并分享以便诊断。';

  @override
  String get failTitleInstallNotFound => '未找到 NieR:Automata 安装目录';

  @override
  String get failExplanationInstallNotFound =>
      'NAMS 无法解析您的 NieR:Automata 安装目录。保存的路径可能有误，或 Steam 自动检测失败。';

  @override
  String get failHintRepickDirectory => '在启动器中重新选择游戏目录以刷新保存的路径。';

  @override
  String get failHintVerifyFiles =>
      '在 Steam 中验证游戏文件（库 → NieR:Automata → 属性 → 本地文件 → 验证）。';

  @override
  String get failTitleFolderCreate => '无法创建所需的文件夹';

  @override
  String get failExplanationFolderCreate =>
      'NAMS 无法在 NAMS.exe 旁创建目录。安装文件夹可能为只读。';

  @override
  String get failHintWritableFolder => '请确保启动器安装文件夹（NAMS.exe 所在位置）可写。';

  @override
  String get failHintProgramFiles =>
      '如果它位于 Program Files 或已由 OneDrive 同步，请将启动器移至普通文件夹，或右键单击 → “始终保留在此设备上”。';

  @override
  String get failTitleRuntimePrep => '运行时准备失败';

  @override
  String get failExplanationRuntimePrep =>
      'NAMS 无法准备其运行时（game.bin / steam_api64.dll）。这通常是可写性或杀毒软件的问题。';

  @override
  String get failHintAntivirusExclusions =>
      '将启动器安装文件夹和游戏文件夹一并添加到杀毒软件的排除项中，然后重试。';

  @override
  String get failHintWritableCache => '请确保安装文件夹可写，以便构建运行时缓存。';

  @override
  String get failTitleHostFailure => 'NAMS 宿主进程故障';

  @override
  String get failExplanationHostFailure =>
      'NAMS 无法加载并启动游戏宿主（game.bin）。这通常是环境问题或文件损坏所致。';

  @override
  String get failHintReboot => '重启后再试一次 — 有时残留的句柄会自行清除。';

  @override
  String get failHintPersistShare => '如果问题持续存在，请复制完整报告并发送给维护者。';

  @override
  String get failTitleSteamNotRunning => 'Steam 未运行或未登录';

  @override
  String get failExplanationSteamNotRunning =>
      'NAMS 无法连接到已登录的 Steam 会话。Steam 必须处于运行且已登录状态。';

  @override
  String get failHintStartSteam => '启动 Steam 并登录，然后重新启动游戏。';

  @override
  String get failTitleSteamNotOwned => '该 Steam 账户未拥有 NieR:Automata';

  @override
  String get failExplanationSteamNotOwned => '当前登录的 Steam 账户未拥有 NieR:Automata。';

  @override
  String get failHintSignInOwner => '请登录拥有 NieR:Automata 的 Steam 账户。';

  @override
  String get failTitleSteamCheckFailed => 'Steam 检查失败';

  @override
  String get failExplanationSteamCheckFailed => 'NAMS 在验证 Steam 所有权时遇到内部错误。';

  @override
  String get failHintRestartSteam => '重启 Steam 和启动器，然后重试。';

  @override
  String get failTitleInvalidArgs => '启动参数无效';

  @override
  String get failExplanationInvalidArgs => '启动器传递了 NAMS 无法解析的参数。这是启动器的缺陷。';

  @override
  String get failTitleExitedUnexpectedly => '游戏意外退出';

  @override
  String get failExplanationExitedUnexpectedly =>
      'NAMS 已启动游戏，但游戏以非零代码退出。游戏可能已崩溃。';

  @override
  String get failHintCheckLogViewer => '请查看应用内的日志查看器（nams.log）以获取崩溃详情。';

  @override
  String get failHeadlinePanicked => 'NAMS 发生严重错误';

  @override
  String get failSectionWhatHappened => '发生了什么';

  @override
  String get failSectionReportedByNams => 'NAMS 报告内容';

  @override
  String get failSectionTryThis => '尝试以下方法';

  @override
  String get failSectionDiagnosticDetail => '诊断详情';

  @override
  String get failSectionLaunchManually => '从终端手动启动';

  @override
  String get failSectionRawOutput => '原始输出';

  @override
  String get failManualCommandHint =>
      '如果启动器界面始终无法正常工作，可将此命令粘贴到终端中手动启动游戏。它与“开始”按钮运行的命令完全相同。';

  @override
  String get failDetailOs => '操作系统';

  @override
  String get failDetailCause => '原因';

  @override
  String get failDetailSuggested => '建议';

  @override
  String get failActionCopyReport => '复制报告';

  @override
  String get failActionOpenLogFile => '打开日志文件';

  @override
  String get logDetailOs => '操作系统';

  @override
  String get logDetailLocale => '区域设置';

  @override
  String get logNoModsInstalled => '未安装任何模组。';

  @override
  String get logSectionSystem => '系统';

  @override
  String get logSectionModsNams => '模组 (NAMS)';

  @override
  String get logSectionCutscenes => '过场动画';

  @override
  String get logSectionTextures => '纹理';

  @override
  String get tooltipOpenInModManager => '在模组管理器中打开';

  @override
  String get tooltipOpenInCutscenesTab => '在过场动画标签页中打开';

  @override
  String tooltipOpenInTexturesTab(String name) {
    return '$name\n\n在纹理标签页中打开';
  }

  @override
  String get actionCancel => '取消';

  @override
  String get tooltipMinimize => '最小化';

  @override
  String get tooltipMaximize => '最大化';

  @override
  String get tooltipRestore => '还原';

  @override
  String get tooltipClose => '关闭';

  @override
  String get infoText => '选择游戏，点击开始，就这么简单。';

  @override
  String get helpPrefix => '启动器无法使用？请尝试 ';

  @override
  String get helpNaoLauncher => 'NAO 启动器';

  @override
  String get helpOr => ' 或 ';

  @override
  String get helpCommandLine => '命令行';

  @override
  String get helpJoinDiscord => '。加入我们的 ';

  @override
  String get helpDiscord => 'Discord';

  @override
  String get helpSuffix => ' 获取帮助。';

  @override
  String get noFileSelected => '未选择文件';

  @override
  String get selectButton => '选择';

  @override
  String get filePickerTitle => '选择游戏可执行文件';

  @override
  String get playButton => '开始';

  @override
  String get stopButton => '停止';

  @override
  String get statusReady => '已准备好启动！';

  @override
  String get statusSelectGame => '请选择游戏可执行文件以开始。';

  @override
  String get statusPreparing => '正在准备启动器文件...';

  @override
  String get statusLaunching => '正在启动游戏...';

  @override
  String get statusRunning => '游戏正在运行。';

  @override
  String get statusStopped => '游戏已停止。';

  @override
  String get statusStopping => '正在停止游戏...';

  @override
  String get errorInvalidExe => '这似乎不是《NieR:Automata》。请确保使用的是最新 Steam 版本。';

  @override
  String get errorFileNotExist => '所选文件不存在';

  @override
  String get errorSelectFailed => '选择文件失败';

  @override
  String get errorStartFailed => '启动游戏失败。';

  @override
  String get errorStopFailed => '停止游戏失败。';

  @override
  String errorFilesQuarantined(String files) {
    return '缺少启动器文件：$files。这通常由杀毒软件导致。我们会在运行时加载模组，这是游戏模组的常规做法，但可能触发误报。请从隔离区恢复文件或重新下载启动器，然后将启动器安装文件夹（包含 NAMS.exe 的文件夹）添加到排除项。';
  }

  @override
  String get notifyFilesQuarantined =>
      '检测到缺少启动器文件。这通常由杀毒软件导致。我们会在运行时加载模组，这是游戏模组的正常做法，但可能触发误报。请从隔离区恢复文件或重新下载启动器，然后将启动器安装文件夹（包含 NAMS.exe 的文件夹）添加到排除项。';

  @override
  String get featureReshade => 'ReShade - 已安装？YP 会自动检测。';

  @override
  String get featureTextures =>
      '高清纹理 - 将纹理放入 nams/inject/textures/，也可从 SK_Res/ 自动读取。';

  @override
  String get featureLodMod => 'LOD Mod - 内置阴影、细节和物体弹出等画面调整。默认关闭。';

  @override
  String get tooltipEditConfigs => '无需编辑文件即可更改画面设置';

  @override
  String get tooltipOpenLogs => '在资源管理器中打开日志文件夹';

  @override
  String get tooltipCreateShortcut => '创建使用 YoRHa Protocol 启动的桌面快捷方式';

  @override
  String get shortcutName => 'NieR Automata (YoRHa Protocol)';

  @override
  String get shortcutDescription => '使用 YoRHa Protocol 启动 NieR:Automata';

  @override
  String get notifyShortcutCreated => '已创建桌面快捷方式！';

  @override
  String get notifyShortcutFailed => '创建桌面快捷方式失败。';

  @override
  String get notifyLodModMigrated =>
      '已找到旧的 LodMod.ini 设置，并导入 lodmod.toml，同时启用了 LodMod。';

  @override
  String get notifyReShadeDetected =>
      '检测到 ReShade，默认已禁用。NAMS 已自带修补后的原生景深效果，因此 ReShade 为可选项。你可以随时在 NAMS 配置选项卡中重新启用（关闭“禁用 ReShade 加载”）。';

  @override
  String get notifyNaiomMigrated =>
      '已找到旧的 NAIOM 设置并导入 nams.toml。请查看 NAIOM 选项卡。你可以从游戏文件夹中删除旧的 NAIOM 文件（dinput8.dll、NAIOM.ini）。';

  @override
  String notifyNaiomSkipped(String entries) {
    return '部分 NAIOM 按键绑定使用了 NAMS 不支持的按键，未被导入：$entries。请在 NAIOM 选项卡中重新绑定。';
  }

  @override
  String notifyPlatformUnsupported(String platform) {
    return '在 $platform 上未找到 Windows 兼容层，因此无法从这里启动游戏。模组、纹理和配置仍可正常使用。请安装 CrossOver 并将 NieR:Automata 放入 bottle 中以启用启动功能。';
  }

  @override
  String get notifyReShadeIncompatible =>
      '检测到支持插件/ImGui 的 ReShade，无法兼容。请使用不含插件支持的标准版 ReShade。';

  @override
  String notifyTexturesDetected(String folder) {
    return '在 $folder 中发现高清纹理，将在启动时加载。';
  }

  @override
  String get errorAppDataNotFound => '未找到 APPDATA 环境变量';

  @override
  String get errorWinePrefixNotFound => '未找到 Wine 前缀。请设置 WINEPREFIX 环境变量。';

  @override
  String get errorHomeNotFound => '未找到 HOME 环境变量';

  @override
  String get errorNoWineUser => 'Wine 前缀中未找到用户目录';

  @override
  String errorWineUsersNotFound(String prefix) {
    return '在 $prefix 中未找到 Wine 的 drive_c/users 目录';
  }

  @override
  String errorPlatformNotSupported(String os) {
    return '不支持平台 $os';
  }

  @override
  String errorExeNotFound(String dir) {
    return '在 $dir 中未找到 NieRAutomata.exe';
  }

  @override
  String get errorDirNotWritable => '启动器文件夹为只读';

  @override
  String errorDirNotWritableBody(String dir) {
    return '无法写入 YP Launcher 文件夹：\n$dir\n\nNAMS 会将运行时缓存、插件和日志写入 NAMS.exe 旁边，因此启动器文件夹必须可写。\n\n解决方法：\n1. 关闭启动器。\n2. 将解压后的整个 YP Launcher 文件夹移出 Program Files（或其他受保护位置），放到你拥有写入权限的普通文件夹中，例如桌面、文档或 D:\\Games\\YP Launcher。\n3. 从新位置重新启动启动器。\n\n替代方案：右键单击启动器 .exe，选择“以管理员身份运行”，以允许写入当前位置。';
  }

  @override
  String get errorGameDirNotWritable => '游戏文件夹为只读';

  @override
  String errorGameDirNotWritableBody(String gameDir, String namsDir) {
    return '无法写入游戏文件夹：\n$gameDir\n\nNAMS 会将模组和配置写入：\n$namsDir\n但无法在该位置创建文件。NieR 很可能安装在 Windows 受保护的 C:\\Program Files (x86)\\Steam 下。\n\n解决方法（推荐：将 Steam 库移出 Program Files）：\n1. 打开 Steam > 设置 > 存储。\n2. 点击驱动器下拉菜单 > “添加驱动器”，并选择另一块驱动器上的文件夹（例如 D:\\SteamLibrary）。\n3. 前往游戏库，右键单击 NieR:Automata > 属性 > 已安装文件 > “移动安装文件夹”，然后将其移动到新库。\n4. 在此启动器中重新选择游戏并再次点击开始。\n\n快速替代方案：右键单击启动器 .exe，选择“以管理员身份运行”，以便写入 Program Files。长期而言，移动游戏库是更干净的解决方案。';
  }

  @override
  String get errorNoCompatLayer => '未找到 CrossOver';

  @override
  String get errorNoCompatLayerBody =>
      '在此系统上运行 NieR:Automata 需要 CrossOver，它可在 macOS 上运行 Windows 程序。未在 /Applications 中找到 CrossOver。\n\n没有它时，启动器仍可管理模组、纹理和配置，但无法启动游戏。\n\n解决方法：\n1. 从 codeweavers.com 安装 CrossOver。\n2. 在 CrossOver 容器中安装 Steam 和 NieR:Automata。\n3. 在此启动器中选择该容器内的 NieRAutomata.exe。';

  @override
  String get errorNoCompatLayerLinux => '未找到 Proton 或 Wine';

  @override
  String get errorNoCompatLayerLinuxBody =>
      '在 Linux 上运行 NieR:Automata 需要 Proton（推荐）或 Wine，但未找到任何一个。\n\n没有它，启动器仍可管理模组、纹理和配置，只是无法启动游戏。\n\n解决方法：\n1. 在 Steam 中安装一个 Proton 版本（推荐 Proton Experimental）。如果它在另一个磁盘上，启动器现在会检查每个 Steam 库。\n2. 确保你从 Steam 库中选择了 NieRAutomata.exe（路径包含 steamapps/common）。\n3. 或在启动启动器前将 YP_PROTON_PATH 设置为你的 proton 可执行文件，例如 YP_PROTON_PATH=\"\$HOME/.steam/steam/steamapps/common/Proton - Experimental/proton\"。';

  @override
  String get errorProtonMissing => '未找到 Proton';

  @override
  String errorProtonMissingBody(String path) {
    return '配置的 Proton 运行时在以下位置缺失：\n$path\n\n请通过 Steam 重新安装 Proton，或在启动启动器前将 YP_PROTON_PATH 设置为有效的 proton 可执行文件。';
  }

  @override
  String get errorNoZDrive => 'Wine 前缀中没有 Z: 驱动器';

  @override
  String errorNoZDriveBody(String prefix) {
    return 'Wine 会将 Z: 映射到主机文件系统，启动器正是通过它将 NAMS.exe 交给游戏。此前缀缺少 dosdevices/z::\n$prefix\n\n这是 CrossOver 的默认设置，因此该容器可能已被修改。最快的解决方法是新建容器并在其中重新安装游戏。';
  }

  @override
  String get errorExeOutsidePrefix => '该可执行文件不在容器内';

  @override
  String errorExeOutsidePrefixBody(String exeName, String path) {
    return '启动器通过 CrossOver 启动游戏，因此 $exeName 必须位于 CrossOver 容器中：\n$path\n\n请将游戏安装到容器中，然后从该位置选择可执行文件。';
  }

  @override
  String get headerNams => 'NAMS';

  @override
  String get headerLodMod => 'LOD MOD';

  @override
  String get headerTextures => '纹理';

  @override
  String get headerYorhaProtocol => 'YORHA PROTOCOL';

  @override
  String get headerNaiom => 'NAIOM';

  @override
  String get headerCutscenes => '过场动画';

  @override
  String get tooltipEditsNamsToml => '编辑 nams/nams.toml';

  @override
  String get tooltipEditsLodmodToml => '编辑 nams/lodmod.toml';

  @override
  String get tooltipEditsTextureInjectionToml =>
      '编辑 nams/texture_injection.toml';

  @override
  String get tooltipEditsSettingsJson => '编辑 %APPDATA%\\NAMS\\settings.json';

  @override
  String get tooltipEditsNaiom =>
      '编辑 nams/nams.toml 中的 [mouse] 设置。大多数设置保存后生效，少数设置需要重启游戏。';

  @override
  String get tooltipCutscenesLocation =>
      '模组：nams/cutscenes/<mod_name>/movie/*.usm';

  @override
  String get cardGeneral => '常规';

  @override
  String get cardLoading => '加载';

  @override
  String get cardHeapOverrides => '堆内存覆盖';

  @override
  String get cardLevelOfDetail => '细节层次';

  @override
  String get cardAmbientOcclusion => '环境光遮蔽';

  @override
  String get cardPostProcessing => '后期处理';

  @override
  String get cardShadows => '阴影';

  @override
  String get cardGlobalIllumination => '全局光照';

  @override
  String get cardPreloading => '预加载';

  @override
  String get cardTextureConfig => '配置';

  @override
  String get cardInstallTextures => '安装纹理';

  @override
  String get cardHowItWorks => '工作原理';

  @override
  String get cardKeybinds => '按键绑定';

  @override
  String get cardWorkspace => '工作区';

  @override
  String get cardCheats => '作弊';

  @override
  String get cardRandomizer => '随机化器';

  @override
  String get cardThirdPersonCamera => '第三人称镜头';

  @override
  String get cardPodAiming => 'Pod / 瞄准';

  @override
  String get cardMisc => '其他';

  @override
  String get cardMovementBindings => '移动按键绑定';

  @override
  String get cardCombatBindings => '战斗按键绑定';

  @override
  String get cardNonStandardBindings => '非标准按键绑定';

  @override
  String get cardMenuNavigation => '菜单导航';

  @override
  String get cardStructure => '结构';

  @override
  String get buttonSave => '保存';

  @override
  String get buttonDiscard => '放弃';

  @override
  String get buttonCancel => '取消';

  @override
  String get buttonInstall => '安装';

  @override
  String get buttonDelete => '删除';

  @override
  String get buttonYes => '是';

  @override
  String get buttonNo => '否';

  @override
  String get buttonContinue => '继续';

  @override
  String get buttonBack => '返回';

  @override
  String get buttonGetStarted => '开始使用';

  @override
  String get buttonAutoDetect => '自动检测';

  @override
  String get buttonSelectManually => '手动选择';

  @override
  String get buttonGoToLauncher => '前往启动器';

  @override
  String get unsavedChangesTitle => '未保存的更改';

  @override
  String get unsavedChangesMessage => '你有未保存的更改。要放弃吗？';

  @override
  String get stay => '保留';

  @override
  String get discard => '放弃';

  @override
  String get enterValidNumber => '请输入有效数字';

  @override
  String get pressKey => '请按键...';

  @override
  String get tabLauncher => '启动器';

  @override
  String get tabYorhaProtocol => 'YP 开发工具包';

  @override
  String get tabNams => 'NAMS';

  @override
  String get tabLodMod => 'LOD Mod';

  @override
  String get tabNaiom => 'NAIOM';

  @override
  String get tabTextures => '纹理';

  @override
  String get tabMods => '模组管理器';

  @override
  String get tabCutscenes => '过场动画';

  @override
  String get tabSectionGeneral => '常规';

  @override
  String get tabSectionConfig => '配置';

  @override
  String get tabSectionMods => '模组';

  @override
  String get tabDividerConfig => '配置';

  @override
  String get tabDividerMods => '模组';

  @override
  String get infoBarVersionPrefix => 'YP Launcher ';

  @override
  String get infoBarLogs => '日志';

  @override
  String get infoBarFaq => '常见问题';

  @override
  String get infoBarWhatsNew => '新增内容';

  @override
  String get infoBarShortcut => '快捷方式';

  @override
  String get tooltipFaq => '我还需要其他模组吗？';

  @override
  String get chipLodModOn => 'LOD MOD 已开启';

  @override
  String get chipLodModOff => 'LOD MOD 已关闭';

  @override
  String get chipReShade => 'ReShade';

  @override
  String get chipNoTextures => '无纹理';

  @override
  String get chipNoMods => '无模组';

  @override
  String get chipNoCutsceneMod => '无过场动画模组';

  @override
  String chipTexturesCount(String details) {
    return '纹理（$details）';
  }

  @override
  String chipModsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '个模组',
      one: '个模组',
    );
    return '$count $_temp0';
  }

  @override
  String chipInjectedCount(int count) {
    return '已注入 $count 个';
  }

  @override
  String get chipSkRes => 'SK_Res';

  @override
  String chipCutsceneMod(int width, int height, String codec) {
    return '过场动画模组（${width}x$height $codec）';
  }

  @override
  String get warningPluginLoadingDisabled => '插件加载已禁用，YoRHa Protocol 工作区不会加载';

  @override
  String get warningReShadeDisabled => 'ReShade 自动加载已禁用';

  @override
  String get warningTextureInjectionDisabled => '纹理注入已禁用';

  @override
  String get heapOverridesDescription =>
      '在 NAMS 默认值之外额外增加内存。父堆会自动扩展。仅当模组需要更多内存时才提高。需要重启。';

  @override
  String get lodModDescription =>
      'NAMS 内置的画质补丁，灵感来自 emoose 的 Automata-LodMod。可消除 LOD 物体弹出、锐化阴影和环境光遮蔽、强制包括植被在内的所有物体投射阴影、禁用手动剔除以避免物体突然出现或消失，并移除暗角。';

  @override
  String get dropModelModHere => '将模型模组文件夹或压缩包拖放到此处';

  @override
  String get dropToInstall => '拖放以安装';

  @override
  String get orClickToBrowse => '或点击浏览';

  @override
  String get mixedModDetected => '检测到混合模组';

  @override
  String get noOutfitsInstalled => '尚未安装服装';

  @override
  String get defaultNoMod => '默认（无模组）';

  @override
  String get clearAllStartupOutfits => '清除所有启动服装';

  @override
  String get removeOutfitTitle => '移除服装？';

  @override
  String get noModelFoundError =>
      '未找到模型。需要 pl000d.dat/.dtt（2B）、pl010d.dat/.dtt（A2）或 pl020d.dat/.dtt（9S）。';

  @override
  String get unsupportedArchiveFormat => '不支持的压缩包格式。';

  @override
  String get extractingArchive => '正在解压压缩包...';

  @override
  String extractingArchivePercent(int percent) {
    return '正在解压 - $percent%';
  }

  @override
  String extractingArchivePercentFile(int percent, String file) {
    return '正在解压 $percent% - $file';
  }

  @override
  String get cutsceneScanningArchive => '正在扫描压缩包中的 movie 文件夹...';

  @override
  String cutsceneCopyingFile(int current, int total, String name) {
    return '正在复制 $current/$total - $name';
  }

  @override
  String cutsceneCopyingFileBytes(
    int current,
    int total,
    String name,
    String mbDone,
    String mbTotal,
  ) {
    return '正在复制 $current/$total - $name（$mbDone / $mbTotal MB）';
  }

  @override
  String get failedToExtractArchive => '解压压缩包失败。';

  @override
  String get extractFailedDiskFull => '解压失败：临时驱动器可用空间不足。请释放磁盘空间后重试。';

  @override
  String get textureDeleteConfirmTitle => '删除纹理包？';

  @override
  String textureDeleteConfirmBody(String name) {
    return '要从 nams/inject/textures/ 永久移除 $name 吗？此操作无法撤销。';
  }

  @override
  String get textureDeleteLabel => '删除';

  @override
  String get busyDeletingTexture => '正在删除纹理包...';

  @override
  String get busyDeletingCutscene => '正在删除过场动画模组...';

  @override
  String get busyCloseTitle => '操作正在进行';

  @override
  String get busyCloseBody =>
      '启动器正在安装、删除或解压文件。此时关闭可能会在磁盘上留下损坏或不完整的文件。请等待当前操作完成，或仍然强制关闭。';

  @override
  String get busyCloseForce => '强制关闭';

  @override
  String extractFailedDetail(String detail) {
    return '解压失败：$detail';
  }

  @override
  String get noOutfitsInstalledNotif => '未安装任何服装。';

  @override
  String get dialogSelectModFolder => '选择模型模组文件夹';

  @override
  String get dialogNameOutfitShown => '切换服装时，此名称将显示在游戏中。';

  @override
  String get dropTextureHere => '将纹理文件夹或压缩包拖放到此处';

  @override
  String get installedToTextures => '已安装到：nams/inject/textures/';

  @override
  String get installingTextures => '正在安装纹理...';

  @override
  String get textureDropAnalyzing => '正在分析拖入的文件...';

  @override
  String get textureDropNoDds => '拖入内容中未找到 .dds 纹理。仅支持 .dds 文件。';

  @override
  String get cutsceneDropAnalyzing => '正在分析拖入的文件...';

  @override
  String get cutsceneDropNoUsm => '拖入内容中未找到 .usm 过场动画文件。';

  @override
  String get modDropAnalyzing => '正在分析拖入的文件...';

  @override
  String get modDropNotAMod =>
      '这似乎不是 NAMS 模组。请拖入包含 entities/、wax/、data/ 或 mod.toml 的文件夹/压缩包。';

  @override
  String get modDropMisroutedCutscenes =>
      '这似乎是独立的过场动画模组，请改为拖放到“过场动画”选项卡。捆绑的过场动画应位于已经包含 entities/ 或 mod.toml 的模组内部。';

  @override
  String modSideInstalledTextures(String names) {
    return '还将捆绑的纹理包安装到了 nams/inject/textures/：$names';
  }

  @override
  String modLooseUnpairedWarning(String names) {
    return '已安装，但部分文件缺少其原版配对文件（.dat/.dtt）：$names。模组可能无法完全生效。';
  }

  @override
  String get modBundledTexturesLabel => '捆绑纹理';

  @override
  String get modBundledCutscenesLabel => '捆绑过场动画';

  @override
  String textureBundledWithMod(String modId) {
    return '随模组捆绑：$modId';
  }

  @override
  String modUninstallAlsoTexturesLabel(String names) {
    return '同时删除捆绑的纹理包：$names';
  }

  @override
  String get noTextures => '无纹理';

  @override
  String get noTexturesInstalled => '未安装纹理';

  @override
  String get textureConflictHint => '两个模组都会加载，并且修改了部分相同内容。请将你更重视的模组放在上方。';

  @override
  String get noConflictsFound => '未发现冲突。所有模组均独立加载。';

  @override
  String get selectTextureFiles => '选择纹理文件或压缩包';

  @override
  String get noTextureFilesFound => '未找到纹理文件（.dds、.png、.tga）';

  @override
  String get cheatsAppliedNote => '在游戏启动时应用一次。';

  @override
  String get wipLabel => '开发中';

  @override
  String get dropCutsceneHere => '将过场动画模组文件夹或压缩包拖放到此处';

  @override
  String get cutsceneMovieHint => '模组必须包含带有 .usm 文件的 movie/ 文件夹';

  @override
  String get cutsceneNamingTitle => '为此过场动画模组命名';

  @override
  String get removeCutsceneModTitle => '移除过场动画模组？';

  @override
  String get noCutsceneModsInstalled => '尚未安装过场动画模组';

  @override
  String get cutsceneHowItWorks1 =>
      '自定义过场动画从 nams/cutscenes/ 加载，而不是 data/movie/。';

  @override
  String get cutsceneHowItWorks2 => '如果自定义文件缺失或损坏，将回退播放原版文件。';

  @override
  String get cutsceneHowItWorks3 => '原始游戏文件绝不会被修改，模组会从单独位置加载。';

  @override
  String get cutsceneStructurePath =>
      'nams/cutscenes/<mod_name>/movie/<filename>.usm';

  @override
  String get cutsceneFolderNameLimit => '文件夹名称最多 27 个字符。';

  @override
  String get cutsceneMigrationTitle => '在 data/movie/ 中检测到自定义过场动画文件';

  @override
  String cutsceneMigrationBody(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '文件',
      one: '文件',
    );
    return '在 data/movie/ 中直接发现 $count 个自定义过场动画$_temp0。这些文件会永久覆盖原版文件。下次请改为在此处安装过场动画模组；如果自定义文件加载失败，将回退播放原版文件。';
  }

  @override
  String get noMovieFolderFound => '未找到包含 .usm 文件的 movie/ 文件夹。';

  @override
  String get noUsmFilesFound => '模组中未找到 .usm 文件。';

  @override
  String get onboardingWelcomeTitle => '欢迎使用 YoRHa Protocol';

  @override
  String get onboardingWelcomeSubtitle =>
      '一个启动器，管理所有模组，体验全新的 NieR 内容。\n\n支持拖放安装模组、游戏中途切换服装、新任务以及内置开发工具包，不再受复杂配置困扰。';

  @override
  String get onboardingSelectTitle => '选择你的 NieR:Automata 安装位置';

  @override
  String get onboardingSearchingDrives => '正在扫描所有驱动器...';

  @override
  String get onboardingSearchingNier => '正在搜索 NieR:Automata...';

  @override
  String get onboardingSelectInstallation => '选择安装位置';

  @override
  String get onboardingFirstPlaythroughTitle => '这是你第一次游玩吗？';

  @override
  String get onboardingFirstPlaythroughHint => '若是，我们会隐藏剧透内容。';

  @override
  String get onboardingFirstYes => '是 - 隐藏剧透功能';

  @override
  String get onboardingFirstNo => '否 - 显示全部内容';

  @override
  String get onboardingMigrationAskTitle => '以前给 NieR 安装过模组吗？';

  @override
  String get onboardingMigrationAskBody => '我们会自动识别你的旧配置。';

  @override
  String get onboardingMigrationYes => '是';

  @override
  String get onboardingMigrationNo => '否';

  @override
  String get onboardingMigrationResultsTitle => '我们发现的内容';

  @override
  String get onboardingMigrationResultsBody => '';

  @override
  String get onboardingMigrationNothingFound => '未检测到任何内容。当前环境很干净。';

  @override
  String get onboardingMigrationActionReshadeKept =>
      '已禁用 - NAMS 自带原生景深。需要时可在 NAMS 配置中重新启用。';

  @override
  String get onboardingMigrationActionReshadeIncompatible =>
      '插件/ImGui 版本 - 请替换或移除。';

  @override
  String get onboardingMigrationActionTextures => '将自动加载。';

  @override
  String get onboardingMigrationActionLodMod => '已导入 LodMod.ini。';

  @override
  String get onboardingMigrationActionSkRes => '已自动识别。';

  @override
  String get onboardingMigrationActionNaiom =>
      '你的 NAIOM 设置会自动导入 NAMS。之后可以删除旧的 NAIOM 文件（dinput8.dll、NAIOM.ini）。';

  @override
  String get onboardingMigrationActionCutscenes => '已集成。';

  @override
  String get onboardingMigrationActionExistingMods => '已经安装 - 保持原样。';

  @override
  String onboardingMigrationLabelExistingMods(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'nams/mods/ 中有 $count 个模组',
      one: 'nams/mods/ 中有 1 个模组',
    );
    return '$_temp0';
  }

  @override
  String get onboardingModInstallAskTitle => '启动前要安装模组吗？';

  @override
  String get onboardingModInstallAskBody =>
      '有 .zip 文件或文件夹？交给我们处理。\n（纹理和过场动画有各自的选项卡，稍后再安装。）';

  @override
  String get onboardingModInstallYes => '是，现在安装模组';

  @override
  String get onboardingModInstallSkip => '否，暂时跳过';

  @override
  String get onboardingModInstallTitle => '添加你的第一个模组';

  @override
  String get onboardingModInstallBody => '支持 .zip、.7z 和文件夹。';

  @override
  String get onboardingModInstallSubBody => '游戏文件不会受到影响。';

  @override
  String get onboardingModInstallBusy => '正在安装…';

  @override
  String get onboardingModInstallInspecting => '正在检查模组…';

  @override
  String onboardingModInstallExtractingPercent(int percent) {
    return '正在解压… $percent%';
  }

  @override
  String onboardingModInstallExtractingFile(int percent, String file) {
    return '正在解压 $percent% - $file';
  }

  @override
  String get onboardingModInstallNotAMod =>
      '这似乎不是有效的 NAMS 模组。请确保它是包含 mod.toml 或可识别数据结构的 .zip / .7z 文件。';

  @override
  String get onboardingOutfitHintHeader => '穿戴方法';

  @override
  String get onboardingOutfitHintCompat => '从物品栏购买或装备。';

  @override
  String get onboardingOutfitHintData => '游戏中按 F1 → 左上角衣柜图标。';

  @override
  String get onboardingOutfitHintMultiOutfit => '也可使用抵抗军营地中的 Multi-Outfit NPC。';

  @override
  String get onboardingOutfitHintMultiOutfitLink => '获取 Multi-Outfit NPC';

  @override
  String get onboardingOutfitHintMultiOutfitUrl =>
      'https://www.nexusmods.com/nierautomata/mods/789';

  @override
  String onboardingModInstallFailed(String reason) {
    return '安装失败：$reason';
  }

  @override
  String get onboardingModInstallNoGameDir => '尚未选择游戏目录。请返回并先选择安装位置。';

  @override
  String get onboardingModInstallInstalledHeader => '已安装：';

  @override
  String get onboardingModInstallSkipButton => '跳过';

  @override
  String get onboardingModInstallDoneButton => '完成';

  @override
  String get onboardingModInstallPickerTitle => '选择模组（.zip、.7z）或打开文件夹';

  @override
  String get onboardingModInstallFolderPickerTitle => '选择模组文件夹';

  @override
  String get onboardingReadyTitle => '一切准备就绪！';

  @override
  String get onboardingCreateShortcut => '创建桌面快捷方式';

  @override
  String get onboardingFirstPlaythroughSpoilerFree => '首次游玩（无剧透）';

  @override
  String get onboardingFullAccess => '完整访问';

  @override
  String get detectionReShade => 'ReShade';

  @override
  String get detectionHDTextures => '高清纹理';

  @override
  String get detectionLodMod => 'LOD Mod（Automata-LodMod）';

  @override
  String get detectionSkRes => 'Special K 纹理（SK_Res）';

  @override
  String get detectionNaiom => 'NAIOM';

  @override
  String get detectionCutscenes => '过场动画模组（nams/cutscenes）';

  @override
  String get detectionCustomMovie => 'data/movie/ 中的自定义过场动画';

  @override
  String get detectionDetected => '已检测到';

  @override
  String get detectionReShadeIncompatible => '不兼容（插件/ImGui）';

  @override
  String get detectionNotFound => '未找到';

  @override
  String get detectionNoneFound => '未发现任何内容';

  @override
  String get detectionLodModMigrated => '已找到 - 已迁移到 NAMS';

  @override
  String get detectionSkResAuto => '已找到 - 将自动加载';

  @override
  String get detectionNaiomPending => '已找到 - 设置会自动导入';

  @override
  String get detectionNoneInstalled => '未安装任何内容';

  @override
  String get detectionCustomMovieHint => '已找到 - 建议改用 nams/cutscenes/，以便安全回退';

  @override
  String get detectionInstalled => '已安装';

  @override
  String get detectionCustomFilesDetected => '检测到自定义文件';

  @override
  String get detectionMigratedIntoNams => '已迁移到 NAMS';

  @override
  String get detectionLoadedAutomatically => '已自动加载';

  @override
  String get detectionMigrationComingSoon => '设置已自动导入';

  @override
  String get detectionNotSet => '未设置';

  @override
  String get labelGame => '游戏';

  @override
  String get labelMode => '模式';

  @override
  String get labelDlc => 'DLC';

  @override
  String get labelNoDlc => '无 DLC';

  @override
  String get searchingForNier => '正在搜索 NieR:Automata...';

  @override
  String get autoButton => '自动';

  @override
  String get nierFound => '已找到 NieR:Automata';

  @override
  String get selectInstallation => '选择你的安装位置：';

  @override
  String get notYourGame => '这不是你的游戏？';

  @override
  String get searchAllDrives => '搜索所有驱动器';

  @override
  String get selectManually => '手动选择';

  @override
  String get notFoundTitle => '未找到';

  @override
  String get notFoundMessage =>
      '无法通过 Steam 找到 NieR:Automata。要扫描所有驱动器吗？这可能最多需要 30 秒。';

  @override
  String get scanDrives => '扫描驱动器';

  @override
  String get confirmInstallation => '这是正确的安装位置吗？';

  @override
  String get cancelButton => '取消';

  @override
  String get noSelectManually => '否，手动选择';

  @override
  String get yesButton => '是';

  @override
  String get installationsFoundTitle => '已找到 NieR:Automata 安装位置';

  @override
  String get validInstallations => '有效安装位置（包含 data 文件夹）：';

  @override
  String get withoutDataFolder => '不含 data 文件夹：';

  @override
  String get noLogEntries => '未找到日志条目';

  @override
  String get noLogMatches => '没有日志条目与你的搜索匹配';

  @override
  String get logViewerTitle => '日志查看器';

  @override
  String get logSearchPlaceholder => '搜索级别 / 模块 / 消息...';

  @override
  String get logLiveBadge => '实时';

  @override
  String get logAutoScroll => '自动滚动到最新内容';

  @override
  String get entriesSuffix => '条记录';

  @override
  String get scrollToBottom => '滚动到底部';

  @override
  String get openLogsFolder => '打开日志文件夹';

  @override
  String get diagnosticsButton => '生成诊断信息';

  @override
  String get diagnosticsTitle => '诊断摘要';

  @override
  String get diagnosticsCollecting => '正在收集诊断信息...';

  @override
  String get diagnosticsCopy => '复制摘要';

  @override
  String get diagnosticsCopied => '摘要已复制到剪贴板';

  @override
  String get diagnosticsSaveFull => '保存完整报告';

  @override
  String diagnosticsSavedAt(String path) {
    return '完整报告已保存到 $path';
  }

  @override
  String get modsTutorialBack => '返回';

  @override
  String get modsTutorialNext => '下一步';

  @override
  String get modsTutorialFinish => '知道了';

  @override
  String get modsTutorialSkip => '跳过';

  @override
  String get modsTutorialMenuEcosystem => 'NAMS 与启动器';

  @override
  String get modsTutorialMenuInstall => '如何安装模组';

  @override
  String get modsTutorialMenuProfiles => '配置档案的工作原理';

  @override
  String get modsTutorialMenuSupporting => '支持生态系统';

  @override
  String get modsTutorialSupportingStep1Title => '这是一个生态系统，而不只是模组管理器';

  @override
  String get modsTutorialSupportingStep1Body =>
      '**NAMS 已经持续开发了 3 年以上。** 它最初只是一个模组加载器，后来逐渐发展成完整的平台，如今各种 **模组**、**插件** 和工具都建立在它之上：\n\n- **模组** 是内容扩展，例如服装、自定义任务、新武器和纹理。它们使用 NAMS 的内容系统，但不会运行自己的代码。\n- **插件** 是与游戏同时运行的程序，可以扩展模组加载器本身，例如多人模组、调试覆盖层或新的游戏系统。它们由代码编写，并在启动时由 NAMS 加载。\n- 你当前使用的启动器**不是插件**，而是一个独立应用，用于在启动前准备模组和配置。\n\n今天你看到的这些模组并不是艰难绕过重重障碍后才勉强运行，而是建立在多年基础工作的成果之上。正因为这些基础已经存在，你才不需要重新做一遍。\n\n本节介绍的是**这个平台如何持续运作**，以及它对你意味着什么——无论你只是游玩、刚开始接触模组制作，还是考虑回馈一些成果。\n\n接下来的页面先讲适用于**所有人**的内容，最后再进入更技术性的部分。觉得了解得够多时可以随时跳过，这些内容都不是必读的。';

  @override
  String get modsTutorialSupportingStep2Title => '无需编程也能创作内容';

  @override
  String get modsTutorialSupportingStep2Body =>
      '**你不需要成为程序员，也能为这个生态系统添砖加瓦。**\n\nNAMS 已经提供了一套内容系统，可支持以下声明式内容扩展，而且支持范围还在不断扩大：\n\n- 在原版剧情之上添加的**自定义任务**。\n- 拥有自身行为、无需覆盖原内容的**新武器和物品**。\n- 使用**新槽位**而非替换现有槽位的**饰品**。\n- 位于 `nams/cutscenes/` 或捆绑在模组内部的**自定义过场动画**（例如自定义任务附带自己的演出动画）——原版过场动画**绝不会被替换**，新内容只会并行加载。\n\n贯穿始终的原则是：**新增，而不是替换。** 原版内容会得到保留，模组内容则叠加在上面。这意味着没有编程经验的建模作者也能制作饰品、武器和角色，并将它们作为新增内容发布——不会覆盖任何东西，不会破坏存档，也不会与采取同样方式的其他模组发生冲突。\n\n这套能力今后还会继续**扩展**。每个版本都会加入更多声明式接口，让非程序员能够完成更多事情。';

  @override
  String get modsTutorialSupportingStep3Title => '贡献方式不只有写代码';

  @override
  String get modsTutorialSupportingStep3Body =>
      '制作内容（上一页所述）是一种回馈方式，但绝不是唯一方式。维持生态健康所需的许多事情，甚至完全不涉及制作模组。真正能带来帮助的包括：\n\n- **撰写指南。** 例如“我是如何用 NAMS 制作 X 的”“我是如何调试 Y 的”“我希望一开始就知道的五件事”。目前许多上手难点，本质上都是文档缺口。\n- **认真报告真实的错误。** 提供可复现步骤、日志，以及日志面板生成的诊断报告。这比十张“它不能用”的工单更有价值。\n- **建模。** 制作饰品、武器、角色和道具。NAMS 的内容系统会把这些作为**新增内容**加载——新槽位、不覆盖原内容——所以即使没有编程经验，建模作者也能发布可以直接加入其他人配置、且不会冲突的作品。\n- **翻译。** 启动器支持本地化。如果还没有你的语言，而你也愿意使用它，字符串位于 `lib/l10n/`，欢迎提交 PR。\n- **在少见硬件上测试。** Steam Deck、AMD GPU、超宽屏、多显示器、少有人使用的手柄。只在罕见配置上出现的问题，在有人报告之前往往会一直隐藏。\n- **在 Discord 中回答问题。** 帮助下一位新用户，本身就是贡献。任何生态系统中真正能长期留存的，往往是早期参与者共同塑造的文化。\n- **逆向一个游戏函数，并把 API 贡献回来。** *（感兴趣的话，下一页会进一步说明）*——对技术型贡献者来说，这是杠杆最高的贡献方式。\n\n### 暂时的结语\n\n这个平台的诞生投入了大量业余时间、个人资源和一股不服输的执着，始终秉持着一个想法：**让其他人能够开始做点什么。** 如果上面有一项你觉得自己能做到，那你已经完成了大半。Discord（**YoRHa Continuum**）是提问的地方。\n\n接下来两页会更偏技术。如果你想了解插件如何共存，以及新的游戏 API 如何进入 NAMS，可以继续阅读；如果你已经了解得够多，也可以到此为止。';

  @override
  String get modsTutorialSupportingStep4Title => '插件从设计上就能共存';

  @override
  String get modsTutorialSupportingStep4Body =>
      '*接下来的两页技术性更强，主要面向考虑开发插件的人。如果你不感兴趣，可以跳过。*\n\nNAMS 平台的一项核心特性是：**多个插件可以在同一游戏会话中同时运行**，而不会互相争夺资源。\n\n**Jasper 的 Multiplayer Mod** 是 NAMS 生态中最重要的项目之一，而且至今仍在积极维护，对这项工作值得给予充分尊重。YP 开发工具包和 MP 插件可以**同时加载**、同时工作，并分别在游戏画面上绘制自己的 UI。这不是偶然，而正是 NAMS 插件宿主从一开始就要实现的目标。\n\n因此，只要你发布的插件遵循平台规范，**它就能与已经运行的其他所有内容共存**——包括你的插件、MP 插件、YP 开发工具包，以及未来由你从未见过的人开发的插件。你不需要争夺 Hook，也不必为加载顺序相互冲突，平台会负责协调。\n\n目前每个月仍在进行重构，以减少一个插件意外破坏另一个插件的边缘情况。这是一个持续变化的目标，但方向很明确，工作也一直在推进。';

  @override
  String get modsTutorialSupportingStep5Title => '你建立在免费逆向成果之上';

  @override
  String get modsTutorialSupportingStep5Body =>
      '构建一个真正复杂插件所需的大多数引擎 API，NAMS 中已经具备——而且**它们之所以存在，是因为有人为了自己的问题逆向了游戏，并把成果贡献了回来。**\n\n游戏本身是闭源的。NAMS 中每一个允许你读取或写入某种游戏状态的 API，背后都有人追踪函数、寻找偏移并验证行为。这些都是大量无偿工作，而它们之所以留在 NAMS 中，正是为了让**下一位**插件作者不必重新做一遍。\n\n在 NAMS 上开发时，你会继承这一切。你不是从 `LoadLibrary` 开始，而是从别人费尽力气实现的 API 开始；而未来需要你所逆向 API 的人，也会得到同样的馈赠。\n\n### 为什么这是杠杆最高的贡献\n\n哪怕你只做过一次，也会永久替所有未来插件作者省下同样的工作。这就是其中的不对称性。指南能帮助读到它的少数人，而 NAMS 中的一个 API 会永远帮助所有需要该能力的人。这个生态正是依靠那些为自己逆向了一件事、又把成果留给所有人的贡献者不断成长。';

  @override
  String get modsTutorialEcosystemStep1Title => '这一切为何存在';

  @override
  String get modsTutorialEcosystemStep1Body =>
      '过去，为 NieR 安装模组一直很痛苦。单独运行正常的模组，只要叠加几个就会开始互相冲突——不同的 DLL Hook（`dxgi`、`d3d11`、`dinput8`）争夺同一个位置，错误的包装器赢得加载顺序，游戏随后在启动时悄无声息地崩溃。装了 5–10 个模组的人，排查冲突花的时间往往比真正游玩的时间还多。\n\n很长一段时间里，唯一的答案都是*只能手动安装*：把零散的 `.dat`/`.dtt` 文件放进 `data/`，手动编辑配置，绝不要使用模组管理器。这对一两个模组还能凑合，但它会覆盖真实的游戏文件，而且不会留下任何修改记录。Vortex 之类的工具也帮不上忙——它们不了解 NieR 的特殊机制。\n\n**NAMS 的存在，是为了在模组加载器层面解决这个问题**；而**这个启动器的存在，是为了给 NAMS 提供一个友好的界面。**';

  @override
  String get modsTutorialEcosystemStep2Title => 'NAMS 的作用';

  @override
  String get modsTutorialEcosystemStep2Body =>
      '**NAMS 是模组加载器。** 它不像旧工具那样，通过代理 DLL 劫持 `dxgi.dll` 或 `d3d11.dll`——这种机制本来就是冲突产生的根源。\n\nNAMS 会作为独立宿主进程运行：它在该进程中把 NieR:Automata 作为库加载（将游戏 exe 转换为可加载的 `game.bin`），并在游戏启动前初始化模组加载器。它不会向另一个进程注入任何东西——NAMS *本身就是*游戏运行所在的进程，因此可以完全控制即将加载的内容。\n\n在此基础上，NAMS 主要完成两件事：\n\n**1. 原生重新实现旧工具提供的功能**——LodMod、Limit Break、纹理注入、快速加载——并把它们统一到同一个协调层中。模组接入 NAMS API，而不是争夺哪个 DLL Hook 最先执行。\n\n**2. 提供虚拟文件系统（VFS）：**\n\n- 每个模组都位于 `nams/mods/<modId>/` 下自己的文件夹中，绝不会覆盖真实游戏文件。\n- 运行时，NAMS 会把启用的模组叠加到引擎所看到的 `data/` 上。\n- 原版 `data/*.cpk` 和 `NieRAutomata.exe` **绝不会被修改**，因此仍可像以前一样通过 Steam 启动未安装模组的游戏。\n\n模组会在清单中声明自己修改的内容。NAMS 会按明确顺序验证并加载它们，于是你终于可以**干净地按模组启用/禁用**，并获得**可明确判断的冲突检测**——旧式把文件直接丢进 `data` 的方法无法做到这些。\n\n### 各部分如何协作\n\n这个启动器**并不是**直接建立在 NieR:Automata 之上的。它不会自行逆向游戏、Hook 引擎函数，也不了解 `.dat`/`.dtt` 格式。整体层级如下：\n\n1. **NieR:Automata**——游戏本体，不做修改。\n2. **NAMS**——模组加载器，首先解决大规模模组化是否可行的问题（引擎功能重实现、VFS、插件宿主、内容框架）。\n3. **这个启动器**——建立在 NAMS 之上的辅助工具。它读取 NAMS 的 TOML 配置、按 NAMS 的文件夹结构写入内容，并调用 NAMS API。仅此而已。\n\n实际结果是：NAMS 才是承重层。启动器只是它前面的友好 UI，即使换成其他 UI（或命令行），你的模组也不会受到影响。\n\n### 而且这已经得到验证\n\n这并非理论。**Rustukun 的 NAO Launcher** 就是建立在同一基础上的独立启动器——使用不同 UI、不同设计选择，但底层仍与同一个 NAMS 通信。无论使用哪个启动器，你的模组、`nams/mods/<modId>/` 文件夹以及 `disabled_mods.toml` 都以完全相同的方式工作。\n\n这证明 NAMS 才是平台，而任何启动器（这个、NAO，或未来尚未出现的其他启动器）都只是前端选择。请选择最适合自己工作流程的工具，你不需要迁移模组库。';

  @override
  String get modsTutorialEcosystemStep3Title => '这个启动器增加了什么，以及它有何不同';

  @override
  String get modsTutorialEcosystemStep3Body =>
      'NAMS 负责加载。启动器则负责**围绕加载的一切工作**——安装、整理和故障排查：\n\n- **模组管理器**——拖放安装 NAMS 格式模组，自动规范目录结构（wax/SK_Res 包装层、捆绑资源），检查清单并提示冲突。\n- **纹理**——无需手动编辑 TOML，即可管理独立纹理包和 `load_order` 优先级。\n- **过场动画**——安装高清过场动画模组，自动检测编码（H264 或 MPEG-2），并配置正确的 NAMS 设置。\n- **配置档案**——并排保存多套模组组合，一键切换，无需复制内容，也不会丢失状态。\n- **诊断**——完整报告哪些内容安装在何处、旧安装留下了什么、NAMS 识别到的内容与磁盘实际内容有何差异。\n\n### 我们为什么制作它\n\n**我们并不反对手动安装。** 对于一两个模组，把某套服装的 `.dat`/`.dtt` 文件放进正确的 `data/` 子文件夹完全可行。这个启动器是为更大的规模而设计的。\n\n如今 NAMS 生态已经支持：\n\n- 每个角色可进行多服装切换的 **30 多个服装模组**。\n- 叠加在原版剧情之上的 **20 多个自定义任务**。\n- 拥有独立行为的 **10 多把新武器**。\n- 此外还有纹理、过场动画、插件、平衡性修改……\n\n手动管理这些内容不是理念选择的问题，而是**根本做不到**。你无法追踪每个文件来自哪个模组，无法干净地启用或禁用单个模组，也无法判断某处为什么损坏。规模一大，手动模组管理就会撞上硬墙——而 NAMS 生态早已跨过这道墙。\n\n### 与 NAMH 和 Vortex 的区别\n\n如果你在 NieR 模组圈待过一段时间，应该记得以前的模组管理器最后都变成了什么样：\n\n- **NAMH**（NieR Automata Mod Helper）后来停止维护，曾以无法恢复的方式破坏游戏，遭遇“程序正在使用”锁定问题，而标准恢复方法竟然是*卸载游戏、重新安装、再改成手动操作。* 它通过**直接替换 `data/` 中的文件**工作——一旦 NAMH 安装出问题，就没有干净的回退路径。\n- **Vortex** 从未真正理解 NieR 的文件结构。它的虚拟文件系统依赖的假设与游戏实际加载内容的方式不匹配，因此安装内容会悄无声息地被放错位置。\n\n这个启动器采用了不同设计。关键选择包括：\n\n1. **绝不替换文件。** 模组位于 `nams/mods/<modId>/`，并在运行时通过 NAMS 的 VFS 叠加到引擎视图中。原版 `data/` 从不被修改。不存在所谓“无法恢复的状态”，因为真实游戏内容从未发生改变。\n2. **每个操作都可撤销。** 卸载模组 → 干净移除其文件夹和捆绑资源。禁用模组 → 在 `disabled_mods.toml` 中写入条目，由 NAMS 跳过它。没有隐藏状态，也没有不可逆写入。\n3. **使用配置档案，而不是单一全局状态。** 过去的管理器把所有内容都塞进一个配置。配置档案可以并排保存多套组合，并以原子方式切换——不会损坏，也不会丢失。\n4. **建立在持续维护的模组加载器之上。** NAMH 的消亡，部分源于模组加载器本身的未来不确定。NAMS 是这里一切内容的基础，而启动器会跟随它的更新。\n\n即使这个启动器有一天停止维护，你的模组仍只是磁盘上由 NAMS 读取的文件——你不会因此失去对自己安装内容的访问权。';

  @override
  String get modsTutorialEcosystemStep4Title => '下一步该做什么';

  @override
  String get modsTutorialEcosystemStep4Body =>
      '如果你以前从未在这里安装过模组：\n\n- **如何安装模组**——按选项卡逐步介绍安装流程。\n- **配置档案的工作原理**——说明不同模组组合以及何时应该使用它们。\n\n这两项都在同一个帮助菜单中（也就是你用来打开此页面的 **?** 图标）。\n\n**简单来说：**把压缩包拖到正确的选项卡（角色/数据模组放到“模组管理器”，独立纹理包放到“纹理”，高清过场动画放到“过场动画”），点击开始，出问题时查看日志。启动器会尽量自动做出正确选择；如果你不认同某项选择，每个操作都可以从 UI 中撤销。';

  @override
  String get modsTutorialHelpTooltip => '教程与帮助';

  @override
  String get modsTutorialInstallStep1Title => '把模组拖放到这里';

  @override
  String get modsTutorialInstallStep1Body =>
      '这里是**模组管理器**选项卡，用于安装角色、服装和其他玩法模组。\n\n将 Nexus 下载的 `.zip`、`.7z` 或 `.rar` 拖到这个区域（也可以点击浏览）。启动器会解压内容、检查目录结构，并把它放到正确位置。你不需要自行解压。\n\n**请放心：**原版游戏文件不会被修改。模组位于单独的 `nams/` 文件夹中，因此需要时你仍可随时通过 Steam 启动未安装模组的游戏。';

  @override
  String get modsTutorialInstallStep2Title => '准备安装 WAX 模组？请先阅读这里';

  @override
  String get modsTutorialInstallStep2Body =>
      '**WAX 模组可以在这里运行**——NAMS 已重新实现 WAX，兼容范围覆盖经过测试的特定版本。Nexus 上以该版本或更早版本为目标的大多数模组，都能正常安装和运行。\n\n### 兼容机制\n\nNAMS 会针对一个特定的 WAX 版本进行验证。WAX 在该版本及之前提供的内容：受支持。WAX 之后在**更新版本**中加入的内容：不会自动支持——那是 WAX 一侧的新功能，需要在 NAMS 一侧从头重新实现。\n\n### WAX 添加新功能时会怎样\n\n当 WAX 在未来版本中发布新功能时，会根据 NAMS 路线图对其进行评估：\n\n- **在范围内**——如果该功能符合 NAMS 已经前进的方向，就会得到实现，未来的 NAMS 更新将支持使用它的模组。\n- **不在范围内**——NAMS 有自己的内容扩展重点（自定义任务、自定义世界地图、自定义插件芯片、更广泛的模组 API 等）。重新实现 WAX 的每项功能并非优先事项，因此某些较小众的 WAX 功能可能永远不会拥有 NAMS 对应实现。\n\n**这并不是在贬低 WAX。** 它们是目标不同的独立项目。NAMS 并不试图成为可直接替换 WAX 的产品，而是一个恰好与 WAX **广泛兼容**的独立平台，因为大多数用户都希望现有模组库能够继续工作。\n\n### 这种情况很正常\n\n这种分化是**所有模组游戏生态都会经历的演变方式**，并非 NieR 独有的怪事。具体例子：**Skyrim Legendary Edition（LE）**和**Skyrim Special Edition（SE）**是同一引擎的分支。SE 与 LE 模组广泛兼容，但并非 100%——有些 LE 模组需要转换，还有少数模组从未移植，因为它们依赖 SE 已改变的引擎特性。Skyrim 社区并未把这视为缺陷，而是将其视为生态运作方式的一部分。**OpenMW 与原版 Morrowind**、**Java 版与基岩版 Minecraft**、**KSP1 与 KSP2 模组**等也是同样的情况。\n\n每当一个平台重新实现另一个平台的行为，就会形成一个兼容范围——大部分内容可用，边缘情况不可用。任何存在足够久、最终产生分支的模组生态都会如此。\n\n### 不确定时的最佳做法\n\n1. **创建一个全新的空配置档案**（参见帮助菜单中的*配置档案的工作原理*），并切换到它。\n2. **只把该 WAX 模组**放进这个配置档案，不要加入其他内容。\n3. **点击开始。** 能运行？再把它安装到你的正式配置档案。\n4. **不能运行？** 该模组可能使用了超出 NAMS 已测试 WAX 版本的功能，或使用了 NAMS 决定不重新实现的功能。\n\n### 你可以预期什么\n\n- 如果 NAMS 中的 X、Y、Z 功能都能工作，而你想用的 WAX 模组需要不受支持的 W 功能——并且你可以不用 W——那么 X、Y、Z 仍能与它一起正常运行。\n- 如果 W 功能对该模组不可或缺，而 NAMS 没有它，你就需要在两者之间选择：WAX（获得 W，但失去 NAMS 的其他优势）或 NAMS（保留其他所有内容，但没有 W）。\n\n**也不要忘记取舍的另一面：**坚持使用 WAX，意味着无法使用那些根本不能在 WAX 上运行的**NAMS 独占模组**——包括每个角色的多服装切换、自定义任务，以及更广泛的插件生态（Multiplayer Mod、YP 开发工具包和未来插件）。它们依赖 WAX 不具备的 NAMS API。因此真正的选择不是“缺少 W 的 NAMS”与“拥有 W 的 WAX”，而是“缺少 W 的 NAMS 生态”与“拥有 W、但缺少所有 NAMS 独占内容的 WAX”。\n\n这是一个真实的取舍，最终由你决定。至于某个 WAX 独占功能将来是否会获得 NAMS 支持，我们并不是最合适的询问对象——这是生态路线图问题，最好前往 YoRHa Continuum Discord 咨询。';

  @override
  String get modsTutorialInstallStep3Title => '已安装的模组';

  @override
  String get modsTutorialInstallStep3Body =>
      '你安装的每个模组都会显示在这里。\n\n**右侧开关**——启用或禁用模组。禁用后模组仍会保留安装，但模组加载器会在下次启动时跳过它。\n\n**游戏启动时崩溃？** 先禁用一半模组，启动游戏，然后重复。利用这些开关可以快速二分排查。\n\n**警告徽章**会标记相互冲突的模组（例如两个模组都替换同一套服装），这通常就是游戏无法进入标题画面的原因。';

  @override
  String get modsTutorialInstallStep4Title => '模组详情';

  @override
  String get modsTutorialInstallStep4Body =>
      '点击列表中的任意模组，即可在这里查看其详情：作者、版本、修改内容、与其他模组的冲突，以及它附带的任何**捆绑纹理包或过场动画**。\n\n如果模组无法运行，最常见的原因会显示在这里，例如*需要更新版本的 NAMS*或*与另一个模组冲突*。在你点击开始**之前**，这些信息就已经可见。\n\n点击**卸载**按钮可正确清理模组，包括其捆绑的附加内容。';

  @override
  String get modsTutorialInstallStep5Title => '独立纹理模组 → 纹理选项卡';

  @override
  String get modsTutorialInstallStep5Body =>
      '**纯纹理模组**（高清放大包、颜色重绘）不会安装在这里。它们有自己的选项卡。\n\n点击侧栏中的**纹理**来安装。以同样方式拖入 `.zip` 压缩包，启动器会识别其中内容。\n\n**注意：**如果角色模组*捆绑*了自己的纹理，这些纹理会随模组自动安装。只有**独立**纹理包才需要使用“纹理”选项卡。';

  @override
  String get modsTutorialInstallStep6Title => '过场动画模组 → 过场动画选项卡';

  @override
  String get modsTutorialInstallStep6Body =>
      '**过场动画模组**（高清过场动画、替换视频）也有自己的选项卡。\n\n点击侧栏中的**过场动画**来安装。\n\n**规则与纹理相同：**如果角色模组捆绑了过场动画，它们会自动安装；只有**独立**过场动画包才需要使用“过场动画”选项卡。';

  @override
  String get modsTutorialInstallStep7Title => '点击开始';

  @override
  String get modsTutorialInstallStep7Body =>
      '返回**启动器**选项卡并点击**开始**。模组加载器会在每次启动游戏时重新读取模组，因此无需重启此启动器。\n\n### 如果游戏崩溃\n\n打开右下角的**日志**——模组加载器输出通常会指出损坏的模组。然后回到这里将其禁用。\n\n### 全部禁用后仍然有问题？\n\n如果你（或以前的模组管理器）曾把零散的 `.dat` / `.dtt` 文件直接放进 `<gameDir>/data/`，引擎仍会读取它们，而模组加载器无法发现或禁用它们。这正是本启动器专门避免的混乱：每个模组都隔离在 `nams/mods/<modId>/` 中，而不是覆盖真实游戏文件。\n\n打开**日志 → 诊断**，检查*原版 data/ 叠加层*部分。列在其中的任何内容都是旧安装留下的文件；将这些文件夹移出 `data/`，游戏即可恢复到干净状态。';

  @override
  String get modsTutorialProfilesStep1Title => '配置档案的用途';

  @override
  String get modsTutorialProfilesStep1Body =>
      '配置档案允许你并排保存多套彼此独立的模组组合。\n\n例如：\n\n- 一个包含你实际游玩模组的**主用**配置档案。\n- 一个用于尝试新内容的**测试**配置档案。\n\n如果某个可疑的新模组破坏了游戏，只需切回**主用**配置档案，就能继续游玩。不同组合永远不会混在一起。\n\n**重要：**当前未使用的模组不会被删除，只是暂时停放，切换回来时随时可用。';

  @override
  String get modsTutorialProfilesStep2Title => '创建配置档案';

  @override
  String get modsTutorialProfilesStep2Body =>
      '在配置档案栏中点击**新建**，输入名称并确认。\n\n启动器会创建一个全新的空配置档案并切换过去。之前配置档案中的模组仍安全保存在磁盘上——它们没有消失，只是暂时停放。\n\n现在，你可以在新配置档案中安装任何内容，而不会影响其他模组组合。';

  @override
  String get modsTutorialProfilesStep3Title => '切换、重命名和删除';

  @override
  String get modsTutorialProfilesStep3Body =>
      '**切换**——从下拉菜单选择任意配置档案，模组列表会立即切换。\n\n**重命名**——修改配置档案名称，不会丢失任何内容。\n\n**删除**——永久移除非活动配置档案（无法删除当前活动档案或最后一个剩余档案）。\n\n整个切换会一步完成；如果发生任何问题，会自动回滚，因此不会留下损坏状态。';

  @override
  String get modsTutorialProfilesStep4Title => '哪些内容随配置档案切换，哪些不会';

  @override
  String get modsTutorialProfilesStep4Body =>
      '**按配置档案保存**（切换时会变化）：\n\n- 已安装的模组\n- 哪些模组启用或禁用\n- 随模组捆绑的纹理\n\n**所有配置档案共享**（不会变化）：\n\n- 通过“纹理”选项卡安装的独立纹理包\n- 过场动画模组\n- 插件\n- 所有启动器设置\n\n因此，配置档案只切换真正与模组相关的内容。全局设置会始终随你保留。';

  @override
  String get platformUnsupportedTitle => '无法在此平台上启动';

  @override
  String get platformUnsupportedLinux =>
      'NieR:Automata 是 Windows 游戏，因此在 Linux 上运行需要兼容层。\n\n请安装带 Proton 的 Steam（游戏可在 Proton 下正常运行），或安装 CrossOver/Wine。存在可用运行时后，启动器即可启动游戏。\n\n没有转换层的原生 Linux 无法运行该游戏。';

  @override
  String get platformUnsupportedMacos =>
      'NieR:Automata 是 Windows 游戏。请通过 CrossOver/Wine 运行启动器——过去曾经可用，但近期尚未重新测试。没有转换层的原生 macOS 无法运行该游戏。\n\n如果你通过其他方式成功运行，请直接使用命令行，而不要使用此启动器。';

  @override
  String get playDisabledTooltip => '此平台无法启动游戏';

  @override
  String get diagnosticsClose => '关闭';

  @override
  String get diagnosticsSectionDataOverlay => '原版 data/ 叠加层（手动放入）';

  @override
  String get diagnosticsSectionExternalLegacy => '外部 / 旧版';

  @override
  String get diagnosticsSectionGameRootExtras => '游戏根目录额外文件（非原版）';

  @override
  String diagnosticsFileCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count 个文件',
      one: '1 个文件',
    );
    return '$_temp0';
  }

  @override
  String diagnosticsMoreItems(int count) {
    return '... 另有 $count 项';
  }

  @override
  String get refreshButton => '刷新';

  @override
  String get tabModloaderLabel => '模组加载器';

  @override
  String get tabYorhaLabel => 'YoRHa Protocol';

  @override
  String get configEditorTitle => '配置编辑器';

  @override
  String get changelogTitle => '新增内容';

  @override
  String get tipDragTextures => '将纹理模组直接拖入“纹理”选项卡';

  @override
  String get tipHdCutscenes => '高清过场动画模组会自动检测并配置';

  @override
  String get tipLodModPreviews => 'LOD Mod 设置附带调整前/后的预览图';

  @override
  String get tipFaqButton => '使用常见问题按钮查看 YoRHa Protocol 会替代哪些模组';

  @override
  String get tipReShadeAuto => 'ReShade 会自动检测，无需手动配置';

  @override
  String get tipFreecam => 'YoRHa Protocol 包含带保存槽位的内置自由镜头';

  @override
  String get tipCustomQuests => '自定义任务即将推出，敬请期待';

  @override
  String get sectionNams => 'NAMS';

  @override
  String get sectionTextureInjection => '纹理注入';

  @override
  String get sectionLodMod => 'LOD MOD';

  @override
  String get sectionLevelOfDetail => '细节层次';

  @override
  String get sectionAmbientOcclusion => '环境光遮蔽';

  @override
  String get sectionShadows => '阴影';

  @override
  String get sectionPostProcessing => '后期处理';

  @override
  String get labelValidateModelData => '验证模型数据';

  @override
  String get tooltipValidateModelData =>
      '游戏会在加载时验证模型数据。通常验证失败时不会提示，而是继续使用损坏的数据，这可能导致模型不可见或出现故障。启用后，NAMS 会以对话框显示验证结果，让你准确看到哪个模型失败以及失败原因。';

  @override
  String get labelPreloadMaxDimension => '预加载最大尺寸';

  @override
  String get tooltipPreloadMaxDimension =>
      '启动时预加载到内存中的最大纹理尺寸。2048 = 默认，4096 = 预加载最高 4K 纹理，16384 = 预加载全部纹理。数值越高，加载时间越长，但游戏内卡顿越少。';

  @override
  String get labelPreloadAllTextures => '预加载所有纹理';

  @override
  String get tooltipPreloadAllTextures =>
      '无论尺寸大小，将所有纹理预加载到内存中。可消除所有纹理突然出现造成的卡顿，但需要 32GB 以上内存，并会显著延长启动时间。';

  @override
  String get labelEnableLodMod => '启用 LodMod';

  @override
  String get tooltipEnableLodMod => '所有 LodMod 画面补丁/重写功能的总开关。';

  @override
  String get labelLodMultiplier => 'LOD 倍率';

  @override
  String get tooltipLodMultiplier =>
      '控制 LOD（细节层次）的绘制距离。0 = 禁用 LOD（最佳画质，无物体弹出）；1 = 原版；10 以上可在不完全禁用 LOD 的情况下减少 AO 渗漏。数值越低，画面越好，但可能降低性能。';

  @override
  String get labelDisableManualCulling => '禁用手动剔除';

  @override
  String get tooltipDisableManualCulling =>
      '防止模型/几何体在特定距离或镜头角度下随机消失。可修复过桥后商场内部消失、营地外建筑消失等问题。偶尔会出现的难看 LOD 模型将被过滤。';

  @override
  String get labelAoWidth => 'AO 宽度';

  @override
  String get tooltipAoWidth =>
      'AO 水平分辨率倍率。原版 AO 以屏幕分辨率的 1/4 运行。2.0 = 半分辨率（AO 更清晰，但性能开销较大）；1.5 是较好的平衡。范围：0.1–2.0。只将一个轴设为 2 可作为开销较低的替代方案。';

  @override
  String get labelAoHeight => 'AO 高度';

  @override
  String get tooltipAoHeight =>
      'AO 垂直分辨率倍率。原版 AO 以屏幕分辨率的 1/4 运行。2.0 = 半分辨率（AO 更清晰，但性能开销较大）；1.5 是较好的平衡。范围：0.1–2.0。两个轴都设为 2.0 时，最坏情况下可能损失约 10 FPS。';

  @override
  String get labelShadowResolution => '阴影分辨率';

  @override
  String get tooltipShadowResolution =>
      '阴影贴图纹理尺寸。数值越高，阴影越清晰，但 GPU 负担越重。2048 = 原版，4096 = 不错的提升，8192 = 非常清晰。必须为 2 的幂。清晰度同时取决于分辨率和距离（距离越大，需要覆盖的区域越大，因此质量会下降）。';

  @override
  String get labelDistanceMultiplier => '距离倍率';

  @override
  String get tooltipDistanceMultiplier =>
      '将每个场景的阴影绘制距离乘以此值。2.0 = 阴影可见距离加倍；原版为 1.0。请禁用下方最小值/最大值以使其正常工作，或使用它们限制该倍率设置的范围。';

  @override
  String get labelDistanceMinimum => '最小距离';

  @override
  String get tooltipDistanceMinimum =>
      '阴影绘制距离的最小限制。0 = 关闭（无最小值）。在 8192 分辨率下设为约 70，可在大幅增加阴影距离的同时保持接近原版的质量。';

  @override
  String get labelDistanceMaximum => '最大距离';

  @override
  String get tooltipDistanceMaximum =>
      '阴影绘制距离的最大限制。0 = 关闭（无最大值）。仅当游戏默认距离造成性能问题时才值得设置。';

  @override
  String get labelDistancePss => 'PSS 距离';

  @override
  String get tooltipDistancePss =>
      '启用 PSS 阴影分布，使阴影质量更均匀。0 = 关闭。推荐值：0.5–0.9。在某些区域效果很好，但在其他区域可能显得模糊。该值应远大于其他距离值（大型开放区域约为 1500）。';

  @override
  String get labelFilterStrengthBias => '滤镜强度偏移';

  @override
  String get tooltipFilterStrengthBias =>
      '按场景调整阴影模糊滤镜强度。0 = 关闭；-1 = 阴影更锐利；正值 = 更柔和。不同区域使用不同强度（森林更柔和）。可与最小值/最大值组合以限制范围。';

  @override
  String get labelFilterStrengthMin => '滤镜强度最小值';

  @override
  String get tooltipFilterStrengthMin =>
      '强制所有区域使用不低于此值的阴影滤镜强度。0 = 关闭。游戏默认值会随场景变化（通常约为 4）。可用于防止任何区域中的阴影过于锐利。';

  @override
  String get labelFilterStrengthMax => '滤镜强度最大值';

  @override
  String get tooltipFilterStrengthMax =>
      '强制所有区域使用不高于此值的阴影滤镜强度。0 = 关闭。游戏默认值会随场景变化（通常约为 4）。可用于防止任何区域中的阴影过于模糊。';

  @override
  String get labelHqShadowModels => '高质量阴影模型';

  @override
  String get tooltipHqShadowModels =>
      '使用实时高质量模型生成阴影，而不是静态低质量模型。树木阴影会随风摆动，不再冻结。实验性功能——在城市废墟中表现良好，但少数区域可能出现问题。';

  @override
  String get labelForceAllShadowModels => '强制所有模型投射阴影';

  @override
  String get tooltipForceAllShadowModels =>
      '强制所有模型投射阴影，包括石头和草等小型物体。实验性功能——极少数情况下可能导致不可见模型仍然投射阴影。目前尚未发现问题。';

  @override
  String get labelDisableVignette => '禁用暗角';

  @override
  String get tooltipDisableVignette =>
      '移除屏幕边缘的黑色暗角效果。某些加载画面的暗角可能已经烘焙进纹理，因此仍会显示。';

  @override
  String get configAppliesOnRestart => '重启后生效';

  @override
  String get configAppliesLive => '立即生效（实时）';

  @override
  String get dropZoneBrowseFolder => '或选择文件夹';

  @override
  String get labelGiEnabled => '启用全局光照';

  @override
  String get tooltipGiEnabled => 'FAR 风格的全局光照。以牺牲部分光照准确性换取大幅 FPS 提升。';

  @override
  String get labelGiWorkgroupSize => 'GI 工作组大小';

  @override
  String get tooltipGiWorkgroupSize =>
      '每次 GI 调度处理的光照体积数量。128 = 原版质量；64/32/16 会逐级加快，但结果更粗糙。数值越低，越是以光照精度换取 FPS。';

  @override
  String get labelGiMinLightExtent => 'GI 最小光源范围';

  @override
  String get tooltipGiMinLightExtent =>
      '从 GI 中剔除远处的小型光源。0.0 = 不剔除（所有光源均参与）；0.5 = 激进剔除。范围 0.0–1.0。';

  @override
  String get cardExperimental => '实验性';

  @override
  String get lodModResetButton => '恢复默认值';

  @override
  String get lodModResetConfirmTitle => '重置 LodMod 设置？';

  @override
  String get lodModResetConfirmBody =>
      '这会把此选项卡中的所有 LodMod 字段恢复为默认值，并覆盖当前值。要继续吗？';

  @override
  String get lodModResetConfirmAction => '重置';

  @override
  String get lodModResetToast => 'LodMod 设置已恢复默认值';

  @override
  String get experimentalWarningTitle => '实验性功能——可能造成故障';

  @override
  String get experimentalWarningBody =>
      '这些设置会绕过引擎依赖的游戏限制。它们不受支持，而且已知会导致问题。仅在你清楚自己所做操作时启用。NAMS 和启动器不会针对由这些设置引发的问题进行调试。';

  @override
  String get labelFpsUncapInMenus => '解除菜单 / 加载时的 FPS 限制';

  @override
  String get tooltipFpsUncapInMenus =>
      '移除菜单和加载画面中的 60 FPS 锁定。加载体感更快，菜单动画也更流畅。安全：不会影响实际游戏。\n\n如果游戏启动时此功能已经启用，则可实时切换。如果启动时处于禁用状态，之后再启用需要重启游戏。';

  @override
  String get labelFpsUncapInGameplay => '解除游戏过程中的 FPS 限制';

  @override
  String get tooltipFpsUncapInGameplay =>
      '移除游戏过程中的 60 FPS 锁定。警告：NieR:Automata 的物理、动画和过场动画时序都与 60 FPS 锁定绑定。解除限制会导致物理异常（跳跃高度、闪避无敌帧）、动画速度变化、过场动画音画不同步，以及脚本流程软锁。只有在你完全清楚自己接受的取舍时才使用。\n\n如果游戏启动时此功能已经启用，则可实时切换。如果启动时处于禁用状态，之后再启用需要重启游戏。';

  @override
  String get labelFpsLimit => 'FPS 上限';

  @override
  String get tooltipFpsLimit =>
      '解除限制时应用的 FPS 上限。0 = 无限制；其他有效范围为 60–1000（超出范围的值会被 NAMS 限制）。低于 60 的值会被限制，因为游戏内部的自旋等待循环会忽略比原版 60 FPS 目标更长的帧时间。提示：将上限设为显示器刷新率的一半，会比原版 60 FPS 获得更平滑的运动效果（例如 144Hz 设为 72、165Hz 设为 82、240Hz 设为 120）。';

  @override
  String get tutorialValidateModel => '模组模型损坏时会直接提示，而不是静默失败。';

  @override
  String get labelValidateScripts => '验证脚本';

  @override
  String get tooltipValidateScripts => '以对话框显示脚本错误，而不是让游戏静默崩溃。';

  @override
  String get previewValidationDialog => '验证对话框';

  @override
  String get previewScriptErrorDialog => '脚本错误对话框';

  @override
  String get labelLoadingStallHints => '加载卡住提示';

  @override
  String get tooltipLoadingStallHints => '加载画面持续过久时显示逐级提示，帮助识别缺失或已删除的模组文件。';

  @override
  String get labelFixWindTimerBug => '修复风动画计时器错误';

  @override
  String get tooltipFixWindTimerBug => '修复原版游戏中达到最大游玩时间后风动画停止的问题，改为使用游戏速度系数。';

  @override
  String get labelDisablePluginLoading => '禁用插件加载';

  @override
  String get tooltipDisablePluginLoading =>
      '跳过所有插件 DLL 的加载（例如 YoRHa Protocol 工作区）。所有 NAMS 引擎功能仍可正常工作。';

  @override
  String get labelDisableContentFeatures => '禁用内容功能';

  @override
  String get tooltipDisableContentFeatures =>
      '所有内容层功能的总开关。启用后，NAMS 将作为纯引擎层运行（鼠标修复、验证、堆内存调整、崩溃修复），不再支持任何物品 / 武器 / 服装 / 任务 / 饰品模组。适合性能基准测试，或用于区分引擎问题与内容问题。';

  @override
  String get labelContentItems => '物品 / 武器 / 商店';

  @override
  String get tooltipContentItems =>
      '自定义物品、武器、服装和商店条目。禁用后可在不加载任何物品相关模组的情况下游玩。需要重启。';

  @override
  String get labelContentAccessories => '饰品';

  @override
  String get tooltipContentAccessories =>
      '自定义饰品（面具、月之泪、正宗面具等）及饰品装备/卸下流程。禁用后可在不加载饰品模组的情况下游玩。需要重启。';

  @override
  String get labelContentAssembleMeshes => '玩家网格';

  @override
  String get tooltipContentAssembleMeshes =>
      '自定义玩家网格（网格替换、发型 / 服装 / 面具覆盖）。禁用后将保持原版玩家模型不变。需要重启。';

  @override
  String get labelContentQuestIntegration => '任务 / 邮件 / 语音';

  @override
  String get tooltipContentQuestIntegration =>
      '自定义任务、自定义邮件、自定义语音，以及用于激活它们的任务 UI 集成。禁用后可在不加载任务模组的情况下游玩。需要重启。';

  @override
  String get labelContentEffectsApplier => '效果规则';

  @override
  String get tooltipContentEffectsApplier =>
      '每帧将武器/服装效果规则应用到玩家属性（伤害倍率、闪避调整、免疫等）。';

  @override
  String get labelContentEquipTracker => '装备追踪器';

  @override
  String get tooltipContentEquipTracker => '检测武器装备/卸下变化，用于驱动效果规则和装备时 SDK 回调。';

  @override
  String get labelContentMcd => '自定义文本';

  @override
  String get tooltipContentMcd => '自定义游戏内文本（由模组提供的物品名称、说明和对话字符串）。';

  @override
  String get labelContentBuddyRubySelector => '伙伴服装选择器（实验性）';

  @override
  String get tooltipContentBuddyRubySelector =>
      '修补全局伙伴对话脚本，添加“更换服装”条目，并列出模组提供的伙伴服装。如果修补后的对话脚本导致不稳定或干扰其他脚本模组，请禁用此功能。';

  @override
  String get cardContentFeatures => '内容功能';

  @override
  String get contentFeaturesDescription =>
      '内容层各项功能的独立开关，默认全部开启。适合将问题缩小到特定子系统。需要重启游戏。';

  @override
  String get labelDisableReShadeLoading => '禁用 ReShade 加载';

  @override
  String get tooltipDisableReShadeLoading =>
      '跳过对 reshade/ 文件夹中 ReShade DLL 的自动检测，并停止加载它。';

  @override
  String get labelDisableTextureInjection => '禁用纹理注入';

  @override
  String get tooltipDisableTextureInjection =>
      '跳过从模组文件夹注入纹理。适合排查问题，或在已安装纹理模组但不想使用时启用。';

  @override
  String get labelOutfitSwapVisualEffects => '服装切换视觉效果';

  @override
  String get tooltipOutfitSwapVisualEffects =>
      '在热切换服装时播放视觉效果：Pod 出现时的致盲动画、帷幕以及骇入画面故障滤镜。关闭后将立即完成无特效切换，但模型仍会重新加载。立即生效，无需重启。';

  @override
  String get labelExperimentalDefaultOutfits => '默认服装（实验性）';

  @override
  String get tooltipExperimentalDefaultOutfits =>
      '可将已安装的服装 Mod 设为游戏启动时即生效，如同将其文件放入游戏的 data 文件夹。开启后，Mod 详情面板会为每个玩家模型显示一个星标按钮，用于将其设为启动默认项。功能稳定前默认关闭。需要重启游戏。';

  @override
  String get labelDisableSplashScreen => '禁用启动画面';

  @override
  String get tooltipDisableSplashScreen =>
      '跳过游戏加载时显示的启动画面窗口。原版游戏会在窗口准备完成前就将其显示出来，导致缩放和闪烁问题；NAMS 完善了启动画面，使窗口仅在准备好后显示。启用此选项会重新带回原版启动时的这些问题。';

  @override
  String get tooltipValidateModelDataSettings => '以对话框显示模型验证错误，而不是静默失败。';

  @override
  String get heapDefault => '默认';

  @override
  String get heapPlus16MB => '+16 MB';

  @override
  String get heapPlus32MB => '+32 MB';

  @override
  String get heapPlus64MB => '+64 MB';

  @override
  String get heapPlus128MB => '+128 MB';

  @override
  String get heapPlus256MB => '+256 MB';

  @override
  String heapCustomMB(String mb) {
    return '+$mb MB';
  }

  @override
  String get heapScriptEngine => '脚本引擎';

  @override
  String get heapScriptEngineDesc => '用于复杂脚本模组（mRuby / HAP）。';

  @override
  String get heapPlayerModels => '玩家模型';

  @override
  String get heapPlayerModelsDesc => '用于大型玩家模型替换模组。';

  @override
  String get heapPlayerTextures => '玩家纹理';

  @override
  String get heapPlayerTexturesDesc => '用于高分辨率玩家纹理模组。';

  @override
  String get heapEnemyBgModels => '敌人/背景模型';

  @override
  String get heapEnemyBgModelsDesc => '用于敌人或环境模型模组。';

  @override
  String get heapEnemyBgTextures => '敌人/背景纹理';

  @override
  String get heapEnemyBgTexturesDesc => '用于高分辨率敌人/环境纹理。';

  @override
  String get tutorialLodModEnable => '开启此功能可改善画面，这是影响最大的设置。';

  @override
  String get tutorialLodModShadowRes => '数值越高，阴影越清晰。推荐使用 8192。';

  @override
  String get tutorialLodModComparison => '点击任意对比图即可全屏查看差异。';

  @override
  String get comparisonVanilla => '原版';

  @override
  String get comparisonDefaultEnabled => '默认（已启用）';

  @override
  String get comparisonAo05x => 'AO 0.5x';

  @override
  String get comparisonAo20x => 'AO 2.0x';

  @override
  String get comparisonVignetteOn => '暗角开启';

  @override
  String get comparisonVignetteOff => '暗角关闭';

  @override
  String get comparison2048 => '2048';

  @override
  String get comparison8192 => '8192';

  @override
  String get comparisonDefault => '默认';

  @override
  String get comparison20x => '2.0x';

  @override
  String get comparisonPssMinus5 => 'PSS -5.0';

  @override
  String get comparisonBiasMinus5 => '偏移 -5.0';

  @override
  String get comparisonOff => '关闭';

  @override
  String get comparison30 => '3.0';

  @override
  String get comparisonHqForceAll => '高质量 + 强制全部';

  @override
  String get tutorialKeybind => '点击以更改按键绑定，然后按任意键进行分配。';

  @override
  String get tutorialDamageMultiplier => '调整游戏玩法——叠加伤害、启用无限生命等。';

  @override
  String get labelOpenWorkspace => '打开工作区';

  @override
  String get tooltipOpenWorkspace => '在游戏中打开 YoRHa Protocol 工作区。';

  @override
  String get labelFreezeGame => '冻结游戏';

  @override
  String get tooltipFreezeGame => '冻结/解除冻结游戏。适合截图和逐帧推进。';

  @override
  String get labelMaxSpeed => '最高速度';

  @override
  String get tooltipMaxSpeed => '切换到最高游戏速度，用于快速移动或测试。';

  @override
  String get labelFreeCam => '自由镜头';

  @override
  String get tooltipFreeCam => '切换自由镜头。完整支持键盘/鼠标和手柄。';

  @override
  String get labelPhaseJump => '阶段跳转';

  @override
  String get tooltipPhaseJump => '跳转到预先选择的游戏阶段/场景。请在游戏中设置目标。';

  @override
  String get labelToggleInput => '切换输入';

  @override
  String get tooltipToggleInput => '工作区打开时，切换游戏输入的开/关。';

  @override
  String get labelAdvanceFrame => '推进一帧';

  @override
  String get tooltipAdvanceFrame => '游戏冻结时向前推进一帧。长按可更快推进。';

  @override
  String get labelDevMode => '开发者模式';

  @override
  String get tooltipDevMode => '切换开发者模式。启用穿透/压力测试按钮和调试工具。';

  @override
  String get labelWarpSave1 => '保存传送点 1';

  @override
  String get tooltipWarpSave1 => '将当前位置和旋转角度保存到传送槽位 1。';

  @override
  String get labelWarpGoto1 => '传送到 1';

  @override
  String get tooltipWarpGoto1 => '传送到槽位 1 中保存的位置。';

  @override
  String get labelWarpSave2 => '保存传送点 2';

  @override
  String get tooltipWarpSave2 => '将当前位置和旋转角度保存到传送槽位 2。';

  @override
  String get labelWarpGoto2 => '传送到 2';

  @override
  String get tooltipWarpGoto2 => '传送到槽位 2 中保存的位置。';

  @override
  String get labelGlobalKeybinds => '全局按键绑定';

  @override
  String get tooltipGlobalKeybinds => '工作区关闭时，按键绑定仍然有效。';

  @override
  String get labelLoadingSpeedup => '加载加速';

  @override
  String get tooltipLoadingSpeedup => '缩短加载画面时间。';

  @override
  String get labelShaders => '着色器';

  @override
  String get tooltipShaders => '工作区着色器。关闭可提高性能。';

  @override
  String get labelSound => '声音';

  @override
  String get tooltipSound => '工作区 UI 交互音效。';

  @override
  String get labelDamageMultiplier => '伤害倍率';

  @override
  String get tooltipDamageMultiplier => '2.0 = 双倍伤害。';

  @override
  String get labelSyncEnemyLevels => '同步敌人等级';

  @override
  String get tooltipSyncEnemyLevels => '使敌人等级与你的等级一致。';

  @override
  String get labelInfiniteHealth => '无限生命';

  @override
  String get tooltipInfiniteHealth => '不会受到伤害。';

  @override
  String get labelInfiniteJump => '无限跳跃';

  @override
  String get tooltipInfiniteJump => '可在空中无限跳跃。';

  @override
  String get labelNoPodCooldown => 'Pod 无冷却';

  @override
  String get tooltipNoPodCooldown => 'Pod 程序没有冷却时间。';

  @override
  String get labelInfiniteAirDash => '无限空中冲刺';

  @override
  String get tooltipInfiniteAirDash => '可在空中无限冲刺。';

  @override
  String get labelAutoStart => '自动启动';

  @override
  String get tooltipAutoStart => '游戏启动时自动启动随机化器。';

  @override
  String get labelGroundEnemies => '地面敌人';

  @override
  String get tooltipGroundEnemies => '随机化地面生成的敌人。';

  @override
  String get labelFlyingEnemies => '飞行敌人';

  @override
  String get tooltipFlyingEnemies => '随机化飞行敌人的生成。';

  @override
  String get labelAllowBigEnemies => '允许大型敌人';

  @override
  String get tooltipAllowBigEnemies => '允许生成大型敌人。';

  @override
  String get labelIncludeDlcEnemies => '包含 DLC 敌人';

  @override
  String get tooltipIncludeDlcEnemies => '将 DLC 敌人纳入随机范围。';

  @override
  String get tutorialCameraAccel => '移除鼠标加速度，实现 1:1 输入。';

  @override
  String get tutorialWipBanner => '这些功能将在未来的 NAMS 更新中推出。';

  @override
  String get labelFixCameraAcceleration => '修复镜头加速度';

  @override
  String get tooltipFixCameraAcceleration => '实现线性 1:1 鼠标移动，移除镜头旋转中的死区和加速度曲线。';

  @override
  String get labelSensitivity => '灵敏度';

  @override
  String get tooltipSensitivity => '镜头灵敏度倍率。数值越高，旋转越快。2.0 是较好的默认值。';

  @override
  String get labelAimSensitivity => '瞄准灵敏度';

  @override
  String get tooltipAimSensitivity =>
      '俯视/横版视角下的瞄准灵敏度。约 3500 DPI 使用 0.001，约 800 DPI 使用 0.003。';

  @override
  String get labelAimOutputMultiplier => '瞄准输出倍率';

  @override
  String get tooltipAimOutputMultiplier =>
      '标准化后准星移动速度的原始倍率。数值越高，准星越快。大多数用户无需更改。';

  @override
  String get labelDisablePodPet => '禁用抚摸 Pod';

  @override
  String get tooltipDisablePodPet => '禁用由鼠标移动触发的抚摸 Pod 动画。推荐启用。';

  @override
  String get labelDebugMenuKey => '调试菜单按键';

  @override
  String get tooltipDebugMenuKey => '打开通关后可使用的调试菜单。通常需要手柄，此按键绑定可让键盘也能打开它。';

  @override
  String get labelThirdPersonMode => '第三人称镜头修复';

  @override
  String get tooltipThirdPersonMode =>
      '为第三人称镜头使用原始鼠标输入。提供平滑、直接的镜头控制，并忽略游戏内鼠标设置。';

  @override
  String get labelThirdPersonCharFollow => '镜头跟随角色';

  @override
  String get tooltipThirdPersonCharFollow => '移动时保留游戏的自动镜头跟随，就像使用手柄一样。';

  @override
  String get labelThirdPersonSensX => '水平灵敏度';

  @override
  String get tooltipThirdPersonSensX => '镜头左右移动速度。负值会反转方向。';

  @override
  String get labelThirdPersonSensY => '垂直灵敏度';

  @override
  String get tooltipThirdPersonSensY => '镜头上下移动速度。负值会反转方向。';

  @override
  String get labelAimMode => '修复 Pod 瞄准';

  @override
  String get tooltipAimMode => '移除俯视和横版视角中 Pod/机甲瞄准的限制范围与死区。';

  @override
  String get labelAimCrosshair => '准星模式';

  @override
  String get tooltipAimCrosshair =>
      '通过指向进行瞄准：Pod 会瞄准跟随鼠标移动的准星，操作方式类似双摇杆射击游戏。准星由游戏自身的 UI 元素组成，因此看起来和操作起来都像 NieR:Automata 原生功能。推荐启用。';

  @override
  String get labelAimCrosshairAlways => '始终显示准星';

  @override
  String get tooltipAimCrosshairAlways => '即使未开火也保持准星可见。关闭时仅在射击期间显示。';

  @override
  String get naiomNeedsCrosshair => '请开启准星模式以使用此功能';

  @override
  String get labelAimSensX => '瞄准水平灵敏度';

  @override
  String get tooltipAimSensX => '左右瞄准速度倍率。负值会反转方向。';

  @override
  String get labelAimSensY => '瞄准垂直灵敏度';

  @override
  String get tooltipAimSensY => '上下瞄准速度倍率。负值会反转方向。';

  @override
  String get labelDisableTapEvade => '禁用双击闪避';

  @override
  String get tooltipDisableTapEvade => '阻止双击移动键触发闪避。仅在设置了独立闪避键时有用。';

  @override
  String get labelCustomCursorMenu => '菜单光标';

  @override
  String get tooltipCustomCursorMenu =>
      '菜单使用的自定义鼠标光标（.cur 或 .ani 文件）。留空 = 使用内置默认光标。';

  @override
  String get labelCustomCursorHacking => '骇入光标';

  @override
  String get tooltipCustomCursorHacking => '骇入小游戏使用的自定义光标。留空 = 与菜单光标相同。';

  @override
  String get labelDisableDefaultCursor => '保留系统光标';

  @override
  String get tooltipDisableDefaultCursor =>
      '不使用内置光标；除非在上方选择了自定义文件，否则保留普通 Windows 光标。';

  @override
  String get labelBindMoveForward => '向前移动';

  @override
  String get tooltipBindMoveForward => '与游戏内按键绑定相同。';

  @override
  String get labelBindMoveBackward => '向后移动';

  @override
  String get tooltipBindMoveBackward => '与游戏内按键绑定相同。';

  @override
  String get labelBindMoveLeft => '向左移动';

  @override
  String get tooltipBindMoveLeft => '与游戏内按键绑定相同。';

  @override
  String get labelBindMoveRight => '向右移动';

  @override
  String get tooltipBindMoveRight => '与游戏内按键绑定相同。';

  @override
  String get labelBindJump => '跳跃';

  @override
  String get tooltipBindJump => '与游戏内按键绑定相同。';

  @override
  String get labelBindWalk => '步行';

  @override
  String get tooltipBindWalk => '按住可缓慢行走。';

  @override
  String get labelBindAutoRun => '自动奔跑';

  @override
  String get tooltipBindAutoRun => '无需持续按住移动键即可保持奔跑。';

  @override
  String get labelBindLightAttack => '轻攻击';

  @override
  String get tooltipBindLightAttack => '与游戏内按键绑定相同。';

  @override
  String get labelBindHeavyAttack => '重攻击';

  @override
  String get tooltipBindHeavyAttack => '与游戏内按键绑定相同。';

  @override
  String get labelBindFire => '开火 / Pod 冲刺';

  @override
  String get tooltipBindFire => '让 Pod 开火。与跳跃键同时使用可执行 Pod 冲刺，即使开启自动射击也可使用。';

  @override
  String get labelBindProgram => '使用程序';

  @override
  String get tooltipBindProgram => '使用 Pod / 飞行单位程序。';

  @override
  String get labelBindLockOn => '锁定目标';

  @override
  String get tooltipBindLockOn => '锁定当前目标。';

  @override
  String get labelBindUse => '使用 / 互动';

  @override
  String get tooltipBindUse => '与游戏内按键绑定相同。';

  @override
  String get labelBindSelfDestruct => '自爆';

  @override
  String get tooltipBindSelfDestruct => '与游戏内按键绑定相同。';

  @override
  String get labelBindLight => '切换照明';

  @override
  String get tooltipBindLight => '与游戏内按键绑定相同。';

  @override
  String get labelBindResetCamera => '重置镜头';

  @override
  String get tooltipBindResetCamera => '与游戏内按键绑定相同。';

  @override
  String get labelBindSwitchWeapon => '切换武器组';

  @override
  String get tooltipBindSwitchWeapon => '循环切换已装备的武器组。';

  @override
  String get labelBindNextProgram => '下一个程序';

  @override
  String get tooltipBindNextProgram => '选择下一个 Pod 程序。';

  @override
  String get labelBindPreviousProgram => '上一个程序';

  @override
  String get tooltipBindPreviousProgram => '选择上一个 Pod 程序。';

  @override
  String get labelBindMenuUp => '菜单向上';

  @override
  String get tooltipBindMenuUp => '在菜单中向上导航。';

  @override
  String get labelBindMenuDown => '菜单向下';

  @override
  String get tooltipBindMenuDown => '在菜单中向下导航。';

  @override
  String get labelBindMenuLeft => '菜单向左';

  @override
  String get tooltipBindMenuLeft => '在菜单中向左导航。';

  @override
  String get labelBindMenuRight => '菜单向右';

  @override
  String get tooltipBindMenuRight => '在菜单中向右导航。';

  @override
  String get labelBindMenuOpen => '打开菜单';

  @override
  String get tooltipBindMenuOpen => '打开系统菜单。';

  @override
  String get labelBindMenuBack => '菜单返回 / 关闭';

  @override
  String get tooltipBindMenuBack => '在菜单中返回，或在顶层关闭菜单。';

  @override
  String get labelBindMenuEnter => '菜单确认 / 跳过对话';

  @override
  String get tooltipBindMenuEnter => '进入所选子菜单或跳过对话。';

  @override
  String get labelBindShortcutMenu => '快捷菜单';

  @override
  String get tooltipBindShortcutMenu => '打开快捷菜单。';

  @override
  String get labelBindEvade => '闪避（独立按键）';

  @override
  String get tooltipBindEvade => '按一次即可沿当前移动方向闪避，无需双击移动键。';

  @override
  String get labelBindAutoFire => '自动射击开关';

  @override
  String get tooltipBindAutoFire => '切换 Pod 持续射击的开/关，无需一直按住开火键。';

  @override
  String get labelBindNextItem => '下一个物品';

  @override
  String get tooltipBindNextItem =>
      '立即切换到下一个快捷物品。此操作会在后台静默完成，不会在游戏中显示物品菜单，这是预期行为。';

  @override
  String get labelBindPreviousItem => '上一个物品';

  @override
  String get tooltipBindPreviousItem =>
      '立即切换到上一个快捷物品。此操作会在后台静默完成，不会在游戏中显示物品菜单，这是预期行为。';

  @override
  String get labelBindUseItem => '使用物品';

  @override
  String get tooltipBindUseItem =>
      '立即使用所选快捷物品。此操作会在后台静默完成，不会在游戏中显示物品菜单，这是预期行为。';

  @override
  String get labelBindThirdPersonToggle => '镜头修复开关';

  @override
  String get tooltipBindThirdPersonToggle => '游玩时开启/关闭第三人称镜头修复。';

  @override
  String get labelBindAimToggle => '瞄准修复开关';

  @override
  String get tooltipBindAimToggle => '游玩时开启/关闭 Pod 瞄准修复。';

  @override
  String get keybindUnbound => '未绑定';

  @override
  String keybindConflict(String other) {
    return '同时用于：$other';
  }

  @override
  String get keybindMouseNotSupported => '此操作不支持鼠标按键，必须使用键盘按键。';

  @override
  String get naiomResetConfirmTitle => '重置 NAIOM 设置？';

  @override
  String get naiomResetConfirmBody =>
      '这会将此选项卡中的所有镜头、瞄准、光标和按键绑定设置恢复为默认值。在你点击保存之前不会写入任何内容，因此之后仍可放弃更改。要继续吗？';

  @override
  String get naiomControllerNote =>
      '使用手柄游玩？这些设置主要为鼠标和键盘设计，但其中一些——尤其是镜头和瞄准修复——也会影响手柄输入。如果切回手柄游玩，请先禁用这些设置，以恢复原版手柄操作感。';

  @override
  String get cardCheatEngine => 'CHEAT ENGINE';

  @override
  String get cheatTableConvertDesc =>
      '有无法与 NAMS 配合使用的 Cheat Engine 表（.CT）？可在此修复。修复后的副本会保存在原文件旁边。';

  @override
  String get cheatTableConvertButton => '修复作弊表...';

  @override
  String cheatTableConvertSuccess(String file) {
    return '修复完成！已保存为 $file';
  }

  @override
  String get cheatTableConvertNone => '此表已经可以与 NAMS 配合使用，无需修复。';

  @override
  String get cheatTableConvertError => '无法修复此表。请确保文件是有效的 .CT 文件。';

  @override
  String get naiomBetaBadge => '测试版';

  @override
  String get naiomRestartBadge => '需重启';

  @override
  String get naiomRestartTooltip => '重启游戏后生效。';

  @override
  String get naiomNeedsCameraFix => '请开启“修复镜头加速度”以使用此功能';

  @override
  String get naiomNeedsThirdPerson => '请开启“第三人称镜头修复”以使用此功能';

  @override
  String get naiomNeedsAimMode => '请开启“修复 Pod 瞄准”以使用此功能';

  @override
  String get naiomCrosshairOverrides => '准星模式开启时不使用此项，准星有独立速度设置';

  @override
  String get naiomThirdPersonRestartNote => '开启此功能需要重启游戏；关闭可在游玩过程中即时生效。';

  @override
  String get naiomTapEvadeWarning =>
      '未绑定闪避键！禁用双击闪避且没有独立闪避键时，你将完全无法闪避。请在“非标准操作”下绑定闪避键。';

  @override
  String get naiomCrosshairNote =>
      '准星仅会在使用鼠标进行普通俯视/横版玩法时显示。如果某些场景中看不到，通常属于正常情况，并非错误。';

  @override
  String get naiomBindingsIntro => '这些是游戏原有控制之外的额外按键，原始按键仍可使用。保存后立即生效，无需重启。';

  @override
  String get naiomCrosshairPreviewLabel => '游戏中的准星模式';

  @override
  String get naiomCursorPick => '选择文件...';

  @override
  String get naiomCursorClear => '移除';

  @override
  String get naiomCursorInvalid => '不是有效的光标文件，需要真实的 .cur 或 .ani 文件';

  @override
  String get naiomLiveBadge => '实时';

  @override
  String get naiomLiveTooltip => '保存后生效，无需重启游戏。';

  @override
  String get labelPreloadMaxDimensionShort => '预加载最大尺寸';

  @override
  String get tooltipPreloadMaxDimensionShort =>
      '0 = 禁用（纯流式加载），2048 = 默认，4096 = 4K 纹理，16384 = 全部纹理。';

  @override
  String get labelPreloadAllTexturesShort => '预加载所有纹理';

  @override
  String get tooltipPreloadAllTexturesShort => '预加载所有纹理。无卡顿，但需要 32GB 以上内存。';

  @override
  String get labelVramBudget => '显存预算（MB）';

  @override
  String get tooltipVramBudget =>
      '纹理模组系统可以使用的 GPU 显存量。选择数值可设置硬上限，例如 8192 表示“模组纹理绝不使用超过 8 GB”，16384 表示“绝不使用超过 16 GB”。自动（推荐）会跳过硬上限，并根据 GPU 实际可用显存使用。';

  @override
  String get labelStreamingEnabled => '后台加载';

  @override
  String get tooltipStreamingEnabled =>
      '游玩过程中在后台加载纹理，可防止进入新区域时冻结和卡顿。仅在遇到问题时关闭；关闭后，游戏加载新纹理时可能短暂冻结。';

  @override
  String get labelLoadOnlyRelevant => '仅加载相关纹理';

  @override
  String get tooltipLoadOnlyRelevant =>
      '对于大型纹理包（400 个以上文件），仅加载与精选优先列表匹配的纹理，以节省显存和加载时间。小型纹理包（服装、武器）始终会完整加载。使用超大型纹理包并希望节省内存时可开启。';

  @override
  String get tutorialDropTextures => '将纹理模组拖到这里进行安装。Zip 文件会自动解压。';

  @override
  String get tutorialLoadOrder => '如果模组有重叠，可拖动调整顺序。顶部 = 最高优先级。';

  @override
  String get textureOverlapLabel => '重叠';

  @override
  String tooltipTextureOverlap(String mods) {
    return '与以下模组修改了相同纹理：$mods。列表中位置更高（更接近“最高”）的模组会显示在游戏中。';
  }

  @override
  String get tooltipFolderNotFound => '在 nams/inject/textures/ 中未找到文件夹';

  @override
  String get priorityHighest => '最高';

  @override
  String get priorityMedium => '中等';

  @override
  String get priorityLowest => '最低';

  @override
  String nameOutfitTitle(String character) {
    return '为此服装命名（$character）';
  }

  @override
  String get outfitNameHint => '服装名称';

  @override
  String installedTextureCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '文件',
      one: '文件',
    );
    return '已安装 $count 个纹理$_temp0';
  }

  @override
  String installationFailed(String error) {
    return '安装失败：$error';
  }

  @override
  String removedItem(String name) {
    return '已移除“$name”';
  }

  @override
  String get tutorialStarIcon => '点击星标可设置游戏启动时加载的默认服装。';

  @override
  String installedOutfitsCount(int count) {
    return '已安装服装（$count）';
  }

  @override
  String get tooltipDlcDetected =>
      '检测到 DLC（data100.cpk）。模型文件使用 DLC 命名方式（pl000d）。';

  @override
  String get tooltipNoDlcDetected => '未检测到 DLC。模型文件将重命名为非 DLC 命名方式（pl0000）。';

  @override
  String installConfirmMod(String name, String character) {
    return '安装“$name”（$character）？';
  }

  @override
  String installedOutfit(String name, String character) {
    return '已安装“$name”（$character）';
  }

  @override
  String get crossInstallTextures =>
      '此模组还包含纹理文件。要将它们安装到 nams/inject/textures/ 吗？';

  @override
  String alsoInstalledTextures(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '文件',
      one: '文件',
    );
    return '还安装了 $count 个纹理$_temp0';
  }

  @override
  String get clearedAllStartupOutfits => '已清除所有启动服装';

  @override
  String get clearedStartupOutfit => '已清除启动服装';

  @override
  String setStartupOutfit(String name) {
    return '已将“$name”设为启动服装';
  }

  @override
  String get tutorialDropCutscenes => '将过场动画模组压缩包拖到这里。支持 .zip、.7z 和 .rar。';

  @override
  String get tutorialInstalledCutscenes =>
      '这里显示已安装的过场动画模组。自定义过场动画会从此处加载，而不是 data/movie/。';

  @override
  String get selectCutsceneModFolder => '选择过场动画模组文件夹';

  @override
  String cutsceneNamingHint(int max) {
    return '最多 $max 个字符。该名称将作为 nams/cutscenes/ 中的文件夹名称。';
  }

  @override
  String cutsceneNameTooLong(int max) {
    return '名称不得超过 $max 个字符。';
  }

  @override
  String get preparingInstall => '正在准备...';

  @override
  String installedCutsceneMod(String name, int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '文件',
      one: '文件',
    );
    return '已安装“$name”（$count 个 USM $_temp0）';
  }

  @override
  String deleteCutsceneConfirm(String name) {
    return '删除“$name”及其所有文件？';
  }

  @override
  String installedCutsceneModsCount(int count) {
    return '已安装过场动画模组（$count）';
  }

  @override
  String cutsceneUsmCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '文件',
      one: '文件',
    );
    return '$count 个 USM $_temp0';
  }

  @override
  String cutsceneMatchCount(int matching, int total) {
    return '$matching/$total 与原版匹配';
  }

  @override
  String tooltipMissingOriginals(String files) {
    return '与原版名称不匹配的文件：$files';
  }

  @override
  String get cutsceneMismatchHint => '部分文件名与原版过场动画不匹配。缺失文件将回退到原版过场动画。';

  @override
  String cutsceneMigrationBannerBody(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '文件',
      one: '文件',
    );
    return '在 data/movie/ 中直接发现 $count 个自定义过场动画$_temp0。这些文件会永久覆盖原版。下次请改为在此处安装过场动画模组；如果自定义文件加载失败，将回退播放原版文件。';
  }

  @override
  String hardwareInfoLabel(int ram, String gpu) {
    return '${ram}GB 内存 | $gpu';
  }

  @override
  String hardwareInfoRamOnly(int ram) {
    return '${ram}GB 内存';
  }

  @override
  String texturesScanResult(int count, int sizeMB, int maxDim) {
    return '$count 个纹理文件，共 ${sizeMB}MB，最大 ${maxDim}px';
  }

  @override
  String recommendedSettings(int dim, String allLabel) {
    return '推荐：预加载 $dim，全部预加载 $allLabel';
  }

  @override
  String get applyRecommended => '应用';

  @override
  String get settingsMatchRecommended => '当前设置与推荐值一致';

  @override
  String get reasonNoTextures => '未安装纹理';

  @override
  String reasonFitsInMemory(int ramGB, int textureSizeMB) {
    return '${ramGB}GB 内存，纹理共 ${textureSizeMB}MB——可全部放入内存，预加载全部纹理可实现零卡顿';
  }

  @override
  String reasonExceedsRam(int ramGB, int estimatedGB) {
    return '${ramGB}GB 内存，预计纹理内存约 ${estimatedGB}GB——全部预加载会导致系统冻结或崩溃。请使用较低的预加载尺寸，或移除部分纹理包。';
  }

  @override
  String reasonTooLargeForAll(int ramGB, int textureSizeMB) {
    return '${ramGB}GB 内存，纹理共 ${textureSizeMB}MB——无法全部预加载，请按需预加载最高 4K 的纹理';
  }

  @override
  String reasonMediumRam(int ramGB) {
    return '${ramGB}GB 内存——预加载最高 4K 的纹理，更大的纹理按需加载';
  }

  @override
  String reasonLowRam(int ramGB) {
    return '${ramGB}GB 内存——仅预加载小型纹理以节省内存';
  }

  @override
  String get analyzingHardware => '正在分析硬件和纹理...';

  @override
  String texturesBloatWarning(int total, int relevant, int excess) {
    return '此模组包含 $total 个纹理，但根据 GPUnity 的精选参考集，只有 $relevant 个在视觉上相关。其余 $excess 个纹理只会增加加载时间和内存占用，却不会带来可见收益。';
  }

  @override
  String cleanUnneededTextures(int count) {
    return '移除 $count 个不需要的纹理';
  }

  @override
  String cleanedTextures(int deleted, int kept) {
    return '已移除 $deleted 个不需要的纹理，保留 $kept 个';
  }

  @override
  String get confirmCleanTextures => '移除不需要的纹理？';

  @override
  String confirmCleanTexturesBody(int count, String sizeMB) {
    return '这将从此模组文件夹永久删除 $count 个纹理文件（$sizeMB MB）。';
  }

  @override
  String get confirmCleanTexturesDetail1 => '仅保留与 GPUnity 精选参考集匹配的纹理';

  @override
  String get confirmCleanTexturesDetail2 => '只影响所选模组文件夹，不影响其他已安装模组';

  @override
  String get confirmCleanTexturesDetail3 => '此操作无法撤销；重新拖入模组可恢复被移除的文件';

  @override
  String get texturesBloatDialogTitle => '检测到不必要的纹理';

  @override
  String texturesBloatDialogBody(int total, int relevant, int excess) {
    return '此纹理包包含 $total 个文件，但只有 $relevant 个与 GPUnity 精选参考集匹配。其余 $excess 个纹理很可能不需要。';
  }

  @override
  String get texturesBloatPoint1 => '游戏启动时间大幅延长——引擎会在启动时加载每个纹理';

  @override
  String get texturesBloatPoint2 => '随机卡顿和掉帧——游戏会流式加载没有视觉收益的纹理';

  @override
  String get texturesBloatPoint3 => '内存占用很高——可能在不可见纹理上浪费数 GB 内存';

  @override
  String get texturesBloatPoint4 => '部分 AI 放大纹理可能包含瑕疵或损坏';

  @override
  String get texturesBloatPoint5 => '几乎没有视觉差异——其中多数只是小型 UI 元素、粒子效果等';

  @override
  String get texturesBloatRecommendation => '安全移除这些纹理可改善性能，推荐这样做。';

  @override
  String get texturesBloatKeepAll => '全部保留';

  @override
  String texturesBloatRemoveUnneeded(int count) {
    return '移除不需要的纹理（$count）';
  }

  @override
  String get texturesProgressExtracting => '正在解压压缩包...';

  @override
  String get texturesProgressCopying => '正在复制文件...';

  @override
  String get texturesProgressAnalyzing => '正在分析纹理...';

  @override
  String get texturesAnalyzingSetup => '正在分析你的纹理配置...';

  @override
  String get texturesBusyMessage => '请稍候，正在安装纹理';

  @override
  String texturesInstallProgress(
    int files,
    int totalFiles,
    int mb,
    int totalMb,
  ) {
    return '正在安装：$files/$totalFiles 个文件 - $mb/$totalMb MB';
  }

  @override
  String texturesAnalyzeProgress(int scanned, int total) {
    return '正在分析：$scanned/$total 个纹理';
  }

  @override
  String get cleaningTextures => '正在移除不需要的纹理...';

  @override
  String get textureMergeTitle => '添加到现有模组还是作为新模组安装？';

  @override
  String get textureMergeDescription => '你已经安装了纹理模组。要将这些文件添加到现有模组，还是作为新模组安装？';

  @override
  String get textureMergeNewMod => '作为新模组安装';

  @override
  String textureMergeAddTo(String name) {
    return '添加到：$name';
  }

  @override
  String get cutsceneMergeTitle => '添加到现有模组还是作为新模组安装？';

  @override
  String get cutsceneMergeDescription => '你已经安装了过场动画模组。多部分过场动画包应合并到同一个模组中。';

  @override
  String get cutsceneMergeNewMod => '作为新模组安装';

  @override
  String cutsceneMergeAddTo(String name) {
    return '添加到：$name';
  }

  @override
  String get headerMods => '模组';

  @override
  String cutsceneBundledWith(String modId) {
    return '随 $modId 捆绑';
  }

  @override
  String get cutsceneStatusHd => '高清';

  @override
  String get cutsceneStatusHdTooltip =>
      'nams.toml 中的 [cutscene] hd_cutscenes；必须设为 true，高清过场动画模组才能加载。';

  @override
  String get cutsceneStatusH264 => 'H264';

  @override
  String get cutsceneStatusH264Tooltip =>
      'nams.toml 中的 [cutscene] enable_h264；必须设为 true，才能播放 H264 编码的过场动画。';

  @override
  String get modIntroTitle => '由 NAMS 驱动——你的 data/ 文件夹保持不变';

  @override
  String get modIntroBody =>
      'NAMS 通过虚拟文件系统，在原始游戏数据之上从 nams/mods/ 加载模组，因此不会向 data/ 复制或覆盖任何内容。模组可随时开启或关闭，无需重新安装；同一角色可以同时存在多套服装；卸载模组只会删除它的文件夹——底层原版游戏始终完整无损。';

  @override
  String get modListEmpty => '未安装模组';

  @override
  String get modListEmptyHint => '将模组文件夹或压缩包拖入上方区域进行安装。';

  @override
  String get modSearchPlaceholder => '搜索模组…';

  @override
  String get modFilterAll => '全部';

  @override
  String get modBulkInstall => '从文件夹批量安装';

  @override
  String modBulkInstallBusy(int done, int total, String name) {
    return '正在安装 $done/$total：$name';
  }

  @override
  String get modBulkInstallScanning => '正在扫描文件夹中的模组压缩包…';

  @override
  String get modBulkInstallNone => '该文件夹中未找到模组压缩包（.zip / .7z / .rar）。';

  @override
  String modBulkInstallDone(int installed, int total) {
    return '已安装 $installed/$total 个模组。';
  }

  @override
  String get modLooseInstall => '从文件夹安装散装文件';

  @override
  String get modLooseInstallScanning => '正在扫描文件夹中的散装游戏文件……';

  @override
  String get modLooseInstallNone => '该文件夹中未找到散装游戏文件（.dat / .dtt）。';

  @override
  String modLooseInstallBusy(int count) {
    return '正在安装 $count 个散装文件……';
  }

  @override
  String modLooseInstallProgress(int done, int total) {
    return '正在复制文件 $done/$total……';
  }

  @override
  String get modLooseInstallFinalizing => '正在将文件放入模组……';

  @override
  String modLooseInstallDone(int count, String id) {
    return '已将 $count 个散装文件安装到 $id。';
  }

  @override
  String get modGroup2b => '2B 服装';

  @override
  String get modGroup9s => '9S 服装';

  @override
  String get modGroupA2 => 'A2 服装';

  @override
  String get modGroupOtherOutfits => '其他服装';

  @override
  String get modGroupWeapons => '武器';

  @override
  String get modGroupAccessories => '配饰';

  @override
  String get modGroupItems => '道具';

  @override
  String get modGroupEnemies => '敌人';

  @override
  String get modGroupWorldProps => '场景物件';

  @override
  String get modGroupModelVariants => '模型变体';

  @override
  String get modGroupMaps => '地图 / 场景';

  @override
  String get modGroupUi => '界面 / 字体';

  @override
  String get modGroupMisc => '杂项纹理';

  @override
  String get modGroupArchives => 'CPK 归档';

  @override
  String get modGroupEffects => '效果';

  @override
  String get modGroupScripting => '脚本';

  @override
  String get modGroupLocalization => '文本与本地化';

  @override
  String get modGroupCutscenes => '过场动画';

  @override
  String get modGroupAudio => '音频';

  @override
  String get modGroupTextures => '纹理';

  @override
  String get modGroupNative => '原生模组';

  @override
  String get modGroupOther => '其他';

  @override
  String get modGroupMixed => '混合内容';

  @override
  String get modGroupMultiHint => '此模组会替换多个角色的模型，因此会列在每个相关角色下。';

  @override
  String get modGroupMixedHint => '此模组会同时更改多种类型的内容。点击查看它包含的全部内容以及涉及的类别。';

  @override
  String get modRename => '重命名';

  @override
  String get modRenameDialogTitle => '重命名模组';

  @override
  String get modRenameReset => '恢复原始名称';

  @override
  String get dropModHere => '将模组拖放到此处';

  @override
  String get dropModHereHint => '或点击浏览';

  @override
  String get modKindNative => '原生';

  @override
  String get modKindNativeTooltip =>
      '带有 entities/ 文件夹的 NAMS 模组。通过 TOML 包定义新物品、武器、服装、饰品、任务等。';

  @override
  String get modKindData => '数据';

  @override
  String get modKindDataTooltip =>
      '经典模组格式——文件通常会放入 NieRAutomata/data/，但这里改由 nams/mods/ 管理，从而保持原始 data 目录干净。';

  @override
  String get textureOutfitLinkedTitle => '服装关联纹理';

  @override
  String get textureOutfitLinkedSubtitle =>
      '这些纹理位于各自的模组文件夹内，仅在装备对应服装时加载。你在游戏中更换服装时，NAMS 会对它们进行热切换。';

  @override
  String textureOutfitLinkedEntry(int count) {
    return '$count 个纹理——仅随此服装启用';
  }

  @override
  String get modKindTexture => '纹理';

  @override
  String get modKindTextureTooltip =>
      '纹理包。其 .dds 文件已安装到 nams/inject/textures/，并可从“纹理”选项卡管理。';

  @override
  String get modKindUnknown => '未知';

  @override
  String get modKindUnknownTooltip => '启动器无法将此文件夹识别为有效模组。';

  @override
  String get modCompatChip => 'wax 兼容';

  @override
  String get modCompatChipTooltip => ' 为兼容现有 wax 模组，NAMS 也会读取这些内容。';

  @override
  String get modDataChip => '+data';

  @override
  String get modDataChipTooltip => '除元数据外还附带 data/ 叠加层。模型、纹理、声音等内容位于此处。';

  @override
  String get modDetailNoSelection => '选择模组以查看详情';

  @override
  String get modAuthor => '作者';

  @override
  String get modVersion => '版本';

  @override
  String get modRootPath => '路径';

  @override
  String get modNativeBundles => '原生包';

  @override
  String get modDataContent => '数据内容';

  @override
  String get modDataPlayerModels => '玩家模型';

  @override
  String get modRequiresLabel => '需要';

  @override
  String get modRequiresPluginsLabel => '需要插件';

  @override
  String get modRequiresMissing => '缺失';

  @override
  String get modConflictsLabel => '冲突';

  @override
  String get modLoadOrderHint => '这些模组替换了相同文件。拖动以调整顺序，顶部优先。';

  @override
  String get modConflictKeep => '保留此模组';

  @override
  String get modConflictResolve => '解决冲突';

  @override
  String get modConflictDialogTitle => '哪个模组应优先？';

  @override
  String modConflictKeepTooltip(String id) {
    return '保留 $id 并禁用其他模组';
  }

  @override
  String modConflictPickBody(int mods, int files) {
    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: '$files 个文件',
      one: '1 个文件',
    );
    return '$mods 个已启用模组替换了相同的 $_temp0。请选择要保留的一个，其他模组将被禁用。';
  }

  @override
  String modConflictOverlapFile(String otherId, String file) {
    return '$otherId 也包含 $file';
  }

  @override
  String get modOpenFolder => '打开文件夹';

  @override
  String get modEnable => '启用';

  @override
  String get modDisable => '禁用';

  @override
  String get modDisabled => '已禁用';

  @override
  String get modDisabledTooltip =>
      '此模组已禁用。NAMS 下次启动游戏时不会加载它。重新启用即可再次加载，无需删除并重新安装。';

  @override
  String get modEnableTooltip => '该模组会由 NAMS 加载。点击可在不移除文件的情况下禁用。';

  @override
  String get modDefaultTooltip =>
      '从游戏启动时起生效，就像其文件位于 NieRAutomata/data 中一样。点击可关闭。';

  @override
  String get modSetDefaultTooltip =>
      '让此模组从游戏启动时起生效，而无需向 NieRAutomata/data 复制任何内容。';

  @override
  String get modSetDefaultOutfitTooltip =>
      '从游戏启动时起穿戴此服装，而无需向 NieRAutomata/data 复制任何内容。它会替换当前默认服装；同一时间只能有一个默认服装。';

  @override
  String get modDefaultChip => '默认';

  @override
  String get modDefaultKindOutfitBare => '服装';

  @override
  String get modDefaultKindOutfitConfig => '服装 + 配置';

  @override
  String get modDefaultKindOutfitAnimation => '动画';

  @override
  String get modDefaultKindOutfitBareTooltip => '直接替换模型文件。同一时间只能有一套服装作为默认服装。';

  @override
  String get modDefaultKindOutfitConfigTooltip =>
      '此模组附带服装配置，因此其网格规则和效果会一起加载。同一时间只能有一套服装作为默认服装。';

  @override
  String get modDefaultKindOutfitAnimationTooltip =>
      '这是动画数据，不是服装。无论穿戴哪套服装，它都会在底层保持启用。';

  @override
  String get modDefaultReplaceTitle => '替换默认服装？';

  @override
  String modDefaultReplaceBody(String model, String current, String next) {
    return '$model 当前在游戏启动时由“$current”穿戴。\n\n将“$next”设为默认会移除当前设置，因为同一时间只能有一个模组为角色更换服装。';
  }

  @override
  String get modDefaultReplaceConfirm => '替换';

  @override
  String get modDefaultOutfitAuto => '默认服装';

  @override
  String get modDefaultOutfitPickTooltip =>
      '此模组包含多套服装。请选择游戏启动时要穿戴的服装。“默认服装”指不使用物品时穿戴的那一套。';

  @override
  String modDefaultRowTooltip(String files) {
    return '从游戏启动时起启用：$files';
  }

  @override
  String get modDisableNotice => '已禁用，将在下次启动游戏时生效。';

  @override
  String get modEnableNotice => '已启用，将在下次启动游戏时生效。';

  @override
  String get modUninstall => '卸载';

  @override
  String get modUninstallConfirmTitle => '卸载模组？';

  @override
  String modUninstallConfirmBody(String id) {
    return '这将永久删除模组文件夹“$id”。';
  }

  @override
  String get modProfileLabel => '配置档案';

  @override
  String get modProfileNewButton => '新建';

  @override
  String get modProfileRenameButton => '重命名';

  @override
  String get modProfileDeleteButton => '删除';

  @override
  String get modProfileNewDialogTitle => '新建配置档案';

  @override
  String get modProfileNewDialogHint => '配置档案名称（字母、数字、_、-）';

  @override
  String get modProfileRenameDialogTitle => '重命名配置档案';

  @override
  String get modProfileDeleteDialogTitle => '删除配置档案？';

  @override
  String modProfileDeleteDialogBody(String name) {
    return '永久移除此配置档案中的 mods_profile_$name/ 文件夹及所有捆绑纹理包。此操作无法撤销。';
  }

  @override
  String get modProfileDeleteConfirm => '删除';

  @override
  String get modProfileErrorNameEmpty => '必须填写名称';

  @override
  String get modProfileErrorNameInvalid => '只能使用字母、数字、_ 或 -';

  @override
  String get modProfileErrorNameCollision => '已存在同名配置档案';

  @override
  String get modProfileErrorDeleteActive => '删除前请先切换到其他配置档案';

  @override
  String get modProfileErrorDeleteLast => '无法删除唯一剩余的配置档案';

  @override
  String get modProfileErrorTargetMissing => '磁盘上的配置档案文件夹缺失';

  @override
  String get modProfileErrorFsBusy => '文件系统正忙。请关闭游戏后重试。';

  @override
  String get modProfileLockedRunning => '更改配置档案前请先停止游戏。';

  @override
  String get modProfileEmptyHint => '空配置档案——拖入模组以开始';

  @override
  String modProfileSwitchedToast(String name) {
    return '已切换到配置档案 $name';
  }

  @override
  String modProfileCreatedToast(String name) {
    return '已创建并切换到配置档案 $name';
  }

  @override
  String modProfileDeletedToast(String name) {
    return '已删除配置档案 $name';
  }

  @override
  String modProfileRenamedToast(String name) {
    return '已将配置档案重命名为 $name';
  }

  @override
  String get modInstallNeedsName => '为此模组命名';

  @override
  String modInstallExistsPickAnother(String id) {
    return '名为“$id”的模组已存在。请选择其他名称。';
  }

  @override
  String get modInspectBusy => '正在检查模组…';

  @override
  String get modInstallBusy => '正在安装模组…';

  @override
  String get modVariantDialogTitle => '选择要安装的内容';

  @override
  String get modVariantDialogSubtitle => '此压缩包包含多个选项。请选择你需要的内容。';

  @override
  String get modOutfitChoiceDialogTitle => '选择要安装的内容';

  @override
  String get modOutfitChoiceDialogSubtitle =>
      '勾选你需要的全部内容。每个项目都会作为独立模组安装。如果服装附带纹理，纹理也会一起安装，之后可在“纹理”选项卡中进一步调整使用哪些纹理组。';

  @override
  String get variantCatPlayer => '服装';

  @override
  String get variantCatWeapon => '武器';

  @override
  String get variantCatAccessory => '饰品';

  @override
  String get variantCatEnemy => '敌人';

  @override
  String get variantCatModelVariant => '模型变体';

  @override
  String get variantCatItem => '物品';

  @override
  String get variantCatWorldProp => '世界场景物件';

  @override
  String get variantCatMap => '地图';

  @override
  String get variantCatEffects => '效果';

  @override
  String get variantCatScripting => '脚本';

  @override
  String get variantCatLocalization => '本地化';

  @override
  String get variantCatUi => 'UI';

  @override
  String get variantCatCutscenes => '过场动画';

  @override
  String get variantCatAudio => '音频';

  @override
  String get variantCatMisc => '其他';

  @override
  String get variantCatOther => '其他';

  @override
  String get variantPickOneSuffix => '请选择一个';

  @override
  String get modVariantSelectAll => '全选';

  @override
  String get modVariantSelectNone => '全不选';

  @override
  String get modVariantInstall => '安装';

  @override
  String modVariantInstallSelected(int count) {
    return '安装 $count 项';
  }

  @override
  String get modVariantTexture => '纹理';

  @override
  String modVariantInstalledToast(int count) {
    return '已安装 $count 个选项';
  }

  @override
  String get modUninstallBusy => '正在卸载模组…';

  @override
  String modInstalled(String id) {
    return '已安装：$id';
  }

  @override
  String modInstallFailed(String reason) {
    return '安装失败：$reason';
  }

  @override
  String get modInstallReasonUnknownDrop => '无法识别拖入内容——该文件夹不符合任何受支持的模组结构。';

  @override
  String get modInstallReasonInvalidMixed =>
      '无效结构——一个模组不能同时混用 entities 和 wax 风格配置。';

  @override
  String get modInstallReasonNativeEmpty => 'entities/ 中未找到实体文件。';

  @override
  String get modInstallReasonDataEmpty => '未找到可识别的内容。';

  @override
  String get modInstallReasonArchiveExtractFailed => '无法解压压缩包。';

  @override
  String get modInstallReasonMoveFailed => '无法将文件移动到 nams/mods/。';

  @override
  String get modInstallReasonTextureOnly => '这是纹理包（仅包含 .dds 文件）。请改从“纹理”选项卡安装。';

  @override
  String modUninstalled(String id) {
    return '已移除：$id';
  }

  @override
  String modCountFiles(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count 个文件',
      one: '1 个文件',
    );
    return '$_temp0';
  }
}
