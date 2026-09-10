import 'package:yp_launcher/l10n/app_localizations.dart';

class RustErrorCodes {
  RustErrorCodes._();

  static const unknown = 1000;

  static const imageUnreadable = 1100;
  static const imageSizeOutOfRange = 1101;
  static const imageSizeNotMultipleOfFour = 1102;

  static const archiveEmpty = 1200;
  static const archiveExtensionTooLong = 1201;
  static const weaponNothingToConvert = 1202;
  static const weaponClassMismatch = 1203;

  static const textureNoWtp = 1210;
  static const textureNotDds = 1211;
  static const textureBadSize = 1212;
  static const textureUnsupportedFormat = 1213;
  static const textureTruncated = 1214;
  static const textureEncodeFailed = 1215;

  static const fileReadFailed = 1300;
  static const fileWriteFailed = 1301;
  static const directoryCreateFailed = 1302;

  static String message(AppLocalizations l10n, int code, String detail) {
    switch (code) {
      case imageUnreadable:
        return l10n.errImageUnreadable;
      case imageSizeOutOfRange:
        return l10n.errImageSizeOutOfRange;
      case imageSizeNotMultipleOfFour:
        return l10n.errImageSizeNotMultipleOfFour;
      case archiveEmpty:
        return l10n.errArchiveEmpty;
      case archiveExtensionTooLong:
        return l10n.errArchiveExtensionTooLong(detail);
      case weaponNothingToConvert:
        return l10n.errWeaponNothingToConvert;
      case weaponClassMismatch:
        return l10n.errWeaponClassMismatch;
      case textureNoWtp:
      case textureNotDds:
      case textureUnsupportedFormat:
      case textureTruncated:
      case textureBadSize:
      case textureEncodeFailed:
        return l10n.errTextureUnreadable;
      case fileReadFailed:
        return l10n.errFileReadFailed(detail);
      case fileWriteFailed:
        return l10n.errFileWriteFailed(detail);
      case directoryCreateFailed:
        return l10n.errDirectoryCreateFailed(detail);
      default:
        return l10n.errUnknownCode(code, detail);
    }
  }
}
