use yp_core::host::SIMULATE_OS_VAR;
use yp_core::{HostEnv, OpenReport};
use yp_i18n::Msg;
use yp_shell::ShellOptions;
use yp_state::{AppEnv, Notice, NoticeKind, NoticeSeverity};

use crate::args::{Args, simulated_os_name};

pub const SETTINGS_RECOVERED_NOTICE: &str = "settings_recovered";
pub const DATA_FOLDER_NOTICE: &str = "data_folder_unavailable";

pub struct Boot {
    pub env: AppEnv,
    pub options: ShellOptions,
    pub notices: Vec<Notice>,
}

fn simulated(host: HostEnv, args: &Args) -> HostEnv {
    match args.simulate {
        Some(os) => host.with_var(SIMULATE_OS_VAR, simulated_os_name(os)),
        None => host,
    }
}

fn import_flutter_settings(env: &AppEnv) {
    let Some(legacy) = env.paths.legacy_prefs_file(&env.host) else {
        return;
    };
    match env.settings.import_flutter_prefs(&legacy) {
        Ok(0) => {}
        Ok(count) => log::info!("imported {count} settings from the previous launcher"),
        Err(err) => log::warn!("could not import the previous launcher settings: {err}"),
    }
}

fn recovery_notice(env: &AppEnv) -> Option<Notice> {
    match env.settings_report.as_ref()? {
        OpenReport::RecoveredCorrupt { backup } => Some(Notice::new(
            SETTINGS_RECOVERED_NOTICE,
            Msg::ShellSettingsRecovered {
                path: backup.to_string_lossy().into_owned(),
            },
            NoticeKind::General,
            NoticeSeverity::Warning,
        )),
        OpenReport::Loaded | OpenReport::Created => None,
    }
}

fn failure_notice(error: &yp_core::Error) -> Notice {
    Notice::new(
        DATA_FOLDER_NOTICE,
        Msg::ShellDataFolderUnavailable {
            reason: error.to_string(),
        },
        NoticeKind::General,
        NoticeSeverity::Error,
    )
}

fn shell_options(args: &Args) -> ShellOptions {
    let base = match &args.sandbox {
        Some(dir) => ShellOptions::sandboxed(dir.clone()),
        None => ShellOptions::default(),
    };
    ShellOptions {
        focus_window: !args.background,
        docs: cfg!(feature = "docs"),
        tools: cfg!(feature = "tools"),
        ..base
    }
}

fn detect(args: &Args) -> yp_core::Result<AppEnv> {
    match &args.sandbox {
        Some(dir) => {
            let env = yp_shell::sandbox::sandbox_env(dir)?;
            let host = simulated(env.host.clone(), args);
            Ok(AppEnv { host, ..env })
        }
        None => {
            let env = AppEnv::detect_for(simulated(HostEnv::current(), args))?;
            import_flutter_settings(&env);
            Ok(env)
        }
    }
}

pub fn boot(args: &Args) -> Boot {
    let options = shell_options(args);
    match detect(args) {
        Ok(env) => {
            let notices = recovery_notice(&env).into_iter().collect();
            Boot {
                env,
                options,
                notices,
            }
        }
        Err(error) => {
            log::error!("could not open the launcher data folder: {error}");
            Boot {
                env: AppEnv::fallback(),
                options,
                notices: vec![failure_notice(&error)],
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yp_core::Os;

    #[test]
    fn sandbox_boot_stays_in_its_folder_and_starts_without_a_game() {
        let root = tempfile::tempdir().unwrap();
        let args = Args {
            sandbox: Some(root.path().to_path_buf()),
            background: true,
            simulate: None,
        };
        let boot = boot(&args);
        assert!(boot.notices.is_empty());
        assert!(boot.env.paths.settings_file.starts_with(root.path()));
        assert_eq!(boot.options.sandbox.as_deref(), Some(root.path()));
        assert!(!boot.options.auto_search);
        assert!(!boot.options.focus_window);
        assert!(yp_host::session::AppSession::saved_directory(&boot.env.settings).is_none());
    }

    #[test]
    fn corrupt_sandbox_settings_are_reported() {
        let root = tempfile::tempdir().unwrap();
        let data = root.path().join(yp_shell::sandbox::DATA_DIR_NAME);
        std::fs::create_dir_all(&data).unwrap();
        std::fs::write(
            data.join(yp_core::paths::SETTINGS_FILE_NAME),
            b"{ not json",
        )
        .unwrap();
        let args = Args {
            sandbox: Some(root.path().to_path_buf()),
            ..Args::default()
        };
        let boot = boot(&args);
        assert_eq!(boot.notices.len(), 1);
        assert_eq!(boot.notices[0].id.as_ref(), SETTINGS_RECOVERED_NOTICE);
        assert_eq!(boot.notices[0].severity, NoticeSeverity::Warning);
        assert!(boot.options.focus_window);
    }

    #[test]
    fn simulated_system_only_changes_what_the_ui_shows() {
        let root = tempfile::tempdir().unwrap();
        let args = Args {
            sandbox: Some(root.path().to_path_buf()),
            background: true,
            simulate: Some(Os::Linux),
        };
        let boot = boot(&args);
        assert_eq!(boot.env.host.ui_os(), Os::Linux);
        assert_eq!(boot.env.host.os(), Os::native());
    }
}
