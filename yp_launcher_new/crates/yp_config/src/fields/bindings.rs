use yp_i18n::MsgKey;

use super::{AnyField, ConfigField, FieldMeta};

pub static STANDARD_MOVE_FORWARD: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_move_forward", MsgKey::LabelBindMoveForward)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindMoveForward),
    String::new,
);

pub static STANDARD_MOVE_BACKWARD: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_move_backward", MsgKey::LabelBindMoveBackward)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindMoveBackward),
    String::new,
);

pub static STANDARD_MOVE_LEFT: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_move_left", MsgKey::LabelBindMoveLeft)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindMoveLeft),
    String::new,
);

pub static STANDARD_MOVE_RIGHT: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_move_right", MsgKey::LabelBindMoveRight)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindMoveRight),
    String::new,
);

pub static STANDARD_JUMP: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_jump", MsgKey::LabelBindJump)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindJump),
    String::new,
);

pub static STANDARD_WALK: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_walk", MsgKey::LabelBindWalk)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindWalk),
    String::new,
);

pub static STANDARD_AUTO_RUN: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_auto_run", MsgKey::LabelBindAutoRun)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindAutoRun),
    String::new,
);

pub static STANDARD_LIGHT_ATTACK: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_light_attack", MsgKey::LabelBindLightAttack)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindLightAttack),
    String::new,
);

pub static STANDARD_HEAVY_ATTACK: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_heavy_attack", MsgKey::LabelBindHeavyAttack)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindHeavyAttack),
    String::new,
);

pub static STANDARD_FIRE: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_fire", MsgKey::LabelBindFire)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindFire),
    String::new,
);

pub static STANDARD_PROGRAM: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_program", MsgKey::LabelBindProgram)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindProgram),
    String::new,
);

pub static STANDARD_LOCK_ON: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_lock_on", MsgKey::LabelBindLockOn)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindLockOn),
    String::new,
);

pub static STANDARD_USE: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_use", MsgKey::LabelBindUse)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindUse),
    String::new,
);

pub static STANDARD_SELF_DESTRUCT: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_self_destruct", MsgKey::LabelBindSelfDestruct)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindSelfDestruct),
    String::new,
);

pub static STANDARD_LIGHT: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_light", MsgKey::LabelBindLight)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindLight),
    String::new,
);

pub static STANDARD_RESET_CAMERA: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_reset_camera", MsgKey::LabelBindResetCamera)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindResetCamera),
    String::new,
);

pub static STANDARD_SWITCH_WEAPON: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_switch_weapon", MsgKey::LabelBindSwitchWeapon)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindSwitchWeapon),
    String::new,
);

pub static STANDARD_NEXT_PROGRAM: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_next_program", MsgKey::LabelBindNextProgram)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindNextProgram),
    String::new,
);

pub static STANDARD_PREVIOUS_PROGRAM: ConfigField<String> = ConfigField::new(
    FieldMeta::new(
        "standard_previous_program",
        MsgKey::LabelBindPreviousProgram,
    )
    .section("mouse.bindings")
    .tooltip(MsgKey::TooltipBindPreviousProgram),
    String::new,
);

pub static STANDARD_MENU_UP: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_menu_up", MsgKey::LabelBindMenuUp)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindMenuUp),
    String::new,
);

pub static STANDARD_MENU_DOWN: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_menu_down", MsgKey::LabelBindMenuDown)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindMenuDown),
    String::new,
);

pub static STANDARD_MENU_LEFT: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_menu_left", MsgKey::LabelBindMenuLeft)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindMenuLeft),
    String::new,
);

pub static STANDARD_MENU_RIGHT: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_menu_right", MsgKey::LabelBindMenuRight)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindMenuRight),
    String::new,
);

pub static STANDARD_MENU_OPEN: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_menu_open", MsgKey::LabelBindMenuOpen)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindMenuOpen),
    String::new,
);

pub static STANDARD_MENU_BACK: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_menu_back", MsgKey::LabelBindMenuBack)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindMenuBack),
    String::new,
);

pub static STANDARD_MENU_ENTER: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_menu_enter", MsgKey::LabelBindMenuEnter)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindMenuEnter),
    String::new,
);

pub static STANDARD_SHORTCUT_MENU: ConfigField<String> = ConfigField::new(
    FieldMeta::new("standard_shortcut_menu", MsgKey::LabelBindShortcutMenu)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindShortcutMenu),
    String::new,
);

pub static NON_STANDARD_EVADE: ConfigField<String> = ConfigField::new(
    FieldMeta::new("non_standard_evade", MsgKey::LabelBindEvade)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindEvade),
    String::new,
);

pub static NON_STANDARD_AUTO_FIRE: ConfigField<String> = ConfigField::new(
    FieldMeta::new("non_standard_auto_fire", MsgKey::LabelBindAutoFire)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindAutoFire),
    String::new,
);

pub static NON_STANDARD_NEXT_ITEM: ConfigField<String> = ConfigField::new(
    FieldMeta::new("non_standard_next_item", MsgKey::LabelBindNextItem)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindNextItem),
    String::new,
);

pub static NON_STANDARD_PREVIOUS_ITEM: ConfigField<String> = ConfigField::new(
    FieldMeta::new("non_standard_previous_item", MsgKey::LabelBindPreviousItem)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindPreviousItem),
    String::new,
);

pub static NON_STANDARD_USE_ITEM: ConfigField<String> = ConfigField::new(
    FieldMeta::new("non_standard_use_item", MsgKey::LabelBindUseItem)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindUseItem),
    String::new,
);

pub static NON_STANDARD_POD_PET: ConfigField<String> = ConfigField::new(
    FieldMeta::new("non_standard_pod_pet", MsgKey::LabelBindPodPet)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindPodPet),
    String::new,
);

pub static THIRD_PERSON_MODE_TOGGLE: ConfigField<String> = ConfigField::new(
    FieldMeta::new(
        "third_person_mode_toggle",
        MsgKey::LabelBindThirdPersonToggle,
    )
    .section("mouse.bindings")
    .tooltip(MsgKey::TooltipBindThirdPersonToggle),
    String::new,
);

pub static AIM_MODE_TOGGLE: ConfigField<String> = ConfigField::new(
    FieldMeta::new("aim_mode_toggle", MsgKey::LabelBindAimToggle)
        .section("mouse.bindings")
        .tooltip(MsgKey::TooltipBindAimToggle),
    String::new,
);

pub const SECTION: &str = "mouse.bindings";

pub static ALL: [&ConfigField<String>; 35] = [
    &STANDARD_MOVE_FORWARD,
    &STANDARD_MOVE_BACKWARD,
    &STANDARD_MOVE_LEFT,
    &STANDARD_MOVE_RIGHT,
    &STANDARD_JUMP,
    &STANDARD_WALK,
    &STANDARD_AUTO_RUN,
    &STANDARD_LIGHT_ATTACK,
    &STANDARD_HEAVY_ATTACK,
    &STANDARD_FIRE,
    &STANDARD_PROGRAM,
    &STANDARD_LOCK_ON,
    &STANDARD_USE,
    &STANDARD_SELF_DESTRUCT,
    &STANDARD_LIGHT,
    &STANDARD_RESET_CAMERA,
    &STANDARD_SWITCH_WEAPON,
    &STANDARD_NEXT_PROGRAM,
    &STANDARD_PREVIOUS_PROGRAM,
    &STANDARD_MENU_UP,
    &STANDARD_MENU_DOWN,
    &STANDARD_MENU_LEFT,
    &STANDARD_MENU_RIGHT,
    &STANDARD_MENU_OPEN,
    &STANDARD_MENU_BACK,
    &STANDARD_MENU_ENTER,
    &STANDARD_SHORTCUT_MENU,
    &NON_STANDARD_EVADE,
    &NON_STANDARD_AUTO_FIRE,
    &NON_STANDARD_NEXT_ITEM,
    &NON_STANDARD_PREVIOUS_ITEM,
    &NON_STANDARD_USE_ITEM,
    &NON_STANDARD_POD_PET,
    &THIRD_PERSON_MODE_TOGGLE,
    &AIM_MODE_TOGGLE,
];

pub fn all() -> impl Iterator<Item = &'static dyn AnyField> {
    ALL.iter().map(|field| *field as &'static dyn AnyField)
}
