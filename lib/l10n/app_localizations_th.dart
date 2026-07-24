// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Thai (`th`).
class AppLocalizationsTh extends AppLocalizations {
  AppLocalizationsTh([String locale = 'th']) : super(locale);

  @override
  String get appTitle => 'ตัวเรียกใช้งาน YoRHa Protocol';

  @override
  String get tooltipLauncherSource => 'ซอร์สโค้ดของตัวเรียกใช้งาน';

  @override
  String get tooltipNamsSource => 'ซอร์สของโปรเจกต์ NAMS';

  @override
  String get tooltipGuide => 'คู่มือ';

  @override
  String get tooltipDiscord => 'Discord';

  @override
  String get tooltipLanguage => 'ภาษา';

  @override
  String get languageSupportNotice =>
      'คำแปลจัดทำโดยชุมชนหรือสร้างขึ้นโดยอัตโนมัติและอาจไม่ถูกต้อง ผู้ดูแลสื่อสารภาษาอังกฤษเท่านั้น โปรดขอความช่วยเหลือเป็นภาษาอังกฤษ';

  @override
  String get tooltipCopyCommand =>
      'คัดลอกคำสั่ง NAMS ไปยังคลิปบอร์ด เพื่อให้คุณนำไปวางในเทอร์มินัลและเริ่มเกมด้วยตนเอง';

  @override
  String get notificationCommandCopied =>
      'คัดลอกคำสั่งเริ่มเกมแล้ว ให้นำไปวางในเทอร์มินัลเพื่อเริ่มเกมด้วยตนเอง';

  @override
  String get notificationCommandNotReady =>
      'ไม่สามารถสร้างคำสั่งเริ่มเกมได้ ไบนารีของตัวเรียกใช้งานยังไม่พร้อม';

  @override
  String get textureAutoRecommended => 'อัตโนมัติ (แนะนำ)';

  @override
  String get detectionDlcPresent => 'DLC: มีอยู่';

  @override
  String get detectionDlcNotDetected => 'DLC: ตรวจไม่พบ';

  @override
  String get detectionDlcPresentTooltip =>
      'พบ DLC data100.cpk ม็อดที่มีไฟล์ชุดเฉพาะ DLC (pl000d, pl010d, pl020d) จะถูกติดตั้งตามเดิม';

  @override
  String get detectionDlcNotDetectedTooltip =>
      'ตรวจไม่พบ DLC ม็อดที่มีไฟล์ชุดเฉพาะ DLC (pl000d, pl010d, pl020d) จะถูกติดตั้งด้วยชื่อแบบไม่มี DLC (pl0000, pl0100, pl0200) เพื่อให้แสดงในเกม';

  @override
  String get detectionExeWolfLimitBreak => 'EXE: Wolf Limit Break';

  @override
  String get detectionExeOriginal => 'EXE: ต้นฉบับ';

  @override
  String get detectionExeMissing => 'EXE: ไม่พบ';

  @override
  String get detectionExeUnrecognised => 'EXE: ไม่รู้จัก';

  @override
  String get detectionExeUnrecognisedTooltip =>
      'มี NieRAutomata.exe อยู่ แต่แฮชของไฟล์ไม่อยู่ในรายการที่เรารู้จัก NAMS ยังทำงานได้ ข้อความนี้เป็นเพียงการแจ้งเตือนว่าเรายังไม่เคยพบเวอร์ชันนี้โดยตรง';

  @override
  String get detectionExeWolfLimitBreakTooltip =>
      'ตรวจพบ NieRAutomata.exe ของ Wolf Limit Break NAMS ไม่ต้องใช้แพตช์นี้และไม่เคยผ่านการทดสอบร่วมกัน เกมอาจยังเปิดได้ แต่อาจเกิดปัญหาประสิทธิภาพ เกมล่มจากหน่วยความจำ หรือม็อดไม่เข้ากัน เพื่อรับการรองรับเต็มรูปแบบ โปรดคืนค่าไฟล์ปฏิบัติการ Steam ต้นฉบับ (ตรวจสอบไฟล์เกมใน Steam)';

  @override
  String get detectionExeLegacyWin7 => 'EXE: เวอร์ชัน Windows 7';

  @override
  String get detectionExeLegacyWin7Tooltip =>
      'นี่คือ NieRAutomata.exe รุ่นเก่าสำหรับ Windows 7/8 NAMS ต้องใช้เวอร์ชัน Steam สำหรับ Windows 10/11 และไม่สามารถเปิดไฟล์นี้ได้ ปัญหานี้พบบ่อยบน Proton/Linux ซึ่งบางครั้ง Steam ดาวน์โหลดไฟล์ปฏิบัติการ Windows 7 มาให้\n\nวิธีแก้ไข:\n1. ลบไฟล์ .exe ทุกไฟล์ในโฟลเดอร์เกม NieRAutomata\n2. ใน Steam ตั้งค่า Proton เป็น \'Proton Experimental\' (คลิกขวาที่เกม > Properties > Compatibility)\n3. ใน Steam คลิกขวาที่เกม > Properties > Installed Files > Verify integrity of game files\n4. เปิดเกมผ่าน Steam หนึ่งครั้งเพื่อให้ระบบเก็บไฟล์ปฏิบัติการที่ถูกต้องไว้ จากนั้นจึงใช้ตัวเรียกใช้งานอีกครั้ง';

  @override
  String get launchOptionsTitle => 'ตัวเลือกการเริ่มเกม';

  @override
  String get launchWrapperTitle => 'ตัวครอบคำสั่งเริ่มเกม (LINUX)';

  @override
  String get launchWrapperDesc =>
      'เพิ่มคำสั่งไว้หน้าคำสั่งเริ่มเกม เช่น gamescope หรือ mangohud ตัวเรียกใช้งานเปิดเกมผ่าน Proton ดังนั้นตัวเลือกการเริ่มเกมของ Steam จะไม่มีผลที่นี่ เว้นว่างไว้เพื่อเริ่มเกมตามปกติ การตั้งค่านี้จะมีผลในการเริ่มเกมครั้งถัดไป';

  @override
  String get launchWrapperHint => 'gamescope -w 2560 -h 1440 -f --';

  @override
  String get launchWrapperExample =>
      'ตัวอย่าง:\ngamescope -w 2560 -h 1440 -f --\nmangohud\ngamemoderun';

  @override
  String get tabLauncherSettings => 'การแก้ไขปัญหา';

  @override
  String get troubleWrongExeTitle => 'เวอร์ชันเกมไม่ถูกต้อง';

  @override
  String get troubleWrongExeSummary =>
      'นี่คือ NieRAutomata.exe รุ่นเก่าสำหรับ Windows 7/8 NAMS ไม่สามารถเปิดได้';

  @override
  String get troubleMissingFilesTitle => 'ไฟล์ตัวเรียกใช้งานหายไป';

  @override
  String troubleMissingFilesSummary(String files) {
    return 'หายไป: $files';
  }

  @override
  String get troubleMissingFilesDesc =>
      'ไฟล์เหล่านี้เป็นส่วนหนึ่งของตัวเรียกใช้งานแบบพกพา แต่หายไป มีความเป็นไปได้สูงว่าโปรแกรมป้องกันไวรัส (มักเป็น Windows Defender) กักกันไว้ กรุณายกเว้นโฟลเดอร์ตัวเรียกใช้งานในโปรแกรมป้องกันไวรัสและกู้คืนไฟล์ หรือดาวน์โหลดตัวเรียกใช้งานใหม่';

  @override
  String get troubleRecentErrorsTitle => 'ข้อผิดพลาด NAMS ล่าสุด';

  @override
  String get troubleFoldersTitle => 'เปิดโฟลเดอร์';

  @override
  String get troubleFoldersDesc =>
      'เข้าถึงโฟลเดอร์ที่ผู้ดูแลอาจขอให้คุณเปิดอย่างรวดเร็ว';

  @override
  String get troubleOpenGame => 'โฟลเดอร์เกม';

  @override
  String get troubleOpenNamsConfig => 'การตั้งค่า NAMS';

  @override
  String get troubleOpenLogs => 'บันทึก';

  @override
  String get troubleOpenBins => 'โฟลเดอร์ตัวเรียกใช้งาน';

  @override
  String get troubleClearCacheTitle => 'ล้างแคช';

  @override
  String get troubleClearCacheDesc =>
      'ลบสถานะแคชที่อาจล้าสมัย: ไฟล์บันทึก แคชการตรวจจับ ไฟล์ settings.json ของม็อด และเครื่องหมายการแตกไฟล์รันไทม์ ม็อดและไฟล์เกมของคุณจะไม่ถูกแตะต้อง';

  @override
  String get troubleClearCacheButton => 'ล้างแคช';

  @override
  String get troubleClearCacheConfirm =>
      'การดำเนินการนี้จะลบบันทึก แคชการตรวจจับ ไฟล์ settings.json ในเกมของม็อด และบังคับให้แตกไฟล์รันไทม์ใหม่เมื่อเริ่มครั้งถัดไป ม็อดและไฟล์เกมจะไม่ถูกแตะต้อง ดำเนินการต่อหรือไม่?';

  @override
  String troubleClearCacheDone(int count) {
    return 'ล้างแคชแล้ว (ลบไฟล์บันทึก $count ไฟล์)';
  }

  @override
  String get verifyInstallTitle => 'การวินิจฉัยการติดตั้ง';

  @override
  String get verifyInstallDesc =>
      'เรียกใช้การตรวจสอบในตัวของ NAMS เพื่อวิเคราะห์สาเหตุที่เกมอาจเปิดไม่ได้ (เวอร์ชัน Windows ไม่ถูกต้อง ไฟล์ Steam ขาดหาย หรือสิทธิ์การเข้าถึง)';

  @override
  String get verifyInstallButton => 'ตรวจสอบการติดตั้ง';

  @override
  String get verifyInstallRunning => 'กำลังตรวจสอบ...';

  @override
  String get verifyInstallOk => 'ผ่านการตรวจสอบทั้งหมด';

  @override
  String get verifyInstallFailed =>
      'การตรวจสอบบางรายการล้มเหลว ดูรายละเอียดด้านล่าง';

  @override
  String get verifyNoRuntime =>
      'ไม่สามารถตรวจสอบได้: ไม่พบรันไทม์ Proton/Wine สำหรับการติดตั้งนี้';

  @override
  String get verifySteamNotRunning =>
      'ไม่สามารถตรวจสอบได้: Steam ต้องกำลังทำงานและบัญชีต้องเป็นเจ้าของเกม';

  @override
  String get verifyInstallError =>
      'ไม่สามารถเรียกใช้การตรวจสอบได้ โปรดตรวจสอบว่าได้เลือกโฟลเดอร์เกมแล้ว';

  @override
  String get verifyInstallNoGameDir => 'โปรดเลือกโฟลเดอร์เกมก่อน';

  @override
  String get verifyCheckSteamInstall => 'การติดตั้ง Steam';

  @override
  String get verifyCheckNierExe => 'ไฟล์ปฏิบัติการของเกม';

  @override
  String get verifyCheckSteamApi64 => 'ไลบรารี Steam API';

  @override
  String get verifyCheckRuntimeWritable => 'รันไทม์สามารถเขียนได้';

  @override
  String get verifyCheckRuntimeCached => 'แคชไลบรารีรันไทม์แล้ว';

  @override
  String get launchOptionMinimizeOnLaunch => 'ย่อตัวเรียกใช้งานขณะเล่นเกม';

  @override
  String get launchOptionPreferDedicatedGpu => 'เลือกใช้ GPU แยกเป็นหลัก';

  @override
  String get launchOptionPreferDedicatedGpuTooltip =>
      'แจ้งระบบให้เปิดเกมด้วยการ์ดจอแยกแทนการ์ดจอประหยัดพลังงาน มีผลเฉพาะคอมพิวเตอร์ที่มี GPU สองตัว เช่น โน้ตบุ๊กเกมมิ่ง';

  @override
  String get failTitlePanic => 'NAMS ขัดข้อง';

  @override
  String get failTitleUnknown => 'เริ่มเกมไม่สำเร็จ';

  @override
  String get failExplanationPanic =>
      'NAMS พบข้อผิดพลาดที่ไม่สามารถกู้คืนได้ก่อนเกมเริ่มทำงาน โดยทั่วไปมักเป็นบั๊ก โปรดส่งรายงานด้านล่างให้ผู้ดูแล';

  @override
  String get failExplanationUnknown =>
      'เกมไม่เริ่มภายใน 60 วินาทีและไม่มีการรายงานข้อผิดพลาด';

  @override
  String get failHintPanicShare =>
      'คัดลอกรายงานฉบับเต็มด้านล่างแล้วส่งให้ผู้ดูแล';

  @override
  String get failHintPanicReboot =>
      'ลองอีกครั้งหลังรีบูต บางครั้งแฮนเดิลเก่าจะถูกล้างออก';

  @override
  String get failHintUnknownSpawned =>
      'ดูเหมือน NAMS จะเริ่มกระบวนการแล้ว แต่หน้าต่างเกมไม่ปรากฏ';

  @override
  String get failHintUnknownTaskManager =>
      'ตรวจสอบ Task Manager ว่า NieRAutomata.exe กำลังทำงานแต่ไม่แสดงหน้าต่างหรือไม่ หากใช่ ให้ปิดกระบวนการแล้วลองใหม่';

  @override
  String get failHintUnknownOtherLauncher =>
      'ตรวจสอบว่าไม่มีตัวเรียกใช้งานหรือเครื่องมือ DRM อื่นกำลังจับไฟล์ exe อยู่ (FAR, Special K เป็นต้น)';

  @override
  String get failTitleNamsFailure => 'NAMS รายงานความล้มเหลว';

  @override
  String get failExplanationNamsFailure =>
      'การตรวจสอบของ NAMS ล้มเหลวก่อนที่เกมจะทำงาน ดูรายละเอียดในรายงานด้านล่าง';

  @override
  String get failHintShareReport =>
      'คัดลอกรายงานฉบับเต็มด้านล่างและแชร์เพื่อใช้วิเคราะห์ปัญหา';

  @override
  String get failTitleInstallNotFound => 'ไม่พบการติดตั้ง NieR:Automata';

  @override
  String get failExplanationInstallNotFound =>
      'NAMS ไม่สามารถระบุตำแหน่งการติดตั้ง NieR:Automata ได้ พาธที่บันทึกไว้อาจผิด หรือการตรวจหา Steam อัตโนมัติล้มเหลว';

  @override
  String get failHintRepickDirectory =>
      'เลือกโฟลเดอร์เกมใหม่ในตัวเรียกใช้งานเพื่ออัปเดตพาธที่บันทึกไว้';

  @override
  String get failHintVerifyFiles =>
      'ตรวจสอบไฟล์เกมใน Steam (Library → NieR:Automata → Properties → Local Files → Verify)';

  @override
  String get failTitleFolderCreate => 'ไม่สามารถสร้างโฟลเดอร์ที่จำเป็นได้';

  @override
  String get failExplanationFolderCreate =>
      'NAMS ไม่สามารถสร้างไดเรกทอรีถัดจาก NAMS.exe ได้ โฟลเดอร์ติดตั้งอาจเป็นแบบอ่านอย่างเดียว';

  @override
  String get failHintWritableFolder =>
      'ตรวจสอบว่าโฟลเดอร์ติดตั้งตัวเรียกใช้งาน (ตำแหน่งที่มี NAMS.exe) สามารถเขียนได้';

  @override
  String get failHintProgramFiles =>
      'หากอยู่ใน Program Files หรือโฟลเดอร์ที่ซิงก์กับ OneDrive ให้ย้ายตัวเรียกใช้งานไปยังโฟลเดอร์ปกติ หรือคลิกขวา → \'Always keep on this device\'';

  @override
  String get failTitleRuntimePrep => 'การเตรียมรันไทม์ล้มเหลว';

  @override
  String get failExplanationRuntimePrep =>
      'NAMS ไม่สามารถเตรียมรันไทม์ (game.bin / steam_api64.dll) ได้ โดยทั่วไปเกิดจากสิทธิ์การเขียนหรือโปรแกรมป้องกันไวรัส';

  @override
  String get failHintAntivirusExclusions =>
      'เพิ่มทั้งโฟลเดอร์ติดตั้งตัวเรียกใช้งานและโฟลเดอร์เกมลงในรายการยกเว้นของโปรแกรมป้องกันไวรัส แล้วลองใหม่';

  @override
  String get failHintWritableCache =>
      'ตรวจสอบว่าโฟลเดอร์ติดตั้งสามารถเขียนได้ เพื่อให้สร้างแคชรันไทม์ได้';

  @override
  String get failTitleHostFailure => 'โฮสต์ NAMS ล้มเหลว';

  @override
  String get failExplanationHostFailure =>
      'NAMS ไม่สามารถโหลดและเริ่มโฮสต์เกม (game.bin) ได้ โดยทั่วไปเกิดจากสภาพแวดล้อมหรือไฟล์เสียหาย';

  @override
  String get failHintReboot =>
      'รีบูตแล้วลองใหม่ บางครั้งแฮนเดิลเก่าจะถูกล้างออก';

  @override
  String get failHintPersistShare =>
      'หากปัญหายังคงอยู่ ให้คัดลอกรายงานฉบับเต็มและส่งให้ผู้ดูแล';

  @override
  String get failTitleSteamNotRunning =>
      'Steam ไม่ทำงาน / ยังไม่ได้เข้าสู่ระบบ';

  @override
  String get failExplanationSteamNotRunning =>
      'NAMS ไม่สามารถเชื่อมต่อกับเซสชัน Steam ที่เข้าสู่ระบบแล้วได้ Steam ต้องกำลังทำงานและเข้าสู่ระบบอยู่';

  @override
  String get failHintStartSteam =>
      'เปิด Steam และเข้าสู่ระบบ จากนั้นลองเริ่มเกมอีกครั้ง';

  @override
  String get failTitleSteamNotOwned =>
      'บัญชี Steam ไม่ได้เป็นเจ้าของ NieR:Automata';

  @override
  String get failExplanationSteamNotOwned =>
      'บัญชี Steam ที่เข้าสู่ระบบอยู่ไม่ได้เป็นเจ้าของ NieR:Automata';

  @override
  String get failHintSignInOwner =>
      'เข้าสู่ระบบด้วยบัญชี Steam ที่เป็นเจ้าของ NieR:Automata';

  @override
  String get failTitleSteamCheckFailed => 'การตรวจสอบ Steam ล้มเหลว';

  @override
  String get failExplanationSteamCheckFailed =>
      'NAMS พบข้อผิดพลาดภายในขณะตรวจสอบความเป็นเจ้าของเกมบน Steam';

  @override
  String get failHintRestartSteam =>
      'รีสตาร์ต Steam และตัวเรียกใช้งาน แล้วลองใหม่';

  @override
  String get failTitleInvalidArgs => 'อาร์กิวเมนต์เริ่มเกมไม่ถูกต้อง';

  @override
  String get failExplanationInvalidArgs =>
      'ตัวเรียกใช้งานส่งอาร์กิวเมนต์ที่ NAMS ไม่สามารถแยกวิเคราะห์ได้ นี่เป็นบั๊กของตัวเรียกใช้งาน';

  @override
  String get failTitleExitedUnexpectedly => 'เกมปิดโดยไม่คาดคิด';

  @override
  String get failExplanationExitedUnexpectedly =>
      'NAMS เริ่มเกมแล้ว แต่เกมปิดด้วยรหัสที่ไม่ใช่ศูนย์ เกมอาจขัดข้อง';

  @override
  String get failHintCheckLogViewer =>
      'ตรวจสอบตัวดูบันทึกภายในแอป (nams.log) เพื่อดูรายละเอียดการขัดข้อง';

  @override
  String get failHeadlinePanicked => 'NAMS เกิด panic';

  @override
  String get failSectionWhatHappened => 'เกิดอะไรขึ้น';

  @override
  String get failSectionReportedByNams => 'รายงานโดย NAMS';

  @override
  String get failSectionTryThis => 'ลองทำดังนี้';

  @override
  String get failSectionDiagnosticDetail => 'รายละเอียดการวินิจฉัย';

  @override
  String get failSectionLaunchManually => 'เริ่มเกมด้วยตนเองจากเทอร์มินัล';

  @override
  String get failSectionRawOutput => 'ผลลัพธ์ดิบ';

  @override
  String get failManualCommandHint =>
      'หาก UI ของตัวเรียกใช้งานยังคงทำงานล้มเหลว ให้วางข้อความนี้ในเทอร์มินัลเพื่อเริ่มเกมด้วยตนเอง นี่คือคำสั่งเดียวกับที่ปุ่ม Play ใช้งาน';

  @override
  String get failDetailOs => 'ระบบปฏิบัติการ';

  @override
  String get failDetailCause => 'สาเหตุ';

  @override
  String get failDetailSuggested => 'คำแนะนำ';

  @override
  String get failActionCopyReport => 'คัดลอกรายงาน';

  @override
  String get failActionOpenLogFile => 'เปิดไฟล์บันทึก';

  @override
  String get logDetailOs => 'ระบบปฏิบัติการ';

  @override
  String get logDetailLocale => 'ภาษา/ภูมิภาค';

  @override
  String get logNoModsInstalled => 'ยังไม่ได้ติดตั้งม็อด';

  @override
  String get logSectionSystem => 'ระบบ';

  @override
  String get logSectionModsNams => 'ม็อด (NAMS)';

  @override
  String get logSectionCutscenes => 'คัตซีน';

  @override
  String get logSectionTextures => 'เท็กซ์เจอร์';

  @override
  String get tooltipOpenInModManager => 'เปิดในตัวจัดการม็อด';

  @override
  String get tooltipOpenInCutscenesTab => 'เปิดในแท็บคัตซีน';

  @override
  String tooltipOpenInTexturesTab(String name) {
    return '$name\n\nเปิดในแท็บเท็กซ์เจอร์';
  }

  @override
  String get actionCancel => 'ยกเลิก';

  @override
  String get tooltipMinimize => 'ย่อหน้าต่าง';

  @override
  String get tooltipMaximize => 'ขยายเต็มหน้าต่าง';

  @override
  String get tooltipRestore => 'คืนค่าหน้าต่าง';

  @override
  String get tooltipClose => 'ปิด';

  @override
  String get infoText => 'เลือกเกม กดเล่น เท่านี้ก็เสร็จ';

  @override
  String get helpPrefix => 'ตัวเรียกใช้งานไม่ทำงาน? ลองใช้ ';

  @override
  String get helpNaoLauncher => 'NAO Launcher';

  @override
  String get helpOr => ' หรือ ';

  @override
  String get helpCommandLine => 'บรรทัดคำสั่ง';

  @override
  String get helpJoinDiscord => '. เข้าร่วม ';

  @override
  String get helpDiscord => 'Discord';

  @override
  String get helpSuffix => ' เพื่อขอความช่วยเหลือ';

  @override
  String get noFileSelected => 'ยังไม่ได้เลือกไฟล์';

  @override
  String get selectButton => 'เลือก';

  @override
  String get filePickerTitle => 'เลือกไฟล์ปฏิบัติการของเกม';

  @override
  String get playButton => 'เล่น';

  @override
  String get stopButton => 'หยุด';

  @override
  String get statusReady => 'พร้อมเริ่มเกม!';

  @override
  String get statusSelectGame => 'เลือกไฟล์ปฏิบัติการของเกมเพื่อเริ่มต้น';

  @override
  String get statusPreparing => 'กำลังเตรียมไฟล์ตัวเรียกใช้งาน...';

  @override
  String get statusLaunching => 'กำลังเริ่มเกม...';

  @override
  String get statusRunning => 'เกมกำลังทำงาน';

  @override
  String get statusStopped => 'เกมหยุดแล้ว';

  @override
  String get statusStopping => 'กำลังหยุดเกม...';

  @override
  String get errorInvalidExe =>
      'ดูเหมือนว่าไฟล์นี้ไม่ใช่ NieR:Automata โปรดตรวจสอบว่าเป็น Steam เวอร์ชันล่าสุด';

  @override
  String get errorFileNotExist => 'ไม่มีไฟล์ที่เลือก';

  @override
  String get errorSelectFailed => 'เลือกไฟล์ไม่สำเร็จ';

  @override
  String get errorStartFailed => 'เริ่มเกมไม่สำเร็จ';

  @override
  String get errorStopFailed => 'หยุดเกมไม่สำเร็จ';

  @override
  String errorFilesQuarantined(String files) {
    return 'ไฟล์ตัวเรียกใช้งานหายไป: $files ปัญหานี้มักเกิดจากโปรแกรมป้องกันไวรัส เราโหลดม็อดขณะรันเกม ซึ่งเป็นวิธีปกติสำหรับการม็อดเกม แต่อาจทำให้เกิดการตรวจจับผิดพลาด โปรดกู้คืนไฟล์จากพื้นที่กักกันหรือดาวน์โหลดตัวเรียกใช้งานใหม่ แล้วเพิ่มโฟลเดอร์ติดตั้งตัวเรียกใช้งาน (โฟลเดอร์ที่มี NAMS.exe) ลงในรายการยกเว้น';
  }

  @override
  String get notifyFilesQuarantined =>
      'ตรวจพบไฟล์ตัวเรียกใช้งานหายไป ปัญหานี้มักเกิดจากโปรแกรมป้องกันไวรัส เราโหลดม็อดขณะรันเกม ซึ่งเป็นเรื่องปกติสำหรับการม็อดเกม แต่อาจทำให้เกิดการตรวจจับผิดพลาด โปรดกู้คืนไฟล์จากพื้นที่กักกันหรือดาวน์โหลดตัวเรียกใช้งานใหม่ แล้วเพิ่มโฟลเดอร์ติดตั้งตัวเรียกใช้งาน (โฟลเดอร์ที่มี NAMS.exe) ลงในรายการยกเว้น';

  @override
  String get featureReshade =>
      'ReShade - ติดตั้งไว้แล้วหรือไม่? YP จะตรวจพบให้อัตโนมัติ';

  @override
  String get featureTextures =>
      'HD Textures - วางเท็กซ์เจอร์ไว้ใน nams/inject/textures/ หรือระบบจะตรวจพบจาก SK_Res/';

  @override
  String get featureLodMod =>
      'LOD Mod - การปรับภาพในตัว เช่น เงา รายละเอียด และอาการวัตถุโผล่ ตั้งค่าเริ่มต้นเป็นปิด';

  @override
  String get tooltipEditConfigs => 'เปลี่ยนการตั้งค่าภาพโดยไม่ต้องแก้ไขไฟล์';

  @override
  String get tooltipOpenLogs => 'เปิดโฟลเดอร์บันทึกใน Explorer';

  @override
  String get tooltipCreateShortcut =>
      'สร้างทางลัดบนเดสก์ท็อปเพื่อเริ่มเกมด้วย YoRHa Protocol';

  @override
  String get shortcutName => 'NieR Automata (YoRHa Protocol)';

  @override
  String get shortcutDescription => 'เริ่ม NieR:Automata ด้วย YoRHa Protocol';

  @override
  String get notifyShortcutCreated => 'สร้างทางลัดบนเดสก์ท็อปแล้ว!';

  @override
  String get notifyShortcutFailed => 'สร้างทางลัดบนเดสก์ท็อปไม่สำเร็จ';

  @override
  String get notifyLodModMigrated =>
      'พบการตั้งค่า LodMod.ini เดิมของคุณ นำเข้าไปยัง lodmod.toml และเปิดใช้งาน LodMod แล้ว';

  @override
  String get notifyReShadeDetected =>
      'ตรวจพบ ReShade และปิดไว้เป็นค่าเริ่มต้น NAMS มี Depth of Field แบบเนทีฟที่แพตช์แล้ว ดังนั้น ReShade จึงเป็นตัวเลือกเสริม คุณสามารถเปิดใช้งานอีกครั้งได้ทุกเมื่อในแท็บตั้งค่า NAMS (Disable ReShade Loading → ปิด)';

  @override
  String get notifyNaiomMigrated =>
      'พบการตั้งค่า NAIOM เดิมของคุณและนำเข้าไปยัง nams.toml แล้ว โปรดตรวจสอบแท็บ NAIOM คุณสามารถลบไฟล์ NAIOM เก่า (dinput8.dll, NAIOM.ini) ออกจากโฟลเดอร์เกมได้';

  @override
  String notifyNaiomSkipped(String entries) {
    return 'ปุ่มบางรายการของ NAIOM ใช้คีย์ที่ NAMS ไม่รองรับ จึงไม่ได้นำเข้า: $entries โปรดตั้งปุ่มใหม่ในแท็บ NAIOM';
  }

  @override
  String notifyPlatformUnsupported(String platform) {
    return 'ไม่พบชั้นความเข้ากันได้ของ Windows บน $platform จึงไม่สามารถเริ่มเกมจากที่นี่ได้ อย่างไรก็ตาม ม็อด เท็กซ์เจอร์ และการตั้งค่าทั้งหมดยังใช้งานได้ โปรดติดตั้ง CrossOver และติดตั้ง NieR:Automata ใน bottle เพื่อเปิดใช้งานการเริ่มเกม';
  }

  @override
  String get notifyReShadeIncompatible =>
      'ตรวจพบ ReShade ที่รองรับ addon/ImGui ซึ่งไม่เข้ากัน โปรดใช้ ReShade มาตรฐานที่ไม่รองรับ addon';

  @override
  String notifyTexturesDetected(String folder) {
    return 'พบเท็กซ์เจอร์ HD ใน $folder และจะโหลดเมื่อเริ่มเกม';
  }

  @override
  String get errorAppDataNotFound => 'ไม่พบตัวแปรสภาพแวดล้อม APPDATA';

  @override
  String get errorWinePrefixNotFound =>
      'ไม่พบ Wine prefix โปรดตั้งค่าตัวแปรสภาพแวดล้อม WINEPREFIX';

  @override
  String get errorHomeNotFound => 'ไม่พบตัวแปรสภาพแวดล้อม HOME';

  @override
  String get errorNoWineUser => 'ไม่พบไดเรกทอรีผู้ใช้ใน Wine prefix';

  @override
  String errorWineUsersNotFound(String prefix) {
    return 'ไม่พบไดเรกทอรี drive_c/users ของ Wine ใน $prefix';
  }

  @override
  String errorPlatformNotSupported(String os) {
    return 'ไม่รองรับแพลตฟอร์ม $os';
  }

  @override
  String errorExeNotFound(String dir) {
    return 'ไม่พบ NieRAutomata.exe ใน $dir';
  }

  @override
  String get errorDirNotWritable =>
      'โฟลเดอร์ตัวเรียกใช้งานเป็นแบบอ่านอย่างเดียว';

  @override
  String errorDirNotWritableBody(String dir) {
    return 'ไม่สามารถเขียนลงในโฟลเดอร์ YP Launcher ได้:\n$dir\n\nNAMS เขียนแคชรันไทม์ ปลั๊กอิน และบันทึกไว้ข้าง NAMS.exe ดังนั้นโฟลเดอร์ตัวเรียกใช้งานต้องสามารถเขียนได้\n\nวิธีแก้ไข:\n1. ปิดตัวเรียกใช้งาน\n2. ย้ายโฟลเดอร์ YP Launcher ที่แตกไฟล์แล้วทั้งหมดออกจาก Program Files (หรือที่ตั้งที่มีการป้องกัน) ไปยังโฟลเดอร์ปกติที่คุณเป็นเจ้าของ เช่น Desktop, Documents หรือ D:\\Games\\YP Launcher\n3. เปิดตัวเรียกใช้งานอีกครั้งจากตำแหน่งใหม่\n\nทางเลือก: คลิกขวาที่ไฟล์ .exe ของตัวเรียกใช้งานแล้วเลือก \"Run as administrator\" เพื่ออนุญาตให้เขียนในตำแหน่งปัจจุบัน';
  }

  @override
  String get errorGameDirNotWritable => 'โฟลเดอร์เกมเป็นแบบอ่านอย่างเดียว';

  @override
  String errorGameDirNotWritableBody(String gameDir, String namsDir) {
    return 'ไม่สามารถเขียนลงในโฟลเดอร์เกมได้:\n$gameDir\n\nNAMS เขียนม็อดและการตั้งค่าลงใน:\n$namsDir\nแต่ไม่สามารถสร้างไฟล์ที่นั่นได้ NieR อาจติดตั้งอยู่ใต้ C:\\Program Files (x86)\\Steam ซึ่ง Windows ป้องกันไว้\n\nวิธีแก้ไข (แนะนำ - ย้ายคลัง Steam ออกจาก Program Files):\n1. เปิด Steam > Settings > Storage\n2. คลิกเมนูไดรฟ์ > \"Add Drive\" แล้วเลือกโฟลเดอร์บนไดรฟ์อื่น เช่น D:\\SteamLibrary\n3. ไปที่ Library คลิกขวา NieR:Automata > Properties > Installed Files > \"Move install folder\" แล้วค่อยย้ายไปยังคลังใหม่\n4. เลือกเกมใหม่ในตัวเรียกใช้งานนี้แล้วกด Play อีกครั้ง\n\nทางเลือกแบบรวดเร็ว: คลิกขวาที่ไฟล์ .exe ของตัวเรียกใช้งานแล้วเลือก \"Run as administrator\" เพื่อให้สามารถเขียนใน Program Files ได้ การย้ายคลังเป็นวิธีแก้ระยะยาวที่สะอาดกว่า';
  }

  @override
  String get errorNoCompatLayer => 'ไม่พบ CrossOver';

  @override
  String get errorNoCompatLayerBody =>
      'การใช้งาน NieR:Automata บนระบบนี้ต้องใช้ CrossOver ซึ่งใช้รันโปรแกรม Windows บน macOS แต่ไม่พบใน /Applications\n\nหากไม่มี CrossOver ตัวเรียกใช้งานยังสามารถจัดการม็อด เท็กซ์เจอร์ และการตั้งค่าได้ เพียงแต่ไม่สามารถเริ่มเกมได้\n\nวิธีแก้ไข:\n1. ติดตั้ง CrossOver จาก codeweavers.com\n2. ติดตั้ง Steam และ NieR:Automata ภายใน CrossOver bottle\n3. เลือก NieRAutomata.exe จากภายใน bottle นั้นในตัวเรียกใช้งานนี้';

  @override
  String get errorNoCompatLayerLinux => 'ไม่พบ Proton หรือ Wine';

  @override
  String get errorNoCompatLayerLinuxBody =>
      'การใช้งาน NieR:Automata บน Linux ต้องใช้ Proton (แนะนำ) หรือ Wine แต่ไม่พบทั้งสองอย่าง\n\nหากไม่มี ตัวเรียกใช้งานยังสามารถจัดการม็อด เท็กซ์เจอร์ และการตั้งค่าได้ เพียงแต่ไม่สามารถเริ่มเกมได้\n\nวิธีแก้ไข:\n1. ใน Steam ติดตั้ง Proton เวอร์ชันหนึ่ง (Proton Experimental ใช้งานได้ดี) หากอยู่บนไดรฟ์อื่น ตัวเรียกใช้งานจะตรวจสอบทุกคลัง Steam\n2. ตรวจสอบว่าเลือก NieRAutomata.exe จากภายในคลัง Steam ของคุณ (พาธที่มี steamapps/common)\n3. หรือกำหนด YP_PROTON_PATH ไปยังไบนารี proton ก่อนเปิดตัวเรียกใช้งาน เช่น YP_PROTON_PATH=\"\$HOME/.steam/steam/steamapps/common/Proton - Experimental/proton\"';

  @override
  String get errorProtonMissing => 'ไม่พบ Proton';

  @override
  String errorProtonMissingBody(String path) {
    return 'ไม่พบรันไทม์ Proton ที่กำหนดค่าไว้ที่:\n$path\n\nโปรดติดตั้ง Proton ใหม่ผ่าน Steam หรือตั้งค่า YP_PROTON_PATH ไปยังไบนารี proton ที่ใช้งานได้ก่อนเปิดตัวเรียกใช้งาน';
  }

  @override
  String get errorNoZDrive => 'Wine prefix ไม่มีไดรฟ์ Z:';

  @override
  String errorNoZDriveBody(String prefix) {
    return 'Wine แมป Z: ไปยังระบบไฟล์ของเครื่องโฮสต์ ซึ่งเป็นวิธีที่ตัวเรียกใช้งานส่ง NAMS.exe ให้เกม แต่ prefix นี้ไม่มี dosdevices/z::\n$prefix\n\nนี่เป็นค่าเริ่มต้นของ CrossOver ดังนั้น bottle อาจถูกแก้ไข การสร้าง bottle ใหม่และติดตั้งเกมใหม่ภายในนั้นเป็นวิธีแก้ที่รวดเร็วที่สุด';
  }

  @override
  String get errorExeOutsidePrefix =>
      'ไฟล์ปฏิบัติการนั้นไม่ได้อยู่ภายใน bottle';

  @override
  String errorExeOutsidePrefixBody(String exeName, String path) {
    return 'ตัวเรียกใช้งานเริ่มเกมผ่าน CrossOver ดังนั้น $exeName ต้องอยู่ภายใน CrossOver bottle:\n$path\n\nติดตั้งเกมลงใน bottle แล้วเลือกไฟล์ปฏิบัติการจากที่นั่น';
  }

  @override
  String get headerNams => 'NAMS';

  @override
  String get headerLodMod => 'LOD MOD';

  @override
  String get headerTextures => 'เท็กซ์เจอร์';

  @override
  String get headerYorhaProtocol => 'YORHA PROTOCOL';

  @override
  String get headerNaiom => 'NAIOM';

  @override
  String get headerCutscenes => 'คัตซีน';

  @override
  String get tooltipEditsNamsToml => 'แก้ไข nams/nams.toml';

  @override
  String get tooltipEditsLodmodToml => 'แก้ไข nams/lodmod.toml';

  @override
  String get tooltipEditsTextureInjectionToml =>
      'แก้ไข nams/texture_injection.toml';

  @override
  String get tooltipEditsSettingsJson => 'แก้ไข %APPDATA%\\NAMS\\settings.json';

  @override
  String get tooltipEditsNaiom =>
      'แก้ไขการตั้งค่า [mouse] ใน nams/nams.toml การตั้งค่าส่วนใหญ่จะมีผลหลังบันทึก แต่บางรายการต้องรีสตาร์ตเกม';

  @override
  String get tooltipCutscenesLocation =>
      'ม็อด: nams/cutscenes/<mod_name>/movie/*.usm';

  @override
  String get cardGeneral => 'ทั่วไป';

  @override
  String get cardLoading => 'การโหลด';

  @override
  String get cardHeapOverrides => 'การแทนค่าฮีป';

  @override
  String get cardLevelOfDetail => 'ระดับรายละเอียด';

  @override
  String get cardAmbientOcclusion => 'Ambient Occlusion';

  @override
  String get cardPostProcessing => 'โพสต์โปรเซสซิง';

  @override
  String get cardShadows => 'เงา';

  @override
  String get cardGlobalIllumination => 'Global Illumination';

  @override
  String get cardPreloading => 'การโหลดล่วงหน้า';

  @override
  String get cardTextureConfig => 'การตั้งค่า';

  @override
  String get cardInstallTextures => 'ติดตั้งเท็กซ์เจอร์';

  @override
  String get cardHowItWorks => 'วิธีการทำงาน';

  @override
  String get cardKeybinds => 'การกำหนดปุ่ม';

  @override
  String get cardWorkspace => 'เวิร์กสเปซ';

  @override
  String get cardCheats => 'สูตรโกง';

  @override
  String get cardRandomizer => 'สุ่มศัตรู';

  @override
  String get cardThirdPersonCamera => 'กล้องบุคคลที่สาม';

  @override
  String get cardPodAiming => 'Pod / การเล็ง';

  @override
  String get cardMisc => 'เบ็ดเตล็ด';

  @override
  String get cardMovementBindings => 'ปุ่มการเคลื่อนไหว';

  @override
  String get cardCombatBindings => 'ปุ่มการต่อสู้';

  @override
  String get cardNonStandardBindings => 'ปุ่มที่ไม่ใช่มาตรฐาน';

  @override
  String get cardMenuNavigation => 'การนำทางเมนู';

  @override
  String get cardStructure => 'โครงสร้าง';

  @override
  String get buttonSave => 'บันทึก';

  @override
  String get buttonDiscard => 'ละทิ้ง';

  @override
  String get buttonCancel => 'ยกเลิก';

  @override
  String get buttonInstall => 'ติดตั้ง';

  @override
  String get buttonDelete => 'ลบ';

  @override
  String get buttonYes => 'ใช่';

  @override
  String get buttonNo => 'ไม่';

  @override
  String get buttonContinue => 'ดำเนินการต่อ';

  @override
  String get buttonBack => 'ย้อนกลับ';

  @override
  String get buttonGetStarted => 'เริ่มต้นใช้งาน';

  @override
  String get buttonAutoDetect => 'ตรวจหาอัตโนมัติ';

  @override
  String get buttonSelectManually => 'เลือกด้วยตนเอง';

  @override
  String get buttonGoToLauncher => 'ไปยังตัวเรียกใช้งาน';

  @override
  String get unsavedChangesTitle => 'การเปลี่ยนแปลงที่ยังไม่ได้บันทึก';

  @override
  String get unsavedChangesMessage =>
      'คุณมีการเปลี่ยนแปลงที่ยังไม่ได้บันทึก ต้องการละทิ้งหรือไม่?';

  @override
  String get stay => 'อยู่ต่อ';

  @override
  String get discard => 'ละทิ้ง';

  @override
  String get enterValidNumber => 'กรอกตัวเลขที่ถูกต้อง';

  @override
  String get pressKey => 'กดปุ่ม...';

  @override
  String get tabLauncher => 'ตัวเรียกใช้งาน';

  @override
  String get tabYorhaProtocol => 'YP Devkit';

  @override
  String get tabNams => 'NAMS';

  @override
  String get tabLodMod => 'LOD Mod';

  @override
  String get tabNaiom => 'NAIOM';

  @override
  String get tabTextures => 'เท็กซ์เจอร์';

  @override
  String get tabMods => 'ตัวจัดการม็อด';

  @override
  String get tabCutscenes => 'คัตซีน';

  @override
  String get tabThirdParty => 'บุคคลที่สาม';

  @override
  String get thirdPartyTitle => 'รันไทม์ของบุคคลที่สาม';

  @override
  String get thirdPartySubtitle =>
      'ReShade และ 3DMigoto ซึ่ง NAMS โหลดจากโฟลเดอร์ที่จัดการไว้ ไม่ต้องเปลี่ยนชื่อและไม่มี DLL ชนกัน';

  @override
  String get thirdPartyReShadeHeader => 'RESHADE';

  @override
  String get thirdPartyMigotoHeader => '3DMIGOTO';

  @override
  String get thirdPartyReShadeHowto =>
      'ติดตั้ง ReShade ตามปกติลงในโฟลเดอร์ NieRAutomata (เลือก dxgi เป็น API) ตัวเรียกใช้งานจะย้ายไฟล์เข้า NAMS และตั้งค่าพาธให้คุณ';

  @override
  String get thirdPartyMigotoHowto =>
      'วางไฟล์เก็บถาวรม็อด shader ของ 3DMigoto ที่นี่ ตัวเรียกใช้งานจะติดตั้งและตั้งค่าเป้าหมายตัวโหลดเพื่อให้ NAMS hook ได้';

  @override
  String get thirdPartyStatusInstalled => 'ติดตั้งแล้ว';

  @override
  String get thirdPartyStatusNotInstalled => 'ยังไม่ได้ติดตั้ง';

  @override
  String get thirdPartyStatusFoundInGame =>
      'พบในโฟลเดอร์เกมของคุณ คลิก นำเข้า เพื่อตั้งค่าให้ใช้กับ NAMS';

  @override
  String get thirdPartyEnable => 'เปิดใช้งาน';

  @override
  String get thirdPartyImport => 'นำเข้าไปยัง NAMS';

  @override
  String get thirdPartyRepair => 'ซ่อมแซม';

  @override
  String get thirdPartyRemove => 'นำออก';

  @override
  String get thirdPartyGetReShade => 'ดาวน์โหลด ReShade';

  @override
  String get thirdPartyShadersMissing =>
      'ยังไม่มีเอฟเฟกต์ติดตั้งอยู่ โปรดเพิ่มพรีเซ็ตหรือชุด shader มิฉะนั้นจะไม่มีผลใด ๆ';

  @override
  String get thirdPartyOpenFolder => 'เปิดโฟลเดอร์';

  @override
  String thirdPartyPresetsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count พรีเซ็ต',
      one: '1 พรีเซ็ต',
      zero: 'ไม่มีพรีเซ็ต',
    );
    return '$_temp0';
  }

  @override
  String get thirdPartyInstallCard => 'ติดตั้ง';

  @override
  String get thirdPartyDropHere => 'วางพรีเซ็ต ReShade / ม็อด 3DMigoto ที่นี่';

  @override
  String get thirdPartyImported => 'นำเข้าไปยัง NAMS แล้ว';

  @override
  String get thirdPartyInstalled => 'ติดตั้งแล้ว';

  @override
  String get thirdPartyInstallFailed => 'ไม่สามารถติดตั้งไฟล์นี้ได้';

  @override
  String get thirdPartyRedirectMods =>
      'นี่เป็นม็อดข้อมูลเกม โปรดใช้แท็บตัวจัดการม็อด';

  @override
  String get thirdPartyRedirectTextures =>
      'นี่เป็นแพ็กเท็กซ์เจอร์ โปรดใช้แท็บเท็กซ์เจอร์';

  @override
  String get thirdPartyUnsupported => 'ไม่รองรับไฟล์ประเภทนี้ที่นี่';

  @override
  String get thirdPartyLodModPrompt =>
      'นี่คือ LodMod โดย NAMS มี LodMod ในตัวอยู่แล้ว ต้องการนำเข้าการตั้งค่าไปยังแท็บ LodMod หรือไม่?';

  @override
  String get thirdPartyStatusActive => 'ทำงานอยู่';

  @override
  String get thirdPartyStatusInactive => 'ติดตั้งแล้ว แต่ปิดใช้งานในแท็บ NAMS';

  @override
  String get thirdPartyVersion => 'เวอร์ชัน';

  @override
  String get thirdPartyAddonBuild => 'เวอร์ชัน Addon';

  @override
  String get thirdPartyPresets => 'พรีเซ็ต';

  @override
  String get thirdPartyNonePresets => 'ยังไม่มีพรีเซ็ตติดตั้งอยู่';

  @override
  String get thirdPartyShaderRepos => 'SHADERS';

  @override
  String get thirdPartyNoneShaders => 'ไม่มี shader เอฟเฟกต์จะคอมไพล์ไม่ได้';

  @override
  String thirdPartyShaderCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count เอฟเฟกต์',
      one: '1 เอฟเฟกต์',
    );
    return '$_temp0';
  }

  @override
  String get thirdPartyAddons => 'ADDONS';

  @override
  String get thirdPartyFiles => 'ไฟล์';

  @override
  String get thirdPartyShaderFixes => 'ShaderFixes';

  @override
  String thirdPartyShaderFixCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count รายการแก้ไข',
      one: '1 รายการแก้ไข',
      zero: 'ไม่มี',
    );
    return '$_temp0';
  }

  @override
  String get thirdPartyLoaderTarget => 'เป้าหมายตัวโหลด';

  @override
  String get thirdPartyHunting => 'โหมดค้นหา shader';

  @override
  String get thirdPartyHuntingOn => 'เปิด';

  @override
  String thirdPartyUpdateTitle(String name) {
    return 'แทนที่ $name ที่ติดตั้งอยู่หรือไม่?';
  }

  @override
  String thirdPartyUpdateBody(String name) {
    return 'ติดตั้ง $name อยู่แล้ว แต่ไฟล์ที่คุณวางเป็นบิลด์คนละตัว ต้องการแทนที่ตัวที่ติดตั้งอยู่ด้วยไฟล์นี้หรือไม่?';
  }

  @override
  String get thirdPartyUpdateInstalledLabel => 'ติดตั้งอยู่ตอนนี้';

  @override
  String get thirdPartyUpdateIncomingLabel => 'ไฟล์ที่คุณวาง';

  @override
  String get thirdPartyUpdateReplace => 'แทนที่';

  @override
  String get thirdPartyUpdateKeep => 'เก็บเวอร์ชันปัจจุบัน';

  @override
  String get thirdPartyUpdateSkipped => 'เก็บเวอร์ชันที่ติดตั้งไว้แล้ว';

  @override
  String get thirdPartyD3dCompilerMissing =>
      'ไม่พบ d3dcompiler_47.dll ที่นี่ บน Wine, ReShade จะถอยไปใช้ stub ของ Wine และเอฟเฟกต์จะคอมไพล์ไม่ได้ โปรดติดตั้ง ReShade เวอร์ชันที่มีไฟล์นี้ใหม่ หรือวาง d3dcompiler_47.dll แบบเนทีฟลงในโฟลเดอร์นี้';

  @override
  String get thirdPartyBanner =>
      'เครื่องมือในแท็บนี้เป็นซอฟต์แวร์จากบุคคลที่สาม NAMS เพียงแค่โหลดมันเท่านั้น ไม่ได้เป็นผู้พัฒนาและไม่สามารถเปลี่ยนพฤติกรรมของมันได้ สิ่งที่คุณติดตั้งผ่านเครื่องมือเหล่านี้จะทำงานด้วยตัวเอง ดังนั้นความถูกต้อง ความเข้ากันได้ และความปลอดภัยเป็นความรับผิดชอบของคุณ หากมีสิ่งใดทำงานผิดปกติ นั่นเป็นเพราะเครื่องมือหรือม็อด ไม่ใช่ NAMS';

  @override
  String get thirdPartyConfigSection => 'การตั้งค่า';

  @override
  String get thirdPartyRestartRequired => 'จะมีผลเมื่อเริ่มเกมครั้งถัดไป';

  @override
  String get thirdPartyReShadePerformanceMode => 'โหมดประสิทธิภาพ';

  @override
  String get thirdPartyReShadePerformanceModeHint =>
      'ข้ามการคอมไพล์เอฟเฟกต์ใหม่ตอนเริ่มเกม ปิดตัวเลือกนี้ระหว่างปรับแต่งเอฟเฟกต์';

  @override
  String get thirdPartyReShadeShowFps => 'แสดง FPS';

  @override
  String get thirdPartyReShadeShowFpsHint =>
      'แสดงอัตราเฟรมที่มุมของโอเวอร์เลย์ ReShade';

  @override
  String get thirdPartyReShadeShowClock => 'แสดงนาฬิกา';

  @override
  String get thirdPartyReShadeShowClockHint =>
      'แสดงเวลาปัจจุบันที่มุมของโอเวอร์เลย์ ReShade';

  @override
  String get thirdPartyReShadeActivePreset => 'พรีเซ็ตที่ใช้งานอยู่';

  @override
  String get thirdPartyReShadeOverlayKey => 'ปุ่มเปิดโอเวอร์เลย์';

  @override
  String get thirdPartyReShadeNoKey => 'ยังไม่ได้ตั้งค่า';

  @override
  String get thirdPartyMigotoHunting => 'โหมดค้นหา shader';

  @override
  String get thirdPartyMigotoHuntingOff => 'ปิด';

  @override
  String get thirdPartyMigotoHuntingOn => 'เปิด';

  @override
  String get thirdPartyMigotoHuntingNoMarking => 'เปิดโดยไม่ทำเครื่องหมาย';

  @override
  String get thirdPartyMigotoHuntingHint =>
      'โหมดนักพัฒนา: วนดู shader เพื่อค้นหาและ dump ไฟล์ ควรปิดไว้สำหรับการเล่นปกติ';

  @override
  String get thirdPartyMigotoMarkingMode => 'โหมดทำเครื่องหมาย';

  @override
  String get thirdPartyMigotoMarkingModeHint =>
      'วิธีไฮไลต์ shader ที่กำลังค้นหาในเกม: ข้าม (ไม่เปลี่ยนแปลง), ต้นฉบับ, สีชมพู หรือขาวดำ';

  @override
  String get thirdPartyMigotoMarkingSkip => 'ข้าม';

  @override
  String get thirdPartyMigotoMarkingOriginal => 'ต้นฉบับ';

  @override
  String get thirdPartyMigotoMarkingPink => 'สีชมพู';

  @override
  String get thirdPartyMigotoMarkingMono => 'ขาวดำ';

  @override
  String get thirdPartyMigotoVerboseOverlay => 'โอเวอร์เลย์แบบละเอียด';

  @override
  String get thirdPartyMigotoVerboseOverlayHint =>
      'แสดงข้อความสถานะ 3DMigoto แบบละเอียดบนหน้าจอ มีประโยชน์ระหว่างดีบัก แต่รบกวนสายตาในการใช้งานปกติ';

  @override
  String get thirdPartyMigotoCacheShaders => 'แคช shader';

  @override
  String get thirdPartyMigotoCacheShadersHint =>
      'นำ shader ที่คอมไพล์แล้วกลับมาใช้ในการเริ่มเกมครั้งถัดไป ปิดเฉพาะเมื่อต้องดีบักการแก้ไข';

  @override
  String get thirdPartyMigotoCheckForegroundWindow =>
      'hook เฉพาะเมื่ออยู่เบื้องหน้า';

  @override
  String get thirdPartyMigotoCheckForegroundWindowHint =>
      'ใช้งานโอเวอร์เลย์และปุ่มลัดเฉพาะเมื่อหน้าต่างเกมอยู่ในโฟกัส';

  @override
  String get thirdPartyConfigApplied => 'บันทึกการตั้งค่าแล้ว';

  @override
  String get thirdPartyConfigNoIni =>
      'ยังไม่มีไฟล์การตั้งค่า การติดตั้งจะสร้างให้';

  @override
  String get tabSectionGeneral => 'ทั่วไป';

  @override
  String get tabSectionConfig => 'การตั้งค่า';

  @override
  String get tabSectionMods => 'ม็อด';

  @override
  String get tabDividerConfig => 'การตั้งค่า';

  @override
  String get tabDividerMods => 'ม็อด';

  @override
  String get infoBarVersionPrefix => 'YP Launcher ';

  @override
  String get infoBarLogs => 'บันทึก';

  @override
  String get infoBarFaq => 'คำถามที่พบบ่อย';

  @override
  String get infoBarWhatsNew => 'มีอะไรใหม่';

  @override
  String get infoBarShortcut => 'ทางลัด';

  @override
  String get tooltipFaq => 'ยังต้องใช้ม็อดอื่นอีกหรือไม่?';

  @override
  String get chipLodModOn => 'เปิด LOD MOD';

  @override
  String get chipLodModOff => 'ปิด LOD MOD';

  @override
  String get chipReShade => 'ReShade';

  @override
  String get chipNoTextures => 'ไม่มีเท็กซ์เจอร์';

  @override
  String get chipNoMods => 'ไม่มีม็อด';

  @override
  String get chipNoCutsceneMod => 'ไม่มีม็อดคัตซีน';

  @override
  String chipTexturesCount(String details) {
    return 'เท็กซ์เจอร์ ($details)';
  }

  @override
  String chipModsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'ม็อด',
      one: 'ม็อด',
    );
    return '$count $_temp0';
  }

  @override
  String chipInjectedCount(int count) {
    return 'ฉีดแล้ว $count รายการ';
  }

  @override
  String get chipSkRes => 'SK_Res';

  @override
  String chipCutsceneMod(int width, int height, String codec) {
    return 'ม็อดคัตซีน (${width}x$height $codec)';
  }

  @override
  String get warningPluginLoadingDisabled =>
      'ปิดการโหลดปลั๊กอินอยู่ เวิร์กสเปซ YoRHa Protocol จะไม่โหลด';

  @override
  String get warningReShadeDisabled => 'ปิดการโหลด ReShade อัตโนมัติอยู่';

  @override
  String get warningTextureInjectionDisabled => 'ปิดการฉีดเท็กซ์เจอร์อยู่';

  @override
  String get heapOverridesDescription =>
      'เพิ่มหน่วยความจำเหนือค่าเริ่มต้นของ NAMS ฮีปหลักจะขยายอัตโนมัติ เพิ่มเฉพาะเมื่อม็อดต้องการหน่วยความจำมากขึ้น ต้องรีสตาร์ตเกม';

  @override
  String get lodModDescription =>
      'แพตช์คุณภาพภาพที่รวมอยู่ใน NAMS โดยได้รับแรงบันดาลใจจาก Automata-LodMod ของ emoose ช่วยลดอาการ LOD โผล่ เพิ่มความคมของเงาและ Ambient Occlusion บังคับให้วัตถุทุกชนิดรวมถึงพืชสร้างเงา ปิดการ culling แบบกำหนดเองเพื่อไม่ให้วัตถุโผล่เข้าออก และลบ vignette';

  @override
  String get dropModelModHere => 'วางโฟลเดอร์หรือไฟล์เก็บถาวรม็อดโมเดลที่นี่';

  @override
  String get dropToInstall => 'วางเพื่อทำการติดตั้ง';

  @override
  String get orClickToBrowse => 'หรือคลิกเพื่อเลือกไฟล์';

  @override
  String get mixedModDetected => 'ตรวจพบม็อดแบบผสม';

  @override
  String get noOutfitsInstalled => 'ยังไม่ได้ติดตั้งชุดใด ๆ';

  @override
  String get defaultNoMod => 'ค่าเริ่มต้น (ไม่มีม็อด)';

  @override
  String get clearAllStartupOutfits => 'ล้างชุดเริ่มต้นทั้งหมด';

  @override
  String get removeOutfitTitle => 'นำชุดออกหรือไม่?';

  @override
  String get noModelFoundError =>
      'ไม่พบโมเดล ต้องมี pl000d.dat/.dtt (2B), pl010d.dat/.dtt (A2) หรือ pl020d.dat/.dtt (9S)';

  @override
  String get unsupportedArchiveFormat => 'รูปแบบไฟล์เก็บถาวรไม่รองรับ';

  @override
  String get extractingArchive => 'กำลังแตกไฟล์เก็บถาวร...';

  @override
  String extractingArchivePercent(int percent) {
    return 'กำลังแตกไฟล์ - $percent%';
  }

  @override
  String extractingArchivePercentFile(int percent, String file) {
    return 'กำลังแตกไฟล์ $percent% - $file';
  }

  @override
  String get cutsceneScanningArchive =>
      'กำลังสแกนไฟล์เก็บถาวรหาโฟลเดอร์ movie...';

  @override
  String cutsceneCopyingFile(int current, int total, String name) {
    return 'กำลังคัดลอก $current/$total - $name';
  }

  @override
  String cutsceneCopyingFileBytes(
    int current,
    int total,
    String name,
    String mbDone,
    String mbTotal,
  ) {
    return 'กำลังคัดลอก $current/$total - $name ($mbDone / $mbTotal MB)';
  }

  @override
  String get failedToExtractArchive => 'แตกไฟล์เก็บถาวรไม่สำเร็จ';

  @override
  String get extractFailedDiskFull =>
      'แตกไฟล์ไม่สำเร็จ: พื้นที่ว่างบนไดรฟ์ชั่วคราวไม่เพียงพอ โปรดเพิ่มพื้นที่ว่างแล้วลองใหม่';

  @override
  String get textureDeleteConfirmTitle => 'ลบแพ็กเท็กซ์เจอร์หรือไม่?';

  @override
  String textureDeleteConfirmBody(String name) {
    return 'ลบ $name ออกจาก nams/inject/textures/ อย่างถาวรหรือไม่? ไม่สามารถย้อนกลับได้';
  }

  @override
  String get textureDeleteLabel => 'ลบ';

  @override
  String get busyDeletingTexture => 'กำลังลบแพ็กเท็กซ์เจอร์...';

  @override
  String get busyDeletingCutscene => 'กำลังลบม็อดคัตซีน...';

  @override
  String get busyCloseTitle => 'กำลังดำเนินการ';

  @override
  String get busyCloseBody =>
      'ตัวเรียกใช้งานกำลังติดตั้ง ลบ หรือแตกไฟล์อยู่ การปิดตอนนี้อาจทำให้มีไฟล์เสียหรือไม่ครบอยู่บนดิสก์ โปรดรอให้การดำเนินการปัจจุบันเสร็จ หรือบังคับปิดต่อไป';

  @override
  String get busyCloseForce => 'บังคับปิด';

  @override
  String extractFailedDetail(String detail) {
    return 'แตกไฟล์ไม่สำเร็จ: $detail';
  }

  @override
  String get noOutfitsInstalledNotif => 'ยังไม่ได้ติดตั้งชุด';

  @override
  String get dialogSelectModFolder => 'เลือกโฟลเดอร์ม็อดโมเดล';

  @override
  String get dialogNameOutfitShown => 'ชื่อนี้จะแสดงในเกมเมื่อสลับชุด';

  @override
  String get dropTextureHere => 'วางโฟลเดอร์หรือไฟล์เก็บถาวรเท็กซ์เจอร์ที่นี่';

  @override
  String get installedToTextures => 'ติดตั้งไปยัง: nams/inject/textures/';

  @override
  String get installingTextures => 'กำลังติดตั้งเท็กซ์เจอร์...';

  @override
  String get textureDropAnalyzing => 'กำลังวิเคราะห์ไฟล์ที่วาง...';

  @override
  String get textureDropNoDds =>
      'ไม่พบเท็กซ์เจอร์ .dds ในไฟล์ที่วาง รองรับเฉพาะไฟล์ .dds';

  @override
  String get cutsceneDropAnalyzing => 'กำลังวิเคราะห์ไฟล์ที่วาง...';

  @override
  String get cutsceneDropNoUsm => 'ไม่พบไฟล์คัตซีน .usm ในไฟล์ที่วาง';

  @override
  String get modDropAnalyzing => 'กำลังวิเคราะห์ไฟล์ที่วาง...';

  @override
  String get modDropNotAMod =>
      'ดูเหมือนไม่ใช่ม็อด NAMS โปรดวางโฟลเดอร์/ไฟล์เก็บถาวรที่มี entities/, wax/, data/ หรือ mod.toml';

  @override
  String get modDropMisroutedCutscenes =>
      'ดูเหมือนเป็นม็อดคัตซีนแบบเดี่ยว โปรดวางในแท็บคัตซีนแทน คัตซีนที่รวมมากับม็อดควรอยู่ภายในม็อดที่มี entities/ หรือ mod.toml อยู่แล้ว';

  @override
  String modSideInstalledTextures(String names) {
    return 'ติดตั้งแพ็กเท็กซ์เจอร์ที่รวมมาด้วยไปยัง nams/inject/textures/ ด้วย: $names';
  }

  @override
  String modLooseUnpairedWarning(String names) {
    return 'ติดตั้งแล้ว แต่บางไฟล์ไม่มีคู่ไฟล์แบบวานิลลา (.dat/.dtt): $names ม็อดอาจทำงานไม่สมบูรณ์';
  }

  @override
  String get modBundledTexturesLabel => 'เท็กซ์เจอร์ที่รวมมาด้วย';

  @override
  String get modBundledCutscenesLabel => 'คัตซีนที่รวมมาด้วย';

  @override
  String textureBundledWithMod(String modId) {
    return 'รวมมากับม็อด: $modId';
  }

  @override
  String modUninstallAlsoTexturesLabel(String names) {
    return 'ลบแพ็กเท็กซ์เจอร์ที่รวมมาด้วยด้วย: $names';
  }

  @override
  String get noTextures => 'ไม่มีเท็กซ์เจอร์';

  @override
  String get noTexturesInstalled => 'ยังไม่ได้ติดตั้งเท็กซ์เจอร์';

  @override
  String get textureConflictHint =>
      'ม็อดทั้งสองถูกโหลดและแก้ไขบางสิ่งเดียวกัน ให้วางม็อดที่คุณให้ความสำคัญมากกว่าไว้ด้านบน';

  @override
  String get noConflictsFound => 'ไม่พบความขัดแย้ง ม็อดทั้งหมดโหลดแยกจากกัน';

  @override
  String get selectTextureFiles => 'เลือกไฟล์หรือไฟล์เก็บถาวรเท็กซ์เจอร์';

  @override
  String get noTextureFilesFound => 'ไม่พบไฟล์เท็กซ์เจอร์ (.dds, .png, .tga)';

  @override
  String get cheatsAppliedNote => 'นำไปใช้หนึ่งครั้งเมื่อเริ่มเกม';

  @override
  String get wipLabel => 'กำลังพัฒนา';

  @override
  String get dropCutsceneHere => 'วางโฟลเดอร์หรือไฟล์เก็บถาวรม็อดคัตซีนที่นี่';

  @override
  String get cutsceneMovieHint => 'ม็อดต้องมีโฟลเดอร์ movie/ ที่มีไฟล์ .usm';

  @override
  String get cutsceneNamingTitle => 'ตั้งชื่อม็อดคัตซีนนี้';

  @override
  String get removeCutsceneModTitle => 'นำม็อดคัตซีนออกหรือไม่?';

  @override
  String get noCutsceneModsInstalled => 'ยังไม่ได้ติดตั้งม็อดคัตซีน';

  @override
  String get cutsceneHowItWorks1 =>
      'คัตซีนแบบกำหนดเองโหลดจาก nams/cutscenes/ แทน data/movie/';

  @override
  String get cutsceneHowItWorks2 =>
      'หากไฟล์แบบกำหนดเองหายหรือเสียหาย ระบบจะเล่นไฟล์ต้นฉบับแทน';

  @override
  String get cutsceneHowItWorks3 =>
      'ไฟล์เกมต้นฉบับของคุณจะไม่ถูกแก้ไข ม็อดโหลดจากตำแหน่งแยกต่างหาก';

  @override
  String get cutsceneStructurePath =>
      'nams/cutscenes/<mod_name>/movie/<filename>.usm';

  @override
  String get cutsceneFolderNameLimit =>
      'ชื่อโฟลเดอร์จำกัดความยาวไม่เกิน 27 ตัวอักษร';

  @override
  String get cutsceneMigrationTitle =>
      'ตรวจพบไฟล์คัตซีนแบบกำหนดเองใน data/movie/';

  @override
  String cutsceneMigrationBody(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'ไฟล์',
      one: 'ไฟล์',
    );
    return 'พบไฟล์คัตซีนแบบกำหนดเอง $count $_temp0 อยู่โดยตรงใน data/movie/ ไฟล์เหล่านี้จะแทนที่ต้นฉบับอย่างถาวร ครั้งถัดไปให้ติดตั้งม็อดคัตซีนที่นี่แทน หากไฟล์แบบกำหนดเองโหลดไม่สำเร็จ ระบบจะเล่นไฟล์ต้นฉบับเป็นตัวสำรอง';
  }

  @override
  String get noMovieFolderFound => 'ไม่พบโฟลเดอร์ movie/ ที่มีไฟล์ .usm';

  @override
  String get noUsmFilesFound => 'ไม่พบไฟล์ .usm ในม็อด';

  @override
  String get onboardingWelcomeTitle => 'ยินดีต้อนรับสู่ YoRHa Protocol';

  @override
  String get onboardingWelcomeSubtitle =>
      'ตัวเรียกใช้งานเดียว ม็อดทั้งหมดของคุณ และเนื้อหา NieR แบบใหม่\n\nลากแล้ววางม็อด สลับชุดระหว่างเล่น เควสต์ใหม่ และ Devkit ในตัว โดยไม่ต้องเจอกับฝันร้ายในการตั้งค่า';

  @override
  String get onboardingSelectTitle => 'เลือกการติดตั้ง NieR:Automata ของคุณ';

  @override
  String get onboardingSearchingDrives => 'กำลังสแกนทุกไดรฟ์...';

  @override
  String get onboardingSearchingNier => 'กำลังค้นหา NieR:Automata...';

  @override
  String get onboardingSelectInstallation => 'เลือกการติดตั้ง';

  @override
  String get onboardingFirstPlaythroughTitle =>
      'นี่เป็นการเล่นครั้งแรกของคุณหรือไม่?';

  @override
  String get onboardingFirstPlaythroughHint => 'หากใช่ เราจะซ่อนเนื้อหาสปอยล์';

  @override
  String get onboardingFirstYes => 'ใช่ - ซ่อนฟีเจอร์ที่มีสปอยล์';

  @override
  String get onboardingFirstNo => 'ไม่ - แสดงทุกอย่าง';

  @override
  String get onboardingMigrationAskTitle => 'เคยม็อด NieR มาก่อนหรือไม่?';

  @override
  String get onboardingMigrationAskBody =>
      'เราจะตรวจพบการตั้งค่าเดิมของคุณโดยอัตโนมัติ';

  @override
  String get onboardingMigrationYes => 'ใช่';

  @override
  String get onboardingMigrationNo => 'ไม่';

  @override
  String get onboardingMigrationResultsTitle => 'สิ่งที่ตรวจพบ';

  @override
  String get onboardingMigrationResultsBody => '';

  @override
  String get onboardingMigrationNothingFound =>
      'ไม่พบอะไร การติดตั้งของคุณสะอาด';

  @override
  String get onboardingMigrationActionReshadeKept =>
      'ปิดใช้งานแล้ว - NAMS มี DoF แบบเนทีฟ เปิดอีกครั้งได้ในตั้งค่า NAMS หากต้องการ';

  @override
  String get onboardingMigrationActionReshadeIncompatible =>
      'เวอร์ชัน Addon/ImGui - โปรดแทนที่หรือนำออก';

  @override
  String get onboardingMigrationActionTextures => 'จะโหลดโดยอัตโนมัติ';

  @override
  String get onboardingMigrationActionLodMod => 'นำเข้า LodMod.ini แล้ว';

  @override
  String get onboardingMigrationActionSkRes => 'ตรวจพบโดยอัตโนมัติ';

  @override
  String get onboardingMigrationActionNaiom =>
      'การตั้งค่า NAIOM ของคุณจะถูกนำเข้าไปยัง NAMS โดยอัตโนมัติ หลังจากนั้นคุณสามารถลบไฟล์ NAIOM เดิม (dinput8.dll, NAIOM.ini) ได้';

  @override
  String get onboardingMigrationActionCutscenes => 'รวมอยู่ในระบบแล้ว';

  @override
  String get onboardingMigrationActionExistingMods =>
      'ติดตั้งอยู่แล้ว - เก็บไว้ตามเดิม';

  @override
  String onboardingMigrationLabelExistingMods(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count ม็อดใน nams/mods/',
      one: '1 ม็อดใน nams/mods/',
    );
    return '$_temp0';
  }

  @override
  String get onboardingModInstallAskTitle =>
      'ต้องการติดตั้งม็อดก่อนเริ่มเกมหรือไม่?';

  @override
  String get onboardingModInstallAskBody =>
      'มีไฟล์ .zip หรือโฟลเดอร์หรือไม่? เราจัดการให้ได้\n(เท็กซ์เจอร์และคัตซีนมีแท็บแยกของตัวเอง สามารถติดตั้งภายหลังได้)';

  @override
  String get onboardingModInstallYes => 'ใช่ ติดตั้งม็อดตอนนี้';

  @override
  String get onboardingModInstallSkip => 'ไม่ ข้ามไปก่อน';

  @override
  String get onboardingModInstallTitle => 'เพิ่มม็อดแรกของคุณ';

  @override
  String get onboardingModInstallBody => 'รองรับ .zip, .7z และโฟลเดอร์';

  @override
  String get onboardingModInstallSubBody => 'ไฟล์เกมยังคงปลอดภัย';

  @override
  String get onboardingModInstallBusy => 'กำลังติดตั้ง…';

  @override
  String get onboardingModInstallInspecting => 'กำลังตรวจสอบม็อด…';

  @override
  String onboardingModInstallExtractingPercent(int percent) {
    return 'กำลังแตกไฟล์… $percent%';
  }

  @override
  String onboardingModInstallExtractingFile(int percent, String file) {
    return 'กำลังแตกไฟล์ $percent% - $file';
  }

  @override
  String get onboardingModInstallNotAMod =>
      'ดูเหมือนไม่ใช่ม็อด NAMS ที่ถูกต้อง โปรดตรวจสอบว่าเป็นไฟล์ .zip / .7z ที่มี mod.toml หรือโครงสร้างข้อมูลที่รองรับ';

  @override
  String get onboardingOutfitHintHeader => 'วิธีสวมใส่';

  @override
  String get onboardingOutfitHintCompat => 'ซื้อหรือสวมใส่จากช่องเก็บของของคุณ';

  @override
  String get onboardingOutfitHintData =>
      'กด F1 ในเกม → ไอคอนตู้เสื้อผ้า (มุมซ้ายบน)';

  @override
  String get onboardingOutfitHintMultiOutfit =>
      'หรือใช้ Multi-Outfit NPC ที่ค่าย Resistance Camp';

  @override
  String get onboardingOutfitHintMultiOutfitLink =>
      'ดาวน์โหลด Multi-Outfit NPC';

  @override
  String get onboardingOutfitHintMultiOutfitUrl =>
      'https://www.nexusmods.com/nierautomata/mods/789';

  @override
  String onboardingModInstallFailed(String reason) {
    return 'ติดตั้งไม่สำเร็จ: $reason';
  }

  @override
  String get onboardingModInstallNoGameDir =>
      'ยังไม่ได้เลือกไดเรกทอรีเกม โปรดย้อนกลับและเลือกการติดตั้งก่อน';

  @override
  String get onboardingModInstallInstalledHeader => 'ติดตั้งแล้ว:';

  @override
  String get onboardingModInstallSkipButton => 'ข้าม';

  @override
  String get onboardingModInstallDoneButton => 'เสร็จสิ้น';

  @override
  String get onboardingModInstallPickerTitle =>
      'เลือกม็อด (.zip, .7z) หรือเปิดโฟลเดอร์';

  @override
  String get onboardingModInstallFolderPickerTitle => 'เลือกโฟลเดอร์ม็อด';

  @override
  String get onboardingReadyTitle => 'พร้อมใช้งานแล้ว!';

  @override
  String get onboardingCreateShortcut => 'สร้างทางลัดบนเดสก์ท็อป';

  @override
  String get onboardingFirstPlaythroughSpoilerFree =>
      'การเล่นครั้งแรก (ไม่มีสปอยล์)';

  @override
  String get onboardingFullAccess => 'เข้าถึงทั้งหมด';

  @override
  String get detectionReShade => 'ReShade';

  @override
  String get detectionHDTextures => 'เท็กซ์เจอร์ HD';

  @override
  String get detectionLodMod => 'LOD Mod (Automata-LodMod)';

  @override
  String get detectionSkRes => 'เท็กซ์เจอร์ Special K (SK_Res)';

  @override
  String get detectionNaiom => 'NAIOM';

  @override
  String get detectionCutscenes => 'ม็อดคัตซีน (nams/cutscenes)';

  @override
  String get detectionCustomMovie => 'คัตซีนแบบกำหนดเองใน data/movie/';

  @override
  String get detectionDetected => 'ตรวจพบ';

  @override
  String get detectionReShadeIncompatible => 'ไม่เข้ากัน (addon/ImGui)';

  @override
  String get detectionNotFound => 'ไม่พบ';

  @override
  String get detectionNoneFound => 'ไม่พบรายการใด';

  @override
  String get detectionLodModMigrated => 'พบแล้ว - ย้ายเข้าสู่ NAMS แล้ว';

  @override
  String get detectionSkResAuto => 'พบแล้ว - โหลดอัตโนมัติ';

  @override
  String get detectionNaiomPending => 'พบแล้ว - นำเข้าการตั้งค่าอัตโนมัติ';

  @override
  String get detectionNoneInstalled => 'ยังไม่ได้ติดตั้ง';

  @override
  String get detectionCustomMovieHint =>
      'พบแล้ว - แนะนำให้ใช้ nams/cutscenes/ แทน เพื่อให้มีไฟล์ต้นฉบับเป็นตัวสำรองอย่างปลอดภัย';

  @override
  String get detectionInstalled => 'ติดตั้งแล้ว';

  @override
  String get detectionCustomFilesDetected => 'ตรวจพบไฟล์แบบกำหนดเอง';

  @override
  String get detectionMigratedIntoNams => 'ย้ายเข้าสู่ NAMS แล้ว';

  @override
  String get detectionLoadedAutomatically => 'โหลดอัตโนมัติ';

  @override
  String get detectionMigrationComingSoon => 'นำเข้าการตั้งค่าอัตโนมัติ';

  @override
  String get detectionNotSet => 'ยังไม่ได้ตั้งค่า';

  @override
  String get labelGame => 'เกม';

  @override
  String get labelMode => 'โหมด';

  @override
  String get labelDlc => 'DLC';

  @override
  String get labelNoDlc => 'ไม่มี DLC';

  @override
  String get searchingForNier => 'กำลังค้นหา NieR:Automata...';

  @override
  String get autoButton => 'อัตโนมัติ';

  @override
  String get nierFound => 'พบ NieR:Automata';

  @override
  String get selectInstallation => 'เลือกการติดตั้งของคุณ:';

  @override
  String get notYourGame => 'ไม่ใช่เกมของคุณหรือ?';

  @override
  String get searchAllDrives => 'ค้นหาทุกไดรฟ์';

  @override
  String get selectManually => 'เลือกด้วยตนเอง';

  @override
  String get notFoundTitle => 'ไม่พบ';

  @override
  String get notFoundMessage =>
      'ไม่พบ NieR:Automata ผ่าน Steam ต้องการสแกนทุกไดรฟ์หรือไม่? อาจใช้เวลาสูงสุด 30 วินาที';

  @override
  String get scanDrives => 'สแกนไดรฟ์';

  @override
  String get confirmInstallation => 'นี่คือการติดตั้งที่ถูกต้องหรือไม่?';

  @override
  String get cancelButton => 'ยกเลิก';

  @override
  String get noSelectManually => 'ไม่ เลือกด้วยตนเอง';

  @override
  String get yesButton => 'ใช่';

  @override
  String get installationsFoundTitle => 'พบการติดตั้ง NieR:Automata';

  @override
  String get validInstallations => 'การติดตั้งที่ถูกต้อง (มีโฟลเดอร์ data):';

  @override
  String get withoutDataFolder => 'ไม่มีโฟลเดอร์ data:';

  @override
  String get noLogEntries => 'ไม่พบรายการบันทึก';

  @override
  String get noLogMatches => 'ไม่มีรายการบันทึกที่ตรงกับการค้นหา';

  @override
  String get logViewerTitle => 'ตัวดูบันทึก';

  @override
  String get logSearchPlaceholder => 'ค้นหาระดับ / โมดูล / ข้อความ...';

  @override
  String get logLiveBadge => 'สด';

  @override
  String get logAutoScroll => 'เลื่อนไปยังรายการล่าสุดอัตโนมัติ';

  @override
  String get entriesSuffix => 'รายการ';

  @override
  String get scrollToBottom => 'เลื่อนไปด้านล่าง';

  @override
  String get openLogsFolder => 'เปิดโฟลเดอร์บันทึก';

  @override
  String get diagnosticsButton => 'สร้างข้อมูลวินิจฉัย';

  @override
  String get diagnosticsSubtitle =>
      'รวบรวมรายงานการติดตั้งทั้งหมด (ไฟล์เกม ม็อด ReShade/3DMigoto การตั้งค่า) เพื่อแชร์เมื่อขอความช่วยเหลือ';

  @override
  String get copyCommandTitle => 'คำสั่งเริ่มเกมด้วยตนเอง';

  @override
  String get copyCommandDesc =>
      'หากเกมไม่เริ่มทำงานจากตัวเรียกใช้งาน ให้คัดลอกคำสั่ง NAMS แล้วรันในเทอร์มินัล NAMS จะแสดงสาเหตุที่ล้มเหลวที่นั่น ซึ่งเป็นวิธีที่เร็วที่สุดในการหาสาเหตุ';

  @override
  String get copyCommandButton => 'คัดลอกคำสั่ง';

  @override
  String get diagnosticsTitle => 'สรุปการวินิจฉัย';

  @override
  String get diagnosticsCollecting => 'กำลังรวบรวมข้อมูลวินิจฉัย...';

  @override
  String get diagnosticsCopy => 'คัดลอกสรุป';

  @override
  String get diagnosticsCopied => 'คัดลอกสรุปไปยังคลิปบอร์ดแล้ว';

  @override
  String get diagnosticsSaveFull => 'บันทึกรายงานฉบับเต็ม';

  @override
  String diagnosticsSavedAt(String path) {
    return 'บันทึกรายงานฉบับเต็มไปยัง $path แล้ว';
  }

  @override
  String get modsTutorialBack => 'ย้อนกลับ';

  @override
  String get modsTutorialNext => 'ถัดไป';

  @override
  String get modsTutorialFinish => 'เข้าใจแล้ว';

  @override
  String get modsTutorialSkip => 'ข้าม';

  @override
  String get modsTutorialMenuEcosystem => 'NAMS และตัวเรียกใช้งาน';

  @override
  String get modsTutorialMenuInstall => 'วิธีติดตั้งม็อด';

  @override
  String get modsTutorialMenuProfiles => 'การทำงานของโปรไฟล์';

  @override
  String get modsTutorialMenuSupporting => 'การสนับสนุนระบบนิเวศ';

  @override
  String get modsTutorialSupportingStep1Title =>
      'ระบบนิเวศ ไม่ใช่แค่ตัวจัดการม็อด';

  @override
  String get modsTutorialSupportingStep1Body =>
      '**NAMS เป็นโปรเจกต์ที่พัฒนามานานกว่า 3 ปี** สิ่งที่เริ่มต้นจากตัวโหลดม็อดเพียงตัวเดียว ได้เติบโตเป็นแพลตฟอร์มเต็มรูปแบบที่รองรับ **ม็อด**, **ปลั๊กอิน** และเครื่องมือต่าง ๆ:\n\n- **ม็อด** คือเนื้อหาที่เพิ่มเข้ามา เช่น ชุด เควสต์แบบกำหนดเอง อาวุธใหม่ และเท็กซ์เจอร์ ม็อดใช้ระบบเนื้อหาของ NAMS แต่ไม่ได้รันโค้ดของตัวเอง\n- **ปลั๊กอิน** คือโปรแกรมที่ทำงานร่วมกับเกมและสามารถขยายตัวโหลดม็อดได้ เช่น ม็อดผู้เล่นหลายคน โอเวอร์เลย์ดีบัก หรือระบบเกมใหม่ ๆ ปลั๊กอินเขียนด้วยโค้ดและถูก NAMS โหลดตอนเริ่มเกม\n- ตัวเรียกใช้งานที่คุณใช้อยู่ตอนนี้ **ไม่ใช่ปลั๊กอิน** แต่เป็นแอปแยกที่เตรียมม็อดและการตั้งค่าก่อนเริ่มเกม\n\nม็อดที่คุณเห็นในวันนี้ไม่ได้ทำงานทั้งที่ต้องฝ่าด่านอุปสรรค แต่มันทำงานอยู่บนพื้นฐานที่ใช้เวลาสร้างมาหลายปี เพื่อให้คุณไม่ต้องทำทุกอย่างซ้ำเอง\n\nส่วนนี้อธิบายว่า **แพลตฟอร์มนี้ดำรงอยู่ได้อย่างไร** และมีความหมายต่อคุณอย่างไร ไม่ว่าคุณจะเป็นผู้เล่นทั่วไป เริ่มลองสร้างม็อด หรือกำลังคิดจะช่วยพัฒนา\n\nหน้าถัดไปจะเริ่มจากสิ่งที่ใช้ได้กับ **ทุกคน** แล้วจึงเข้าสู่เนื้อหาทางเทคนิคมากขึ้นในช่วงท้าย คุณสามารถข้ามได้ทุกเมื่อ ไม่มีส่วนใดเป็นสิ่งที่ต้องอ่าน';

  @override
  String get modsTutorialSupportingStep2Title =>
      'สร้างเนื้อหาโดยไม่ต้องเขียนโค้ด';

  @override
  String get modsTutorialSupportingStep2Body =>
      '**คุณไม่จำเป็นต้องเป็นโปรแกรมเมอร์เพื่อช่วยเพิ่มสิ่งใหม่ให้ระบบนิเวศนี้**\n\nNAMS มีระบบเนื้อหาที่รองรับอยู่แล้ว และกำลังขยายเพื่อรองรับเนื้อหาแบบประกาศกำหนดค่ามากขึ้นเรื่อย ๆ:\n\n- **เควสต์แบบกำหนดเอง** ที่เพิ่มต่อจากเนื้อเรื่องวานิลลา\n- **อาวุธและไอเท็มใหม่** ที่มีพฤติกรรมของตัวเองโดยไม่แทนที่ของเดิม\n- **เครื่องประดับ** ใน **ช่องใหม่** โดยไม่แทนที่ช่องเดิม\n- **คัตซีนแบบกำหนดเอง** ที่อยู่ใน `nams/cutscenes/` หรือรวมมากับม็อด เช่น เควสต์แบบกำหนดเองที่มีฉากภาพยนตร์ของตัวเอง คัตซีนต้นฉบับจะ **ไม่ถูกแทนที่** แต่คัตซีนใหม่จะโหลดควบคู่กัน\n\nหลักสำคัญคือ **เพิ่มเข้าไป ไม่ใช่แทนที่** เนื้อหาวานิลลายังคงอยู่ และเนื้อหาจากม็อดจะถูกซ้อนเพิ่มด้านบน ดังนั้นผู้สร้างโมเดลที่ไม่มีประสบการณ์เขียนโปรแกรมก็สามารถสร้างเครื่องประดับ อาวุธ ตัวละคร และเผยแพร่เป็นเนื้อหาใหม่ได้ โดยไม่เขียนทับอะไร ไม่ทำให้เซฟเสีย และไม่ขัดแย้งกับม็อดอื่นที่ทำแบบเดียวกัน\n\nพื้นที่นี้จะ **ขยายต่อไป** ในอนาคต ทุกเวอร์ชันจะเพิ่มจุดเชื่อมต่อแบบกำหนดค่ามากขึ้น เพื่อให้ผู้ที่ไม่เขียนโปรแกรมทำสิ่งต่าง ๆ ได้มากกว่าเดิม';

  @override
  String get modsTutorialSupportingStep3Title =>
      'วิธีมีส่วนร่วม ไม่ได้มีแค่การเขียนโค้ด';

  @override
  String get modsTutorialSupportingStep3Body =>
      'การสร้างเนื้อหา (หน้าก่อนหน้า) เป็นเพียงหนึ่งวิธีในการตอบแทนระบบนิเวศ ยังมีอีกหลายวิธี และหลายสิ่งที่ทำให้ระบบนิเวศแข็งแรงไม่ได้เกี่ยวกับการสร้างม็อดเลย สิ่งที่ช่วยได้จริงมีดังนี้:\n\n- **เขียนคู่มือ** เช่น \"ฉันสร้าง X ด้วย NAMS อย่างไร\", \"ฉันดีบัก Y อย่างไร\", \"ห้าสิ่งที่อยากรู้ตั้งแต่แรก\" ช่องว่างในการเริ่มต้นใช้งานส่วนใหญ่ตอนนี้คือช่องว่างด้านเอกสาร\n- **รายงานบั๊กจริงอย่างมีคุณภาพ** มีขั้นตอนทำซ้ำได้ มีบันทึก และรายงานวินิจฉัยจากแผง Logs สิ่งนี้มีค่ามากกว่าตั๋ว \"มันไม่ทำงาน\" สิบรายการ\n- **สร้างโมเดล** เครื่องประดับ อาวุธ ตัวละคร และอุปกรณ์ประกอบฉาก ระบบเนื้อหาของ NAMS โหลดสิ่งเหล่านี้เป็น **เนื้อหาเพิ่มเติม** ในช่องใหม่โดยไม่แทนที่ของเดิม ผู้สร้างโมเดลที่ไม่เขียนโปรแกรมจึงสามารถเผยแพร่ผลงานที่เข้าไปอยู่ในชุดม็อดของทุกคนได้โดยไม่เกิดความขัดแย้ง\n- **แปลภาษา** ตัวเรียกใช้งานรองรับหลายภาษา หากยังไม่มีภาษาของคุณและคุณต้องการใช้ สตริงอยู่ใน `lib/l10n/` และยินดีรับ PR\n- **ทดสอบบนฮาร์ดแวร์ที่ไม่ปกติ** Steam Deck, GPU AMD, จอ ultrawide, หลายจอ หรือคอนโทรลเลอร์ที่แทบไม่มีใครใช้ บั๊กที่เกิดเฉพาะกับการตั้งค่าหายากจะไม่ถูกพบจนกว่าจะมีคนรายงาน\n- **ช่วยตอบคำถามใน Discord** การช่วยผู้ใช้ใหม่คนถัดไปก็คือการมีส่วนร่วม สิ่งที่อยู่รอดในระบบนิเวศระยะยาวส่วนใหญ่คือวัฒนธรรมของผู้ที่เข้ามาช่วยกันตั้งแต่แรก\n- **ทำ reverse-engineer ฟังก์ชันเกมหนึ่งรายการและคืน API ให้ชุมชน** *(มีรายละเอียดในหน้าถัดไปหากสนใจ)* สำหรับผู้มีความรู้ด้านเทคนิค นี่คือการมีส่วนร่วมที่สร้างผลกระทบสูงที่สุด\n\n### ข้อคิดส่งท้าย (จนถึงตอนนี้)\n\nเวลาว่าง เงินลงทุนส่วนตัว และความดื้อดึงจำนวนมากถูกใช้เพื่อทำให้แพลตฟอร์มนี้เกิดขึ้น โดยมีเป้าหมายเสมอว่า **ช่วยให้คนอื่นเริ่มทำบางสิ่งได้** หากมีข้อใดด้านบนที่คุณคิดว่าทำได้ คุณก็มาไกลเกือบถึงจุดเริ่มแล้ว Discord (**YoRHa Continuum**) คือสถานที่สำหรับสอบถาม\n\nสองหน้าถัดไปจะเป็นเนื้อหาทางเทคนิคมากขึ้น อ่านต่อหากอยากรู้ว่าปลั๊กอินอยู่ร่วมกันอย่างไรและ API ใหม่ของเกมเข้ามาอยู่ใน NAMS ได้อย่างไร หรือหยุดตรงนี้ได้หากข้อมูลเพียงพอแล้ว';

  @override
  String get modsTutorialSupportingStep4Title =>
      'ปลั๊กอินอยู่ร่วมกันได้โดยการออกแบบ';

  @override
  String get modsTutorialSupportingStep4Body =>
      '*สองหน้าถัดไปเป็นเนื้อหาทางเทคนิคมากขึ้นและเหมาะสำหรับผู้ที่กำลังคิดจะสร้างปลั๊กอิน ข้ามได้หากไม่ใช่สิ่งที่คุณสนใจ*\n\nคุณสมบัติสำคัญของแพลตฟอร์ม NAMS คือ **ปลั๊กอินหลายตัวสามารถทำงานพร้อมกันได้** ในเซสชันเกมเดียว โดยไม่แย่งหรือทำลายกัน\n\n**Multiplayer Mod โดย Jasper** เป็นหนึ่งในสิ่งที่ใหญ่ที่สุดที่เกิดขึ้นในระบบนิเวศ NAMS และยังคงได้รับการดูแลอย่างต่อเนื่อง ขอแสดงความชื่นชมอย่างมากต่อผลงานนี้ YP Devkit และปลั๊กอิน MP **โหลดพร้อมกันได้** ทั้งคู่ทำงานและวาด UI ของตนเองทับบนเกม สิ่งนี้ไม่ใช่ความบังเอิญ แต่เป็นสิ่งที่โฮสต์ปลั๊กอินของ NAMS ถูกสร้างมาเพื่อรองรับ\n\nดังนั้น หากคุณเผยแพร่ปลั๊กอินที่ทำตามข้อกำหนดของแพลตฟอร์ม **ปลั๊กอินนั้นสามารถอยู่ร่วมกับทุกสิ่งที่กำลังทำงานอยู่ได้** ไม่ว่าจะเป็นปลั๊กอินของคุณ ปลั๊กอิน MP, YP Devkit หรือปลั๊กอินในอนาคตจากคนที่คุณไม่เคยพบ คุณไม่ต้องแย่ง hook หรือสู้กับลำดับการโหลด เพราะแพลตฟอร์มเป็นผู้ประสานงาน\n\nยังมีการปรับโครงสร้างเกิดขึ้นทุกเดือน เพื่อลดจุดที่ปลั๊กอินหนึ่งอาจทำให้อีกปลั๊กอินเสียโดยไม่ตั้งใจ เป้าหมายยังเคลื่อนไหวอยู่ แต่ทิศทางชัดเจนและงานกำลังดำเนินต่อไป';

  @override
  String get modsTutorialSupportingStep5Title =>
      'คุณกำลังต่อยอดจากงาน reverse-engineering ที่เปิดให้ใช้ฟรี';

  @override
  String get modsTutorialSupportingStep5Body =>
      'API ของเอนจินส่วนใหญ่ที่จำเป็นสำหรับการสร้างปลั๊กอินจริงจังมีอยู่ใน NAMS แล้ว และ **มันมีอยู่เพราะมีคนทำ reverse-engineer เกม** เพื่อแก้ปัญหาของตนเองและนำผลลัพธ์กลับมาแบ่งปัน\n\nเกมเป็นซอฟต์แวร์ปิด ทุก API ใน NAMS ที่ให้คุณอ่านหรือเขียนสถานะของเกมเกิดจากการติดตามฟังก์ชัน ค้นหา offset และตรวจสอบพฤติกรรม นี่คืองานจำนวนมากที่ทำให้ฟรี และถูกเก็บไว้ใน NAMS โดยเฉพาะเพื่อให้ผู้สร้างปลั๊กอิน **คนถัดไป** ไม่ต้องทำซ้ำ\n\nเมื่อคุณสร้างบน NAMS คุณได้รับงานทั้งหมดนั้นต่อยอดทันที คุณไม่ได้เริ่มจาก `LoadLibrary` แต่เริ่มจาก API ที่มีคนต่อสู้จนสร้างขึ้นมาได้แล้ว และคนถัดไปที่ต้องใช้ API ที่คุณ reverse-engineer ก็จะได้รับของขวัญแบบเดียวกัน\n\n### เหตุใดนี่จึงเป็นการมีส่วนร่วมที่สร้างผลกระทบสูงที่สุด\n\nหากคุณทำสิ่งนี้แม้เพียงครั้งเดียว คุณจะช่วยประหยัดงานเดียวกันให้ผู้สร้างปลั๊กอินทุกคนในอนาคตอย่างถาวร นี่คือความไม่สมมาตร คู่มือช่วยคนที่มาอ่านได้จำนวนหนึ่ง แต่ API ใน NAMS ช่วยทุกคนที่ต้องใช้ความสามารถนั้นตลอดไป ระบบนิเวศเติบโตจากผู้ที่ reverse-engineer สิ่งหนึ่งเพื่อตัวเอง แล้วทิ้งผลลัพธ์ไว้ให้ทุกคน';

  @override
  String get modsTutorialEcosystemStep1Title => 'เหตุผลที่ทั้งหมดนี้มีอยู่';

  @override
  String get modsTutorialEcosystemStep1Body =>
      'การม็อด NieR ในอดีตเป็นเรื่องเจ็บปวด ม็อดที่ทำงานได้ดีเมื่อใช้เดี่ยว ๆ เริ่มขัดแย้งทันทีเมื่อซ้อนหลายตัว hook DLL ต่าง ๆ (`dxgi`, `d3d11`, `dinput8`) แย่งช่องเดียวกัน wrapper ที่ไม่ถูกต้องชนะลำดับการโหลด และเกมปิดเงียบตั้งแต่เริ่ม ผู้ที่มีม็อด 5–10 ตัวใช้เวลาแยกหาสาเหตุมากกว่าเวลาเล่น\n\nเป็นเวลานาน คำตอบเดียวคือ *ติดตั้งด้วยตนเองเท่านั้น* วางไฟล์ `.dat`/`.dtt` ลงใน `data/` แก้การตั้งค่าด้วยมือ และห้ามใช้ตัวจัดการม็อด วิธีนี้ใช้ได้กับหนึ่งหรือสองม็อด แต่เขียนทับไฟล์เกมจริงและไม่ทิ้งบันทึกว่าอะไรถูกเปลี่ยน เครื่องมืออย่าง Vortex ก็ช่วยไม่ได้ เพราะไม่เข้าใจลักษณะเฉพาะของ NieR\n\n**NAMS มีอยู่เพื่อแก้ปัญหานี้ในระดับตัวโหลดม็อด** และ **ตัวเรียกใช้งานนี้มีอยู่เพื่อมอบหน้าตาที่ใช้งานง่ายให้ NAMS**';

  @override
  String get modsTutorialEcosystemStep2Title => 'สิ่งที่ NAMS ทำ';

  @override
  String get modsTutorialEcosystemStep2Body =>
      '**NAMS คือตัวโหลดม็อด** ไม่ใช่ proxy DLL ที่ยึด `dxgi.dll` หรือ `d3d11.dll` แบบเครื่องมือรุ่นเก่า เพราะกลไกนั้นเองคือสาเหตุของความขัดแย้งตั้งแต่แรก\n\nNAMS ทำงานเป็นกระบวนการโฮสต์ของตัวเอง โดยโหลด NieR:Automata เป็นไลบรารีภายในกระบวนการนั้น (แปลง exe ของเกมเป็น `game.bin` ที่โหลดได้) และเริ่มต้นตัวโหลดม็อดก่อนเกม ไม่มีสิ่งใดถูกฉีดเข้าไปในกระบวนการอื่น NAMS *คือ* กระบวนการที่เกมทำงานอยู่ และควบคุมสิ่งที่จะโหลดได้เต็มรูปแบบ\n\nจากนั้น NAMS ทำสองสิ่งหลัก:\n\n**1. สร้างความสามารถเดิมของเครื่องมืออื่นขึ้นใหม่แบบเนทีฟ** เช่น LodMod, Limit Break, การฉีดเท็กซ์เจอร์ และการโหลดเร็ว ทั้งหมดอยู่ในชั้นเดียวที่ทำงานประสานกัน ม็อดเชื่อมต่อกับ API ของ NAMS แทนการแย่งกันว่า hook DLL ใดจะถูกเรียกก่อน\n\n**2. มีระบบไฟล์เสมือน (VFS):**\n\n- ม็อดทุกตัวอยู่ในโฟลเดอร์ของตนเองใต้ `nams/mods/<modId>/` โดยไม่เขียนทับไฟล์เกมจริง\n- ขณะรันเกม NAMS จะซ้อนม็อดที่เปิดใช้งานเข้าไปในมุมมอง `data/` ของเอนจิน\n- `data/*.cpk` และ `NieRAutomata.exe` แบบวานิลลาของคุณ **ไม่เคยถูกแก้ไข** ดังนั้นการเปิดเกมแบบไม่มีม็อดผ่าน Steam ยังทำงานเหมือนเดิม\n\nม็อดประกาศสิ่งที่แก้ไขไว้ใน manifest NAMS ตรวจสอบและโหลดตามลำดับที่กำหนด คุณจึงสามารถ **เปิด/ปิดแต่ละม็อดได้อย่างสะอาด** และ **ตรวจพบความขัดแย้งที่รู้สาเหตุได้** ซึ่งไม่สามารถทำได้ด้วยวิธีวางไฟล์ลง `data` แบบเดิม\n\n### ทุกส่วนทำงานร่วมกันอย่างไร\n\nตัวเรียกใช้งานนี้ **ไม่ได้** สร้างบน NieR:Automata โดยตรง มันไม่ทำ reverse-engineer เกม ไม่ hook ฟังก์ชันเอนจิน และไม่จำเป็นต้องรู้รูปแบบ `.dat`/`.dtt` ด้วยตัวเอง ลำดับคือ:\n\n1. **NieR:Automata** - ตัวเกมที่ไม่ถูกแก้ไข\n2. **NAMS** - ตัวโหลดม็อดที่ถูกออกแบบก่อน เพื่อทำให้การม็อดในระดับใหญ่เป็นไปได้ (การสร้างระบบเอนจินใหม่, VFS, โฮสต์ปลั๊กอิน, เฟรมเวิร์กเนื้อหา)\n3. **ตัวเรียกใช้งานนี้** - ผู้ช่วยที่สร้างบน NAMS อ่านการตั้งค่า TOML ของ NAMS เขียนลงโครงสร้างโฟลเดอร์ NAMS และสื่อสารกับ API ของ NAMS เท่านั้น\n\nผลที่เกิดขึ้นจริงคือ NAMS เป็นชั้นหลักที่รับน้ำหนัก ตัวเรียกใช้งานเป็นเพียง UI ที่ใช้งานง่ายด้านหน้า และสามารถแทนด้วย UI อื่นหรือบรรทัดคำสั่งได้โดยไม่กระทบม็อด\n\n### และมีตัวอย่างแล้ว\n\nนี่ไม่ใช่ทฤษฎี **NAO Launcher โดย Rustukun** เป็นตัวเรียกใช้งานแยกที่สร้างบนพื้นฐานเดียวกัน มี UI และแนวคิดการออกแบบต่างกัน แต่สื่อสารกับ NAMS เดียวกันด้านล่าง ม็อดของคุณ โฟลเดอร์ `nams/mods/<modId>/` และ `disabled_mods.toml` ทั้งหมดทำงานเหมือนกันไม่ว่าคุณใช้ตัวเรียกใช้งานใด\n\nนี่คือหลักฐานว่า NAMS คือแพลตฟอร์ม และตัวเรียกใช้งานใด ๆ (ตัวนี้, NAO หรือของอนาคตที่ยังไม่มีใครเขียน) เป็นเพียงตัวเลือก frontend เลือกสิ่งที่เข้ากับวิธีทำงานของคุณ โดยไม่ต้องย้ายคลังม็อด';

  @override
  String get modsTutorialEcosystemStep3Title =>
      'สิ่งที่ตัวเรียกใช้งานเพิ่ม และสิ่งที่แตกต่าง';

  @override
  String get modsTutorialEcosystemStep3Body =>
      'NAMS จัดการการโหลด ส่วนตัวเรียกใช้งานจัดการ **ทุกอย่างรอบ ๆ** ได้แก่ การติดตั้ง การจัดระเบียบ และการแก้ปัญหา:\n\n- **ตัวจัดการม็อด** - ติดตั้งม็อดรูปแบบ NAMS ด้วยการลากวาง ปรับโครงสร้างอัตโนมัติ (wrapper wax/SK_Res และทรัพยากรที่รวมมา) ตรวจสอบ manifest และเตือนความขัดแย้ง\n- **เท็กซ์เจอร์** - จัดการแพ็กเท็กซ์เจอร์แยกและลำดับความสำคัญ `load_order` โดยไม่ต้องแก้ TOML ด้วยมือ\n- **คัตซีน** - ติดตั้งม็อดคัตซีน HD ตรวจหา codec อัตโนมัติ (H264 หรือ MPEG-2) และตั้งค่า NAMS ที่ถูกต้อง\n- **โปรไฟล์** - เก็บชุดม็อดหลายแบบไว้ข้างกัน สลับได้ด้วยคลิกเดียวโดยไม่ต้องคัดลอกหรือสูญเสียสถานะ\n- **การวินิจฉัย** - รายงานเต็มว่าอะไรติดตั้งที่ไหน อะไรเหลือจากการติดตั้งเก่า และสิ่งที่ NAMS เห็นเทียบกับสิ่งที่อยู่บนดิสก์จริง\n\n### เหตุใดเราจึงสร้างสิ่งนี้\n\n**เราไม่ได้ต่อต้านการติดตั้งด้วยตนเอง** การวางไฟล์ `.dat`/`.dtt` ของชุดหนึ่งลงในโฟลเดอร์ย่อย `data/` ที่ถูกต้องใช้ได้ดีกับหนึ่งหรือสองม็อด ตัวเรียกใช้งานนี้ถูกสร้างสำหรับระดับที่ใหญ่กว่านั้น\n\nระบบนิเวศ NAMS รองรับสิ่งต่าง ๆ เช่น:\n\n- **ม็อดชุดมากกว่า 30 ชุด** พร้อมสลับหลายชุดต่อหนึ่งตัวละคร\n- **เควสต์แบบกำหนดเองมากกว่า 20 รายการ** ที่เพิ่มต่อจากเนื้อเรื่องวานิลลา\n- **อาวุธใหม่มากกว่า 10 ชิ้น** ที่มีพฤติกรรมของตัวเอง\n- รวมถึงเท็กซ์เจอร์ คัตซีน ปลั๊กอิน การปรับสมดุล และอื่น ๆ\n\nการจัดการสิ่งนี้ด้วยมือไม่ใช่แค่เรื่องความชอบ แต่ **เป็นไปไม่ได้ในทางปฏิบัติ** คุณไม่สามารถติดตามว่าไฟล์มาจากม็อดใด เปิดหรือปิดม็อดเดียวอย่างสะอาด หรือรู้ว่าอะไรเสีย เมื่อถึงระดับใหญ่ การม็อดด้วยมือจะชนกำแพง และระบบนิเวศ NAMS ผ่านกำแพงนั้นมานานแล้ว\n\n### ความแตกต่างจาก NAMH และ Vortex\n\nหากคุณอยู่ในชุมชน NieR มานาน คุณอาจจำได้ว่าตัวจัดการม็อดก่อนหน้าจบอย่างไร:\n\n- **NAMH** (NieR Automata Mod Helper) หยุดพัฒนา ทำให้เกมเสียในแบบที่กู้คืนไม่ได้ เจอปัญหาไฟล์ \"program in use\" และวิธีกู้คืนมาตรฐานคือ *ถอนการติดตั้งเกม ติดตั้งใหม่ แล้วทำด้วยมือ* มันทำงานโดย **แทนที่ไฟล์ใน `data/` โดยตรง** เมื่อการติดตั้ง NAMH ผิดพลาด จึงไม่มีทางย้อนกลับอย่างสะอาด\n- **Vortex** ไม่เคยเข้าใจโครงสร้างไฟล์ของ NieR อย่างถูกต้อง ระบบไฟล์เสมือนของมันใช้สมมติฐานที่ไม่ตรงกับวิธีโหลดเนื้อหาของเกม ทำให้ไฟล์ถูกวางผิดตำแหน่งโดยไม่แจ้งเตือน\n\nตัวเรียกใช้งานนี้สร้างต่างออกไป โดยมีหลักการสำคัญดังนี้:\n\n1. **ไม่แทนที่ไฟล์เด็ดขาด** ม็อดอยู่ใน `nams/mods/<modId>/` และถูกซ้อนเข้าในมุมมองเอนจินขณะรันผ่าน VFS ของ NAMS `data/` วานิลลาไม่ถูกแตะ จึงไม่มี \"สถานะที่กู้ไม่ได้\" เพราะไม่มีอะไรในเกมจริงถูกเปลี่ยน\n2. **ทุกการกระทำย้อนกลับได้** ถอนการติดตั้งม็อด → โฟลเดอร์และทรัพยากรที่รวมมาถูกลบอย่างสะอาด ปิดม็อด → เพิ่มรายการใน `disabled_mods.toml` และ NAMS ข้ามม็อดนั้น ไม่มีสถานะซ่อนหรือการเขียนที่ย้อนกลับไม่ได้\n3. **ใช้โปรไฟล์แทนสถานะส่วนกลางเดียว** ตัวจัดการเดิมผูกทุกอย่างไว้กับการตั้งค่าเดียว โปรไฟล์ช่วยเก็บชุดม็อดหลายชุดข้างกันและสลับแบบอะตอม ไม่มีอะไรเสียหายหรือสูญหาย\n4. **สร้างบนตัวโหลดม็อดที่ยังได้รับการดูแล** NAMH สิ้นสุดเพราะฐานตัวโหลดม็อดไม่แน่นอน NAMS คือรากฐานของทุกอย่างที่นี่ และตัวเรียกใช้งานติดตามการอัปเดตของมัน\n\nแม้ตัวเรียกใช้งานนี้จะหยุดดูแล ม็อดของคุณก็ยังเป็นไฟล์บนดิสก์ที่ NAMS อ่านได้ คุณจะไม่ถูกล็อกออกจากการติดตั้งของตัวเอง';

  @override
  String get modsTutorialEcosystemStep4Title => 'ไปที่ใดต่อ';

  @override
  String get modsTutorialEcosystemStep4Body =>
      'หากคุณยังไม่เคยติดตั้งม็อดที่นี่:\n\n- **วิธีติดตั้งม็อด** - อธิบายขั้นตอนการติดตั้งทีละแท็บ\n- **การทำงานของโปรไฟล์** - อธิบายชุดม็อดแยกและควรใช้เมื่อใด\n\nทั้งสองหัวข้ออยู่ในเมนูช่วยเหลือเดียวกันนี้ (ไอคอน **?** ที่คุณใช้เปิดหน้านี้)\n\n**ฉบับสั้น:** วางไฟล์เก็บถาวรลงในแท็บที่ถูกต้อง (Mod Manager สำหรับม็อดตัวละคร/ข้อมูล, Textures สำหรับแพ็กเท็กซ์เจอร์แยก, Cutscenes สำหรับคัตซีน HD) กด Play และดู Logs หากเกิดปัญหา ตัวเรียกใช้งานจะพยายามเลือกสิ่งที่ถูกต้องโดยอัตโนมัติ และหากคุณไม่เห็นด้วยกับการตัดสินใจใด ทุกการกระทำสามารถย้อนกลับได้จาก UI';

  @override
  String get modsTutorialHelpTooltip => 'บทช่วยสอนและความช่วยเหลือ';

  @override
  String get modsTutorialInstallStep1Title => 'วางม็อดของคุณที่นี่';

  @override
  String get modsTutorialInstallStep1Body =>
      'นี่คือแท็บ **ตัวจัดการม็อด** ใช้สำหรับติดตั้งม็อดตัวละคร ชุด และม็อดเกมเพลย์อื่น ๆ\n\nลากไฟล์ `.zip`, `.7z` หรือ `.rar` จาก Nexus มาวางในพื้นที่นี้ (หรือคลิกเพื่อเลือกไฟล์) ตัวเรียกใช้งานจะแตกไฟล์ ตรวจสอบโครงสร้าง และวางไว้ในตำแหน่งที่ถูกต้อง คุณไม่ต้องแตกไฟล์เอง\n\n**ควรรู้:** ไฟล์เกมวานิลลาของคุณจะไม่ถูกแตะ ม็อดอยู่ในโฟลเดอร์ `nams/` แยกต่างหาก ดังนั้นคุณยังสามารถเปิดเกมแบบไม่มีม็อดผ่าน Steam ได้ทุกเมื่อ';

  @override
  String get modsTutorialInstallStep2Title =>
      'กำลังติดตั้งม็อด WAX หรือไม่? โปรดอ่านก่อน';

  @override
  String get modsTutorialInstallStep2Body =>
      '**ม็อด WAX ใช้งานที่นี่ได้** NAMS สร้างความสามารถของ WAX ใหม่จนถึงเวอร์ชันที่ผ่านการทดสอบ ม็อดส่วนใหญ่บน Nexus ที่รองรับเวอร์ชันนั้นหรือเก่ากว่าจะติดตั้งและทำงานตามปกติ\n\n### ความเข้ากันได้ทำงานอย่างไร\n\nNAMS ผ่านการตรวจสอบกับ WAX เวอร์ชันเฉพาะ ทุกสิ่งที่ WAX มีจนถึงและรวมเวอร์ชันนั้น: รองรับ ทุกสิ่งที่ WAX เพิ่มในเวอร์ชัน **ใหม่กว่า** หลังจากนั้น: ไม่รองรับโดยอัตโนมัติ เพราะเป็นฟีเจอร์ใหม่ฝั่ง WAX ที่ต้องสร้างใหม่ตั้งแต่ต้นฝั่ง NAMS\n\n### เกิดอะไรขึ้นเมื่อ WAX เพิ่มฟีเจอร์ใหม่\n\nเมื่อ WAX เพิ่มฟีเจอร์ในเวอร์ชันอนาคต จะมีการประเมินเทียบกับแผนงานของ NAMS:\n\n- **อยู่ในขอบเขต** - หากฟีเจอร์สอดคล้องกับทิศทางของ NAMS จะถูกพัฒนา และ NAMS เวอร์ชันอนาคตจะรองรับม็อดที่ใช้มัน\n- **นอกขอบเขต** - NAMS มีส่วนขยายเนื้อหาของตัวเองที่ต้องให้ความสำคัญ (เควสต์แบบกำหนดเอง แผนที่โลกแบบกำหนดเอง plugin chips แบบกำหนดเอง API ม็อดที่ขยายขึ้น และอื่น ๆ) การสร้างฟีเจอร์ WAX ทุกอย่างใหม่ไม่ใช่เป้าหมายหลัก ฟีเจอร์เฉพาะบางอย่างของ WAX อาจไม่มีสิ่งเทียบเท่าใน NAMS\n\n**นี่ไม่ใช่การดูถูก WAX** ทั้งสองเป็นโปรเจกต์แยกที่มีเป้าหมายต่างกัน NAMS ไม่ได้พยายามเป็นตัวแทน WAX แบบวางแล้วใช้ได้ทันที แต่เป็นแพลตฟอร์มของตัวเองที่ **เข้ากันได้ในวงกว้าง** กับ WAX เพราะผู้ใช้ส่วนใหญ่ต้องการให้คลังม็อดเดิมยังใช้งานได้\n\n### รูปแบบนี้เป็นเรื่องปกติ\n\nการแยกแบบนี้คือ **วิธีที่ระบบนิเวศม็อดทุกเกมพัฒนา** ไม่ใช่ความแปลกเฉพาะ NieR ตัวอย่างชัดเจนคือ **Skyrim Legendary Edition (LE)** และ **Skyrim Special Edition (SE)** ซึ่งแยกมาจากเอนจินเดียวกัน SE เข้ากันได้กับม็อด LE ในวงกว้างแต่ไม่ 100% ม็อด LE บางตัวต้องแปลง และบางตัวไม่เคยถูกพอร์ตเพราะอาศัยพฤติกรรมเอนจินที่ SE เปลี่ยน ชุมชน Skyrim ไม่ถือว่านี่เป็นข้อบกพร่อง แต่เป็นส่วนหนึ่งของระบบนิเวศ เช่นเดียวกับ **OpenMW เทียบกับ Morrowind วานิลลา**, **Minecraft Java เทียบกับ Bedrock**, **ม็อด KSP1 เทียบกับ KSP2** เป็นต้น\n\nเมื่อแพลตฟอร์มหนึ่งสร้างพฤติกรรมของอีกแพลตฟอร์มใหม่ จะเกิดขอบเขตความเข้ากันได้ ส่วนใหญ่ทำงาน แต่บางขอบไม่ทำ นี่คือความจริงของชุมชนม็อดที่มีอายุมากพอจนแตกแขนง\n\n### แนวทางที่ดีที่สุดหากไม่แน่ใจ\n\n1. **สร้างโปรไฟล์ว่างใหม่** (ดู *การทำงานของโปรไฟล์* ในเมนูช่วยเหลือ) แล้วสลับไปยังโปรไฟล์นั้น\n2. **วางม็อด WAX เพียงตัวเดียว** ลงในโปรไฟล์นั้น ไม่ต้องใส่อะไรอื่น\n3. **กด Play** หากทำงาน ให้ติดตั้งในโปรไฟล์จริงของคุณ\n4. **หากไม่ทำงาน** ม็อดอาจใช้ฟีเจอร์หลังเวอร์ชัน WAX ที่ NAMS ทดสอบ หรือเป็นฟีเจอร์ที่ NAMS เลือกไม่สร้างใหม่\n\n### สิ่งที่ควรคาดหวัง\n\n- หากฟีเจอร์ X, Y และ Z ทำงานใน NAMS แต่ม็อด WAX ที่คุณต้องการใช้ฟีเจอร์ W ที่ไม่รองรับ และคุณอยู่ได้โดยไม่มี W คุณยังใช้ X, Y, Z ร่วมกับม็อดนั้นได้\n- หาก W จำเป็นต่อม็อดและ NAMS ไม่มี คุณต้องเลือกระหว่าง WAX (ได้ W แต่เสียประโยชน์อื่นของ NAMS) หรือ NAMS (ได้ทุกอย่างอื่นแต่ไม่มี W)\n\n**อย่าลืมอีกด้านของข้อแลกเปลี่ยน:** การอยู่กับ WAX หมายถึงพลาด **ม็อดเฉพาะ NAMS** ที่ไม่ทำงานบน WAX เช่น การสลับหลายชุดต่อหนึ่งตัวละคร เควสต์แบบกำหนดเอง และระบบปลั๊กอินที่กว้างกว่า (Multiplayer Mod, YP Devkit และปลั๊กอินในอนาคต) สิ่งเหล่านี้ใช้ API ของ NAMS ที่ WAX ไม่มี ดังนั้นตัวเลือกไม่ใช่ \"NAMS ลบ W\" เทียบกับ \"WAX ที่มี W\" แต่คือ \"ระบบนิเวศ NAMS ที่ไม่มี W\" เทียบกับ \"WAX ที่มี W แต่ไม่มีทุกสิ่งเฉพาะ NAMS\"\n\nนี่เป็นข้อแลกเปลี่ยนจริงและเป็นการตัดสินใจของคุณ เราไม่ใช่ผู้ที่ควรถามว่าฟีเจอร์เฉพาะ WAX รายการหนึ่งจะได้รับการรองรับใน NAMS หรือไม่ นั่นเป็นคำถามเกี่ยวกับแผนงานระบบนิเวศ ซึ่งควรถามใน Discord ของ YoRHa Continuum';

  @override
  String get modsTutorialInstallStep3Title => 'ม็อดที่ติดตั้งของคุณ';

  @override
  String get modsTutorialInstallStep3Body =>
      'ม็อดทุกตัวที่คุณติดตั้งจะแสดงที่นี่\n\n**สวิตช์ทางขวา** - เปิดหรือปิดม็อด การปิดจะยังคงติดตั้งม็อดไว้ แต่บอกตัวโหลดม็อดให้ข้ามในการเริ่มเกมครั้งถัดไป\n\n**เกมล่มตอนเริ่มหรือไม่?** ปิดม็อดครึ่งหนึ่ง เริ่มเกม แล้วทำซ้ำ สวิตช์ช่วยให้คุณแยกหาสาเหตุได้อย่างรวดเร็ว\n\n**ป้ายเตือน** ระบุม็อดที่ขัดแย้งกัน (เช่น สองม็อดแทนที่ชุดเดียวกัน) ซึ่งเป็นสาเหตุทั่วไปที่เกมไม่ถึงหน้าจอชื่อเรื่อง';

  @override
  String get modsTutorialInstallStep4Title => 'รายละเอียดม็อด';

  @override
  String get modsTutorialInstallStep4Body =>
      'คลิกม็อดใด ๆ ในรายการเพื่อดูรายละเอียดที่นี่: ผู้สร้าง เวอร์ชัน สิ่งที่เปลี่ยน ความขัดแย้งกับม็อดอื่น และ **แพ็กเท็กซ์เจอร์หรือคัตซีนที่รวมมา**\n\nหากม็อดไม่ทำงาน สาเหตุที่พบบ่อยจะแสดงที่นี่ เช่น *ต้องใช้ NAMS เวอร์ชันใหม่กว่า* หรือ *ขัดแย้งกับม็อดอื่น* ทั้งสองอย่างมองเห็นได้ **ก่อน** กด Play\n\nปุ่ม **ถอนการติดตั้ง** จะล้างม็อดอย่างถูกต้อง รวมถึงส่วนเสริมที่รวมมาด้วย';

  @override
  String get modsTutorialInstallStep5Title =>
      'ม็อดเท็กซ์เจอร์แบบแยก → แท็บเท็กซ์เจอร์';

  @override
  String get modsTutorialInstallStep5Body =>
      '**ม็อดที่มีแต่เท็กซ์เจอร์** (แพ็ก upscale HD หรือเปลี่ยนสี) จะไม่ติดตั้งที่นี่ แต่มีแท็บของตัวเอง\n\nคลิก **Textures** ในแถบด้านข้างเพื่อติดตั้ง ลากไฟล์ `.zip` ในแบบเดียวกัน ตัวเรียกใช้งานรู้ว่ากำลังดูไฟล์ชนิดใด\n\n**หมายเหตุ:** หากม็อดตัวละคร *รวม* เท็กซ์เจอร์ของตัวเอง มันจะติดตั้งอัตโนมัติพร้อมม็อด คุณใช้แท็บ Textures เฉพาะแพ็กเท็กซ์เจอร์ **แบบแยก** เท่านั้น';

  @override
  String get modsTutorialInstallStep6Title => 'ม็อดคัตซีน → แท็บคัตซีน';

  @override
  String get modsTutorialInstallStep6Body =>
      '**ม็อดคัตซีน** (คัตซีน HD หรือวิดีโอแทนที่) มีแท็บของตัวเองเช่นกัน\n\nคลิก **Cutscenes** ในแถบด้านข้างเพื่อติดตั้ง\n\n**กฎเดียวกับเท็กซ์เจอร์:** หากม็อดตัวละครรวมคัตซีน มันจะติดตั้งอัตโนมัติ คุณใช้แท็บ Cutscenes เฉพาะแพ็กคัตซีน **แบบแยก** เท่านั้น';

  @override
  String get modsTutorialInstallStep7Title => 'กด Play';

  @override
  String get modsTutorialInstallStep7Body =>
      'กลับไปที่แท็บ **Launcher** แล้วกด **PLAY** ตัวโหลดม็อดจะอ่านม็อดใหม่ทุกครั้งที่เริ่มเกม คุณจึงไม่ต้องรีสตาร์ตตัวเรียกใช้งานนี้\n\n### หากเกมล่ม\n\nเปิด **Logs** (มุมล่างขวา) ผลลัพธ์ของตัวโหลดม็อดมักระบุชื่อม็อดที่เสีย กลับมาที่นี่แล้วปิดม็อดนั้น\n\n### ยังเสียแม้ปิดทุกม็อดหรือไม่?\n\nหากคุณหรือตัวจัดการม็อดเก่าเคยวางไฟล์ `.dat` / `.dtt` โดยตรงใน `<gameDir>/data/` เอนจินยังคงโหลดไฟล์เหล่านั้น และตัวโหลดม็อดมองไม่เห็นหรือปิดไม่ได้ นี่คือความยุ่งเหยิงที่ตัวเรียกใช้งานนี้หลีกเลี่ยงโดยเฉพาะ ม็อดทุกตัวอยู่แยกใน `nams/mods/<modId>/` แทนการเขียนทับไฟล์เกมจริง\n\nเปิด **Logs → Diagnostics** แล้วตรวจสอบส่วน *Vanilla data/ overlay* รายการใด ๆ ที่อยู่ตรงนั้นคือของเหลือจากการติดตั้งเก่า ย้ายโฟลเดอร์เหล่านั้นออกจาก `data/` แล้วเกมจะกลับสู่สถานะสะอาด';

  @override
  String get modsTutorialProfilesStep1Title => 'โปรไฟล์มีไว้ทำอะไร';

  @override
  String get modsTutorialProfilesStep1Body =>
      'โปรไฟล์ช่วยให้คุณเก็บชุดม็อดแยกหลายชุดไว้ข้างกัน\n\nตัวอย่าง:\n\n- โปรไฟล์ **main** ที่มีม็อดที่คุณใช้เล่นจริง\n- โปรไฟล์ **test** สำหรับลองสิ่งใหม่\n\nหากม็อดใหม่ที่ไม่น่าไว้ใจทำให้เกมเสีย เพียงสลับกลับไป **main** แล้วเล่นต่อ ชุดม็อดจะไม่ปะปนกัน\n\n**สำคัญ:** ม็อดที่คุณไม่ได้ใช้อยู่จะไม่ถูกลบ แต่ถูกพักไว้ พร้อมกลับมาเมื่อสลับโปรไฟล์';

  @override
  String get modsTutorialProfilesStep2Title => 'สร้างโปรไฟล์';

  @override
  String get modsTutorialProfilesStep2Body =>
      'คลิก **NEW** ในแถบโปรไฟล์ พิมพ์ชื่อ แล้วกดยืนยัน\n\nตัวเรียกใช้งานจะสร้างโปรไฟล์ว่างใหม่และสลับไปใช้ ม็อดของโปรไฟล์ก่อนหน้ายังคงปลอดภัยบนดิสก์ ไม่ได้หายไป เพียงถูกพักไว้\n\nตอนนี้คุณสามารถติดตั้งสิ่งใดก็ได้ในโปรไฟล์ใหม่โดยไม่แตะชุดม็อดอื่น';

  @override
  String get modsTutorialProfilesStep3Title => 'สลับ เปลี่ยนชื่อ ลบ';

  @override
  String get modsTutorialProfilesStep3Body =>
      '**สลับ** - เลือกโปรไฟล์จากเมนูดรอปดาวน์ รายการม็อดจะเปลี่ยนทันที\n\n**เปลี่ยนชื่อ** - เปลี่ยนชื่อโปรไฟล์โดยไม่สูญเสียข้อมูล\n\n**ลบ** - ลบโปรไฟล์ที่ไม่ได้ใช้งานอย่างถาวร (ไม่สามารถลบโปรไฟล์ที่กำลังใช้งานหรือโปรไฟล์สุดท้ายได้)\n\nการสลับทั้งหมดเกิดขึ้นในขั้นตอนเดียว หากมีสิ่งผิดพลาด ระบบจะย้อนกลับอัตโนมัติ จึงไม่ทำให้คุณติดอยู่ในสถานะเสีย';

  @override
  String get modsTutorialProfilesStep4Title =>
      'สิ่งใดติดตามโปรไฟล์ และสิ่งใดไม่ติดตาม';

  @override
  String get modsTutorialProfilesStep4Body =>
      '**เฉพาะโปรไฟล์** (เปลี่ยนเมื่อสลับ):\n\n- ม็อดที่ติดตั้ง\n- ม็อดใดเปิดหรือปิด\n- เท็กซ์เจอร์ที่รวมมากับม็อด\n\n**ใช้ร่วมกันทุกโปรไฟล์** (ไม่เปลี่ยน):\n\n- แพ็กเท็กซ์เจอร์แบบแยกที่ติดตั้งผ่านแท็บ Textures\n- ม็อดคัตซีน\n- ปลั๊กอิน\n- การตั้งค่าตัวเรียกใช้งานทั้งหมด\n\nดังนั้นโปรไฟล์จะเปลี่ยนเฉพาะสิ่งที่เป็นของม็อดจริง ๆ การตั้งค่าระดับระบบจะติดตามคุณไปทุกที่';

  @override
  String get platformUnsupportedTitle => 'ไม่สามารถเริ่มเกมบนแพลตฟอร์มนี้ได้';

  @override
  String get platformUnsupportedLinux =>
      'NieR:Automata เป็นเกม Windows จึงต้องมีชั้นความเข้ากันได้เพื่อทำงานบน Linux\n\nติดตั้ง Steam พร้อม Proton (เกมทำงานได้ดีบน Proton) หรือติดตั้ง CrossOver/Wine เมื่อมีรันไทม์ ตัวเรียกใช้งานจะสามารถเริ่มเกมได้\n\nLinux แบบเนทีฟที่ไม่มีชั้นแปลคำสั่งไม่สามารถรันเกมได้';

  @override
  String get platformUnsupportedMacos =>
      'NieR:Automata เป็นเกม Windows โปรดรันตัวเรียกใช้งานผ่าน CrossOver/Wine วิธีนี้เคยใช้งานได้ในอดีต แต่ยังไม่ได้ทดสอบซ้ำเมื่อไม่นานนี้ macOS แบบเนทีฟที่ไม่มีชั้นแปลคำสั่งไม่สามารถรันเกมได้\n\nหากคุณทำให้ใช้งานได้ด้วยวิธีใด โปรดใช้บรรทัดคำสั่งโดยตรงแทนตัวเรียกใช้งานนี้';

  @override
  String get playDisabledTooltip => 'ไม่สามารถเริ่มเกมบนแพลตฟอร์มนี้';

  @override
  String get diagnosticsClose => 'ปิด';

  @override
  String get diagnosticsSectionDataOverlay =>
      'ไฟล์ซ้อนทับใน data/ วานิลลา (วางด้วยมือ)';

  @override
  String get diagnosticsSectionExternalLegacy => 'ภายนอก / รุ่นเก่า';

  @override
  String get diagnosticsSectionGameRootExtras =>
      'ไฟล์เพิ่มเติมในโฟลเดอร์รากเกม (ไม่ใช่วานิลลา)';

  @override
  String get diagnosticsSectionGameIdentity => 'ข้อมูลเกม';

  @override
  String get diagnosticsSectionNamsHealth => 'สถานะ NAMS';

  @override
  String get diagnosticsSectionReshade => 'ReShade';

  @override
  String get diagnosticsSectionMigoto => '3DMigoto';

  @override
  String get diagnosticsSectionTexturePacks => 'แพ็กพื้นผิว';

  @override
  String get diagnosticsSectionVanillaDrops =>
      'ไฟล์ที่วางในโฟลเดอร์ data/ ของวานิลลา';

  @override
  String get diagnosticsSectionNonDefault => 'การตั้งค่าที่ไม่ใช่ค่าเริ่มต้น';

  @override
  String get diagnosticsSectionRecentIssues => 'ปัญหา NAMS ล่าสุด';

  @override
  String diagnosticsExeVariant(String variant) {
    return 'exe: $variant';
  }

  @override
  String get diagnosticsExeUnsupported => 'เวอร์ชันที่ไม่รองรับ';

  @override
  String get diagnosticsDlcPresent => 'DLC';

  @override
  String get diagnosticsGameRunning => 'กำลังทำงาน';

  @override
  String get diagnosticsNamsPresent => 'NAMS.exe';

  @override
  String diagnosticsMissingFiles(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'ไฟล์หายไป $count ไฟล์',
    );
    return '$_temp0';
  }

  @override
  String get diagnosticsInstalled => 'ติดตั้งแล้ว';

  @override
  String get diagnosticsEnabled => 'เปิดใช้งาน';

  @override
  String get diagnosticsShadersMissing => 'ไม่มีเชดเดอร์';

  @override
  String get diagnosticsTexturePacksUnavailable =>
      'ไม่สามารถสอบถามพื้นผิว NAMS ได้';

  @override
  String get diagnosticsExtraFile => 'เพิ่มเติม';

  @override
  String diagnosticsFileCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count ไฟล์',
      one: '1 ไฟล์',
    );
    return '$_temp0';
  }

  @override
  String diagnosticsMoreItems(int count) {
    return '... อีก $count รายการ';
  }

  @override
  String get refreshButton => 'รีเฟรช';

  @override
  String get tabModloaderLabel => 'ตัวโหลดม็อด';

  @override
  String get tabYorhaLabel => 'YoRHa Protocol';

  @override
  String get configEditorTitle => 'ตัวแก้ไขการตั้งค่า';

  @override
  String get changelogTitle => 'มีอะไรใหม่';

  @override
  String get tipDragTextures => 'ลากม็อดเท็กซ์เจอร์ลงในแท็บ Textures โดยตรง';

  @override
  String get tipHdCutscenes => 'ระบบตรวจพบและตั้งค่าม็อดคัตซีน HD อัตโนมัติ';

  @override
  String get tipLodModPreviews => 'การตั้งค่า LOD Mod มีภาพตัวอย่างก่อน/หลัง';

  @override
  String get tipFaqButton =>
      'ใช้ปุ่ม FAQ เพื่อดูว่า YoRHa Protocol แทนที่ม็อดใดบ้าง';

  @override
  String get tipReShadeAuto =>
      'ระบบตรวจพบ ReShade อัตโนมัติ ไม่ต้องตั้งค่าด้วยมือ';

  @override
  String get tipFreecam => 'YoRHa Protocol มี freecam ในตัวพร้อมช่องบันทึก';

  @override
  String get tipCustomQuests => 'เควสต์แบบกำหนดเองกำลังจะมา โปรดติดตาม';

  @override
  String get sectionNams => 'NAMS';

  @override
  String get sectionTextureInjection => 'การฉีดเท็กซ์เจอร์';

  @override
  String get sectionLodMod => 'LOD MOD';

  @override
  String get sectionLevelOfDetail => 'ระดับรายละเอียด';

  @override
  String get sectionAmbientOcclusion => 'Ambient Occlusion';

  @override
  String get sectionShadows => 'เงา';

  @override
  String get sectionPostProcessing => 'โพสต์โปรเซสซิง';

  @override
  String get labelValidateModelData => 'ตรวจสอบข้อมูลโมเดล';

  @override
  String get tooltipValidateModelData =>
      'เกมจะตรวจสอบข้อมูลโมเดลเมื่อโหลด ปกติหากล้มเหลวจะเงียบและดำเนินต่อด้วยข้อมูลที่เสีย ซึ่งอาจทำให้โมเดลมองไม่เห็นหรือเกิดข้อผิดพลาด เมื่อเปิดใช้งาน NAMS จะแสดงผลการตรวจสอบเป็นกล่องข้อความ เพื่อให้เห็นว่าโมเดลใดล้มเหลวและเพราะเหตุใด';

  @override
  String get labelPreloadMaxDimension => 'ขนาดสูงสุดสำหรับการโหลดล่วงหน้า';

  @override
  String get tooltipPreloadMaxDimension =>
      'ขนาดเท็กซ์เจอร์สูงสุดที่จะโหลดล่วงหน้าเข้า RAM ตอนเริ่มเกม 2048 = ค่าเริ่มต้น, 4096 = โหลดล่วงหน้าถึงเท็กซ์เจอร์ 4K, 16384 = โหลดทุกอย่าง ค่าสูงขึ้นทำให้โหลดนานขึ้นแต่ลดอาการกระตุกในเกม';

  @override
  String get labelPreloadAllTextures => 'โหลดเท็กซ์เจอร์ทั้งหมดล่วงหน้า';

  @override
  String get tooltipPreloadAllTextures =>
      'โหลดเท็กซ์เจอร์ทั้งหมดเข้า RAM โดยไม่คำนึงถึงขนาด ช่วยกำจัดอาการกระตุกจากเท็กซ์เจอร์โผล่ทั้งหมด แต่ต้องใช้ RAM 32GB ขึ้นไปและทำให้เริ่มเกมช้าลงมาก';

  @override
  String get labelEnableLodMod => 'เปิดใช้งาน LodMod';

  @override
  String get tooltipEnableLodMod =>
      'สวิตช์หลักสำหรับแพตช์/การสร้างระบบภาพใหม่ทั้งหมดของ LodMod';

  @override
  String get labelLodMultiplier => 'ตัวคูณ LOD';

  @override
  String get tooltipLodMultiplier =>
      'ควบคุมระยะวาด LOD (Level of Detail) 0 = ปิด LOD (คุณภาพดีที่สุด ไม่มีอาการโผล่), 1 = วานิลลา, 10+ ช่วยลด AO bleed โดยไม่ปิด LOD ทั้งหมด ค่าต่ำลงให้ภาพดีขึ้นแต่อาจใช้ประสิทธิภาพมากขึ้น';

  @override
  String get labelDisableManualCulling => 'ปิด Manual Culling';

  @override
  String get tooltipDisableManualCulling =>
      'ป้องกันโมเดล/เรขาคณิตหายแบบสุ่มที่ระยะหรือมุมกล้องบางแบบ แก้ปัญหาเช่น ภายในห้างหายหลังข้ามสะพาน อาคารนอกค่ายหาย เป็นต้น โมเดล LOD ที่ดูแย่และพบไม่บ่อยจะถูกกรองออก';

  @override
  String get labelAoWidth => 'ความกว้าง AO';

  @override
  String get tooltipAoWidth =>
      'ตัวคูณความละเอียดแนวนอนของ AO ค่า AO วานิลลาทำงานที่ 1/4 ของความละเอียดหน้าจอ 2.0 = ครึ่งหนึ่งของความละเอียดหน้าจอ (คมขึ้นแต่หนัก) 1.5 เป็นสมดุลที่ดี ช่วง: 0.1 - 2.0 การตั้งเพียงแกนเดียวเป็น 2 เป็นทางเลือกที่เบากว่า';

  @override
  String get labelAoHeight => 'ความสูง AO';

  @override
  String get tooltipAoHeight =>
      'ตัวคูณความละเอียดแนวตั้งของ AO ค่า AO วานิลลาทำงานที่ 1/4 ของความละเอียดหน้าจอ 2.0 = ครึ่งหนึ่งของความละเอียดหน้าจอ (คมขึ้นแต่หนัก) 1.5 เป็นสมดุลที่ดี ช่วง: 0.1 - 2.0 การตั้งทั้งสองแกนเป็น 2.0 อาจเสียประมาณ 10 FPS ในกรณีแย่ที่สุด';

  @override
  String get labelShadowResolution => 'ความละเอียดเงา';

  @override
  String get tooltipShadowResolution =>
      'ขนาดเท็กซ์เจอร์ shadow map ค่าสูงขึ้นทำให้เงาคมขึ้นแต่ใช้ GPU มากขึ้น 2048 = วานิลลา, 4096 = อัปเกรดที่ดี, 8192 = คมมาก ต้องเป็นเลขยกกำลังของ 2 ความคมขึ้นอยู่กับทั้งความละเอียดและระยะ (ระยะไกลขึ้นหมายถึงต้องครอบคลุมพื้นที่มากขึ้น คุณภาพจึงลดลง)';

  @override
  String get labelDistanceMultiplier => 'ตัวคูณระยะ';

  @override
  String get tooltipDistanceMultiplier =>
      'คูณระยะวาดเงาของแต่ละฉาก 2.0 = มองเห็นเงาไกลเป็นสองเท่า วานิลลา: 1.0 ปิด Min/Max ด้านล่างเพื่อให้ทำงานถูกต้อง หรือใช้เพื่อจำกัดช่วงที่ตัวคูณนี้กำหนด';

  @override
  String get labelDistanceMinimum => 'ระยะขั้นต่ำ';

  @override
  String get tooltipDistanceMinimum =>
      'ค่าจำกัดระยะวาดเงาขั้นต่ำ 0 = ปิด (ไม่มีขั้นต่ำ) การตั้งประมาณ 70 พร้อมความละเอียด 8192 ให้คุณภาพใกล้วานิลลาแต่เพิ่มระยะเงาอย่างมาก';

  @override
  String get labelDistanceMaximum => 'ระยะสูงสุด';

  @override
  String get tooltipDistanceMaximum =>
      'ค่าจำกัดระยะวาดเงาสูงสุด 0 = ปิด (ไม่มีค่าสูงสุด) ควรตั้งเฉพาะเมื่อระยะเริ่มต้นของเกมทำให้เกิดปัญหาประสิทธิภาพ';

  @override
  String get labelDistancePss => 'ระยะ PSS';

  @override
  String get tooltipDistancePss =>
      'เปิดใช้การกระจายเงาแบบ PSS เพื่อให้คุณภาพเงาสม่ำเสมอขึ้น 0 = ปิด ค่าที่เหมาะสม: 0.5 - 0.9 ดูดีมากในบางพื้นที่ แต่อาจดูเบลอในบางพื้นที่ ควรตั้งให้สูงกว่าค่าระยะอื่นมาก (~1500 สำหรับพื้นที่เปิดขนาดใหญ่)';

  @override
  String get labelFilterStrengthBias => 'อคติความแรงของฟิลเตอร์';

  @override
  String get tooltipFilterStrengthBias =>
      'ปรับความแรงของฟิลเตอร์เบลอเงาในแต่ละฉาก 0 = ปิด, -1 = เงาคมขึ้น, ค่าบวก = นุ่มขึ้น แต่ละพื้นที่ใช้ความแรงต่างกัน (ป่า = นุ่มกว่า) สามารถใช้ร่วมกับ Min/Max เพื่อจำกัดช่วง';

  @override
  String get labelFilterStrengthMin => 'ความแรงฟิลเตอร์ขั้นต่ำ';

  @override
  String get tooltipFilterStrengthMin =>
      'บังคับค่าความแรงฟิลเตอร์เงาขั้นต่ำในทุกพื้นที่ 0 = ปิด ค่าเริ่มต้นของเกมแตกต่างตามฉาก (โดยทั่วไปประมาณ 4) ใช้เพื่อป้องกันไม่ให้เงาคมเกินไปในบางพื้นที่';

  @override
  String get labelFilterStrengthMax => 'ความแรงฟิลเตอร์สูงสุด';

  @override
  String get tooltipFilterStrengthMax =>
      'บังคับค่าความแรงฟิลเตอร์เงาสูงสุดในทุกพื้นที่ 0 = ปิด ค่าเริ่มต้นของเกมแตกต่างตามฉาก (โดยทั่วไปประมาณ 4) ใช้เพื่อป้องกันไม่ให้เงาเบลอเกินไปในบางพื้นที่';

  @override
  String get labelHqShadowModels => 'โมเดลเงาคุณภาพสูง';

  @override
  String get tooltipHqShadowModels =>
      'ใช้โมเดล HQ แบบเรียลไทม์สำหรับเงาแทนโมเดล LQ แบบคงที่ เงาต้นไม้จะไหวตามลมแทนที่จะหยุดนิ่ง อยู่ในขั้นทดลอง ใช้งานได้ดีในซากเมือง แต่อาจมีปัญหาในบางพื้นที่ที่พบไม่บ่อย';

  @override
  String get labelForceAllShadowModels => 'บังคับใช้โมเดลเงาทั้งหมด';

  @override
  String get tooltipForceAllShadowModels =>
      'บังคับให้โมเดลทั้งหมดสร้างเงา รวมถึงวัตถุขนาดเล็กอย่างก้อนหินและหญ้า อยู่ในขั้นทดลอง อาจมีบางกรณีที่โมเดลมองไม่เห็นแต่ยังสร้างเงา ยังไม่พบปัญหาในการทดสอบปัจจุบัน';

  @override
  String get labelDisableVignette => 'ปิด Vignette';

  @override
  String get tooltipDisableVignette =>
      'ลบเอฟเฟกต์ขอบจอมืด คัตซีนหรือหน้าจอโหลดบางส่วนอาจยังมีเอฟเฟกต์นี้ฝังอยู่ในเท็กซ์เจอร์';

  @override
  String get configAppliesOnRestart => 'มีผลหลังรีสตาร์ต';

  @override
  String get configAppliesLive => 'มีผลทันที';

  @override
  String get dropZoneBrowseFolder => 'หรือเลือกโฟลเดอร์';

  @override
  String get labelGiEnabled => 'เปิดใช้งาน Global Illumination';

  @override
  String get tooltipGiEnabled =>
      'Global Illumination แบบ FAR ให้ FPS เพิ่มขึ้นมาก แลกกับความแม่นยำของแสงบางส่วน';

  @override
  String get labelGiWorkgroupSize => 'ขนาด Workgroup ของ GI';

  @override
  String get tooltipGiWorkgroupSize =>
      'จำนวน light volume ที่ประมวลผลต่อการ dispatch ของ GI 128 = คุณภาพวานิลลา, 64/32/16 = เร็วขึ้นตามลำดับแต่หยาบขึ้น ค่าต่ำลงแลกความแม่นยำของแสงกับ FPS';

  @override
  String get labelGiMinLightExtent => 'ขนาดแสงขั้นต่ำของ GI';

  @override
  String get tooltipGiMinLightExtent =>
      'ตัดแสงขนาดเล็กที่อยู่ไกลออกจาก GI 0.0 = ไม่ตัด (แสงทั้งหมดมีผล), 0.5 = ตัดอย่างเข้ม ช่วง 0.0-1.0';

  @override
  String get cardExperimental => 'ทดลอง';

  @override
  String get lodModResetButton => 'รีเซ็ตเป็นค่าเริ่มต้น';

  @override
  String get lodModResetConfirmTitle => 'รีเซ็ตการตั้งค่า LodMod หรือไม่?';

  @override
  String get lodModResetConfirmBody =>
      'การดำเนินการนี้จะรีเซ็ตทุกช่องของ LodMod ในแท็บนี้กลับเป็นค่าเริ่มต้น ค่าปัจจุบันจะถูกเขียนทับ ต้องการดำเนินการต่อหรือไม่?';

  @override
  String get lodModResetConfirmAction => 'รีเซ็ต';

  @override
  String get lodModResetToast => 'รีเซ็ตการตั้งค่า LodMod เป็นค่าเริ่มต้นแล้ว';

  @override
  String get experimentalWarningTitle =>
      'อยู่ในขั้นทดลองและจะทำให้บางอย่างเสีย';

  @override
  String get experimentalWarningBody =>
      'การตั้งค่าเหล่านี้ข้ามข้อจำกัดของเกมที่เอนจินต้องพึ่งพา ไม่ได้รับการรองรับและทราบว่าอาจทำให้เกิดปัญหา เปิดใช้เฉพาะเมื่อคุณเข้าใจสิ่งที่กำลังทำ NAMS และตัวเรียกใช้งานจะไม่ดีบักปัญหาที่เกิดจากตัวเลือกเหล่านี้';

  @override
  String get labelFpsUncapInMenus => 'ปลดล็อก FPS ในเมนู / หน้าจอโหลด';

  @override
  String get tooltipFpsUncapInMenus =>
      'ปลดล็อกขีดจำกัด 60 FPS ระหว่างเมนูและหน้าจอโหลด ทำให้การโหลดรู้สึกเร็วขึ้นและแอนิเมชันเมนูลื่นขึ้น ปลอดภัย: เกมเพลย์ไม่ได้รับผลกระทบ\n\nสามารถเปิด/ปิดระหว่างเล่นได้หากเปิดไว้ตั้งแต่ตอนเริ่มเกม หากปิดไว้ตอนเริ่มเกม การเปิดภายหลังต้องรีสตาร์ต';

  @override
  String get labelFpsUncapInGameplay => 'ปลดล็อก FPS ระหว่างเกมเพลย์';

  @override
  String get tooltipFpsUncapInGameplay =>
      'ปลดล็อกขีดจำกัด 60 FPS ระหว่างเกมเพลย์ คำเตือน: ฟิสิกส์ แอนิเมชัน และจังหวะคัตซีนของ NieR:Automata ผูกกับขีดจำกัด 60 FPS การปลดล็อกอาจทำให้ฟิสิกส์ผิดปกติ (ความสูงการกระโดด, i-frame การหลบ), ความเร็วแอนิเมชันเปลี่ยน, เสียงคัตซีนไม่ตรง และเกิด softlock ในลำดับเหตุการณ์แบบสคริปต์ ใช้เฉพาะเมื่อคุณเข้าใจข้อแลกเปลี่ยนอย่างชัดเจน\n\nสามารถเปิด/ปิดระหว่างเล่นได้หากเปิดไว้ตั้งแต่ตอนเริ่มเกม หากปิดไว้ตอนเริ่มเกม การเปิดภายหลังต้องรีสตาร์ต';

  @override
  String get labelFpsLimit => 'จำกัด FPS';

  @override
  String get tooltipFpsLimit =>
      'ขีดจำกัด FPS ที่ใช้เมื่อเปิดการปลดล็อก 0 = ไม่จำกัด มิฉะนั้น 60-1000 (NAMS จะปรับค่าที่อยู่นอกช่วง) ค่าต่ำกว่า 60 จะถูกปรับขึ้น เพราะ spin-wait loop ภายในเกมไม่รองรับ frametime ที่ยาวกว่าเป้าหมายวานิลลา 60 FPS เคล็ดลับ: จำกัดที่ครึ่งหนึ่งของรีเฟรชเรตจอจะให้การเคลื่อนไหวลื่นกว่าวานิลลา 60 เช่น 72 บน 144Hz, 82 บน 165Hz, 120 บน 240Hz';

  @override
  String get tutorialValidateModel =>
      'แจ้งให้คุณทราบเมื่อโมเดลของม็อดเสีย แทนที่จะล้มเหลวแบบเงียบ';

  @override
  String get labelValidateScripts => 'ตรวจสอบสคริปต์';

  @override
  String get tooltipValidateScripts =>
      'แสดงข้อผิดพลาดของสคริปต์เป็นกล่องข้อความ แทนที่จะทำให้เกมล่มแบบเงียบ';

  @override
  String get previewValidationDialog => 'กล่องข้อความตรวจสอบ';

  @override
  String get previewScriptErrorDialog => 'กล่องข้อความข้อผิดพลาดสคริปต์';

  @override
  String get labelLoadingStallHints => 'คำแนะนำเมื่อการโหลดค้าง';

  @override
  String get tooltipLoadingStallHints =>
      'แสดงคำแนะนำเพิ่มขึ้นตามระยะเวลาเมื่อหน้าจอโหลดใช้เวลานานเกินไป ช่วยระบุไฟล์ม็อดที่หายหรือถูกลบ';

  @override
  String get labelFixWindTimerBug => 'แก้บั๊กตัวจับเวลาลม';

  @override
  String get tooltipFixWindTimerBug =>
      'แก้บั๊กวานิลลาที่แอนิเมชันลมหยุดหลังเวลาเล่นสูงสุด โดยใช้ตัวคูณความเร็วของเกมแทน';

  @override
  String get labelDisablePluginLoading => 'ปิดการโหลดปลั๊กอิน';

  @override
  String get tooltipDisablePluginLoading =>
      'ข้ามการโหลด DLL ปลั๊กอินทั้งหมด (เช่น เวิร์กสเปซ YoRHa Protocol) ฟีเจอร์เอนจินทั้งหมดของ NAMS ยังทำงานตามปกติ';

  @override
  String get labelDisableContentFeatures => 'ปิดฟีเจอร์เนื้อหา';

  @override
  String get tooltipDisableContentFeatures =>
      'สวิตช์หลักสำหรับฟีเจอร์ชั้นเนื้อหาทั้งหมด เมื่อเปิดตัวเลือกนี้ NAMS จะทำงานเป็นชั้นเอนจินล้วน (แก้เมาส์, ตรวจสอบข้อมูล, ปรับฮีป, แก้การล่ม) โดยไม่มีการรองรับม็อดไอเท็ม / อาวุธ / ชุด / เควสต์ / เครื่องประดับ เหมาะสำหรับวัดประสิทธิภาพหรือแยกปัญหาเอนจินออกจากปัญหาเนื้อหา';

  @override
  String get labelContentItems => 'ไอเท็ม / อาวุธ / ร้านค้า';

  @override
  String get tooltipContentItems =>
      'ไอเท็ม อาวุธ ชุด และรายการร้านค้าแบบกำหนดเอง ปิดเพื่อเล่นโดยไม่มีม็อดที่เกี่ยวข้องกับไอเท็ม ต้องรีสตาร์ตเกม';

  @override
  String get labelContentAccessories => 'เครื่องประดับ';

  @override
  String get tooltipContentAccessories =>
      'เครื่องประดับแบบกำหนดเอง (หน้ากาก, Lunar Tear, หน้ากาก Masamune เป็นต้น) และขั้นตอนสวม/ถอดเครื่องประดับ ปิดเพื่อเล่นโดยไม่มีม็อดเครื่องประดับ ต้องรีสตาร์ตเกม';

  @override
  String get labelContentAssembleMeshes => 'โมเดลผู้เล่น';

  @override
  String get tooltipContentAssembleMeshes =>
      'โมเดลผู้เล่นแบบกำหนดเอง (สลับ mesh, แทนที่ผม / ชุด / หน้ากาก) ปิดเพื่อแสดงโมเดลผู้เล่นวานิลลาโดยไม่เปลี่ยนแปลง ต้องรีสตาร์ตเกม';

  @override
  String get labelContentQuestIntegration => 'เควสต์ / เมล / เสียงพากย์';

  @override
  String get tooltipContentQuestIntegration =>
      'เควสต์แบบกำหนดเอง เมลแบบกำหนดเอง เสียงพากย์แบบกำหนดเอง และการเชื่อมต่อ UI เควสต์ที่เปิดใช้งานเนื้อหาเหล่านี้ ปิดเพื่อเล่นโดยไม่มีม็อดเควสต์ ต้องรีสตาร์ตเกม';

  @override
  String get labelContentEffectsApplier => 'กฎเอฟเฟกต์';

  @override
  String get tooltipContentEffectsApplier =>
      'นำกฎเอฟเฟกต์ของอาวุธ / ชุดไปใช้กับค่าสถานะผู้เล่นทุกเฟรม (ตัวคูณความเสียหาย การปรับการหลบ ภูมิคุ้มกัน เป็นต้น)';

  @override
  String get labelContentEquipTracker => 'ตัวติดตามการสวมใส่';

  @override
  String get tooltipContentEquipTracker =>
      'ตรวจจับการสวม/ถอดอาวุธ ใช้ขับเคลื่อนกฎเอฟเฟกต์และ callback ของ SDK ตอนสวมใส่';

  @override
  String get labelContentMcd => 'ข้อความแบบกำหนดเอง';

  @override
  String get tooltipContentMcd =>
      'ปรับข้อความในเกมแบบกำหนดเอง (ชื่อไอเท็ม คำอธิบาย และบทสนทนาที่ม็อดกำหนด)';

  @override
  String get labelContentBuddyRubySelector => 'ตัวเลือกชุดคู่หู (ทดลอง)';

  @override
  String get tooltipContentBuddyRubySelector =>
      'แพตช์สคริปต์บทสนทนาคู่หูส่วนกลางเพื่อเพิ่มรายการ \"เปลี่ยนชุด\" ที่แสดงชุดคู่หูจากม็อด ปิดหากสคริปต์บทสนทนาที่แพตช์แล้วทำให้ไม่เสถียรหรือรบกวนม็อดสคริปต์อื่น';

  @override
  String get cardContentFeatures => 'ฟีเจอร์เนื้อหา';

  @override
  String get contentFeaturesDescription =>
      'สวิตช์แยกแต่ละฟีเจอร์ของชั้นเนื้อหา ค่าเริ่มต้นทั้งหมดเป็นเปิด ใช้เพื่อจำกัดปัญหาไปยังระบบย่อยเฉพาะ ต้องรีสตาร์ตเกม';

  @override
  String get labelDisableReShadeLoading => 'ปิดการโหลด ReShade';

  @override
  String get tooltipDisableReShadeLoading =>
      'ข้ามการตรวจหา DLL ของ ReShade อัตโนมัติจากโฟลเดอร์ reshade/ และจะไม่โหลดอีก';

  @override
  String get labelDisable3dmigotoLoading => 'ปิดการโหลด 3DMigoto';

  @override
  String get tooltipDisable3dmigotoLoading =>
      'ข้ามการโหลดรันไทม์ 3DMigoto จาก thirdparty/3dmigoto/ ปิดเพื่อหยุดโหลดม็อด shader';

  @override
  String get labelDisableTextureInjection => 'ปิดการฉีดเท็กซ์เจอร์';

  @override
  String get tooltipDisableTextureInjection =>
      'ข้ามการฉีดเท็กซ์เจอร์จากโฟลเดอร์ม็อด มีประโยชน์สำหรับแยกปัญหา หรือเมื่อไม่ต้องการใช้ม็อดเท็กซ์เจอร์แม้จะติดตั้งไว้';

  @override
  String get labelOutfitSwapVisualEffects => 'เอฟเฟกต์ภาพตอนสลับชุด';

  @override
  String get tooltipOutfitSwapVisualEffects =>
      'เล่นเอฟเฟกต์ภาพระหว่าง hot-swap ชุด: แอนิเมชัน blinder ตอน Pod ปรากฏ ม่าน และฟิลเตอร์ glitch ของหน้าจอแฮ็ก ปิดเพื่อสลับทันทีโดยไม่มีเอฟเฟกต์ โมเดลยังคงโหลดใหม่ มีผลทันที ไม่ต้องรีสตาร์ต';

  @override
  String get labelExperimentalDefaultOutfits => 'ชุดเริ่มต้น (ทดลอง)';

  @override
  String get tooltipExperimentalDefaultOutfits =>
      'ให้คุณกำหนดม็อดชุดที่ติดตั้งให้ทำงานตั้งแต่เกมเริ่ม ราวกับวางไฟล์ไว้ในโฟลเดอร์ data ของเกม เมื่อเปิด แผงรายละเอียดม็อดจะแสดงปุ่มดาวสำหรับโมเดลผู้เล่นแต่ละตัวเพื่อกำหนดเป็นค่าเริ่มต้นตอนบูต ปิดไว้เป็นค่าเริ่มต้นระหว่างที่ฟีเจอร์ยังปรับเสถียรภาพ ต้องรีสตาร์ตเกม';

  @override
  String get labelDisableSplashScreen => 'ปิด Splash Screen';

  @override
  String get tooltipDisableSplashScreen =>
      'ข้ามหน้าต่าง splash ตอนเริ่มเกมที่แสดงระหว่างโหลด เกมต้นฉบับเปิดหน้าต่างก่อนพร้อมใช้งาน ทำให้เกิดอาการปรับขนาดและกระพริบ NAMS ทำ splash ให้สมบูรณ์เพื่อให้หน้าต่างแสดงเมื่อพร้อมเท่านั้น การเปิดตัวเลือกนี้จะนำอาการเริ่มเกมแบบวานิลลากลับมา';

  @override
  String get tooltipValidateModelDataSettings =>
      'แสดงข้อผิดพลาดตรวจสอบโมเดลเป็นกล่องข้อความ แทนการล้มเหลวแบบเงียบ';

  @override
  String get heapDefault => 'ค่าเริ่มต้น';

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
  String get heapScriptEngine => 'เอนจินสคริปต์';

  @override
  String get heapScriptEngineDesc => 'สำหรับม็อดสคริปต์ซับซ้อน (mRuby / HAP)';

  @override
  String get heapPlayerModels => 'โมเดลผู้เล่น';

  @override
  String get heapPlayerModelsDesc => 'สำหรับม็อดแทนที่โมเดลผู้เล่นขนาดใหญ่';

  @override
  String get heapPlayerTextures => 'เท็กซ์เจอร์ผู้เล่น';

  @override
  String get heapPlayerTexturesDesc =>
      'สำหรับม็อดเท็กซ์เจอร์ผู้เล่นความละเอียดสูง';

  @override
  String get heapEnemyBgModels => 'โมเดลศัตรู/ฉากหลัง';

  @override
  String get heapEnemyBgModelsDesc => 'สำหรับม็อดโมเดลศัตรูหรือสภาพแวดล้อม';

  @override
  String get heapEnemyBgTextures => 'เท็กซ์เจอร์ศัตรู/ฉากหลัง';

  @override
  String get heapEnemyBgTexturesDesc =>
      'สำหรับเท็กซ์เจอร์ศัตรู/สภาพแวดล้อมความละเอียดสูง';

  @override
  String get tutorialLodModEnable =>
      'เปิดตัวเลือกนี้เพื่อภาพที่ดีขึ้น นี่เป็นการตั้งค่าที่มีผลมากที่สุด';

  @override
  String get tutorialLodModShadowRes => 'ค่าสูงขึ้นหมายถึงเงาคมขึ้น แนะนำ 8192';

  @override
  String get tutorialLodModComparison =>
      'คลิกภาพเปรียบเทียบเพื่อดูความแตกต่างแบบเต็มหน้าจอ';

  @override
  String get comparisonVanilla => 'วานิลลา';

  @override
  String get comparisonDefaultEnabled => 'ค่าเริ่มต้น (เปิดใช้งาน)';

  @override
  String get comparisonAo05x => 'AO 0.5x';

  @override
  String get comparisonAo20x => 'AO 2.0x';

  @override
  String get comparisonVignetteOn => 'เปิด VIGNETTE';

  @override
  String get comparisonVignetteOff => 'ปิด VIGNETTE';

  @override
  String get comparison2048 => '2048';

  @override
  String get comparison8192 => '8192';

  @override
  String get comparisonDefault => 'ค่าเริ่มต้น';

  @override
  String get comparison20x => '2.0x';

  @override
  String get comparisonPssMinus5 => 'PSS -5.0';

  @override
  String get comparisonBiasMinus5 => 'BIAS -5.0';

  @override
  String get comparisonOff => 'ปิด';

  @override
  String get comparison30 => '3.0';

  @override
  String get comparisonHqForceAll => 'HQ + บังคับทั้งหมด';

  @override
  String get tutorialKeybind =>
      'คลิกเพื่อเปลี่ยนปุ่มลัด จากนั้นกดปุ่มใดก็ได้เพื่อกำหนด';

  @override
  String get tutorialDamageMultiplier =>
      'ปรับแต่งเกมเพลย์ เพิ่มความเสียหาย เปิดพลังชีวิตไม่จำกัด และอื่น ๆ';

  @override
  String get labelOpenWorkspace => 'เปิดเวิร์กสเปซ';

  @override
  String get tooltipOpenWorkspace => 'เปิดเวิร์กสเปซ YoRHa Protocol ภายในเกม';

  @override
  String get labelFreezeGame => 'หยุดเกมชั่วคราว';

  @override
  String get tooltipFreezeGame =>
      'หยุด/เล่นเกมต่อ เหมาะสำหรับภาพหน้าจอและการเลื่อนทีละเฟรม';

  @override
  String get labelMaxSpeed => 'ความเร็วสูงสุด';

  @override
  String get tooltipMaxSpeed =>
      'เปิด/ปิดความเร็วเกมสูงสุดสำหรับเดินทางเร็วหรือทดสอบ';

  @override
  String get labelFreeCam => 'กล้องอิสระ';

  @override
  String get tooltipFreeCam =>
      'เปิด/ปิดกล้องอิสระ รองรับคีย์บอร์ด/เมาส์และคอนโทรลเลอร์เต็มรูปแบบ';

  @override
  String get labelPhaseJump => 'Phase Jump';

  @override
  String get tooltipPhaseJump =>
      'กระโดดไปยังเฟส/ฉากเกมที่เลือกไว้ กำหนดเป้าหมายภายในเกม';

  @override
  String get labelToggleInput => 'เปิด/ปิดอินพุต';

  @override
  String get tooltipToggleInput =>
      'เปิดหรือปิดอินพุตของเกมขณะเวิร์กสเปซเปิดอยู่';

  @override
  String get labelAdvanceFrame => 'เลื่อนไปหนึ่งเฟรม';

  @override
  String get tooltipAdvanceFrame =>
      'เลื่อนไปข้างหน้าหนึ่งเฟรมขณะเกมหยุด กดค้างเพื่อเลื่อนเร็วขึ้น';

  @override
  String get labelDevMode => 'โหมดนักพัฒนา';

  @override
  String get tooltipDevMode =>
      'เปิด/ปิดโหมดนักพัฒนา เปิดใช้งานปุ่มทดสอบ penetration/stress และเครื่องมือดีบัก';

  @override
  String get labelWarpSave1 => 'บันทึก Warp 1';

  @override
  String get tooltipWarpSave1 => 'บันทึกตำแหน่งและการหมุนปัจจุบันลงช่อง Warp 1';

  @override
  String get labelWarpGoto1 => 'Warp ไปยัง 1';

  @override
  String get tooltipWarpGoto1 => 'เทเลพอร์ตไปยังตำแหน่งที่บันทึกในช่อง Warp 1';

  @override
  String get labelWarpSave2 => 'บันทึก Warp 2';

  @override
  String get tooltipWarpSave2 => 'บันทึกตำแหน่งและการหมุนปัจจุบันลงช่อง Warp 2';

  @override
  String get labelWarpGoto2 => 'Warp ไปยัง 2';

  @override
  String get tooltipWarpGoto2 => 'เทเลพอร์ตไปยังตำแหน่งที่บันทึกในช่อง Warp 2';

  @override
  String get labelGlobalKeybinds => 'ปุ่มลัดส่วนกลาง';

  @override
  String get tooltipGlobalKeybinds => 'ปุ่มลัดยังทำงานเมื่อปิดเวิร์กสเปซ';

  @override
  String get labelLoadingSpeedup => 'เร่งความเร็วการโหลด';

  @override
  String get tooltipLoadingSpeedup => 'ทำให้หน้าจอโหลดเร็วขึ้น';

  @override
  String get labelShaders => 'Shaders';

  @override
  String get tooltipShaders => 'Shader ของเวิร์กสเปซ ปิดเพื่อเพิ่มประสิทธิภาพ';

  @override
  String get labelSound => 'เสียง';

  @override
  String get tooltipSound => 'เสียงตอบสนองของ UI เวิร์กสเปซ';

  @override
  String get labelDamageMultiplier => 'ตัวคูณความเสียหาย';

  @override
  String get tooltipDamageMultiplier => '2.0 = ความเสียหายสองเท่า';

  @override
  String get labelSyncEnemyLevels => 'ปรับระดับศัตรูตามผู้เล่น';

  @override
  String get tooltipSyncEnemyLevels => 'ทำให้ระดับศัตรูเท่ากับระดับของคุณ';

  @override
  String get labelInfiniteHealth => 'พลังชีวิตไม่จำกัด';

  @override
  String get tooltipInfiniteHealth => 'ไม่ได้รับความเสียหาย';

  @override
  String get labelInfiniteJump => 'กระโดดไม่จำกัด';

  @override
  String get tooltipInfiniteJump => 'กระโดดกลางอากาศได้ไม่จำกัด';

  @override
  String get labelNoPodCooldown => 'Pod ไม่มีคูลดาวน์';

  @override
  String get tooltipNoPodCooldown => 'โปรแกรม Pod ไม่มีเวลาคูลดาวน์';

  @override
  String get labelInfiniteAirDash => 'แดชกลางอากาศไม่จำกัด';

  @override
  String get tooltipInfiniteAirDash => 'แดชกลางอากาศได้ไม่จำกัด';

  @override
  String get labelAutoStart => 'เริ่มอัตโนมัติ';

  @override
  String get tooltipAutoStart => 'เริ่มระบบสุ่มอัตโนมัติเมื่อเปิดเกม';

  @override
  String get labelGroundEnemies => 'ศัตรูภาคพื้นดิน';

  @override
  String get tooltipGroundEnemies => 'สุ่มจุดเกิดของศัตรูภาคพื้นดิน';

  @override
  String get labelFlyingEnemies => 'ศัตรูบิน';

  @override
  String get tooltipFlyingEnemies => 'สุ่มจุดเกิดของศัตรูบิน';

  @override
  String get labelAllowBigEnemies => 'อนุญาตศัตรูขนาดใหญ่';

  @override
  String get tooltipAllowBigEnemies => 'อนุญาตศัตรูขนาดใหญ่';

  @override
  String get labelIncludeDlcEnemies => 'รวมศัตรู DLC';

  @override
  String get tooltipIncludeDlcEnemies => 'รวมศัตรูจาก DLC';

  @override
  String get tutorialCameraAccel => 'ลบการเร่งเมาส์เพื่อให้อินพุตแบบ 1:1';

  @override
  String get tutorialWipBanner => 'ฟีเจอร์เหล่านี้จะมาในอัปเดต NAMS ในอนาคต';

  @override
  String get labelFixCameraAcceleration => 'แก้การเร่งกล้อง';

  @override
  String get tooltipFixCameraAcceleration =>
      'การเคลื่อนไหวเมาส์แบบเส้นตรง 1:1 ลบ deadzone และเส้นโค้งเร่งจากการหมุนกล้อง';

  @override
  String get labelSensitivity => 'ความไว';

  @override
  String get tooltipSensitivity =>
      'ตัวคูณความไวกล้อง ค่าสูงขึ้นทำให้หมุนเร็วขึ้น 2.0 เป็นค่าเริ่มต้นที่ดี';

  @override
  String get labelAimSensitivity => 'ความไวการเล็ง';

  @override
  String get tooltipAimSensitivity =>
      'ความไวการเล็งสำหรับมุมมองบนลงล่าง/ด้านข้าง 0.001 สำหรับประมาณ 3500 DPI, 0.003 สำหรับประมาณ 800 DPI';

  @override
  String get labelAimOutputMultiplier => 'ตัวคูณเอาต์พุตการเล็ง';

  @override
  String get tooltipAimOutputMultiplier =>
      'ตัวคูณตรงสำหรับความเร็วการเคลื่อนเป้าเล็งหลัง normalization ค่าสูงขึ้นทำให้เป้าเล็งเร็วขึ้น ผู้ใช้ส่วนใหญ่ไม่จำเป็นต้องเปลี่ยน';

  @override
  String get labelDisablePodPet => 'ปิดการลูบ Pod';

  @override
  String get tooltipDisablePodPet =>
      'ปิดแอนิเมชันลูบ Pod ที่ถูกเรียกจากการเคลื่อนไหวเมาส์ แนะนำ';

  @override
  String get labelDebugMenuKey => 'ปุ่มเมนูดีบัก';

  @override
  String get tooltipDebugMenuKey =>
      'เปิดเมนูดีบักที่เข้าถึงได้หลังจบเกม ปกติต้องใช้คอนโทรลเลอร์ การกำหนดปุ่มนี้ทำให้ใช้คีย์บอร์ดได้';

  @override
  String get labelThirdPersonMode => 'แก้กล้องบุคคลที่สาม';

  @override
  String get tooltipThirdPersonMode =>
      'ใช้อินพุตเมาส์ดิบสำหรับกล้องบุคคลที่สาม ให้การควบคุมกล้องที่ลื่นและตรง โดยไม่สนการตั้งค่าเมาส์ในเกม';

  @override
  String get labelThirdPersonCharFollow => 'กล้องติดตามตัวละคร';

  @override
  String get tooltipThirdPersonCharFollow =>
      'คงการติดตามกล้องอัตโนมัติของเกมระหว่างเคลื่อนไหว เหมือนใช้คอนโทรลเลอร์';

  @override
  String get labelThirdPersonSensX => 'ความไวแนวนอน';

  @override
  String get tooltipThirdPersonSensX =>
      'ความเร็วกล้องซ้าย/ขวา ค่าติดลบจะกลับทิศทาง';

  @override
  String get labelThirdPersonSensY => 'ความไวแนวตั้ง';

  @override
  String get tooltipThirdPersonSensY =>
      'ความเร็วกล้องขึ้น/ลง ค่าติดลบจะกลับทิศทาง';

  @override
  String get labelAimMode => 'แก้การเล็งของ Pod';

  @override
  String get tooltipAimMode =>
      'ลบ clamp และ deadzone จากการเล็ง Pod/หุ่นในมุมมองบนลงล่างและด้านข้าง';

  @override
  String get labelAimCrosshair => 'โหมดเป้าเล็ง';

  @override
  String get tooltipAimCrosshair =>
      'เล็งด้วยการชี้: Pod จะเล็งไปยังเป้าเล็งที่ตามเมาส์ คล้ายเกม twin-stick shooter เป้าเล็งสร้างจากองค์ประกอบ UI เดิมของเกม จึงดูและรู้สึกเหมือนเป็นส่วนหนึ่งของ NieR:Automata ตั้งแต่แรก แนะนำ';

  @override
  String get labelAimCrosshairAlways => 'แสดงเป้าเล็งตลอดเวลา';

  @override
  String get tooltipAimCrosshairAlways =>
      'คงเป้าเล็งไว้แม้ไม่ได้ยิง ปิด = แสดงเฉพาะขณะยิง';

  @override
  String get naiomNeedsCrosshair => 'เปิดโหมดเป้าเล็งเพื่อใช้ตัวเลือกนี้';

  @override
  String get labelAimSensX => 'ความไวการเล็งแนวนอน';

  @override
  String get tooltipAimSensX =>
      'ตัวคูณความเร็วการเล็งซ้าย/ขวา ค่าติดลบจะกลับทิศทาง';

  @override
  String get labelAimSensY => 'ความไวการเล็งแนวตั้ง';

  @override
  String get tooltipAimSensY =>
      'ตัวคูณความเร็วการเล็งขึ้น/ลง ค่าติดลบจะกลับทิศทาง';

  @override
  String get labelDisableTapEvade => 'ปิดการหลบด้วยการแตะสองครั้ง';

  @override
  String get tooltipDisableTapEvade =>
      'หยุดการหลบจากการแตะปุ่มเคลื่อนไหวสองครั้ง ใช้เมื่อกำหนดปุ่ม Evade แยกไว้เท่านั้น';

  @override
  String get labelCustomCursorMenu => 'เคอร์เซอร์เมนู';

  @override
  String get tooltipCustomCursorMenu =>
      'เคอร์เซอร์เมาส์แบบกำหนดเองสำหรับเมนู (ไฟล์ .cur หรือ .ani) เว้นว่าง = ใช้เคอร์เซอร์เริ่มต้นที่รวมมา';

  @override
  String get labelCustomCursorHacking => 'เคอร์เซอร์มินิเกมแฮ็ก';

  @override
  String get tooltipCustomCursorHacking =>
      'เคอร์เซอร์แบบกำหนดเองสำหรับมินิเกมแฮ็ก เว้นว่าง = ใช้เคอร์เซอร์เดียวกับเมนู';

  @override
  String get labelDisableDefaultCursor => 'ใช้เคอร์เซอร์ระบบต่อไป';

  @override
  String get tooltipDisableDefaultCursor =>
      'ไม่ใช้เคอร์เซอร์ที่รวมมา ให้ใช้เคอร์เซอร์ Windows ปกติ เว้นแต่คุณเลือกไฟล์ของตนเองด้านบน';

  @override
  String get labelBindMoveForward => 'เดินหน้า';

  @override
  String get tooltipBindMoveForward => 'เหมือนกับปุ่มที่ตั้งไว้ในเกม';

  @override
  String get labelBindMoveBackward => 'ถอยหลัง';

  @override
  String get tooltipBindMoveBackward => 'เหมือนกับปุ่มที่ตั้งไว้ในเกม';

  @override
  String get labelBindMoveLeft => 'ไปทางซ้าย';

  @override
  String get tooltipBindMoveLeft => 'เหมือนกับปุ่มที่ตั้งไว้ในเกม';

  @override
  String get labelBindMoveRight => 'ไปทางขวา';

  @override
  String get tooltipBindMoveRight => 'เหมือนกับปุ่มที่ตั้งไว้ในเกม';

  @override
  String get labelBindJump => 'กระโดด';

  @override
  String get tooltipBindJump => 'เหมือนกับปุ่มที่ตั้งไว้ในเกม';

  @override
  String get labelBindWalk => 'เดิน';

  @override
  String get tooltipBindWalk => 'กดค้างเพื่อเดินช้า';

  @override
  String get labelBindAutoRun => 'วิ่งอัตโนมัติ';

  @override
  String get tooltipBindAutoRun => 'วิ่งต่อโดยไม่ต้องกดปุ่มเคลื่อนไหวค้าง';

  @override
  String get labelBindLightAttack => 'โจมตีเบา';

  @override
  String get tooltipBindLightAttack => 'เหมือนกับปุ่มที่ตั้งไว้ในเกม';

  @override
  String get labelBindHeavyAttack => 'โจมตีหนัก';

  @override
  String get tooltipBindHeavyAttack => 'เหมือนกับปุ่มที่ตั้งไว้ในเกม';

  @override
  String get labelBindFire => 'ยิง / Pod Dash';

  @override
  String get tooltipBindFire =>
      'ยิงด้วย Pod เมื่อใช้ร่วมกับ Jump จะทำ Pod Dash ได้ แม้เปิด Auto-Fire อยู่';

  @override
  String get labelBindProgram => 'ใช้โปรแกรม';

  @override
  String get tooltipBindProgram => 'ใช้โปรแกรมของ Pod / ยูนิตบิน';

  @override
  String get labelBindLockOn => 'ล็อกเป้าหมาย';

  @override
  String get tooltipBindLockOn => 'ล็อกเป้าหมายปัจจุบัน';

  @override
  String get labelBindUse => 'ใช้ / โต้ตอบ';

  @override
  String get tooltipBindUse => 'เหมือนกับปุ่มที่ตั้งไว้ในเกม';

  @override
  String get labelBindSelfDestruct => 'ทำลายตัวเอง';

  @override
  String get tooltipBindSelfDestruct => 'เหมือนกับปุ่มที่ตั้งไว้ในเกม';

  @override
  String get labelBindLight => 'เปิด/ปิดไฟ';

  @override
  String get tooltipBindLight => 'เหมือนกับปุ่มที่ตั้งไว้ในเกม';

  @override
  String get labelBindResetCamera => 'รีเซ็ตกล้อง';

  @override
  String get tooltipBindResetCamera => 'เหมือนกับปุ่มที่ตั้งไว้ในเกม';

  @override
  String get labelBindSwitchWeapon => 'สลับชุดอาวุธ';

  @override
  String get tooltipBindSwitchWeapon => 'วนเปลี่ยนชุดอาวุธที่สวมใส่';

  @override
  String get labelBindNextProgram => 'โปรแกรมถัดไป';

  @override
  String get tooltipBindNextProgram => 'เลือกโปรแกรม Pod ถัดไป';

  @override
  String get labelBindPreviousProgram => 'โปรแกรมก่อนหน้า';

  @override
  String get tooltipBindPreviousProgram => 'เลือกโปรแกรม Pod ก่อนหน้า';

  @override
  String get labelBindMenuUp => 'เมนูขึ้น';

  @override
  String get tooltipBindMenuUp => 'เลื่อนขึ้นในเมนู';

  @override
  String get labelBindMenuDown => 'เมนูลง';

  @override
  String get tooltipBindMenuDown => 'เลื่อนลงในเมนู';

  @override
  String get labelBindMenuLeft => 'เมนูซ้าย';

  @override
  String get tooltipBindMenuLeft => 'เลื่อนไปทางซ้ายในเมนู';

  @override
  String get labelBindMenuRight => 'เมนูขวา';

  @override
  String get tooltipBindMenuRight => 'เลื่อนไปทางขวาในเมนู';

  @override
  String get labelBindMenuOpen => 'เปิดเมนู';

  @override
  String get tooltipBindMenuOpen => 'เปิดเมนูระบบ';

  @override
  String get labelBindMenuBack => 'ย้อนกลับ / ปิดเมนู';

  @override
  String get tooltipBindMenuBack => 'ย้อนกลับในเมนู หรือปิดเมื่ออยู่ระดับบนสุด';

  @override
  String get labelBindMenuEnter => 'ยืนยันเมนู / ข้ามบทสนทนา';

  @override
  String get tooltipBindMenuEnter => 'เข้าสู่เมนูย่อยที่เลือกหรือข้ามบทสนทนา';

  @override
  String get labelBindShortcutMenu => 'เมนูทางลัด';

  @override
  String get tooltipBindShortcutMenu => 'เปิดเมนูทางลัด';

  @override
  String get labelBindEvade => 'หลบ (ปุ่มเฉพาะ)';

  @override
  String get tooltipBindEvade =>
      'หลบไปตามทิศทางการเคลื่อนไหวปัจจุบันด้วยปุ่มเดียว ไม่ต้องแตะสองครั้ง';

  @override
  String get labelBindAutoFire => 'เปิด/ปิดยิงอัตโนมัติ';

  @override
  String get tooltipBindAutoFire =>
      'เปิดหรือปิดการยิง Pod ต่อเนื่อง เพื่อไม่ต้องกดปุ่มยิงค้าง';

  @override
  String get labelBindNextItem => 'ไอเท็มถัดไป';

  @override
  String get tooltipBindNextItem =>
      'สลับไปยังไอเท็มด่วนถัดไปทันที ทำงานเบื้องหลังโดยไม่แสดงเมนูไอเท็มในเกม ซึ่งเป็นพฤติกรรมที่ตั้งใจไว้';

  @override
  String get labelBindPreviousItem => 'ไอเท็มก่อนหน้า';

  @override
  String get tooltipBindPreviousItem =>
      'สลับไปยังไอเท็มด่วนก่อนหน้าทันที ทำงานเบื้องหลังโดยไม่แสดงเมนูไอเท็มในเกม ซึ่งเป็นพฤติกรรมที่ตั้งใจไว้';

  @override
  String get labelBindUseItem => 'ใช้ไอเท็ม';

  @override
  String get tooltipBindUseItem =>
      'ใช้ไอเท็มด่วนที่เลือกทันที ทำงานเบื้องหลังโดยไม่แสดงเมนูไอเท็มในเกม ซึ่งเป็นพฤติกรรมที่ตั้งใจไว้';

  @override
  String get labelBindThirdPersonToggle => 'เปิด/ปิดการแก้กล้อง';

  @override
  String get tooltipBindThirdPersonToggle =>
      'เปิดหรือปิดการแก้กล้องบุคคลที่สามระหว่างเล่น';

  @override
  String get labelBindAimToggle => 'เปิด/ปิดการแก้การเล็ง';

  @override
  String get tooltipBindAimToggle =>
      'เปิดหรือปิดการแก้การเล็งของ Pod ระหว่างเล่น';

  @override
  String get keybindUnbound => 'ไม่ได้กำหนดปุ่ม';

  @override
  String keybindConflict(String other) {
    return 'ใช้งานโดย: $other เช่นกัน';
  }

  @override
  String get keybindMouseNotSupported =>
      'ปุ่มเมาส์ใช้กับการกระทำนี้ไม่ได้ ต้องใช้ปุ่มคีย์บอร์ด';

  @override
  String get naiomResetConfirmTitle => 'รีเซ็ตการตั้งค่า NAIOM หรือไม่?';

  @override
  String get naiomResetConfirmBody =>
      'การดำเนินการนี้จะรีเซ็ตการตั้งค่ากล้อง การเล็ง เคอร์เซอร์ และปุ่มทั้งหมดในแท็บนี้กลับเป็นค่าเริ่มต้น ระบบจะยังไม่เขียนไฟล์จนกว่าคุณกด Save ดังนั้นยังสามารถละทิ้งภายหลังได้ ต้องการดำเนินการต่อหรือไม่?';

  @override
  String get naiomControllerNote =>
      'เล่นด้วยคอนโทรลเลอร์หรือไม่? การตั้งค่าเหล่านี้ออกแบบสำหรับเมาส์และคีย์บอร์ด แต่บางรายการ โดยเฉพาะการแก้กล้องและการเล็ง จะมีผลกับคอนโทรลเลอร์ด้วย หากกลับไปเล่นด้วยคอนโทรลเลอร์ ให้ปิดการตั้งค่าเหล่านั้นก่อนเพื่อคืนความรู้สึกแบบเดิมของเกมแพด';

  @override
  String get cardCheatEngine => 'CHEAT ENGINE';

  @override
  String get cheatTableConvertDesc =>
      'มีตาราง Cheat Engine (.CT) ที่ใช้กับ NAMS ไม่ได้หรือไม่? แก้ไขได้ที่นี่ สำเนาที่แก้แล้วจะถูกบันทึกไว้ข้างไฟล์ต้นฉบับ';

  @override
  String get cheatTableConvertButton => 'แก้ไขตารางโกง...';

  @override
  String cheatTableConvertSuccess(String file) {
    return 'แก้ไขแล้ว! บันทึกเป็น $file';
  }

  @override
  String get cheatTableConvertNone =>
      'ตารางนี้ใช้กับ NAMS ได้อยู่แล้ว ไม่ต้องแก้ไข';

  @override
  String get cheatTableConvertError =>
      'ไม่สามารถแก้ไขตารางนี้ได้ โปรดตรวจสอบว่าเป็นไฟล์ .CT ที่ถูกต้อง';

  @override
  String get naiomBetaBadge => 'เบต้า';

  @override
  String get naiomRestartBadge => 'รีสตาร์ต';

  @override
  String get naiomRestartTooltip => 'มีผลหลังรีสตาร์ตเกม';

  @override
  String get naiomNeedsCameraFix =>
      'เปิด Fix Camera Acceleration เพื่อใช้ตัวเลือกนี้';

  @override
  String get naiomNeedsThirdPerson =>
      'เปิด Third-Person Camera Fix เพื่อใช้ตัวเลือกนี้';

  @override
  String get naiomNeedsAimMode => 'เปิด Fix Pod Aiming เพื่อใช้ตัวเลือกนี้';

  @override
  String get naiomCrosshairOverrides =>
      'ไม่ใช้ขณะเปิด Crosshair Mode เพราะเป้าเล็งมีความเร็วของตัวเอง';

  @override
  String get naiomThirdPersonRestartNote =>
      'การเปิดตัวเลือกนี้ต้องรีสตาร์ตเกม แต่การปิดทำได้ระหว่างเล่น';

  @override
  String get naiomTapEvadeWarning =>
      'ยังไม่ได้กำหนดปุ่ม Evade! หากปิด Tap Evade และไม่มีปุ่ม Evade เฉพาะ คุณจะหลบไม่ได้เลย โปรดกำหนดปุ่ม Evade ใต้ Non-Standard Actions';

  @override
  String get naiomCrosshairNote =>
      'เป้าเล็งจะแสดงเฉพาะเกมเพลย์มุมมองบนลงล่าง / ด้านข้างแบบปกติเมื่อใช้เมาส์ หากบางจุดไม่แสดง โดยทั่วไปเป็นพฤติกรรมปกติ ไม่ใช่บั๊ก';

  @override
  String get naiomBindingsIntro =>
      'ปุ่มเพิ่มเติมจากการควบคุมเดิมของเกม ปุ่มเดิมยังทำงาน การเปลี่ยนแปลงมีผลหลังบันทึก ไม่ต้องรีสตาร์ต';

  @override
  String get naiomCrosshairPreviewLabel => 'Crosshair Mode ในเกม';

  @override
  String get naiomCursorPick => 'เลือกไฟล์...';

  @override
  String get naiomCursorClear => 'นำออก';

  @override
  String get naiomCursorInvalid =>
      'ไฟล์เคอร์เซอร์ไม่ถูกต้อง ต้องเป็นไฟล์ .cur หรือ .ani จริง';

  @override
  String get naiomLiveBadge => 'สด';

  @override
  String get naiomLiveTooltip => 'มีผลหลังบันทึก ไม่ต้องรีสตาร์ตเกม';

  @override
  String get labelPreloadMaxDimensionShort => 'ขนาดสูงสุดสำหรับการโหลดล่วงหน้า';

  @override
  String get tooltipPreloadMaxDimensionShort =>
      '0 = ปิดใช้งาน (สตรีมอย่างเดียว), 2048 = ค่าเริ่มต้น, 4096 = เท็กซ์เจอร์ 4K, 16384 = ทุกอย่าง';

  @override
  String get labelPreloadAllTexturesShort => 'โหลดเท็กซ์เจอร์ทั้งหมดล่วงหน้า';

  @override
  String get tooltipPreloadAllTexturesShort =>
      'โหลดเท็กซ์เจอร์ทั้งหมดล่วงหน้า ไม่มีอาการกระตุกแต่ต้องใช้ RAM 32GB ขึ้นไป';

  @override
  String get labelVramBudget => 'งบ VRAM (MB)';

  @override
  String get tooltipVramBudget =>
      'จำนวนหน่วยความจำ GPU ที่ระบบม็อดเท็กซ์เจอร์สามารถใช้ได้ เลือกค่าเพื่อกำหนดขีดจำกัดตายตัว เช่น 8192 หมายถึง \"ห้ามใช้เกิน 8 GB สำหรับเท็กซ์เจอร์ม็อด\" และ 16384 หมายถึง \"ห้ามใช้เกิน 16 GB\" อัตโนมัติ (แนะนำ) จะไม่กำหนดขีดจำกัดและใช้ตามหน่วยความจำที่ GPU ของคุณมีจริง';

  @override
  String get labelStreamingEnabled => 'การโหลดเบื้องหลัง';

  @override
  String get tooltipStreamingEnabled =>
      'โหลดเท็กซ์เจอร์เบื้องหลังระหว่างเล่น ช่วยป้องกันเกมค้างและกระตุกเมื่อโหลดพื้นที่ใหม่ ปิดเฉพาะเมื่อมีปัญหา หากปิด เกมอาจค้างชั่วครู่เมื่อโหลดเท็กซ์เจอร์ใหม่';

  @override
  String get labelLoadOnlyRelevant => 'โหลดเฉพาะที่เกี่ยวข้อง';

  @override
  String get tooltipLoadOnlyRelevant =>
      'สำหรับแพ็กขนาดใหญ่มาก (400+ ไฟล์) โหลดเฉพาะเท็กซ์เจอร์ที่ตรงกับรายการลำดับความสำคัญที่คัดไว้ ช่วยประหยัด VRAM และเวลาโหลด แพ็กขนาดเล็ก (เสื้อผ้า อาวุธ) จะโหลดเต็มเสมอ เปิดหากใช้แพ็กขนาดใหญ่และต้องการประหยัดหน่วยความจำ';

  @override
  String get tutorialDropTextures =>
      'ลากม็อดเท็กซ์เจอร์มาวางที่นี่เพื่อติดตั้ง ไฟล์ Zip จะถูกแตกโดยอัตโนมัติ';

  @override
  String get tutorialLoadOrder =>
      'หากม็อดทับกัน ให้ลากเพื่อจัดลำดับ ด้านบน = ความสำคัญสูงสุด';

  @override
  String get textureOverlapLabel => 'ทับซ้อน';

  @override
  String tooltipTextureOverlap(String mods) {
    return 'เปลี่ยนเท็กซ์เจอร์เดียวกับ: $mods ม็อดที่อยู่สูงกว่าในรายการ (ใกล้ HIGHEST) คือสิ่งที่จะแสดงในเกม';
  }

  @override
  String get tooltipFolderNotFound => 'ไม่พบโฟลเดอร์ใน nams/inject/textures/';

  @override
  String get priorityHighest => 'สูงสุด';

  @override
  String get priorityMedium => 'ปานกลาง';

  @override
  String get priorityLowest => 'ต่ำสุด';

  @override
  String nameOutfitTitle(String character) {
    return 'ตั้งชื่อชุดนี้ ($character)';
  }

  @override
  String get outfitNameHint => 'ชื่อชุด';

  @override
  String installedTextureCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'ไฟล์',
      one: 'ไฟล์',
    );
    return 'ติดตั้งเท็กซ์เจอร์ $count $_temp0';
  }

  @override
  String installationFailed(String error) {
    return 'ติดตั้งไม่สำเร็จ: $error';
  }

  @override
  String removedItem(String name) {
    return 'นำ \"$name\" ออกแล้ว';
  }

  @override
  String get tutorialStarIcon =>
      'คลิกดาวเพื่อกำหนดชุดเริ่มต้นที่จะโหลดเมื่อเกมเริ่ม';

  @override
  String installedOutfitsCount(int count) {
    return 'ชุดที่ติดตั้ง ($count)';
  }

  @override
  String get tooltipDlcDetected =>
      'ตรวจพบ DLC (data100.cpk) ไฟล์โมเดลใช้การตั้งชื่อแบบ DLC (pl000d)';

  @override
  String get tooltipNoDlcDetected =>
      'ตรวจไม่พบ DLC ไฟล์โมเดลจะถูกเปลี่ยนชื่อเป็นแบบไม่มี DLC (pl0000)';

  @override
  String installConfirmMod(String name, String character) {
    return 'ติดตั้ง \"$name\" ($character) หรือไม่?';
  }

  @override
  String installedOutfit(String name, String character) {
    return 'ติดตั้ง \"$name\" ($character) แล้ว';
  }

  @override
  String get crossInstallTextures =>
      'ม็อดนี้มีไฟล์เท็กซ์เจอร์ด้วย ต้องการติดตั้งไปยัง nams/inject/textures/ หรือไม่?';

  @override
  String alsoInstalledTextures(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'ไฟล์',
      one: 'ไฟล์',
    );
    return 'ติดตั้งเท็กซ์เจอร์เพิ่มเติม $count $_temp0';
  }

  @override
  String get clearedAllStartupOutfits => 'ล้างชุดเริ่มต้นทั้งหมดแล้ว';

  @override
  String get clearedStartupOutfit => 'ล้างชุดเริ่มต้นแล้ว';

  @override
  String setStartupOutfit(String name) {
    return 'ตั้ง \"$name\" เป็นชุดเริ่มต้นแล้ว';
  }

  @override
  String get tutorialDropCutscenes =>
      'วางไฟล์เก็บถาวรม็อดคัตซีนที่นี่ รองรับ .zip, .7z และ .rar';

  @override
  String get tutorialInstalledCutscenes =>
      'ม็อดคัตซีนที่ติดตั้ง คัตซีนแบบกำหนดเองจะโหลดจากที่นี่แทน data/movie/';

  @override
  String get selectCutsceneModFolder => 'เลือกโฟลเดอร์ม็อดคัตซีน';

  @override
  String cutsceneNamingHint(int max) {
    return 'สูงสุด $max ตัวอักษร ชื่อนี้จะกลายเป็นชื่อโฟลเดอร์ใน nams/cutscenes/';
  }

  @override
  String cutsceneNameTooLong(int max) {
    return 'ชื่อต้องมีความยาวไม่เกิน $max ตัวอักษร';
  }

  @override
  String get preparingInstall => 'กำลังเตรียม...';

  @override
  String installedCutsceneMod(String name, int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'ไฟล์',
      one: 'ไฟล์',
    );
    return 'ติดตั้ง \"$name\" แล้ว ($count USM $_temp0)';
  }

  @override
  String deleteCutsceneConfirm(String name) {
    return 'ลบ \"$name\" และไฟล์ทั้งหมดหรือไม่?';
  }

  @override
  String installedCutsceneModsCount(int count) {
    return 'ม็อดคัตซีนที่ติดตั้ง ($count)';
  }

  @override
  String cutsceneUsmCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'ไฟล์ USM',
      one: 'ไฟล์ USM',
    );
    return '$count $_temp0';
  }

  @override
  String cutsceneMatchCount(int matching, int total) {
    return '$matching/$total ตรงกับไฟล์ต้นฉบับ';
  }

  @override
  String tooltipMissingOriginals(String files) {
    return 'ไฟล์ที่ไม่ตรงกับต้นฉบับ: $files';
  }

  @override
  String get cutsceneMismatchHint =>
      'บางไฟล์ไม่ตรงกับชื่อคัตซีนต้นฉบับ ไฟล์ที่หายจะกลับไปใช้คัตซีนต้นฉบับ';

  @override
  String cutsceneMigrationBannerBody(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'ไฟล์',
      one: 'ไฟล์',
    );
    return 'พบไฟล์คัตซีนแบบกำหนดเอง $count $_temp0 อยู่โดยตรงใน data/movie/ ไฟล์เหล่านี้จะแทนที่ต้นฉบับอย่างถาวร ครั้งถัดไปให้ติดตั้งม็อดคัตซีนที่นี่แทน หากไฟล์แบบกำหนดเองโหลดไม่สำเร็จ ระบบจะเล่นไฟล์ต้นฉบับเป็นตัวสำรอง';
  }

  @override
  String hardwareInfoLabel(int ram, String gpu) {
    return 'RAM ${ram}GB | $gpu';
  }

  @override
  String hardwareInfoRamOnly(int ram) {
    return 'RAM ${ram}GB';
  }

  @override
  String texturesScanResult(int count, int sizeMB, int maxDim) {
    return 'เท็กซ์เจอร์ $count ไฟล์ รวม ${sizeMB}MB สูงสุด ${maxDim}px';
  }

  @override
  String recommendedSettings(int dim, String allLabel) {
    return 'แนะนำ: โหลดล่วงหน้า $dim, โหลดทั้งหมด $allLabel';
  }

  @override
  String get applyRecommended => 'นำไปใช้';

  @override
  String get settingsMatchRecommended => 'การตั้งค่าของคุณตรงกับคำแนะนำ';

  @override
  String get reasonNoTextures => 'ยังไม่ได้ติดตั้งเท็กซ์เจอร์';

  @override
  String reasonFitsInMemory(int ramGB, int textureSizeMB) {
    return 'RAM ${ramGB}GB, เท็กซ์เจอร์ ${textureSizeMB}MB - พอดีกับหน่วยความจำ โหลดทั้งหมดล่วงหน้าเพื่อไม่มีอาการกระตุก';
  }

  @override
  String reasonExceedsRam(int ramGB, int estimatedGB) {
    return 'RAM ${ramGB}GB, ประมาณ ${estimatedGB}GB สำหรับหน่วยความจำเท็กซ์เจอร์ - การโหลดทั้งหมดล่วงหน้าจะทำให้ระบบค้างหรือล่ม โปรดใช้ขนาดโหลดล่วงหน้าต่ำหรือนำแพ็กเท็กซ์เจอร์บางส่วนออก';
  }

  @override
  String reasonTooLargeForAll(int ramGB, int textureSizeMB) {
    return 'RAM ${ramGB}GB, เท็กซ์เจอร์ ${textureSizeMB}MB - ใหญ่เกินกว่าจะโหลดทั้งหมดล่วงหน้า ให้โหลดถึง 4K ตามต้องการ';
  }

  @override
  String reasonMediumRam(int ramGB) {
    return 'RAM ${ramGB}GB - โหลดล่วงหน้าถึง 4K เท็กซ์เจอร์ขนาดใหญ่กว่าจะโหลดตามต้องการ';
  }

  @override
  String reasonLowRam(int ramGB) {
    return 'RAM ${ramGB}GB - โหลดเท็กซ์เจอร์ขนาดเล็กล่วงหน้าเท่านั้นเพื่อประหยัดหน่วยความจำ';
  }

  @override
  String get analyzingHardware => 'กำลังวิเคราะห์ฮาร์ดแวร์และเท็กซ์เจอร์...';

  @override
  String texturesBloatWarning(int total, int relevant, int excess) {
    return 'ม็อดนี้มีเท็กซ์เจอร์ $total รายการ แต่มีเพียง $relevant รายการที่เกี่ยวข้องกับภาพจริง (ตามชุดอ้างอิงที่ GPUnity คัดไว้) อีก $excess รายการเพิ่มเวลาโหลดและการใช้ RAM โดยไม่มีประโยชน์ต่อภาพ';
  }

  @override
  String cleanUnneededTextures(int count) {
    return 'นำสิ่งที่ไม่จำเป็นออก $count รายการ';
  }

  @override
  String cleanedTextures(int deleted, int kept) {
    return 'นำเท็กซ์เจอร์ที่ไม่จำเป็นออก $deleted รายการ เก็บไว้ $kept รายการ';
  }

  @override
  String get confirmCleanTextures => 'นำเท็กซ์เจอร์ที่ไม่จำเป็นออกหรือไม่?';

  @override
  String confirmCleanTexturesBody(int count, String sizeMB) {
    return 'การดำเนินการนี้จะลบเท็กซ์เจอร์ $count ไฟล์ ($sizeMB MB) ออกจากโฟลเดอร์ม็อดนี้อย่างถาวร';
  }

  @override
  String get confirmCleanTexturesDetail1 =>
      'จะเก็บเฉพาะเท็กซ์เจอร์ที่ตรงกับชุดอ้างอิงที่ GPUnity คัดไว้';

  @override
  String get confirmCleanTexturesDetail2 =>
      'มีผลเฉพาะโฟลเดอร์ม็อดที่เลือก ไม่กระทบม็อดอื่นที่ติดตั้ง';

  @override
  String get confirmCleanTexturesDetail3 =>
      'ไม่สามารถย้อนกลับได้ ให้วางม็อดอีกครั้งเพื่อคืนไฟล์ที่นำออก';

  @override
  String get texturesBloatDialogTitle => 'ตรวจพบเท็กซ์เจอร์ที่ไม่จำเป็น';

  @override
  String texturesBloatDialogBody(int total, int relevant, int excess) {
    return 'แพ็กเท็กซ์เจอร์นี้มี $total ไฟล์ แต่มีเพียง $relevant ไฟล์ที่ตรงกับชุดอ้างอิง GPUnity ที่คัดไว้ อีก $excess ไฟล์มีแนวโน้มว่าไม่จำเป็น';
  }

  @override
  String get texturesBloatPoint1 =>
      'เริ่มเกมช้าลงมาก เพราะเอนจินโหลดทุกเท็กซ์เจอร์ตอนเริ่ม';

  @override
  String get texturesBloatPoint2 =>
      'เกิดการกระตุกและเฟรมตกแบบสุ่ม เพราะเกมสตรีมเท็กซ์เจอร์ที่ไม่เพิ่มประโยชน์ด้านภาพ';

  @override
  String get texturesBloatPoint3 =>
      'ใช้ RAM สูง อาจสิ้นเปลืองหลาย GB กับเท็กซ์เจอร์ที่คุณมองไม่เห็น';

  @override
  String get texturesBloatPoint4 =>
      'เท็กซ์เจอร์ที่อัปสเกลด้วย AI บางรายการอาจมีสิ่งผิดเพี้ยนหรือไฟล์เสีย';

  @override
  String get texturesBloatPoint5 =>
      'แทบไม่เห็นความแตกต่าง เพราะส่วนใหญ่เป็นองค์ประกอบ UI ขนาดเล็ก เอฟเฟกต์อนุภาค เป็นต้น';

  @override
  String get texturesBloatRecommendation =>
      'การนำออกปลอดภัยและแนะนำเพื่อประสิทธิภาพที่ดีขึ้น';

  @override
  String get texturesBloatKeepAll => 'เก็บทั้งหมด';

  @override
  String texturesBloatRemoveUnneeded(int count) {
    return 'นำสิ่งที่ไม่จำเป็นออก ($count)';
  }

  @override
  String get texturesProgressExtracting => 'กำลังแตกไฟล์เก็บถาวร...';

  @override
  String get texturesProgressCopying => 'กำลังคัดลอกไฟล์...';

  @override
  String get texturesProgressAnalyzing => 'กำลังวิเคราะห์เท็กซ์เจอร์...';

  @override
  String get texturesAnalyzingSetup =>
      'กำลังวิเคราะห์การตั้งค่าเท็กซ์เจอร์ของคุณ...';

  @override
  String get texturesBusyMessage => 'โปรดรอ กำลังติดตั้งเท็กซ์เจอร์';

  @override
  String texturesInstallProgress(
    int files,
    int totalFiles,
    int mb,
    int totalMb,
  ) {
    return 'กำลังติดตั้ง: $files/$totalFiles ไฟล์ - $mb/$totalMb MB';
  }

  @override
  String texturesAnalyzeProgress(int scanned, int total) {
    return 'กำลังวิเคราะห์: $scanned/$total เท็กซ์เจอร์';
  }

  @override
  String get cleaningTextures => 'กำลังนำเท็กซ์เจอร์ที่ไม่จำเป็นออก...';

  @override
  String get textureMergeTitle => 'เพิ่มไปยังม็อดเดิมหรือติดตั้งใหม่?';

  @override
  String get textureMergeDescription =>
      'คุณมีม็อดเท็กซ์เจอร์ติดตั้งอยู่แล้ว ต้องการเพิ่มไฟล์เหล่านี้ไปยังม็อดเดิมหรือติดตั้งเป็นม็อดใหม่?';

  @override
  String get textureMergeNewMod => 'ติดตั้งเป็นม็อดใหม่';

  @override
  String textureMergeAddTo(String name) {
    return 'เพิ่มไปยัง: $name';
  }

  @override
  String get cutsceneMergeTitle => 'เพิ่มไปยังม็อดเดิมหรือติดตั้งใหม่?';

  @override
  String get cutsceneMergeDescription =>
      'คุณมีม็อดคัตซีนติดตั้งอยู่แล้ว แพ็กคัตซีนหลายส่วนควรถูกรวมไว้ในม็อดเดียวกัน';

  @override
  String get cutsceneMergeNewMod => 'ติดตั้งเป็นม็อดใหม่';

  @override
  String cutsceneMergeAddTo(String name) {
    return 'เพิ่มไปยัง: $name';
  }

  @override
  String get headerMods => 'ม็อด';

  @override
  String cutsceneBundledWith(String modId) {
    return 'รวมมากับ $modId';
  }

  @override
  String get cutsceneStatusHd => 'HD';

  @override
  String get cutsceneStatusHdTooltip =>
      '[cutscene] hd_cutscenes ใน nams.toml ต้องเป็น true เพื่อโหลดม็อดคัตซีน HD';

  @override
  String get cutsceneStatusH264 => 'H264';

  @override
  String get cutsceneStatusH264Tooltip =>
      '[cutscene] enable_h264 ใน nams.toml ต้องเป็น true เพื่อเล่นคัตซีนที่เข้ารหัส H264';

  @override
  String get modIntroTitle =>
      'ขับเคลื่อนโดย NAMS โฟลเดอร์ data/ ของคุณจะไม่ถูกแตะ';

  @override
  String get modIntroBody =>
      'NAMS โหลดม็อดจาก nams/mods/ ผ่านระบบไฟล์เสมือนที่ซ้อนอยู่บนข้อมูลเกมต้นฉบับ จึงไม่มีสิ่งใดถูกคัดลอกหรือเขียนทับใน data/ ม็อดสามารถเปิดหรือปิดได้ทุกเมื่อโดยไม่ต้องติดตั้งใหม่ ชุดหลายชุดสามารถอยู่ร่วมกันสำหรับตัวละครเดียว และการถอนการติดตั้งม็อดเพียงลบโฟลเดอร์ของมัน เกมวานิลลายังคงสมบูรณ์อยู่ด้านล่างเสมอ';

  @override
  String get modListEmpty => 'ยังไม่ได้ติดตั้งม็อด';

  @override
  String get modListEmptyHint =>
      'วางโฟลเดอร์หรือไฟล์เก็บถาวรม็อดลงในกล่องด้านบนเพื่อติดตั้ง';

  @override
  String get modSearchPlaceholder => 'ค้นหาม็อด…';

  @override
  String get modFilterAll => 'ทั้งหมด';

  @override
  String get modCollapseAll => 'ยุบทุกกลุ่ม';

  @override
  String get modExpandAll => 'ขยายทุกกลุ่ม';

  @override
  String get modBulkInstall => 'ติดตั้งหลายรายการจากโฟลเดอร์';

  @override
  String modBulkInstallBusy(int done, int total, String name) {
    return 'กำลังติดตั้ง $done จาก $total: $name';
  }

  @override
  String get modBulkInstallScanning => 'กำลังสแกนโฟลเดอร์หาไฟล์เก็บถาวรม็อด…';

  @override
  String get modBulkInstallNone =>
      'ไม่พบไฟล์เก็บถาวรม็อด (.zip / .7z / .rar) ในโฟลเดอร์นั้น';

  @override
  String modBulkInstallDone(int installed, int total) {
    return 'ติดตั้งม็อด $installed จาก $total รายการแล้ว';
  }

  @override
  String get modLooseInstall => 'ติดตั้งไฟล์แบบแยกจากโฟลเดอร์';

  @override
  String get modLooseInstallScanning => 'กำลังสแกนโฟลเดอร์หาไฟล์เกมแบบแยก…';

  @override
  String get modLooseInstallNone =>
      'ไม่พบไฟล์เกมแบบแยก (.dat / .dtt) ในโฟลเดอร์นั้น';

  @override
  String modLooseInstallBusy(int count) {
    return 'กำลังติดตั้งไฟล์แบบแยก $count ไฟล์…';
  }

  @override
  String modLooseInstallProgress(int done, int total) {
    return 'กำลังคัดลอก $done จาก $total ไฟล์…';
  }

  @override
  String get modLooseInstallFinalizing => 'กำลังวางไฟล์ลงในม็อด…';

  @override
  String modLooseInstallDone(int count, String id) {
    return 'ติดตั้งไฟล์แบบแยก $count ไฟล์ลงใน $id แล้ว';
  }

  @override
  String get modGroup2b => 'ชุดของ 2B';

  @override
  String get modGroup9s => 'ชุดของ 9S';

  @override
  String get modGroupA2 => 'ชุดของ A2';

  @override
  String get modGroupOtherOutfits => 'ชุดอื่น ๆ';

  @override
  String get modGroupWeapons => 'อาวุธ';

  @override
  String get modGroupAccessories => 'เครื่องประดับ';

  @override
  String get modGroupItems => 'ไอเท็ม';

  @override
  String get modGroupEnemies => 'ศัตรู';

  @override
  String get modGroupWorldProps => 'วัตถุประกอบฉากในโลก';

  @override
  String get modGroupModelVariants => 'รูปแบบโมเดล';

  @override
  String get modGroupMaps => 'แผนที่ / ด่าน';

  @override
  String get modGroupUi => 'UI / ฟอนต์';

  @override
  String get modGroupMisc => 'เท็กซ์เจอร์เบ็ดเตล็ด';

  @override
  String get modGroupArchives => 'ไฟล์เก็บถาวร CPK';

  @override
  String get modGroupEffects => 'เอฟเฟกต์';

  @override
  String get modGroupScripting => 'สคริปต์';

  @override
  String get modGroupLocalization => 'ข้อความและภาษา';

  @override
  String get modGroupCutscenes => 'คัตซีน';

  @override
  String get modGroupAudio => 'เสียง';

  @override
  String get modGroupTextures => 'เท็กซ์เจอร์';

  @override
  String get modGroupNative => 'ม็อดเนทีฟ';

  @override
  String get modGroupOther => 'อื่น ๆ';

  @override
  String get modGroupMixed => 'เนื้อหาแบบผสม';

  @override
  String get modGroupWax => 'WAX แบบกะทัดรัด';

  @override
  String get modGroupMultiHint =>
      'ม็อดนี้แทนที่โมเดลของหลายตัวละคร จึงถูกแสดงอยู่ในแต่ละหมวดที่เกี่ยวข้อง';

  @override
  String get modGroupMixedHint =>
      'ม็อดนี้เปลี่ยนเนื้อหาหลายประเภทพร้อมกัน คลิกเพื่อดูสิ่งที่รวมอยู่ทั้งหมดและหมวดหมู่ที่ได้รับผลกระทบ';

  @override
  String get modRename => 'เปลี่ยนชื่อ';

  @override
  String get modRenameDialogTitle => 'เปลี่ยนชื่อม็อด';

  @override
  String get modRenameReset => 'คืนชื่อเดิม';

  @override
  String get dropModHere => 'วางม็อดที่นี่';

  @override
  String get dropModHereHint => 'หรือคลิกเพื่อเลือกไฟล์';

  @override
  String get modKindNative => 'เนทีฟ';

  @override
  String get modKindNativeTooltip =>
      'ม็อด NAMS ที่มีโฟลเดอร์ entities/ กำหนดไอเท็ม อาวุธ ชุด เครื่องประดับ เควสต์ และอื่น ๆ ผ่านชุดไฟล์ TOML';

  @override
  String get modKindData => 'ข้อมูล';

  @override
  String get modKindDataTooltip =>
      'รูปแบบม็อดแบบดั้งเดิม ไฟล์เดียวกับที่ปกติจะถูกวางใน NieRAutomata/data/ แต่ถูกจัดการภายใต้ nams/mods/ แทน เพื่อรักษาโฟลเดอร์ data ต้นฉบับให้สะอาด';

  @override
  String get textureOutfitLinkedTitle => 'เท็กซ์เจอร์ที่เชื่อมกับชุด';

  @override
  String get textureOutfitLinkedSubtitle =>
      'เท็กซ์เจอร์เหล่านี้อยู่ภายในโฟลเดอร์ม็อดและโหลดเฉพาะเมื่อสวมชุดนั้น NAMS จะ hot-swap เมื่อคุณเปลี่ยนชุดในเกม';

  @override
  String textureOutfitLinkedEntry(int count) {
    return '$count เท็กซ์เจอร์ - ทำงานเฉพาะกับชุดนี้';
  }

  @override
  String get modKindTexture => 'เท็กซ์เจอร์';

  @override
  String get modKindTextureTooltip =>
      'แพ็กเท็กซ์เจอร์ ไฟล์ .dds ถูกติดตั้งไปยัง nams/inject/textures/ และจัดการจากแท็บ Textures';

  @override
  String get modKindUnknown => 'ไม่ทราบ';

  @override
  String get modKindUnknownTooltip =>
      'ตัวเรียกใช้งานไม่สามารถระบุโฟลเดอร์นี้ว่าเป็นม็อดที่ถูกต้องได้';

  @override
  String get modCompatChip => 'รองรับ wax';

  @override
  String get modCompatChipTooltip =>
      ' NAMS อ่านไฟล์เหล่านี้ด้วยเพื่อให้เข้ากันได้กับม็อด wax ที่มีอยู่';

  @override
  String get modDataChip => '+data';

  @override
  String get modDataChipTooltip =>
      'มี data/ overlay มาพร้อมกับ metadata โมเดล เท็กซ์เจอร์ เสียง และอื่น ๆ อยู่ที่นี่';

  @override
  String get modDetailNoSelection => 'เลือกม็อดเพื่อดูรายละเอียด';

  @override
  String get modAuthor => 'ผู้สร้าง';

  @override
  String get modVersion => 'เวอร์ชัน';

  @override
  String get modRootPath => 'พาธ';

  @override
  String get modNativeBundles => 'ชุดเนทีฟ';

  @override
  String get modDataContent => 'เนื้อหา data';

  @override
  String get modDataPlayerModels => 'โมเดลผู้เล่น';

  @override
  String get modRequiresLabel => 'ต้องใช้';

  @override
  String get modRequiresPluginsLabel => 'ต้องใช้ปลั๊กอิน';

  @override
  String get modRequiresMissing => 'หายไป';

  @override
  String get modConflictsLabel => 'ความขัดแย้ง';

  @override
  String get modLoadOrderHint =>
      'ม็อดเหล่านี้แทนที่ไฟล์เดียวกัน ลากเพื่อจัดลำดับ โดยม็อดด้านบนชนะ';

  @override
  String get modConflictKeep => 'เก็บม็อดนี้';

  @override
  String get modConflictResolve => 'แก้ไข';

  @override
  String get modConflictDialogTitle => 'ม็อดใดควรมีลำดับเหนือกว่า?';

  @override
  String modConflictKeepTooltip(String id) {
    return 'เก็บ $id และปิดม็อดอื่น';
  }

  @override
  String modConflictPickBody(int mods, int files) {
    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: 'ไฟล์เดียวกัน $files ไฟล์',
      one: 'ไฟล์เดียวกัน 1 ไฟล์',
    );
    return 'ม็อดที่เปิดอยู่ $mods รายการแทนที่ $_temp0 เลือกม็อดที่จะเก็บไว้ ม็อดอื่นจะถูกปิด';
  }

  @override
  String modConflictOverlapFile(String otherId, String file) {
    return '$otherId มีไฟล์ $file เช่นกัน';
  }

  @override
  String get modOpenFolder => 'เปิดโฟลเดอร์';

  @override
  String get modEnable => 'เปิดใช้งาน';

  @override
  String get modDisable => 'ปิดใช้งาน';

  @override
  String get modDisabled => 'ปิดใช้งานแล้ว';

  @override
  String get modDisabledTooltip =>
      'ม็อดนี้ถูกปิดอยู่ NAMS จะไม่โหลดในการเริ่มเกมครั้งถัดไป เปิดใช้งานอีกครั้งเพื่อโหลด ไม่ต้องลบและติดตั้งใหม่';

  @override
  String get modEnableTooltip =>
      'NAMS กำลังโหลดม็อดนี้ คลิกเพื่อปิดโดยไม่ลบไฟล์';

  @override
  String get modDefaultTooltip =>
      'ทำงานตั้งแต่เกมเริ่ม ราวกับไฟล์อยู่ใน NieRAutomata/data คลิกเพื่อปิด';

  @override
  String get modSetDefaultTooltip =>
      'ทำให้ม็อดนี้ทำงานตั้งแต่เกมเริ่ม โดยไม่คัดลอกอะไรไปยัง NieRAutomata/data';

  @override
  String get modSetDefaultOutfitTooltip =>
      'สวมชุดนี้ตั้งแต่เกมเริ่ม โดยไม่คัดลอกอะไรไปยัง NieRAutomata/data จะแทนที่ชุดที่เป็นค่าเริ่มต้นอยู่ และมีได้เพียงหนึ่งชุด';

  @override
  String get modDefaultChip => 'ค่าเริ่มต้น';

  @override
  String get modDefaultKindOutfitBare => 'ชุด';

  @override
  String get modDefaultKindOutfitConfig => 'ชุด + การตั้งค่า';

  @override
  String get modDefaultKindOutfitAnimation => 'แอนิเมชัน';

  @override
  String get modDefaultKindOutfitBareTooltip =>
      'แทนที่ไฟล์โมเดลโดยตรง สามารถมีชุดเริ่มต้นได้เพียงชุดเดียวในแต่ละครั้ง';

  @override
  String get modDefaultKindOutfitConfigTooltip =>
      'ม็อดนี้มีการตั้งค่าชุดมาด้วย ดังนั้นกฎ mesh และเอฟเฟกต์จะโหลดพร้อมกัน สามารถมีชุดเริ่มต้นได้เพียงชุดเดียวในแต่ละครั้ง';

  @override
  String get modDefaultKindOutfitAnimationTooltip =>
      'ข้อมูลแอนิเมชัน ไม่ใช่ชุด จะยังทำงานอยู่ใต้ชุดใดก็ตามที่คุณสวม';

  @override
  String get modDefaultReplaceTitle => 'แทนที่ค่าเริ่มต้นหรือไม่?';

  @override
  String modDefaultReplaceBody(String model, String current, String next) {
    return 'ขณะนี้ $model สวม \"$current\" ตั้งแต่เริ่มเกม\n\nการกำหนด \"$next\" เป็นค่าเริ่มต้นจะนำชุดเดิมออก เพราะตัวละครหนึ่งตัวสวมม็อดได้เพียงชุดเดียวในเวลาเดียวกัน';
  }

  @override
  String get modDefaultReplaceConfirm => 'แทนที่';

  @override
  String get modDefaultOutfitAuto => 'ชุดเริ่มต้น';

  @override
  String get modDefaultOutfitPickTooltip =>
      'ม็อดนี้มีหลายชุด เลือกชุดที่ต้องการสวมตั้งแต่เกมเริ่ม \"ชุดเริ่มต้น\" คือชุดที่สวมโดยไม่ต้องใช้ไอเท็ม';

  @override
  String modDefaultRowTooltip(String files) {
    return 'ทำงานตั้งแต่เกมเริ่ม: $files';
  }

  @override
  String get modDisableNotice => 'ปิดใช้งานแล้ว มีผลในการเริ่มเกมครั้งถัดไป';

  @override
  String get modEnableNotice => 'เปิดใช้งานแล้ว มีผลในการเริ่มเกมครั้งถัดไป';

  @override
  String get modUninstall => 'ถอนการติดตั้ง';

  @override
  String get modUninstallConfirmTitle => 'ถอนการติดตั้งม็อดหรือไม่?';

  @override
  String modUninstallConfirmBody(String id) {
    return 'การดำเนินการนี้จะลบโฟลเดอร์ม็อด \"$id\" อย่างถาวร';
  }

  @override
  String get modProfileLabel => 'โปรไฟล์';

  @override
  String get modProfileNewButton => 'ใหม่';

  @override
  String get modProfileRenameButton => 'เปลี่ยนชื่อ';

  @override
  String get modProfileDeleteButton => 'ลบ';

  @override
  String get modProfileNewDialogTitle => 'โปรไฟล์ใหม่';

  @override
  String get modProfileNewDialogHint => 'ชื่อโปรไฟล์ (ตัวอักษร ตัวเลข _ -)';

  @override
  String get modProfileRenameDialogTitle => 'เปลี่ยนชื่อโปรไฟล์';

  @override
  String get modProfileDeleteDialogTitle => 'ลบโปรไฟล์หรือไม่?';

  @override
  String modProfileDeleteDialogBody(String name) {
    return 'ลบโฟลเดอร์ mods_profile_$name/ และแพ็กเท็กซ์เจอร์ที่รวมมากับม็อดในโปรไฟล์นี้อย่างถาวร ไม่สามารถย้อนกลับได้';
  }

  @override
  String get modProfileDeleteConfirm => 'ลบ';

  @override
  String get modProfileErrorNameEmpty => 'ต้องระบุชื่อ';

  @override
  String get modProfileErrorNameInvalid =>
      'ใช้ได้เฉพาะตัวอักษร ตัวเลข _ หรือ -';

  @override
  String get modProfileErrorNameCollision => 'มีโปรไฟล์ชื่อนี้อยู่แล้ว';

  @override
  String get modProfileErrorDeleteActive =>
      'สลับไปยังโปรไฟล์อื่นก่อนลบโปรไฟล์นี้';

  @override
  String get modProfileErrorDeleteLast =>
      'ไม่สามารถลบโปรไฟล์สุดท้ายที่เหลืออยู่ได้';

  @override
  String get modProfileErrorTargetMissing => 'ไม่พบโฟลเดอร์โปรไฟล์บนดิสก์';

  @override
  String get modProfileErrorFsBusy =>
      'ระบบไฟล์กำลังถูกใช้งาน โปรดปิดเกมแล้วลองใหม่';

  @override
  String get modProfileLockedRunning => 'หยุดเกมก่อนเปลี่ยนโปรไฟล์';

  @override
  String get modProfileEmptyHint => 'โปรไฟล์ว่าง วางม็อดเพื่อเริ่มต้น';

  @override
  String modProfileSwitchedToast(String name) {
    return 'สลับไปยังโปรไฟล์ $name แล้ว';
  }

  @override
  String modProfileCreatedToast(String name) {
    return 'สร้างและสลับไปยังโปรไฟล์ $name แล้ว';
  }

  @override
  String modProfileDeletedToast(String name) {
    return 'ลบโปรไฟล์ $name แล้ว';
  }

  @override
  String modProfileRenamedToast(String name) {
    return 'เปลี่ยนชื่อโปรไฟล์เป็น $name แล้ว';
  }

  @override
  String get modInstallNeedsName => 'ตั้งชื่อม็อดนี้';

  @override
  String modInstallExistsPickAnother(String id) {
    return 'มีม็อดชื่อ \"$id\" อยู่แล้ว โปรดเลือกชื่ออื่น';
  }

  @override
  String get modInspectBusy => 'กำลังตรวจสอบม็อด…';

  @override
  String get modInstallBusy => 'กำลังติดตั้งม็อด…';

  @override
  String get modVariantDialogTitle => 'เลือกสิ่งที่ต้องการติดตั้ง';

  @override
  String get modVariantDialogSubtitle =>
      'ไฟล์เก็บถาวรนี้มีหลายตัวเลือก โปรดเลือกรายการที่ต้องการ';

  @override
  String get modOutfitChoiceDialogTitle => 'เลือกสิ่งที่ต้องการติดตั้ง';

  @override
  String get modOutfitChoiceDialogSubtitle =>
      'ทำเครื่องหมายทุกรายการที่ต้องการ แต่ละรายการจะติดตั้งเป็นม็อดแยก หากชุดมีเท็กซ์เจอร์มาด้วย เท็กซ์เจอร์จะติดตั้งพร้อมกัน และคุณสามารถปรับชุดที่ใช้ภายหลังในแท็บ Textures';

  @override
  String get variantCatPlayer => 'ชุด';

  @override
  String get variantCatWeapon => 'อาวุธ';

  @override
  String get variantCatAccessory => 'เครื่องประดับ';

  @override
  String get variantCatEnemy => 'ศัตรู';

  @override
  String get variantCatModelVariant => 'รูปแบบโมเดล';

  @override
  String get variantCatItem => 'ไอเท็ม';

  @override
  String get variantCatWorldProp => 'วัตถุประกอบฉากในโลก';

  @override
  String get variantCatMap => 'แผนที่';

  @override
  String get variantCatEffects => 'เอฟเฟกต์';

  @override
  String get variantCatScripting => 'สคริปต์';

  @override
  String get variantCatLocalization => 'ภาษา';

  @override
  String get variantCatUi => 'UI';

  @override
  String get variantCatCutscenes => 'คัตซีน';

  @override
  String get variantCatAudio => 'เสียง';

  @override
  String get variantCatMisc => 'เบ็ดเตล็ด';

  @override
  String get variantCatOther => 'อื่น ๆ';

  @override
  String get variantPickOneSuffix => 'เลือกหนึ่งรายการ';

  @override
  String get modVariantSelectAll => 'เลือกทั้งหมด';

  @override
  String get modVariantSelectNone => 'ไม่เลือก';

  @override
  String get modVariantInstall => 'ติดตั้ง';

  @override
  String modVariantInstallSelected(int count) {
    return 'ติดตั้ง $count รายการ';
  }

  @override
  String get modVariantTexture => 'เท็กซ์เจอร์';

  @override
  String modVariantInstalledToast(int count) {
    return 'ติดตั้งตัวเลือก $count รายการแล้ว';
  }

  @override
  String get modUninstallBusy => 'กำลังถอนการติดตั้งม็อด…';

  @override
  String modInstalled(String id) {
    return 'ติดตั้งแล้ว: $id';
  }

  @override
  String modInstallFailed(String reason) {
    return 'ติดตั้งไม่สำเร็จ: $reason';
  }

  @override
  String get modInstallReasonUnknownDrop =>
      'ไม่รู้จักไฟล์ที่วาง โฟลเดอร์ไม่ตรงกับโครงสร้างม็อดที่รองรับ';

  @override
  String get modInstallReasonUnsupportedNasa =>
      'นี่คือม็อด NASA (มี sadfutago.cpk) ซึ่งตัวเรียกใช้งานนี้ไม่รองรับ';

  @override
  String get modInstallReasonInvalidMixed =>
      'โครงสร้างไม่ถูกต้อง ม็อดไม่สามารถรวม entities และการตั้งค่าแบบ wax ไว้ด้วยกัน';

  @override
  String get modInstallReasonNativeEmpty => 'ไม่พบไฟล์ entity ใน entities/';

  @override
  String get modInstallReasonDataEmpty => 'ไม่พบเนื้อหาที่รู้จัก';

  @override
  String get modInstallReasonArchiveExtractFailed =>
      'ไม่สามารถแตกไฟล์เก็บถาวรได้';

  @override
  String get modInstallReasonMoveFailed =>
      'ไม่สามารถย้ายไฟล์ไปยัง nams/mods/ ได้';

  @override
  String get modInstallReasonTextureOnly =>
      'นี่คือแพ็กเท็กซ์เจอร์ (มีเฉพาะไฟล์ .dds) โปรดติดตั้งจากแท็บ Textures แทน';

  @override
  String modUninstalled(String id) {
    return 'นำออกแล้ว: $id';
  }

  @override
  String modCountFiles(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count ไฟล์',
      one: '1 ไฟล์',
    );
    return '$_temp0';
  }
}
