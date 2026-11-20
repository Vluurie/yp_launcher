use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tempfile::TempDir;
use yp_core::GamePaths;
use yp_mods::profiles::{ProfileFs, RealProfileFs};
use yp_mods::{ModProfiles, ModsError, ProfileRejection, ProfileStep, disabled};

struct Fixture {
    _dir: TempDir,
    game: GamePaths,
}

impl Fixture {
    fn new() -> Self {
        let dir = tempfile::Builder::new()
            .prefix("yp_profiles_")
            .tempdir()
            .unwrap();
        let game = GamePaths::new(dir.path());
        std::fs::create_dir_all(game.mods_dir()).unwrap();
        Self { _dir: dir, game }
    }

    fn nams(&self, name: &str) -> PathBuf {
        self.game.nams_dir().join(name)
    }

    fn profiles(&self) -> ModProfiles<'_> {
        ModProfiles::new(&self.game)
    }

    fn write_sidecars(&self, marker: &str) {
        std::fs::write(
            self.nams("disabled_mods.toml"),
            format!("disabled = [\"mods/{marker}\"]\n"),
        )
        .unwrap();
        std::fs::write(
            self.nams("default_mods.toml"),
            format!("outfit_bare = [\"mods/{marker}\"]\n"),
        )
        .unwrap();
        std::fs::write(
            self.nams("mod_load_order.toml"),
            format!("load_order = [\"mods/{marker}\"]\n"),
        )
        .unwrap();
    }

    fn has_marker(&self, file: &str, marker: &str) -> bool {
        std::fs::read_to_string(self.nams(file))
            .map(|text| text.contains(&format!("mods/{marker}")))
            .unwrap_or(false)
    }

    fn expect_active_sidecars(&self, marker: &str) {
        for file in [
            "disabled_mods.toml",
            "default_mods.toml",
            "mod_load_order.toml",
        ] {
            assert!(self.has_marker(file, marker), "{file} lacks {marker}");
        }
    }

    fn expect_parked_sidecars(&self, profile: &str, marker: &str) {
        for prefix in [
            "disabled_mods_profile_",
            "default_mods_profile_",
            "mod_load_order_profile_",
        ] {
            let file = format!("{prefix}{profile}.toml");
            assert!(self.has_marker(&file, marker), "{file} lacks {marker}");
        }
    }

    fn expect_no_active_sidecars(&self) {
        for file in [
            "disabled_mods.toml",
            "default_mods.toml",
            "mod_load_order.toml",
        ] {
            assert!(!self.nams(file).exists(), "{file} should not exist");
        }
    }

    fn disabled(&self) -> Vec<String> {
        let mut entries = disabled::list(&self.game).unwrap();
        entries.sort();
        entries
    }

    fn set_disabled(&self, rel: &str, value: bool) {
        disabled::set_disabled(&self.game, rel, value).unwrap();
    }
}

fn sorted(items: &[&str]) -> Vec<String> {
    let mut out: Vec<String> = items.iter().map(|item| (*item).to_owned()).collect();
    out.sort();
    out
}

#[test]
fn create_parks_every_sidecar_under_the_previous_profile() {
    let fx = Fixture::new();
    fx.write_sidecars("from_default");
    fx.profiles().create("second").unwrap();
    fx.expect_parked_sidecars("default", "from_default");
    fx.expect_no_active_sidecars();
}

#[test]
fn create_a_fresh_profile_starts_with_an_empty_mods_folder() {
    let fx = Fixture::new();
    std::fs::write(fx.game.mods_dir().join("marker.txt"), "x").unwrap();
    let state = fx.profiles().create("second").unwrap();
    assert_eq!(state.active_name, "second");
    assert_eq!(std::fs::read_dir(fx.game.mods_dir()).unwrap().count(), 0);
    assert!(fx.nams("mods_profile_default").join("marker.txt").is_file());
}

#[test]
fn switch_shared_texture_disables_survive_profile_creation_and_switching() {
    let fx = Fixture::new();
    fx.set_disabled("inject/textures/pack", true);
    fx.set_disabled("mods/alpha", true);

    fx.profiles().create("beta").unwrap();
    assert_eq!(fx.disabled(), sorted(&["inject/textures/pack"]));
    fx.set_disabled("mods/beta", true);

    fx.profiles().switch_to("default").unwrap();
    assert_eq!(
        fx.disabled(),
        sorted(&["mods/alpha", "inject/textures/pack"])
    );

    fx.profiles().switch_to("beta").unwrap();
    assert_eq!(
        fx.disabled(),
        sorted(&["mods/beta", "inject/textures/pack"])
    );
}

#[test]
fn switch_re_enabled_textures_stay_enabled_when_an_older_profile_is_restored() {
    let fx = Fixture::new();
    fx.set_disabled("inject/textures/pack", true);
    fx.profiles().create("beta").unwrap();
    fx.set_disabled("inject/textures/pack", false);

    fx.profiles().switch_to("default").unwrap();
    assert!(fx.disabled().is_empty());

    fx.profiles().switch_to("beta").unwrap();
    assert!(fx.disabled().is_empty());
}

#[test]
fn switch_every_sidecar_follows_the_profile_it_belongs_to() {
    let fx = Fixture::new();
    fx.write_sidecars("alpha");
    fx.profiles().create("beta").unwrap();
    fx.write_sidecars("beta");

    fx.profiles().switch_to("default").unwrap();

    fx.expect_active_sidecars("alpha");
    fx.expect_parked_sidecars("beta", "beta");
}

#[test]
fn switch_switching_back_restores_the_other_profile_again() {
    let fx = Fixture::new();
    fx.write_sidecars("alpha");
    fx.profiles().create("beta").unwrap();
    fx.write_sidecars("beta");

    fx.profiles().switch_to("default").unwrap();
    fx.profiles().switch_to("beta").unwrap();

    fx.expect_active_sidecars("beta");
    fx.expect_parked_sidecars("default", "alpha");
}

#[test]
fn switch_mods_folders_swap_with_the_profile() {
    let fx = Fixture::new();
    std::fs::write(fx.game.mods_dir().join("alpha.txt"), "x").unwrap();
    fx.profiles().create("beta").unwrap();
    std::fs::write(fx.game.mods_dir().join("beta.txt"), "x").unwrap();

    fx.profiles().switch_to("default").unwrap();

    assert!(fx.game.mods_dir().join("alpha.txt").is_file());
    assert!(!fx.game.mods_dir().join("beta.txt").exists());
}

#[test]
fn switch_a_profile_with_no_sidecars_leaves_none_behind() {
    let fx = Fixture::new();
    fx.profiles().create("beta").unwrap();
    fx.write_sidecars("beta");

    fx.profiles().switch_to("default").unwrap();

    fx.expect_no_active_sidecars();
    fx.expect_parked_sidecars("beta", "beta");
}

#[test]
fn switch_a_partial_sidecar_set_travels_without_inventing_the_others() {
    let fx = Fixture::new();
    std::fs::write(
        fx.nams("default_mods.toml"),
        "outfit_bare = [\"mods/only_default\"]\n",
    )
    .unwrap();

    fx.profiles().create("second").unwrap();

    assert!(fx.nams("default_mods_profile_default.toml").is_file());
    assert!(!fx.nams("disabled_mods_profile_default.toml").exists());
    assert!(!fx.nams("mod_load_order_profile_default.toml").exists());
}

#[test]
fn switch_to_the_active_profile_is_a_no_op() {
    let fx = Fixture::new();
    fx.write_sidecars("alpha");
    fx.profiles().switch_to("default").unwrap();
    fx.expect_active_sidecars("alpha");
}

#[test]
fn rename_an_inactive_profile_takes_all_its_sidecars_along() {
    let fx = Fixture::new();
    fx.write_sidecars("alpha");
    fx.profiles().create("beta").unwrap();

    fx.profiles().rename_profile("default", "renamed").unwrap();

    fx.expect_parked_sidecars("renamed", "alpha");
    assert!(!fx.nams("default_mods_profile_default.toml").exists());
}

#[test]
fn delete_removes_every_sidecar_of_the_deleted_profile() {
    let fx = Fixture::new();
    fx.write_sidecars("alpha");
    fx.profiles().create("beta").unwrap();

    fx.profiles().delete("default").unwrap();

    assert!(!fx.nams("disabled_mods_profile_default.toml").exists());
    assert!(!fx.nams("default_mods_profile_default.toml").exists());
    assert!(!fx.nams("mod_load_order_profile_default.toml").exists());
}

#[test]
fn delete_leaves_the_surviving_profile_untouched() {
    let fx = Fixture::new();
    fx.profiles().create("beta").unwrap();
    fx.write_sidecars("beta");
    fx.profiles().create("gamma").unwrap();

    fx.profiles().delete("default").unwrap();

    fx.expect_parked_sidecars("beta", "beta");
}

fn rejection(result: Result<yp_mods::ModProfileState, ModsError>) -> Option<ProfileRejection> {
    result.err().and_then(|err| err.profile_rejection())
}

#[test]
fn profile_requests_are_validated() {
    let fx = Fixture::new();
    assert_eq!(
        rejection(fx.profiles().create("___")),
        Some(ProfileRejection::InvalidName)
    );
    assert_eq!(
        rejection(fx.profiles().create("mods")),
        Some(ProfileRejection::InvalidName)
    );
    assert_eq!(
        rejection(fx.profiles().create("Default")),
        Some(ProfileRejection::NameCollision)
    );
    assert_eq!(
        rejection(fx.profiles().switch_to("nope")),
        Some(ProfileRejection::UnknownTarget)
    );
    assert_eq!(
        rejection(fx.profiles().delete("default")),
        Some(ProfileRejection::CannotDeleteLast)
    );
    fx.profiles().create("beta").unwrap();
    assert_eq!(
        rejection(fx.profiles().delete("beta")),
        Some(ProfileRejection::CannotDeleteActive)
    );
    assert_eq!(
        rejection(fx.profiles().delete("ghost")),
        Some(ProfileRejection::UnknownTarget)
    );
    assert_eq!(
        rejection(fx.profiles().rename_profile("ghost", "new")),
        Some(ProfileRejection::UnknownTarget)
    );
    assert_eq!(
        rejection(fx.profiles().rename_profile("beta", "default")),
        Some(ProfileRejection::NameCollision)
    );
}

#[test]
fn first_run_writes_profiles_toml_and_adopts_stray_folders() {
    let fx = Fixture::new();
    std::fs::create_dir_all(fx.nams("mods_profile_extra")).unwrap();
    std::fs::create_dir_all(fx.nams("mods_profile_Bad Name")).unwrap();
    let state = fx.profiles().load().unwrap();
    assert_eq!(state.active_name, "default");
    let names: Vec<&str> = state
        .profiles
        .iter()
        .map(|profile| profile.name.as_str())
        .collect();
    assert_eq!(names, vec!["default", "extra"]);
    let text = std::fs::read_to_string(fx.game.profiles_toml()).unwrap();
    assert!(text.starts_with(
        "# YP Launcher mod profile state. Edit via the launcher.\nactive = \"default\""
    ));
    let reloaded = fx.profiles().load().unwrap();
    assert_eq!(reloaded.profiles.len(), 2);
}

#[test]
fn bundled_packs_are_scoped_and_disabled_per_profile() {
    let fx = Fixture::new();
    std::fs::write(
        fx.game.texture_injection_toml(),
        "load_order = [\"HD\"]\ndisabled_packs = []\n",
    )
    .unwrap();
    let kimono = fx.game.mods_dir().join("kimono");
    std::fs::create_dir_all(&kimono).unwrap();
    std::fs::write(kimono.join(".bundled_textures.json"), r#"{"packs":["HD"]}"#).unwrap();
    std::fs::create_dir_all(fx.game.inject_textures_dir().join("HD")).unwrap();

    fx.profiles().load().unwrap();
    assert!(
        fx.game
            .inject_textures_dir()
            .join("HD (kimono@default)")
            .is_dir()
    );
    let toml = std::fs::read_to_string(fx.game.texture_injection_toml()).unwrap();
    assert!(
        toml.contains("load_order = [\"HD (kimono@default)\"]"),
        "{toml}"
    );

    fx.profiles().create("beta").unwrap();
    let toml = std::fs::read_to_string(fx.game.texture_injection_toml()).unwrap();
    assert!(
        toml.contains("disabled_packs = [\"HD (kimono@default)\"]"),
        "{toml}"
    );

    fx.profiles().rename_profile("default", "main").unwrap();
    assert!(
        fx.game
            .inject_textures_dir()
            .join("HD (kimono@main)")
            .is_dir()
    );
    let toml = std::fs::read_to_string(fx.game.texture_injection_toml()).unwrap();
    assert!(
        toml.contains("disabled_packs = [\"HD (kimono@main)\"]"),
        "{toml}"
    );
    assert!(
        toml.contains("load_order = [\"HD (kimono@main)\"]"),
        "{toml}"
    );

    fx.profiles().switch_to("main").unwrap();
    let toml = std::fs::read_to_string(fx.game.texture_injection_toml()).unwrap();
    assert!(toml.contains("disabled_packs = []"), "{toml}");

    fx.profiles().delete("beta").unwrap();
    fx.profiles().create("gamma").unwrap();
    fx.profiles().delete("main").unwrap();
    assert!(
        !fx.game
            .inject_textures_dir()
            .join("HD (kimono@main)")
            .exists()
    );
    let toml = std::fs::read_to_string(fx.game.texture_injection_toml()).unwrap();
    assert!(toml.contains("load_order = []"), "{toml}");
}

struct FailingFs {
    fail_rename_to: Mutex<Option<PathBuf>>,
    fail_rename_from: Mutex<Option<PathBuf>>,
    fail_write_to: Mutex<Option<PathBuf>>,
    fail_create: Mutex<Option<PathBuf>>,
}

impl FailingFs {
    fn new() -> Self {
        Self {
            fail_rename_to: Mutex::new(None),
            fail_rename_from: Mutex::new(None),
            fail_write_to: Mutex::new(None),
            fail_create: Mutex::new(None),
        }
    }

    fn injected() -> io::Error {
        io::Error::other("injected failure")
    }

    fn take_if(slot: &Mutex<Option<PathBuf>>, path: &Path) -> bool {
        let mut guard = slot.lock().unwrap();
        if guard.as_deref() == Some(path) {
            *guard = None;
            return true;
        }
        false
    }
}

impl ProfileFs for FailingFs {
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        if Self::take_if(&self.fail_rename_to, to) || Self::take_if(&self.fail_rename_from, from) {
            return Err(Self::injected());
        }
        RealProfileFs.rename(from, to)
    }

    fn create_dir(&self, path: &Path) -> io::Result<()> {
        if Self::take_if(&self.fail_create, path) {
            return Err(Self::injected());
        }
        RealProfileFs.create_dir(path)
    }

    fn remove_dir(&self, path: &Path) -> io::Result<()> {
        RealProfileFs.remove_dir(path)
    }

    fn write(&self, path: &Path, contents: &str) -> io::Result<()> {
        if Self::take_if(&self.fail_write_to, path) {
            return Err(Self::injected());
        }
        RealProfileFs.write(path, contents)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        RealProfileFs.remove_file(path)
    }
}

fn two_profiles() -> Fixture {
    let fx = Fixture::new();
    std::fs::write(
        fx.game.texture_injection_toml(),
        "load_order = []\ndisabled_packs = []\n",
    )
    .unwrap();
    fx.write_sidecars("alpha");
    std::fs::write(fx.game.mods_dir().join("alpha.txt"), "x").unwrap();
    fx.profiles().create("beta").unwrap();
    fx.write_sidecars("beta");
    std::fs::write(fx.game.mods_dir().join("beta.txt"), "x").unwrap();
    fx
}

fn expect_untouched_beta(fx: &Fixture) {
    assert!(fx.game.mods_dir().join("beta.txt").is_file());
    assert!(!fx.game.mods_dir().join("alpha.txt").exists());
    assert!(fx.nams("mods_profile_default").join("alpha.txt").is_file());
    assert!(!fx.nams("mods_profile_beta").exists());
    fx.expect_active_sidecars("beta");
    fx.expect_parked_sidecars("default", "alpha");
    assert_eq!(
        ModProfiles::new(&fx.game).load().unwrap().active_name,
        "beta"
    );
}

fn step(result: Result<yp_mods::ModProfileState, ModsError>) -> Option<ProfileStep> {
    result.err().and_then(|err| err.profile_step())
}

#[test]
fn switch_rolls_back_when_parking_mods_fails() {
    let fx = two_profiles();
    let fs = FailingFs::new();
    *fs.fail_rename_to.lock().unwrap() = Some(fx.nams("mods_profile_beta"));
    let result = ModProfiles::with_fs(&fx.game, &fs).switch_to("default");
    assert_eq!(step(result), Some(ProfileStep::ParkMods));
    expect_untouched_beta(&fx);
}

#[test]
fn switch_rolls_back_when_activating_the_target_mods_fails() {
    let fx = two_profiles();
    let fs = FailingFs::new();
    *fs.fail_rename_from.lock().unwrap() = Some(fx.nams("mods_profile_default"));
    let result = ModProfiles::with_fs(&fx.game, &fs).switch_to("default");
    assert_eq!(step(result), Some(ProfileStep::ActivateMods));
    expect_untouched_beta(&fx);
}

#[test]
fn switch_rolls_back_when_activating_the_target_sidecars_fails() {
    let fx = two_profiles();
    let fs = FailingFs::new();
    *fs.fail_rename_from.lock().unwrap() = Some(fx.nams("mod_load_order_profile_default.toml"));
    let result = ModProfiles::with_fs(&fx.game, &fs).switch_to("default");
    assert_eq!(step(result), Some(ProfileStep::ActivateSidecars));
    expect_untouched_beta(&fx);
}

#[test]
fn switch_rolls_back_when_parking_sidecars_fails() {
    let fx = two_profiles();
    let fs = FailingFs::new();
    *fs.fail_rename_to.lock().unwrap() = Some(fx.nams("mod_load_order_profile_beta.toml"));
    let result = ModProfiles::with_fs(&fx.game, &fs).switch_to("default");
    assert_eq!(step(result), Some(ProfileStep::ParkSidecars));
    expect_untouched_beta(&fx);
}

#[test]
fn switch_rolls_back_when_the_texture_state_cannot_be_written() {
    let fx = two_profiles();
    fx.set_disabled("inject/textures/shared", true);
    let before_disabled = std::fs::read_to_string(fx.nams("disabled_mods.toml")).unwrap();
    let fs = FailingFs::new();
    *fs.fail_write_to.lock().unwrap() = Some(fx.game.disabled_mods_toml());
    let result = ModProfiles::with_fs(&fx.game, &fs).switch_to("default");
    assert_eq!(step(result), Some(ProfileStep::TextureState));
    expect_untouched_beta(&fx);
    assert_eq!(
        std::fs::read_to_string(fx.nams("disabled_mods.toml")).unwrap(),
        before_disabled
    );
}

#[test]
fn switch_reports_a_profiles_write_failure_after_the_swap() {
    let fx = two_profiles();
    let fs = FailingFs::new();
    *fs.fail_write_to.lock().unwrap() = Some(fx.game.profiles_toml());
    let result = ModProfiles::with_fs(&fx.game, &fs).switch_to("default");
    assert_eq!(step(result), Some(ProfileStep::WriteProfiles));
    assert!(fx.game.mods_dir().join("alpha.txt").is_file());
}

#[test]
fn create_rolls_back_when_the_fresh_mods_folder_cannot_be_created() {
    let fx = Fixture::new();
    fx.write_sidecars("alpha");
    std::fs::write(fx.game.mods_dir().join("alpha.txt"), "x").unwrap();
    let fs = FailingFs::new();
    *fs.fail_create.lock().unwrap() = Some(fx.game.mods_dir());
    let result = ModProfiles::with_fs(&fx.game, &fs).create("beta");
    assert_eq!(step(result), Some(ProfileStep::CreateMods));
    assert!(fx.game.mods_dir().join("alpha.txt").is_file());
    assert!(!fx.nams("mods_profile_default").exists());
    fx.expect_active_sidecars("alpha");
    let state = ModProfiles::new(&fx.game).load().unwrap();
    assert_eq!(state.active_name, "default");
    assert_eq!(state.profiles.len(), 1);
}

#[test]
fn create_rolls_back_when_the_texture_state_cannot_be_written() {
    let fx = Fixture::new();
    fx.write_sidecars("alpha");
    fx.set_disabled("inject/textures/shared", true);
    std::fs::write(fx.game.mods_dir().join("alpha.txt"), "x").unwrap();
    let fs = FailingFs::new();
    *fs.fail_write_to.lock().unwrap() = Some(fx.game.disabled_mods_toml());
    let result = ModProfiles::with_fs(&fx.game, &fs).create("beta");
    assert_eq!(step(result), Some(ProfileStep::TextureState));
    assert!(fx.game.mods_dir().join("alpha.txt").is_file());
    assert!(!fx.nams("mods_profile_default").exists());
    assert!(fx.has_marker("disabled_mods.toml", "alpha"));
    fx.expect_active_sidecars("alpha");
}

#[test]
fn rename_rolls_back_the_folder_when_sidecars_cannot_follow() {
    let fx = two_profiles();
    let fs = FailingFs::new();
    *fs.fail_rename_from.lock().unwrap() = Some(fx.nams("default_mods_profile_default.toml"));
    let result = ModProfiles::with_fs(&fx.game, &fs).rename_profile("default", "main");
    assert_eq!(step(result), Some(ProfileStep::RenameSidecars));
    assert!(fx.nams("mods_profile_default").join("alpha.txt").is_file());
    assert!(!fx.nams("mods_profile_main").exists());
    fx.expect_parked_sidecars("default", "alpha");
}
