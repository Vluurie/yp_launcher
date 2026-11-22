use std::path::{Path, PathBuf};

use gpui::SharedString;
use yp_core::GamePaths;
use yp_i18n::Strings;
use yp_thirdparty::ReShadeStatus;
use yp_ui::IconName;

pub const STEP_COUNT: usize = 6;
const LIST_SEPARATOR: &str = ", ";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum Step {
    #[default]
    Welcome,
    SelectGame,
    FirstPlaythrough,
    Migration,
    Mods,
    Ready,
}

impl Step {
    pub const ALL: [Step; STEP_COUNT] = [
        Step::Welcome,
        Step::SelectGame,
        Step::FirstPlaythrough,
        Step::Migration,
        Step::Mods,
        Step::Ready,
    ];

    pub fn index(self) -> usize {
        Step::ALL
            .iter()
            .position(|step| *step == self)
            .unwrap_or_default()
    }

    pub fn from_index(index: usize) -> Option<Step> {
        Step::ALL.get(index).copied()
    }

    pub fn following(self) -> Option<Step> {
        Step::from_index(self.index() + 1)
    }

    pub fn previous(self) -> Option<Step> {
        self.index().checked_sub(1).and_then(Step::from_index)
    }

    pub fn needs_findings(self) -> bool {
        matches!(self, Step::Migration | Step::Ready)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationFindings {
    pub reshade: ReShadeStatus,
    pub hd_textures: Vec<&'static str>,
    pub has_lodmod: bool,
    pub has_sk_res: bool,
    pub has_naiom: bool,
    pub has_cutscene_mods: bool,
    pub existing_mods: usize,
}

impl Default for MigrationFindings {
    fn default() -> Self {
        Self {
            reshade: ReShadeStatus::NotFound,
            hd_textures: Vec::new(),
            has_lodmod: false,
            has_sk_res: false,
            has_naiom: false,
            has_cutscene_mods: false,
            existing_mods: 0,
        }
    }
}

fn sub_directories(dir: &Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default()
}

fn is_mod_folder(name: &str) -> bool {
    !name.starts_with('.') && !name.starts_with('_')
}

pub fn detect_migration(game: &GamePaths) -> MigrationFindings {
    let special_k_root = game
        .special_k_textures_dir()
        .ancestors()
        .nth(2)
        .map(Path::to_path_buf);
    MigrationFindings {
        reshade: yp_thirdparty::detect_reshade(game),
        hd_textures: yp_host::game_probe::detect_hd_textures(game.root()),
        has_lodmod: yp_config::legacy::detect_lodmod_ini(game)
            .ok()
            .flatten()
            .is_some(),
        has_sk_res: special_k_root.is_some_and(|dir| dir.is_dir()),
        has_naiom: yp_config::legacy::detect_naiom_ini(game)
            .ok()
            .flatten()
            .is_some(),
        has_cutscene_mods: !sub_directories(&game.cutscenes_dir()).is_empty(),
        existing_mods: sub_directories(&game.mods_dir())
            .iter()
            .filter(|name| is_mod_folder(name))
            .count(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationRow {
    pub icon: IconName,
    pub label: SharedString,
    pub action: &'static str,
    pub ok: bool,
}

pub fn migration_rows(findings: &MigrationFindings, strings: &Strings) -> Vec<MigrationRow> {
    let mut rows = Vec::new();
    let mut push = |icon: IconName, label: SharedString, action: &'static str, ok: bool| {
        rows.push(MigrationRow {
            icon,
            label,
            action,
            ok,
        });
    };
    if findings.existing_mods > 0 {
        push(
            IconName::ExtensionOutlined,
            strings
                .onboarding_migration_label_existing_mods(findings.existing_mods as i64)
                .into(),
            strings.onboarding_migration_action_existing_mods(),
            true,
        );
    }
    match findings.reshade {
        ReShadeStatus::Detected => push(
            IconName::AutoFixHigh,
            strings.detection_re_shade().into(),
            strings.onboarding_migration_action_reshade_kept(),
            true,
        ),
        ReShadeStatus::IncompatibleAddon => push(
            IconName::AutoFixHigh,
            strings.detection_re_shade().into(),
            strings.onboarding_migration_action_reshade_incompatible(),
            false,
        ),
        ReShadeStatus::NotFound => {}
    }
    if !findings.hd_textures.is_empty() {
        push(
            IconName::Image,
            strings.detection_hd_textures().into(),
            strings.onboarding_migration_action_textures(),
            true,
        );
    }
    if findings.has_lodmod {
        push(
            IconName::Visibility,
            strings.detection_lod_mod().into(),
            strings.onboarding_migration_action_lod_mod(),
            true,
        );
    }
    if findings.has_sk_res {
        push(
            IconName::Texture,
            strings.detection_sk_res().into(),
            strings.onboarding_migration_action_sk_res(),
            true,
        );
    }
    if findings.has_naiom {
        push(
            IconName::Mouse,
            strings.detection_naiom().into(),
            strings.onboarding_migration_action_naiom(),
            true,
        );
    }
    if findings.has_cutscene_mods {
        push(
            IconName::Movie,
            strings.detection_cutscenes().into(),
            strings.onboarding_migration_action_cutscenes(),
            true,
        );
    }
    rows
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummaryRow {
    pub icon: IconName,
    pub label: &'static str,
    pub value: SharedString,
}

pub fn summary_rows(
    path: Option<&Path>,
    first_playthrough: bool,
    findings: &MigrationFindings,
    strings: &Strings,
) -> Vec<SummaryRow> {
    let mut rows = vec![
        SummaryRow {
            icon: IconName::Folder,
            label: strings.label_game(),
            value: match path {
                Some(path) => path.to_string_lossy().into_owned().into(),
                None => strings.detection_not_set().into(),
            },
        },
        SummaryRow {
            icon: IconName::Visibility,
            label: strings.label_mode(),
            value: if first_playthrough {
                strings.onboarding_first_playthrough_spoiler_free().into()
            } else {
                strings.onboarding_full_access().into()
            },
        },
    ];
    let mut push = |icon: IconName, label: &'static str, value: SharedString| {
        rows.push(SummaryRow { icon, label, value });
    };
    match findings.reshade {
        ReShadeStatus::Detected => push(
            IconName::AutoFixHigh,
            strings.detection_re_shade(),
            strings.detection_detected().into(),
        ),
        ReShadeStatus::IncompatibleAddon => push(
            IconName::AutoFixHigh,
            strings.detection_re_shade(),
            strings.detection_re_shade_incompatible().into(),
        ),
        ReShadeStatus::NotFound => {}
    }
    if !findings.hd_textures.is_empty() {
        push(
            IconName::Image,
            strings.detection_hd_textures(),
            findings.hd_textures.join(LIST_SEPARATOR).into(),
        );
    }
    if findings.has_lodmod {
        push(
            IconName::Visibility,
            strings.detection_lod_mod(),
            strings.detection_migrated_into_nams().into(),
        );
    }
    if findings.has_sk_res {
        push(
            IconName::Texture,
            strings.detection_sk_res(),
            strings.detection_loaded_automatically().into(),
        );
    }
    if findings.has_naiom {
        push(
            IconName::Mouse,
            strings.detection_naiom(),
            strings.detection_migration_coming_soon().into(),
        );
    }
    if findings.has_cutscene_mods {
        push(
            IconName::Movie,
            strings.detection_cutscenes(),
            strings.detection_installed().into(),
        );
    }
    rows
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnboardingFlow {
    step: Step,
    selected_path: Option<PathBuf>,
    first_playthrough: bool,
    had_mods_before: Option<bool>,
    wants_mod_install: Option<bool>,
    create_shortcut: bool,
    findings: Option<MigrationFindings>,
}

impl Default for OnboardingFlow {
    fn default() -> Self {
        Self::new(None)
    }
}

impl OnboardingFlow {
    pub fn new(selected_path: Option<PathBuf>) -> Self {
        Self {
            step: Step::Welcome,
            selected_path,
            first_playthrough: true,
            had_mods_before: None,
            wants_mod_install: None,
            create_shortcut: true,
            findings: None,
        }
    }

    pub fn step(&self) -> Step {
        self.step
    }

    pub fn selected_path(&self) -> Option<&Path> {
        self.selected_path.as_deref()
    }

    pub fn first_playthrough(&self) -> bool {
        self.first_playthrough
    }

    pub fn had_mods_before(&self) -> Option<bool> {
        self.had_mods_before
    }

    pub fn wants_mod_install(&self) -> Option<bool> {
        self.wants_mod_install
    }

    pub fn create_shortcut(&self) -> bool {
        self.create_shortcut
    }

    pub fn findings(&self) -> Option<&MigrationFindings> {
        self.findings.as_ref()
    }

    pub fn needs_detection(&self) -> bool {
        self.step.needs_findings() && self.findings.is_none() && self.selected_path.is_some()
    }

    pub fn can_leave_select_game(&self) -> bool {
        self.selected_path.is_some()
    }

    pub fn advance(&mut self) -> bool {
        if self.step == Step::SelectGame && !self.can_leave_select_game() {
            return false;
        }
        match self.step.following() {
            Some(step) => {
                self.step = step;
                true
            }
            None => false,
        }
    }

    pub fn go_back(&mut self) -> bool {
        if self.step == Step::Migration && self.had_mods_before == Some(true) {
            self.had_mods_before = None;
            return true;
        }
        match self.step.previous() {
            Some(step) => {
                self.step = step;
                true
            }
            None => false,
        }
    }

    pub fn jump_to(&mut self, step: Step) {
        self.step = step;
    }

    pub fn set_selected_path(&mut self, path: PathBuf) {
        if self.selected_path.as_ref() != Some(&path) {
            self.findings = None;
        }
        self.selected_path = Some(path);
    }

    pub fn set_first_playthrough(&mut self, first_playthrough: bool) {
        self.first_playthrough = first_playthrough;
    }

    pub fn answer_had_mods(&mut self, had_mods: bool) {
        if had_mods {
            self.had_mods_before = Some(true);
        } else {
            self.had_mods_before = Some(false);
            self.advance();
        }
    }

    pub fn answer_wants_mod_install(&mut self, wants: bool) {
        self.wants_mod_install = Some(wants);
        if !wants {
            self.advance();
        }
    }

    pub fn set_create_shortcut(&mut self, create: bool) {
        self.create_shortcut = create;
    }

    pub fn set_findings(&mut self, path: &Path, findings: MigrationFindings) {
        if self.selected_path.as_deref() == Some(path) {
            self.findings = Some(findings);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yp_i18n::Locale;

    fn strings() -> &'static Strings {
        Strings::for_locale(Locale::En)
    }

    #[test]
    fn steps_run_in_order_and_stop_at_both_ends() {
        let mut flow = OnboardingFlow::new(Some(PathBuf::from("G:/game")));
        assert!(!flow.go_back());
        let mut visited = vec![flow.step()];
        while flow.advance() {
            visited.push(flow.step());
        }
        assert_eq!(visited, Step::ALL.to_vec());
        assert!(!flow.advance());
        assert_eq!(flow.step(), Step::Ready);
    }

    #[test]
    fn game_selection_is_required_before_continuing() {
        let mut flow = OnboardingFlow::new(None);
        assert!(flow.advance());
        assert_eq!(flow.step(), Step::SelectGame);
        assert!(!flow.advance());
        assert_eq!(flow.step(), Step::SelectGame);
        flow.set_selected_path(PathBuf::from("G:/game"));
        assert!(flow.advance());
        assert_eq!(flow.step(), Step::FirstPlaythrough);
    }

    #[test]
    fn detection_runs_once_per_selected_folder() {
        let mut flow = OnboardingFlow::new(Some(PathBuf::from("G:/game")));
        flow.jump_to(Step::Migration);
        assert!(flow.needs_detection());
        flow.set_findings(Path::new("G:/other"), MigrationFindings::default());
        assert!(flow.needs_detection());
        flow.set_findings(Path::new("G:/game"), MigrationFindings::default());
        assert!(!flow.needs_detection());
        flow.set_selected_path(PathBuf::from("G:/game"));
        assert!(!flow.needs_detection());
        flow.set_selected_path(PathBuf::from("H:/game"));
        assert!(flow.needs_detection());
        flow.jump_to(Step::Mods);
        assert!(!flow.needs_detection());
    }

    #[test]
    fn migration_question_branches() {
        let mut flow = OnboardingFlow::new(Some(PathBuf::from("G:/game")));
        flow.jump_to(Step::Migration);
        flow.answer_had_mods(true);
        assert_eq!(flow.step(), Step::Migration);
        assert_eq!(flow.had_mods_before(), Some(true));
        assert!(flow.go_back());
        assert_eq!(flow.step(), Step::Migration);
        assert_eq!(flow.had_mods_before(), None);
        flow.answer_had_mods(false);
        assert_eq!(flow.step(), Step::Mods);
    }

    #[test]
    fn skipping_mod_install_moves_on() {
        let mut flow = OnboardingFlow::new(Some(PathBuf::from("G:/game")));
        flow.jump_to(Step::Mods);
        flow.answer_wants_mod_install(true);
        assert_eq!(flow.step(), Step::Mods);
        flow.answer_wants_mod_install(false);
        assert_eq!(flow.step(), Step::Ready);
    }

    #[test]
    fn migration_rows_follow_the_findings() {
        assert!(migration_rows(&MigrationFindings::default(), strings()).is_empty());
        let findings = MigrationFindings {
            reshade: ReShadeStatus::IncompatibleAddon,
            hd_textures: vec!["SK_Res/"],
            has_lodmod: true,
            has_sk_res: true,
            has_naiom: true,
            has_cutscene_mods: true,
            existing_mods: 2,
        };
        let rows = migration_rows(&findings, strings());
        assert_eq!(rows.len(), 7);
        assert_eq!(rows[0].label.as_ref(), "2 mods in nams/mods/");
        assert!(!rows[1].ok);
        assert!(rows.iter().skip(2).all(|row| row.ok));
    }

    #[test]
    fn summary_always_names_game_and_mode() {
        let rows = summary_rows(None, true, &MigrationFindings::default(), strings());
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].value.as_ref(), strings().detection_not_set());
        assert_eq!(
            rows[1].value.as_ref(),
            strings().onboarding_first_playthrough_spoiler_free()
        );
        let findings = MigrationFindings {
            reshade: ReShadeStatus::Detected,
            ..MigrationFindings::default()
        };
        let rows = summary_rows(Some(Path::new("G:/game")), false, &findings, strings());
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].value.as_ref(), "G:/game");
        assert_eq!(rows[1].value.as_ref(), strings().onboarding_full_access());
    }

    #[test]
    fn detection_reads_a_game_folder() {
        let root = tempfile::tempdir().unwrap();
        let game = GamePaths::new(root.path());
        assert_eq!(detect_migration(&game), MigrationFindings::default());
        std::fs::create_dir_all(game.mods_dir().join("outfit")).unwrap();
        std::fs::create_dir_all(game.mods_dir().join(".hidden")).unwrap();
        std::fs::create_dir_all(game.mods_dir().join("_staging")).unwrap();
        std::fs::create_dir_all(game.cutscenes_dir().join("hd")).unwrap();
        std::fs::create_dir_all(game.special_k_textures_dir()).unwrap();
        let findings = detect_migration(&game);
        assert_eq!(findings.existing_mods, 1);
        assert!(findings.has_cutscene_mods);
        assert!(findings.has_sk_res);
        assert_eq!(findings.hd_textures, vec!["SK_Res/"]);
    }
}
