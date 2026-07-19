class NaiomKeys {
  static const List<String> letters = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
    'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
  ];

  static const List<String> digits = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
  ];

  static const List<String> special = [
    'SPACE',
    'END',
    'UPARROW',
    'DOWNARROW',
    'LEFTARROW',
    'RIGHTARROW',
    'LSHIFT',
    'RSHIFT',
    'LCTRL',
    'RCTRL',
  ];

  static const List<String> mouseButtons = [
    'MOUSE1',
    'MOUSE2',
    'MOUSE3',
    'MOUSE4',
    'MOUSE5',
  ];

  static const List<String> genericModifiers = ['SHIFT', 'CTRL', 'ALT'];

  static const List<String> sideModifiers = [
    'LSHIFT',
    'RSHIFT',
    'LCTRL',
    'RCTRL',
  ];

  static final List<String> all = [
    ...letters,
    ...digits,
    ...special,
    ...mouseButtons,
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

  static bool comboContainsMouse(String binding) =>
      comboParts(binding).any(mouseButtons.contains);

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
    }
    return null;
  }

  static String? vkToKeyName(int vk) {
    if (vk >= 0x41 && vk <= 0x5A) return String.fromCharCode(vk);
    if (vk >= 0x30 && vk <= 0x39) return String.fromCharCode(vk);
    switch (vk) {
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
