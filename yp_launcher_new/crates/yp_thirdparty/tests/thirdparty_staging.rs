mod common;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use common::{archive_of_dir, core_fixture, fresh_game, nams_flag, temp, write, write_marked_dll, write_plain_dll};
use tempfile::TempDir;
use yp_core::archive::Extractor;
use yp_core::testing::archives::{write_encrypted_zip, write_raw_zip};
use yp_core::{ArchiveFailure, CancelToken, GamePaths, HostEnv, NoProgress, Os, Progress};
use yp_thirdparty::staging::{source_kind, stage_source, LOOSE_FILE_TEMP_PREFIX, PICKER_EXTENSIONS};
use yp_thirdparty::{
    DropRoute, LoadingFlag, RuntimePresence, SourceKind, StageFailure, StagedSource, ThirdPartyKind, ThirdPartyRuntime,
    ThirdPartyState,
};

struct Bench {
    root: TempDir,
    host: HostEnv,
}

fn bench() -> Bench {
    let root = temp("yp_tp_stage_");
    let host = HostEnv::for_test(Os::Windows, &[], root.path());
    Bench { root, host }
}

impl Bench {
    fn drop_dir(&self) -> PathBuf {
        let dir = self.root.path().join("drop");
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn stage(&self, source: &Path) -> Result<StagedSource, StageFailure> {
        stage_source(&Extractor::new(), &self.host, None, source, &NoProgress, &CancelToken::new())
    }
}

fn extract_failure(result: Result<StagedSource, StageFailure>) -> Option<ArchiveFailure> {
    match result {
        Err(StageFailure::ExtractFailed(error)) => error.archive_failure(),
        other => panic!("expected a failed extraction, got {other:?}"),
    }
}

fn dir_names(dir: &Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .map(|read| read.filter_map(Result::ok).map(|entry| entry.file_name().to_string_lossy().into_owned()).collect())
        .unwrap_or_default()
}

mod what_a_drop_is {
    use super::*;

    #[test]
    fn the_extension_decides_before_anything_else() {
        let bench = bench();
        let drop = bench.drop_dir();
        for name in ["Pack.zip", "Pack.7Z", "pack.rar"] {
            assert_eq!(source_kind(&drop.join(name)), SourceKind::Archive, "{name}");
        }
        for name in ["pack.tar", "pack.GZ", "pack.bz2", "pack.tar.gz"] {
            assert_eq!(source_kind(&drop.join(name)), SourceKind::Unsupported, "{name}");
        }
        assert_eq!(source_kind(&drop.join("ReShade_Setup_6.7.3.EXE")), SourceKind::Executable);
        assert_eq!(source_kind(&drop.join("Preset.INI")), SourceKind::LooseFile);
        assert_eq!(source_kind(&drop.join("celestial.dll")), SourceKind::LooseFile);
        assert_eq!(source_kind(&drop), SourceKind::Folder);
        assert_eq!(source_kind(&drop.join("notes.txt")), SourceKind::Unsupported);
        assert_eq!(source_kind(&drop.join("missing-folder")), SourceKind::Unsupported);

        let folder_named_like_an_archive = drop.join("Unpacked.zip");
        std::fs::create_dir(&folder_named_like_an_archive).unwrap();
        assert_eq!(source_kind(&folder_named_like_an_archive), SourceKind::Archive);
    }

    #[test]
    fn the_file_picker_offers_what_staging_handles() {
        let bench = bench();
        let drop = bench.drop_dir();
        for extension in PICKER_EXTENSIONS {
            let kind = source_kind(&drop.join(format!("file.{extension}")));
            assert!(
                matches!(kind, SourceKind::Archive | SourceKind::Executable | SourceKind::LooseFile),
                "{extension}"
            );
        }
    }

    #[test]
    fn every_kind_has_one_route() {
        let install = [
            ThirdPartyKind::ReShadeWholeInstall,
            ThirdPartyKind::ReShadePreset,
            ThirdPartyKind::Migoto,
            ThirdPartyKind::GameMod,
            ThirdPartyKind::GraphicMod,
        ];
        let unsupported = [
            ThirdPartyKind::LodMod,
            ThirdPartyKind::SpecialK,
            ThirdPartyKind::SkShaderEdit,
            ThirdPartyKind::BrokenLegacy,
            ThirdPartyKind::MultiBundle,
            ThirdPartyKind::Unknown,
        ];
        for kind in install {
            assert_eq!(kind.drop_route(), DropRoute::Install, "{kind:?}");
            assert!(kind.is_installable());
        }
        for kind in unsupported {
            assert_eq!(kind.drop_route(), DropRoute::Unsupported, "{kind:?}");
        }
        assert_eq!(ThirdPartyKind::GameData.drop_route(), DropRoute::ModsTab);
        assert_eq!(ThirdPartyKind::Textures.drop_route(), DropRoute::TexturesTab);
    }
}

mod folders {
    use super::*;

    #[test]
    fn a_dropped_folder_is_used_as_it_is() {
        let bench = bench();
        let folder = bench.drop_dir().join("Celestial_Timer-42");
        write_plain_dll(&folder.join("celestial.dll"));

        let staged = bench.stage(&folder).unwrap();

        assert_eq!(staged.root(), folder);
        assert_eq!(staged.source(), folder);
        assert!(!staged.is_temporary());
        let c = staged.classify();
        assert_eq!(c.kind, ThirdPartyKind::GameMod);
        assert_eq!(c.source_name.as_deref(), Some("Celestial Timer"));

        drop(staged);
        assert!(folder.join("celestial.dll").is_file(), "the user's folder is never deleted");
    }

    #[test]
    fn a_single_wrapping_folder_is_unwrapped_by_classify() {
        let bench = bench();
        let folder = bench.drop_dir().join("download");
        write_marked_dll(&folder.join("Pack").join("d3d11.dll"), "HackerDevice");

        let c = bench.stage(&folder).unwrap().classify();

        assert_eq!(c.kind, ThirdPartyKind::Migoto);
        assert_eq!(c.source_root, folder.join("Pack"));
    }
}

mod loose_files {
    use super::*;

    #[test]
    fn a_dropped_dll_is_copied_into_a_fresh_temp_folder() {
        let bench = bench();
        let dll = bench.drop_dir().join("celestial.dll");
        write_plain_dll(&dll);
        write(&bench.drop_dir().join("unrelated.ini"), "x=1");

        let staged = bench.stage(&dll).unwrap();

        assert!(staged.is_temporary());
        assert!(staged.root().starts_with(bench.host.temp_dir()));
        assert!(staged.root().file_name().unwrap().to_string_lossy().starts_with(LOOSE_FILE_TEMP_PREFIX));
        assert_eq!(dir_names(staged.root()), ["celestial.dll"], "only the dropped file is staged");
        let c = staged.classify();
        assert_eq!(c.kind, ThirdPartyKind::GameMod);
        assert_eq!(c.game_mod_dlls, [staged.root().join("celestial.dll")]);
        assert_eq!(c.source_name.as_deref(), Some("celestial"));

        let root = staged.root().to_path_buf();
        drop(staged);
        assert!(!root.exists(), "the temp folder goes away with the staged source");
        assert!(dll.is_file(), "the dropped file stays where it was");
    }

    #[test]
    fn a_dropped_preset_ini_is_a_reshade_preset() {
        let bench = bench();
        let preset = bench.drop_dir().join("Cinematic_Look.ini");
        write(&preset, "Techniques=SMAA@SMAA.fx\n");

        let staged = bench.stage(&preset).unwrap();
        let c = staged.classify();

        assert_eq!(c.kind, ThirdPartyKind::ReShadePreset);
        assert_eq!(c.kind.drop_route(), DropRoute::Install);
        assert_eq!(c.source_name.as_deref(), Some("Cinematic Look"));
    }

    #[test]
    fn the_temp_folder_lives_in_the_game_folder_when_one_is_given() {
        let bench = bench();
        let game = GamePaths::new(bench.root.path().join("game"));
        let dll = bench.drop_dir().join("celestial.dll");
        write_plain_dll(&dll);

        let staged =
            stage_source(&Extractor::new(), &bench.host, Some(&game), &dll, &NoProgress, &CancelToken::new()).unwrap();

        assert!(staged.root().starts_with(game.scoped_temp_root()));
    }

    #[test]
    fn a_file_that_vanished_stages_nothing_and_routes_to_unsupported() {
        let bench = bench();

        let staged = bench.stage(&bench.drop_dir().join("gone.dll")).unwrap();

        assert!(dir_names(staged.root()).is_empty());
        let c = staged.classify();
        assert_eq!(c.kind, ThirdPartyKind::Unknown);
        assert_eq!(c.kind.drop_route(), DropRoute::Unsupported);
    }
}

mod archives {
    use super::*;

    #[test]
    fn a_dropped_archive_is_extracted_into_a_temp_folder() {
        for extension in ["zip", "7z", "rar"] {
            let bench = bench();
            let content = bench.root.path().join("content");
            write_marked_dll(&content.join("3DMigoto").join("d3d11.dll"), "HackerDevice 3DMigoto");
            write(&content.join("3DMigoto").join("d3dx.ini"), "[Loader]\n;target = NieRAutomata.exe\n");
            write(&content.join("3DMigoto").join("ShaderFixes").join("abc-ps_replace.txt"), "fix");
            let archive = bench.drop_dir().join(format!("Bloom_Fix-769.{extension}"));
            archive_of_dir(&content, &archive);
            let seen = Mutex::new(Vec::new());
            let progress = |progress: Progress| seen.lock().unwrap().push(progress);

            let staged =
                stage_source(&Extractor::new(), &bench.host, None, &archive, &progress, &CancelToken::new()).unwrap();

            assert!(staged.is_temporary(), "{extension}");
            assert!(staged.root().starts_with(bench.host.temp_dir()), "{extension}");
            assert!(staged.root().join("3DMigoto").join("d3d11.dll").is_file(), "{extension}");
            assert_eq!(
                std::fs::read_to_string(staged.root().join("3DMigoto").join("ShaderFixes").join("abc-ps_replace.txt"))
                    .unwrap(),
                "fix",
                "{extension}"
            );
            let c = staged.classify();
            assert_eq!(c.kind, ThirdPartyKind::Migoto, "{extension}");
            assert_eq!(c.source_root, staged.root().join("3DMigoto"), "{extension}");
            assert_eq!(c.source_name.as_deref(), Some("Bloom Fix"), "{extension}");
            assert_eq!(seen.lock().unwrap().last().and_then(|progress| progress.fraction), Some(1.0), "{extension}");

            let root = staged.root().to_path_buf();
            drop(staged);
            assert!(!root.exists(), "{extension}");
            assert!(archive.is_file(), "{extension}");
        }
    }

    #[test]
    fn a_broken_archive_fails_and_leaves_no_temp_folder_behind() {
        for name in ["broken.zip", "broken.7z", "broken.rar"] {
            let bench = bench();
            let archive = bench.drop_dir().join(name);
            write(&archive, "not an archive");

            assert_eq!(extract_failure(bench.stage(&archive)), Some(ArchiveFailure::Corrupt), "{name}");
            assert!(dir_names(bench.host.temp_dir()).is_empty(), "{name}");
        }
    }

    #[test]
    fn an_archive_that_yields_no_files_fails() {
        let bench = bench();
        let zip = bench.drop_dir().join("empty.zip");
        write_raw_zip(&zip, &[]);

        assert_eq!(extract_failure(bench.stage(&zip)), Some(ArchiveFailure::NoFiles));
        assert!(dir_names(bench.host.temp_dir()).is_empty());
    }

    #[test]
    fn a_password_protected_archive_fails_with_its_reason() {
        let bench = bench();
        let locked = bench.drop_dir().join("locked.zip");
        write_encrypted_zip(&locked, b"secret", &[("ReShade/dxgi.dll", b"MZ")]);

        for archive in [locked, core_fixture("enc_headers.7z"), core_fixture("enc_data.rar")] {
            assert_eq!(
                extract_failure(bench.stage(&archive)),
                Some(ArchiveFailure::PasswordRequired),
                "{}",
                archive.display()
            );
        }
        assert!(dir_names(bench.host.temp_dir()).is_empty());
    }

    #[test]
    fn a_cancelled_archive_drop_leaves_no_temp_folder_behind() {
        let bench = bench();
        let content = bench.root.path().join("content");
        std::fs::create_dir_all(&content).unwrap();
        std::fs::write(content.join("big.bin"), yp_core::testing::archives::noise(1024 * 1024, 5)).unwrap();
        let archive = bench.drop_dir().join("big.7z");
        archive_of_dir(&content, &archive);
        let cancel = CancelToken::new();
        let stop = cancel.clone();
        let progress = move |progress: Progress| {
            if progress.fraction.is_some_and(|fraction| fraction > 0.0) {
                stop.cancel();
            }
        };

        let result = stage_source(&Extractor::new(), &bench.host, None, &archive, &progress, &cancel);

        assert!(matches!(result, Err(StageFailure::Cancelled)));
        assert!(dir_names(bench.host.temp_dir()).is_empty());
    }
}

mod refusals {
    use super::*;

    #[test]
    fn a_dropped_exe_is_refused_without_unpacking_it() {
        let bench = bench();
        let exe = bench.drop_dir().join("ReShade_Setup_6.7.3_Addon.exe");
        write_raw_zip(&exe, &[yp_core::testing::archives::RawZipEntry::utf8("ReShade64.dll", b"MZ")]);

        let result = bench.stage(&exe);

        assert!(matches!(result, Err(StageFailure::ExecutableNotSupported)));
        assert!(!bench.host.temp_dir().exists() || dir_names(bench.host.temp_dir()).is_empty());
    }

    #[test]
    fn tarballs_are_unsupported_files_not_broken_archives() {
        let bench = bench();
        for name in ["shaders.tar", "shaders.tar.gz", "shaders.gz", "shaders.bz2"] {
            let tarball = bench.drop_dir().join(name);
            write(&tarball, "not something the launcher unpacks");

            assert_eq!(source_kind(&tarball), SourceKind::Unsupported, "{name}");
            assert!(matches!(bench.stage(&tarball), Err(StageFailure::UnsupportedFile)), "{name}");
        }
        assert!(!bench.host.temp_dir().exists() || dir_names(bench.host.temp_dir()).is_empty());
    }

    #[test]
    fn other_files_and_missing_paths_are_unsupported() {
        let bench = bench();
        let notes = bench.drop_dir().join("notes.txt");
        write(&notes, "hello");

        assert!(matches!(bench.stage(&notes), Err(StageFailure::UnsupportedFile)));
        assert!(matches!(bench.stage(&bench.drop_dir().join("nowhere")), Err(StageFailure::UnsupportedFile)));
    }

    #[test]
    fn a_cancelled_drop_stages_nothing() {
        let bench = bench();
        let dll = bench.drop_dir().join("celestial.dll");
        write_plain_dll(&dll);
        let cancel = CancelToken::new();
        cancel.cancel();

        let result = stage_source(&Extractor::new(), &bench.host, None, &dll, &NoProgress, &cancel);

        assert!(matches!(result, Err(StageFailure::Cancelled)));
        assert!(!bench.host.temp_dir().exists() || dir_names(bench.host.temp_dir()).is_empty());
    }
}

mod from_drop_to_install {
    use super::*;

    #[test]
    fn a_staged_dll_installs_through_the_state_while_it_is_alive() {
        let bench = bench();
        let fresh = fresh_game("yp_tp_stage_game_", "disable_game_mods_loading = true\n");
        let dll = bench.drop_dir().join("Celestial_Timer-12.dll");
        write_plain_dll(&dll);
        let mut state = ThirdPartyState::with_legacy_root(&bench.host, bench.root.path().join("no-legacy"));
        state.set_game(Some(fresh.game.clone()));

        let staged = stage_source(&Extractor::new(), &bench.host, state.game(), &dll, &NoProgress, &CancelToken::new()).unwrap();
        let c = staged.classify();
        assert_eq!(c.kind.drop_route(), DropRoute::Install);
        assert_eq!(state.would_update(&c), None);
        let outcome = state.install(&c).unwrap().unwrap();
        drop(staged);

        assert!(outcome.is_ok());
        let installed = yp_thirdparty::paths::game_mods(&fresh.game).join("Celestial Timer").join("Celestial_Timer-12.dll");
        assert!(installed.is_file());
        assert_eq!(nams_flag(&fresh.game, LoadingFlag::GameMods.key()), Some(false));
        assert_eq!(state.presence(ThirdPartyRuntime::GameMods), RuntimePresence::Installed);
        assert_eq!(dir_names(&fresh.game.scoped_temp_root()), Vec::<String>::new());
    }
}

mod game_root_copies {
    use super::*;

    fn state_for(bench: &Bench, game: &GamePaths) -> ThirdPartyState {
        let mut state = ThirdPartyState::with_legacy_root(&bench.host, bench.root.path().join("no-legacy"));
        state.set_game(Some(game.clone()));
        state.refresh(false).unwrap();
        state
    }

    #[test]
    fn a_copy_in_the_game_folder_is_reported_while_loading_is_off() {
        let bench = bench();
        let fresh = fresh_game(
            "yp_tp_presence_",
            "disable_reshade_loading = true\ndisable_3dmigoto_loading = true\n",
        );
        let mut state = state_for(&bench, &fresh.game);
        for which in ThirdPartyRuntime::ALL {
            assert_eq!(state.presence(which), RuntimePresence::NotInstalled, "{which:?}");
            assert_eq!(state.existing_install_dir(which), None, "{which:?}");
        }
        assert_eq!(state.existing_shaders_dir(), None);

        let dxgi = fresh.game.root().join("dxgi.dll");
        write_marked_dll(&dxgi, "reshade.me crosire");

        assert!(!state.on_game_root_changed(&[fresh.game.root().join("notes.txt")]).unwrap());
        assert_eq!(state.presence(ThirdPartyRuntime::ReShade), RuntimePresence::NotInstalled);
        assert!(state.on_game_root_changed(std::slice::from_ref(&dxgi)).unwrap());
        assert_eq!(state.presence(ThirdPartyRuntime::ReShade), RuntimePresence::FoundInGameRoot);
        assert_eq!(state.presence(ThirdPartyRuntime::Migoto), RuntimePresence::NotInstalled);
        assert!(!state.on_game_root_changed(std::slice::from_ref(&dxgi)).unwrap(), "nothing new the second time");
        assert!(!state.statuses().reshade.installed, "loading is off, so the copy is not imported");

        assert!(state.import_from_game_root(ThirdPartyRuntime::ReShade).unwrap());
        assert_eq!(state.presence(ThirdPartyRuntime::ReShade), RuntimePresence::Installed);
        let install_dir = state.existing_install_dir(ThirdPartyRuntime::ReShade).unwrap();
        assert!(install_dir.join("ReShade64.dll").is_file());
        assert_eq!(state.existing_shaders_dir(), Some(install_dir.join("reshade-shaders").join("Shaders")));
    }

    #[test]
    fn a_copy_is_imported_right_away_while_loading_is_on() {
        let bench = bench();
        let fresh = fresh_game("yp_tp_presence_", "disable_3dmigoto_loading = false\n");
        let mut state = state_for(&bench, &fresh.game);
        let d3d11 = fresh.game.root().join("d3d11.dll");
        write_marked_dll(&d3d11, "HackerDevice");

        assert!(state.on_game_root_changed(std::slice::from_ref(&d3d11)).unwrap());

        assert_eq!(state.presence(ThirdPartyRuntime::Migoto), RuntimePresence::Installed);
        assert!(d3d11.is_file(), "the game-folder copy is left in place");
    }

    #[test]
    fn switching_the_game_forgets_the_old_findings() {
        let bench = bench();
        let fresh = fresh_game("yp_tp_presence_", "disable_reshade_loading = true\n");
        write_marked_dll(&fresh.game.root().join("dxgi.dll"), "crosire");
        let mut state = state_for(&bench, &fresh.game);
        assert_eq!(state.presence(ThirdPartyRuntime::ReShade), RuntimePresence::FoundInGameRoot);

        state.set_game(Some(GamePaths::new(bench.root.path().join("other-game"))));

        assert_eq!(state.presence(ThirdPartyRuntime::ReShade), RuntimePresence::NotInstalled);
    }
}
