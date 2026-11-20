use yp_mods::camera::CameraField;
use yp_mods::defaults::DefaultKind;
use yp_mods::grouping::ModGroupKind;
use yp_mods::{
    DataCategory, DropRejection, InstallFailure, ModKind, ModsError, OutfitChoice, ProfileRejection,
};
use yp_core::ArchiveFailure;
use yp_state::archive_failure_key;
use yp_ui::{ChipTone, IconName, MsgKey, Strings};

pub fn group_label(kind: ModGroupKind, strings: &Strings) -> &'static str {
    match kind {
        ModGroupKind::Mixed => strings.mod_group_mixed(),
        ModGroupKind::Wax => strings.mod_group_wax(),
        ModGroupKind::Outfit2b => strings.mod_group2b(),
        ModGroupKind::Outfit9s => strings.mod_group9s(),
        ModGroupKind::OutfitA2 => strings.mod_group_a2(),
        ModGroupKind::OutfitOther => strings.mod_group_other_outfits(),
        ModGroupKind::Weapon => strings.mod_group_weapons(),
        ModGroupKind::Accessory => strings.mod_group_accessories(),
        ModGroupKind::Item => strings.mod_group_items(),
        ModGroupKind::Enemy => strings.mod_group_enemies(),
        ModGroupKind::WorldProp => strings.mod_group_world_props(),
        ModGroupKind::ModelVariant => strings.mod_group_model_variants(),
        ModGroupKind::Map => strings.mod_group_maps(),
        ModGroupKind::Effect => strings.mod_group_effects(),
        ModGroupKind::Scripting => strings.mod_group_scripting(),
        ModGroupKind::Localization => strings.mod_group_localization(),
        ModGroupKind::Ui => strings.mod_group_ui(),
        ModGroupKind::Cutscene => strings.mod_group_cutscenes(),
        ModGroupKind::Audio => strings.mod_group_audio(),
        ModGroupKind::Texture => strings.mod_group_textures(),
        ModGroupKind::Misc => strings.mod_group_misc(),
        ModGroupKind::Archive => strings.mod_group_archives(),
        ModGroupKind::Native => strings.mod_group_native(),
        ModGroupKind::Other => strings.mod_group_other(),
    }
}

pub fn group_icon(kind: ModGroupKind) -> IconName {
    match kind {
        ModGroupKind::Mixed => IconName::DashboardCustomizeOutlined,
        ModGroupKind::Wax
        | ModGroupKind::Outfit2b
        | ModGroupKind::Outfit9s
        | ModGroupKind::OutfitA2
        | ModGroupKind::OutfitOther => IconName::CheckroomOutlined,
        ModGroupKind::Weapon => IconName::GavelOutlined,
        ModGroupKind::Accessory => IconName::DiamondOutlined,
        ModGroupKind::Item => IconName::Inventory2Outlined,
        ModGroupKind::Enemy => IconName::BugReportOutlined,
        ModGroupKind::WorldProp => IconName::ChairOutlined,
        ModGroupKind::ModelVariant => IconName::CategoryOutlined,
        ModGroupKind::Map => IconName::MapOutlined,
        ModGroupKind::Effect => IconName::AutoAwesomeOutlined,
        ModGroupKind::Scripting => IconName::CodeOutlined,
        ModGroupKind::Localization => IconName::TranslateOutlined,
        ModGroupKind::Ui => IconName::WebOutlined,
        ModGroupKind::Cutscene => IconName::MovieCreationOutlined,
        ModGroupKind::Audio => IconName::VolumeUpOutlined,
        ModGroupKind::Texture => IconName::TextureOutlined,
        ModGroupKind::Misc => IconName::ImageOutlined,
        ModGroupKind::Archive => IconName::ArchiveOutlined,
        ModGroupKind::Native => IconName::ExtensionOutlined,
        ModGroupKind::Other => IconName::FolderOutlined,
    }
}

pub struct KindBadge {
    pub label: &'static str,
    pub tooltip: &'static str,
    pub tone: ChipTone,
}

pub fn kind_badge(kind: ModKind, strings: &Strings) -> KindBadge {
    match kind {
        ModKind::Native => KindBadge {
            label: strings.mod_kind_native(),
            tooltip: strings.mod_kind_native_tooltip(),
            tone: ChipTone::Accent,
        },
        ModKind::Data => KindBadge {
            label: strings.mod_kind_data(),
            tooltip: strings.mod_kind_data_tooltip(),
            tone: ChipTone::Muted,
        },
        ModKind::Texture => KindBadge {
            label: strings.mod_kind_texture(),
            tooltip: strings.mod_kind_texture_tooltip(),
            tone: ChipTone::Accent,
        },
        ModKind::Unknown => KindBadge {
            label: strings.mod_kind_unknown(),
            tooltip: strings.mod_kind_unknown_tooltip(),
            tone: ChipTone::Error,
        },
    }
}

pub fn variant_category_label(category: Option<DataCategory>, strings: &Strings) -> &'static str {
    match category {
        Some(DataCategory::Player) => strings.variant_cat_player(),
        Some(DataCategory::Weapon) => strings.variant_cat_weapon(),
        Some(DataCategory::Accessory) => strings.variant_cat_accessory(),
        Some(DataCategory::Enemy) => strings.variant_cat_enemy(),
        Some(DataCategory::ModelVariant) => strings.variant_cat_model_variant(),
        Some(DataCategory::Item) => strings.variant_cat_item(),
        Some(DataCategory::WorldProp) => strings.variant_cat_world_prop(),
        Some(DataCategory::Map) => strings.variant_cat_map(),
        Some(DataCategory::Effects) => strings.variant_cat_effects(),
        Some(DataCategory::Scripting) => strings.variant_cat_scripting(),
        Some(DataCategory::Localization) => strings.variant_cat_localization(),
        Some(DataCategory::Ui) => strings.variant_cat_ui(),
        Some(DataCategory::Cutscenes) => strings.variant_cat_cutscenes(),
        Some(DataCategory::Audio) => strings.variant_cat_audio(),
        Some(DataCategory::Misc) => strings.variant_cat_misc(),
        Some(DataCategory::Archive) | Some(DataCategory::Other) | None => {
            strings.variant_cat_other()
        }
    }
}

pub fn data_category_label(category: DataCategory, dir_name: &str, strings: &Strings) -> String {
    match category {
        DataCategory::Player => strings.mods_ui_cat_player().to_owned(),
        DataCategory::Weapon => strings.mods_ui_cat_weapon().to_owned(),
        DataCategory::Enemy => strings.mods_ui_cat_enemy().to_owned(),
        DataCategory::Accessory => strings.mods_ui_cat_accessory().to_owned(),
        DataCategory::Item => strings.mods_ui_cat_item().to_owned(),
        DataCategory::WorldProp => strings.mods_ui_cat_world_prop(dir_name),
        DataCategory::ModelVariant => strings.mods_ui_cat_model_variant().to_owned(),
        DataCategory::Map => strings.mods_ui_cat_map(dir_name),
        DataCategory::Scripting => strings.mods_ui_cat_scripting(dir_name),
        DataCategory::Localization => strings.mods_ui_cat_localization(dir_name),
        DataCategory::Effects => strings.mods_ui_cat_effects(dir_name),
        DataCategory::Ui => strings.mods_ui_cat_ui(dir_name),
        DataCategory::Misc => strings.mods_ui_cat_misc().to_owned(),
        DataCategory::Cutscenes => strings.mods_ui_cat_cutscenes().to_owned(),
        DataCategory::Audio => strings.mods_ui_cat_audio().to_owned(),
        DataCategory::Archive => strings.mods_ui_cat_archive().to_owned(),
        DataCategory::Other => strings.mods_ui_cat_other(dir_name),
    }
}

pub fn default_kind_label(kind: DefaultKind, strings: &Strings) -> &'static str {
    match kind {
        DefaultKind::OutfitBare => strings.mod_default_kind_outfit_bare(),
        DefaultKind::OutfitConfig => strings.mod_default_kind_outfit_config(),
        DefaultKind::OutfitAnimation => strings.mod_default_kind_outfit_animation(),
    }
}

pub fn default_kind_tooltip(kind: DefaultKind, strings: &Strings) -> &'static str {
    match kind {
        DefaultKind::OutfitBare => strings.mod_default_kind_outfit_bare_tooltip(),
        DefaultKind::OutfitConfig => strings.mod_default_kind_outfit_config_tooltip(),
        DefaultKind::OutfitAnimation => strings.mod_default_kind_outfit_animation_tooltip(),
    }
}

pub fn camera_field_label(field: CameraField, strings: &Strings) -> &'static str {
    match field {
        CameraField::SideOffset => strings.mod_camera_side_offset(),
        CameraField::HeightOffset => strings.mod_camera_height_offset(),
        CameraField::DistanceOffset => strings.mod_camera_distance_offset(),
        CameraField::FovOffset => strings.mod_camera_fov_offset(),
        CameraField::PitchUpLimit => strings.mod_camera_pitch_up_limit(),
        CameraField::PitchDownLimit => strings.mod_camera_pitch_down_limit(),
    }
}

pub fn camera_field_hint(field: CameraField, strings: &Strings) -> &'static str {
    match field {
        CameraField::SideOffset => strings.mod_camera_side_offset_hint(),
        CameraField::HeightOffset => strings.mod_camera_height_offset_hint(),
        CameraField::DistanceOffset => strings.mod_camera_distance_offset_hint(),
        CameraField::FovOffset => strings.mod_camera_fov_offset_hint(),
        CameraField::PitchUpLimit => strings.mod_camera_pitch_up_limit_hint(),
        CameraField::PitchDownLimit => strings.mod_camera_pitch_down_limit_hint(),
    }
}

pub fn outfit_choice_label(choice: &OutfitChoice, strings: &Strings) -> String {
    match &choice.name {
        Some(name) if !name.trim().is_empty() => name.clone(),
        _ => strings.mods_ui_outfit_fallback(choice.outfit_id),
    }
}

pub fn drop_rejection_reason(rejection: DropRejection, strings: &Strings) -> &'static str {
    match rejection {
        DropRejection::TextureOnly => strings.mod_install_reason_texture_only(),
        DropRejection::ArchiveExtractFailed(ArchiveFailure::UnsupportedFormat) => {
            strings.mod_install_reason_archive_extract_failed()
        }
        DropRejection::ArchiveExtractFailed(failure) => archive_failure_key(failure).text(strings),
        DropRejection::UnknownDrop => strings.mod_install_reason_unknown_drop(),
        DropRejection::UnsupportedNasa => strings.mod_install_reason_unsupported_nasa(),
        DropRejection::InvalidMixed => strings.mod_install_reason_invalid_mixed(),
    }
}

pub fn install_failure_reason(failure: &InstallFailure, strings: &Strings) -> String {
    match failure {
        InstallFailure::Rejected(rejection) => {
            drop_rejection_reason(*rejection, strings).to_owned()
        }
        InstallFailure::VariantMissing => strings.mods_ui_reason_variant_missing().to_owned(),
        InstallFailure::Exists { id } => strings.mod_install_exists_pick_another(id),
        InstallFailure::NativeEmpty => strings.mod_install_reason_native_empty().to_owned(),
        InstallFailure::DataEmpty => strings.mod_install_reason_data_empty().to_owned(),
        InstallFailure::MoveFailed { .. } => strings.mod_install_reason_move_failed().to_owned(),
        InstallFailure::Io { path, .. } => strings.mods_ui_reason_io(&path.to_string_lossy()),
        InstallFailure::Cancelled => strings.mods_ui_reason_cancelled().to_owned(),
    }
}

pub fn profile_rejection_key(rejection: ProfileRejection) -> MsgKey {
    match rejection {
        ProfileRejection::InvalidName => MsgKey::ModProfileErrorNameInvalid,
        ProfileRejection::NameCollision => MsgKey::ModProfileErrorNameCollision,
        ProfileRejection::UnknownTarget
        | ProfileRejection::TargetMissing
        | ProfileRejection::ActiveMissing => MsgKey::ModProfileErrorTargetMissing,
        ProfileRejection::CannotDeleteLast => MsgKey::ModProfileErrorDeleteLast,
        ProfileRejection::CannotDeleteActive => MsgKey::ModProfileErrorDeleteActive,
    }
}

pub fn profile_error_key(error: Option<&ModsError>) -> MsgKey {
    match error {
        Some(ModsError::Profile(rejection)) => profile_rejection_key(*rejection),
        Some(ModsError::ProfileStep { .. }) | Some(ModsError::Core(_)) | None => {
            MsgKey::ModProfileErrorFsBusy
        }
    }
}

const KIND_DOC_PAGES: [(&str, &str); 22] = [
    ("item", "03_item.md"),
    ("weapon", "04_weapon.md"),
    ("weapon_patch", "30_weapon_patch.md"),
    ("outfit", "05_outfit.md"),
    ("accessory", "06_accessory.md"),
    ("quest", "07_quest.md"),
    ("mail", "08_mail.md"),
    ("reward", "09_reward.md"),
    ("voice_bank", "10_voice_lines.md"),
    ("hairspray", "19_hairspray.md"),
    ("shop", "03_item.md"),
    ("shop_item", "03_item.md"),
    ("ruby_patch", "24_ruby_patches.md"),
    ("char_name", "12_translate.md"),
    ("effect_area", "29_effect_areas.md"),
    ("mesh_visibility", "28_mesh_visibility.md"),
    ("music", "27_custom_music.md"),
    ("scenario_variables", "23_scenario_variables.md"),
    ("phase", "25_phases.md"),
    ("graphic_adjust", "26_graphic_adjust.md"),
    ("hap_group", "31_hap_groups.md"),
    ("custom_map", "32_custom_maps.md"),
];

const NATIVE_DOCS_DIR: &str = "docs/native";

pub fn kind_doc_page(kind: &str) -> Option<String> {
    KIND_DOC_PAGES
        .iter()
        .find(|(name, _)| *name == kind)
        .map(|(_, page)| format!("{NATIVE_DOCS_DIR}/{page}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use yp_ui::Locale;

    fn strings() -> &'static Strings {
        Strings::for_locale(Locale::En)
    }

    #[test]
    fn every_install_failure_has_a_reason() {
        let failures = [
            InstallFailure::Rejected(DropRejection::TextureOnly),
            InstallFailure::Rejected(DropRejection::ArchiveExtractFailed(ArchiveFailure::UnsupportedFormat)),
            InstallFailure::Rejected(DropRejection::ArchiveExtractFailed(ArchiveFailure::UnsupportedMethod)),
            InstallFailure::Rejected(DropRejection::ArchiveExtractFailed(ArchiveFailure::PasswordRequired)),
            InstallFailure::Rejected(DropRejection::ArchiveExtractFailed(ArchiveFailure::Corrupt)),
            InstallFailure::Rejected(DropRejection::ArchiveExtractFailed(ArchiveFailure::MissingVolume)),
            InstallFailure::Rejected(DropRejection::ArchiveExtractFailed(ArchiveFailure::DiskFull)),
            InstallFailure::Rejected(DropRejection::ArchiveExtractFailed(ArchiveFailure::NoFiles)),
            InstallFailure::Rejected(DropRejection::UnknownDrop),
            InstallFailure::Rejected(DropRejection::UnsupportedNasa),
            InstallFailure::Rejected(DropRejection::InvalidMixed),
            InstallFailure::VariantMissing,
            InstallFailure::Exists {
                id: "kimono".to_owned(),
            },
            InstallFailure::NativeEmpty,
            InstallFailure::DataEmpty,
            InstallFailure::MoveFailed {
                path: "x".into(),
                kind: std::io::ErrorKind::PermissionDenied,
            },
            InstallFailure::Cancelled,
        ];
        for failure in failures {
            assert!(
                !install_failure_reason(&failure, strings()).is_empty(),
                "{failure:?}"
            );
        }
        assert!(
            install_failure_reason(
                &InstallFailure::Exists {
                    id: "kimono".to_owned()
                },
                strings()
            )
            .contains("kimono")
        );
    }

    #[test]
    fn profile_errors_map_rejections_and_fall_back_to_busy() {
        assert_eq!(
            profile_error_key(Some(&ModsError::Profile(
                ProfileRejection::CannotDeleteActive
            ))),
            MsgKey::ModProfileErrorDeleteActive
        );
        assert_eq!(
            profile_error_key(Some(&ModsError::Profile(ProfileRejection::NameCollision))),
            MsgKey::ModProfileErrorNameCollision
        );
        assert_eq!(
            profile_error_key(Some(&ModsError::Profile(ProfileRejection::TargetMissing))),
            MsgKey::ModProfileErrorTargetMissing
        );
        assert_eq!(profile_error_key(None), MsgKey::ModProfileErrorFsBusy);
        for rejection in [
            ProfileRejection::InvalidName,
            ProfileRejection::NameCollision,
            ProfileRejection::UnknownTarget,
            ProfileRejection::TargetMissing,
            ProfileRejection::ActiveMissing,
            ProfileRejection::CannotDeleteLast,
            ProfileRejection::CannotDeleteActive,
        ] {
            assert!(!profile_rejection_key(rejection).text(strings()).is_empty());
        }
    }

    #[test]
    fn unnamed_outfits_fall_back_to_their_id() {
        let named = OutfitChoice {
            outfit_id: 7,
            name: Some("Kimono".to_owned()),
            needs_item: false,
        };
        let unnamed = OutfitChoice {
            outfit_id: 42,
            name: None,
            needs_item: true,
        };
        assert_eq!(outfit_choice_label(&named, strings()), "Kimono");
        assert_eq!(outfit_choice_label(&unnamed, strings()), "Outfit 42");
    }

    #[test]
    fn native_kinds_link_to_their_guide_pages() {
        assert_eq!(
            kind_doc_page("weapon").as_deref(),
            Some("docs/native/04_weapon.md")
        );
        assert_eq!(kind_doc_page("unknown_kind"), None);
    }

    #[test]
    fn every_group_has_a_label() {
        for kind in yp_mods::grouping::MOD_GROUP_ORDER {
            assert!(!group_label(kind, strings()).is_empty());
        }
    }
}
