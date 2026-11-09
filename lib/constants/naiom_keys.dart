class NaiomKeys {
  static const List<String> letters = [
    'A',
    'B',
    'C',
    'D',
    'E',
    'F',
    'G',
    'H',
    'I',
    'J',
    'K',
    'L',
    'M',
    'N',
    'O',
    'P',
    'Q',
    'R',
    'S',
    'T',
    'U',
    'V',
    'W',
    'X',
    'Y',
    'Z',
  ];

  static const List<String> digits = [
    '0',
    '1',
    '2',
    '3',
    '4',
    '5',
    '6',
    '7',
    '8',
    '9',
  ];

  static const List<String> special = [
    'SPACE',
    'TAB',
    'ENTER',
    'ESCAPE',
    'BACKSPACE',
    'INS',
    'DEL',
    'HOME',
    'END',
    'PGUP',
    'PGDN',
    'UPARROW',
    'DOWNARROW',
    'LEFTARROW',
    'RIGHTARROW',
    'LSHIFT',
    'RSHIFT',
    'LCTRL',
    'RCTRL',
    'LALT',
    'RALT',
  ];

  static final List<String> functionKeys = [
    for (var n = 1; n <= 12; n++) 'F$n',
  ];

  static final List<String> keypad = [
    for (var n = 0; n <= 9; n++) 'KP_$n',
    'KP_PLUS',
    'KP_MINUS',
    'KP_SLASH',
    'KP_STAR',
  ];

  static const List<String> mouseButtons = [
    'MOUSE1',
    'MOUSE2',
    'MOUSE3',
    'MOUSE4',
    'MOUSE5',
    'MOUSE_WHEEL_UP',
    'MOUSE_WHEEL_DOWN',
  ];

  static const List<String> controllerButtons = [
    'CONTROLLER_A',
    'CONTROLLER_B',
    'CONTROLLER_X',
    'CONTROLLER_Y',
    'CONTROLLER_LB',
    'CONTROLLER_RB',
    'CONTROLLER_LT',
    'CONTROLLER_RT',
    'CONTROLLER_LTHUMB',
    'CONTROLLER_RTHUMB',
    'CONTROLLER_DUP',
    'CONTROLLER_DDOWN',
    'CONTROLLER_DLEFT',
    'CONTROLLER_DRIGHT',
    'CONTROLLER_BACK',
    'CONTROLLER_START',
  ];

  static const List<String> genericModifiers = ['SHIFT', 'CTRL', 'ALT'];

  static const List<String> sideModifiers = [
    'LSHIFT',
    'RSHIFT',
    'LCTRL',
    'RCTRL',
    'LALT',
    'RALT',
  ];

  static final List<String> all = [
    ...letters,
    ...digits,
    ...special,
    ...functionKeys,
    ...keypad,
    ...mouseButtons,
    ...controllerButtons,
  ];

  static final Set<String> validSet = all.toSet();

  static final Set<String> modifierSet = {
    ...genericModifiers,
    ...sideModifiers,
  };

  static List<String> comboParts(String binding) => binding
      .split('+')
      .map((p) => p.trim().toUpperCase())
      .where((p) => p.isNotEmpty)
      .toList();

  static bool isValidBinding(String binding) {
    if (binding.isEmpty) return true;
    final parts = comboParts(binding);
    if (parts.isEmpty) return false;
    if (!validSet.contains(parts.last)) return false;
    return parts
        .sublist(0, parts.length - 1)
        .every((p) => modifierSet.contains(p) || validSet.contains(p));
  }

  static bool comboContainsNonKeyboard(String binding) => comboParts(
    binding,
  ).any((p) => mouseButtons.contains(p) || controllerButtons.contains(p));

  static const Set<String> keyboardOnlyBindings = {
    'standard_move_forward',
    'standard_move_backward',
    'standard_move_left',
    'standard_move_right',
    'standard_auto_run',
    'standard_light',
    'standard_self_destruct',
    'standard_walk',
    'standard_reset_camera',
    'standard_menu_up',
    'standard_menu_down',
    'standard_menu_left',
    'standard_menu_right',
  };

  static bool isMouseButton(String keyName) =>
      mouseButtons.contains(keyName.toUpperCase());

  static bool isValid(String keyName) =>
      keyName.isEmpty || validSet.contains(keyName.trim().toUpperCase());

  static bool allowsMouse(String bindingKey) =>
      !keyboardOnlyBindings.contains(bindingKey);

  static int? keyNameToVk(String keyName) {
    final upper = keyName.trim().toUpperCase();
    if (upper.length == 1) {
      final code = upper.codeUnitAt(0);
      if ((code >= 0x41 && code <= 0x5A) || (code >= 0x30 && code <= 0x39)) {
        return code;
      }
      return null;
    }
    switch (upper) {
      case 'SPACE':
        return 0x20;
      case 'END':
        return 0x23;
      case 'LEFTARROW':
        return 0x25;
      case 'UPARROW':
        return 0x26;
      case 'RIGHTARROW':
        return 0x27;
      case 'DOWNARROW':
        return 0x28;
      case 'LSHIFT':
        return 0xA0;
      case 'RSHIFT':
        return 0xA1;
      case 'LCTRL':
        return 0xA2;
      case 'RCTRL':
        return 0xA3;
      case 'LALT':
        return 0xA4;
      case 'RALT':
        return 0xA5;
      case 'TAB':
        return 0x09;
      case 'ENTER':
        return 0x0D;
      case 'ESCAPE':
        return 0x1B;
      case 'BACKSPACE':
        return 0x08;
      case 'INS':
        return 0x2D;
      case 'DEL':
        return 0x2E;
      case 'HOME':
        return 0x24;
      case 'PGUP':
        return 0x21;
      case 'PGDN':
        return 0x22;
      case 'KP_PLUS':
        return 0x6B;
      case 'KP_MINUS':
        return 0x6D;
      case 'KP_SLASH':
        return 0x6F;
      case 'KP_STAR':
        return 0x6A;
    }
    final fn = functionKeys.indexOf(upper);
    if (fn != -1) return 0x70 + fn;
    final kp = keypad.indexOf(upper);
    if (kp != -1 && kp <= 9) return 0x60 + kp;
    return null;
  }

  static String? vkToKeyName(int vk) {
    if (vk >= 0x41 && vk <= 0x5A) return String.fromCharCode(vk);
    if (vk >= 0x30 && vk <= 0x39) return String.fromCharCode(vk);
    if (vk >= 0x70 && vk <= 0x7B) return functionKeys[vk - 0x70];
    if (vk >= 0x60 && vk <= 0x69) return keypad[vk - 0x60];
    switch (vk) {
      case 0x12:
      case 0xA4:
        return 'LALT';
      case 0xA5:
        return 'RALT';
      case 0x09:
        return 'TAB';
      case 0x0D:
        return 'ENTER';
      case 0x1B:
        return 'ESCAPE';
      case 0x08:
        return 'BACKSPACE';
      case 0x2D:
        return 'INS';
      case 0x2E:
        return 'DEL';
      case 0x24:
        return 'HOME';
      case 0x21:
        return 'PGUP';
      case 0x22:
        return 'PGDN';
      case 0x6B:
        return 'KP_PLUS';
      case 0x6D:
        return 'KP_MINUS';
      case 0x6F:
        return 'KP_SLASH';
      case 0x6A:
        return 'KP_STAR';
      case 0x20:
        return 'SPACE';
      case 0x23:
        return 'END';
      case 0x25:
        return 'LEFTARROW';
      case 0x26:
        return 'UPARROW';
      case 0x27:
        return 'RIGHTARROW';
      case 0x28:
        return 'DOWNARROW';
      case 0x10:
      case 0xA0:
        return 'LSHIFT';
      case 0xA1:
        return 'RSHIFT';
      case 0x11:
      case 0xA2:
        return 'LCTRL';
      case 0xA3:
        return 'RCTRL';
    }
    return null;
  }
}
