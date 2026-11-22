use yp_core::{Os, SettingsStore};
use yp_host::launch::command::LaunchCommand;
use yp_host::launch::wrapper::{apply_wrapper, read, save, tokenize, wrap};

fn base() -> LaunchCommand {
    LaunchCommand {
        command: "/opt/proton/proton".to_owned(),
        args: ["run", "NAMS.exe", "run", "--nier-path", r"Z:\game"].map(str::to_owned).to_vec(),
        cwd: "/launcher".to_owned(),
        env: None,
        label: "Proton".to_owned(),
    }
}

#[test]
fn splits_a_plain_command_on_spaces() {
    assert_eq!(
        tokenize("gamescope -w 2560 -h 1440 -f --"),
        ["gamescope", "-w", "2560", "-h", "1440", "-f", "--"]
    );
}

#[test]
fn keeps_double_quoted_segments_together() {
    assert_eq!(tokenize("sh -c \"echo hi there\""), ["sh", "-c", "echo hi there"]);
}

#[test]
fn keeps_single_quoted_segments_together() {
    assert_eq!(tokenize("run 'a b' c"), ["run", "a b", "c"]);
}

#[test]
fn collapses_repeated_spaces() {
    assert_eq!(tokenize("a    b"), ["a", "b"]);
}

#[test]
fn empty_input_yields_no_tokens() {
    assert!(tokenize("   ").is_empty());
}

#[test]
fn quotes_nest_only_inside_the_other_kind_and_empty_quotes_are_tokens() {
    assert_eq!(tokenize("a \"it's\" 'say \"hi\"' \"\""), ["a", "it's", "say \"hi\"", ""]);
    assert_eq!(tokenize("x\"y z\"w"), ["xy zw"]);
}

#[test]
fn empty_wrapper_returns_the_base_command_unchanged() {
    assert_eq!(wrap(Os::Linux, base(), ""), base());
    assert_eq!(wrap(Os::Linux, base(), "   "), base());
}

#[test]
fn wrap_is_a_no_op_off_linux() {
    assert_eq!(wrap(Os::Windows, base(), "gamescope --"), base());
    assert_eq!(wrap(Os::MacOs, base(), "gamescope --"), base());
    assert_eq!(wrap(Os::Linux, base(), "gamescope --").command, "gamescope");
}

#[test]
fn apply_wrapper_prepends_the_wrapper_and_preserves_its_separator() {
    let wrapped = apply_wrapper(base(), "gamescope -w 2560 -h 1440 -f --");
    assert_eq!(wrapped.command, "gamescope");
    assert_eq!(
        wrapped.args,
        [
            "-w", "2560", "-h", "1440", "-f", "--", "/opt/proton/proton", "run", "NAMS.exe", "run", "--nier-path",
            r"Z:\game",
        ]
    );
    assert_eq!(wrapped.cwd, base().cwd);
    assert_eq!(wrapped.env, base().env);
    assert_eq!(wrapped.label, base().label);
}

#[test]
fn trailing_separator_is_not_duplicated() {
    let wrapped = apply_wrapper(base(), "gamescope --");
    assert_eq!(wrapped.command, "gamescope");
    assert_eq!(wrapped.args.iter().filter(|arg| *arg == "--").count(), 1);
    assert_eq!(wrapped.args[0], "--");
}

#[test]
fn single_word_wrapper_prepends_correctly() {
    let wrapped = apply_wrapper(base(), "mangohud");
    assert_eq!(wrapped.command, "mangohud");
    let mut expected = vec![base().command];
    expected.extend(base().args);
    assert_eq!(wrapped.args, expected);
}

#[test]
fn the_wrapper_setting_is_trimmed_on_save_and_read() {
    let settings = SettingsStore::in_memory();
    assert_eq!(read(&settings), "");
    save(&settings, "  gamescope --  ").unwrap();
    assert_eq!(read(&settings), "gamescope --");
}
