use std::path::PathBuf;

use yp_core::Os;

pub const SANDBOX_FLAG: &str = "--sandbox";
pub const SANDBOX_ASSIGN: &str = "--sandbox=";
pub const BACKGROUND_FLAG: &str = "--background";
pub const SIMULATE_LINUX_FLAG: &str = "--simulate-linux";
pub const SIMULATE_MACOS_FLAG: &str = "--simulate-macos";
pub const SIMULATE_WINDOWS_FLAG: &str = "--simulate-windows";

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Args {
    pub sandbox: Option<PathBuf>,
    pub background: bool,
    pub simulate: Option<Os>,
}

fn folder(text: &str) -> Option<PathBuf> {
    Some(text.trim())
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
}

impl Args {
    pub fn parse(args: impl IntoIterator<Item = String>, sandbox_var: Option<String>) -> Self {
        let mut parsed = Args {
            sandbox: sandbox_var.as_deref().and_then(folder),
            ..Args::default()
        };
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                SANDBOX_FLAG => {
                    if let Some(dir) = args.next().as_deref().and_then(folder) {
                        parsed.sandbox = Some(dir);
                    }
                }
                BACKGROUND_FLAG => parsed.background = true,
                SIMULATE_LINUX_FLAG => parsed.simulate = Some(Os::Linux),
                SIMULATE_MACOS_FLAG => parsed.simulate = Some(Os::MacOs),
                SIMULATE_WINDOWS_FLAG => parsed.simulate = Some(Os::Windows),
                other => {
                    if let Some(dir) = other.strip_prefix(SANDBOX_ASSIGN).and_then(folder) {
                        parsed.sandbox = Some(dir);
                    }
                }
            }
        }
        parsed
    }

    pub fn from_process() -> Self {
        Self::parse(
            std::env::args().skip(1),
            std::env::var(yp_shell::sandbox::SANDBOX_DIR_VAR).ok(),
        )
    }
}

pub fn simulated_os_name(os: Os) -> &'static str {
    match os {
        Os::Windows => "windows",
        Os::Linux => "linux",
        Os::MacOs => "macos",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str], var: Option<&str>) -> Args {
        Args::parse(
            args.iter().map(|arg| (*arg).to_owned()),
            var.map(str::to_owned),
        )
    }

    #[test]
    fn no_arguments_means_the_real_environment() {
        assert_eq!(parse(&[], None), Args::default());
        assert_eq!(parse(&[], Some("  ")), Args::default());
    }

    #[test]
    fn sandbox_comes_from_the_flag_or_the_variable() {
        assert_eq!(
            parse(&["--sandbox", "D:/box"], None).sandbox,
            Some(PathBuf::from("D:/box"))
        );
        assert_eq!(
            parse(&["--sandbox=D:/box"], None).sandbox,
            Some(PathBuf::from("D:/box"))
        );
        assert_eq!(
            parse(&[], Some("E:/env")).sandbox,
            Some(PathBuf::from("E:/env"))
        );
        assert_eq!(
            parse(&["--sandbox", "D:/flag"], Some("E:/env")).sandbox,
            Some(PathBuf::from("D:/flag"))
        );
        assert_eq!(parse(&["--sandbox"], None).sandbox, None);
    }

    #[test]
    fn simulate_and_background_flags_are_read() {
        let args = parse(&["--background", "--simulate-linux"], None);
        assert!(args.background);
        assert_eq!(args.simulate, Some(Os::Linux));
        assert_eq!(
            parse(&["--simulate-macos"], None).simulate,
            Some(Os::MacOs)
        );
        assert_eq!(
            parse(&["--simulate-windows"], None).simulate,
            Some(Os::Windows)
        );
        assert_eq!(parse(&["--unknown"], None), Args::default());
    }

    #[test]
    fn simulated_names_round_trip_through_the_host_parser() {
        for os in [Os::Windows, Os::Linux, Os::MacOs] {
            assert_eq!(Os::from_name(simulated_os_name(os)), Some(os));
        }
    }
}
