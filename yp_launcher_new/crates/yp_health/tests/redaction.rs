mod support;

use support::{FakeGame, Rig, bare_report, write};
use yp_core::Os;
use yp_health::diagnostics::model::{
    GAME_DIR_KEY, GAME_DIR_NOT_SELECTED, LAUNCHER_DIR_KEY, ReportHost,
};
use yp_health::diagnostics::redact::{
    GAME_DIR_TOKEN, LAUNCHER_DIR_TOKEN, USER_HOME_TOKEN, USER_TOKEN,
};
use yp_health::diagnostics::{Redactor, format_full_report, format_summary, redact};

fn report_for(user_home: &str, user_name: &str) -> yp_health::DiagnosticsReport {
    let mut report = bare_report("");
    report.host = ReportHost {
        os: Os::native(),
        user_home: user_home.to_owned(),
        user_name: user_name.to_owned(),
    };
    report
}

#[test]
fn home_folder_is_replaced_in_every_separator_style() {
    let report = report_for("C:\\Users\\Alice", "Alice");
    let out = redact(
        "C:\\Users\\Alice\\Documents and C:/Users/Alice/Saved Games and c:\\users\\alice\\x",
        &report,
    );
    assert_eq!(
        out,
        "<userHome>\\Documents and <userHome>/Saved Games and <userHome>\\x"
    );
}

#[test]
fn posix_home_is_replaced_behind_the_wine_host_drive_too() {
    let report = report_for("/home/alice", "alice");
    let out = redact(
        "/home/alice/.steam and Z:\\home\\alice\\.wine and z:\\home\\alice\\x and Z:/home/alice/y",
        &report,
    );
    assert_eq!(
        out,
        "<userHome>/.steam and <userHome>\\.wine and <userHome>\\x and <userHome>/y"
    );
}

#[test]
fn paths_match_without_regard_to_case() {
    let mut report = bare_report("D:\\Games\\NieRAutomata");
    report.launcher_info.insert(
        LAUNCHER_DIR_KEY.to_owned(),
        "D:\\Tools\\YP Launcher\\assets\\bins".to_owned(),
    );
    let out = redact(
        "d:\\games\\nierautomata\\data, D:/GAMES/NIERAUTOMATA/nams, d:\\tools\\yp launcher\\ASSETS\\bins\\NAMS.exe",
        &report,
    );
    assert_eq!(
        out,
        "<gameDir>\\data, <gameDir>/nams, <launcherDir>\\NAMS.exe"
    );
}

#[test]
fn game_folder_inside_the_home_folder_keeps_the_more_specific_token() {
    let mut report = bare_report("/home/alice/Games/NieRAutomata");
    report.host.user_home = "/home/alice".to_owned();
    report.host.user_name = "alice".to_owned();
    let out = redact(
        "/home/alice/Games/NieRAutomata/data /home/alice/Games/Other /home/alice",
        &report,
    );
    assert_eq!(out, "<gameDir>/data <userHome>/Games/Other <userHome>");
}

#[test]
fn username_needs_a_users_or_home_parent_segment() {
    let report = report_for("", "sam");
    let out = redact(
        "C:\\Users\\sam\\x D:\\Home\\SAM\\y /home/sam/z /opt/sam/bin /mnt/users2/sam/a sam/b \\\\server\\sam\\c",
        &report,
    );
    assert_eq!(
        out,
        "C:\\Users\\<user>\\x D:\\Home\\<user>\\y /home/<user>/z /opt/sam/bin /mnt/users2/sam/a sam/b \\\\server\\sam\\c"
    );
}

#[test]
fn username_must_end_at_a_word_boundary() {
    let report = report_for("", "al");
    let out = redact(
        "/home/alice /home/al.bak /home/al-old /home/al_2 /home/al9 /home/al /home/al/x /home/al: /home/al\"",
        &report,
    );
    assert_eq!(
        out,
        "/home/alice /home/al.bak /home/al-old /home/al_2 /home/al9 /home/<user> /home/<user>/x /home/<user>: /home/<user>\""
    );
}

#[test]
fn username_at_the_very_end_of_the_text_is_redacted() {
    let report = report_for("", "sam");
    assert_eq!(redact("cwd=/home/sam", &report), "cwd=/home/<user>");
    assert_eq!(redact("C:\\Users\\Sam", &report), "C:\\Users\\<user>");
}

#[test]
fn username_is_matched_in_any_letter_case_and_keeps_surrounding_text() {
    let report = report_for("", "Björn");
    let out = redact(
        "C:\\Users\\BJÖRN\\Saved Games and c:\\users\\björn\\x and Björn said",
        &report,
    );
    assert_eq!(
        out,
        "C:\\Users\\<user>\\Saved Games and c:\\users\\<user>\\x and Björn said"
    );
}

#[test]
fn single_character_usernames_and_tiny_paths_are_left_alone() {
    let report = report_for("C:", "a");
    let input = "C:\\Users\\a\\x and /home/a/y";
    assert_eq!(redact(input, &report), input);
    assert_eq!(
        Redactor::new()
            .with_path("ab", GAME_DIR_TOKEN)
            .redact("ab/cd"),
        "ab/cd"
    );
    assert_eq!(
        Redactor::new()
            .with_path("abc", GAME_DIR_TOKEN)
            .redact("abc/cd"),
        "<gameDir>/cd"
    );
}

#[test]
fn unselected_game_folder_placeholder_is_not_a_path() {
    let mut report = bare_report("");
    report
        .launcher_info
        .insert(GAME_DIR_KEY.to_owned(), GAME_DIR_NOT_SELECTED.to_owned());
    let input = "Game dir: <not selected>";
    assert_eq!(redact(input, &report), input);
    assert_eq!(redact("", &report), "");
}

#[test]
fn typed_game_dir_is_redacted_even_when_the_launcher_table_lacks_it() {
    let mut report = bare_report("E:\\SteamLibrary\\steamapps\\common\\NieRAutomata");
    report.launcher_info.clear();
    assert_eq!(
        redact(
            "E:\\SteamLibrary\\steamapps\\common\\NieRAutomata\\nams\\nams.toml",
            &report
        ),
        "<gameDir>\\nams\\nams.toml"
    );
}

#[test]
fn tokens_are_the_ones_support_expects() {
    assert_eq!(GAME_DIR_TOKEN, "<gameDir>");
    assert_eq!(LAUNCHER_DIR_TOKEN, "<launcherDir>");
    assert_eq!(USER_HOME_TOKEN, "<userHome>");
    assert_eq!(USER_TOKEN, "<user>");
}

#[test]
fn collected_report_never_leaks_the_user_or_any_private_folder() {
    let rig = Rig::native();
    let game = FakeGame::create();
    let home = rig.host.home().unwrap();
    let home_text = home.to_string_lossy().into_owned();
    rig.write_log(
        "nams.log",
        &format!(
            "2026-11-06T10:00:00.000000Z  WARN modloader: could not open {}\\Documents\\save.dat\n2026-11-06T10:00:01.000000Z ERROR modloader: bad file C:\\Users\\tester\\Desktop\\mod.zip\n",
            home_text
        ),
    );
    write(
        &rig.paths.logs_dir.join("crashdump_1.txt"),
        format!(
            "module {}\\nams\\mods\\x.dll\nuser dir /home/tester/.cache\n",
            game.text()
        ),
    );
    let preview = format!(
        "cd /d \"{}\"\n\"{}\" run --nier-path \"{}\"",
        rig.paths.runtime_dir.display(),
        rig.paths.nams_exe().display(),
        game.text()
    );

    let report = rig.collect_with_preview(Some(game.path()), Some(preview));
    assert_eq!(report.recent_log_issues.len(), 2);
    let launcher_text = rig.paths.runtime_dir.to_string_lossy().into_owned();
    for text in [format_full_report(&report), format_summary(&report)] {
        assert!(!text.contains(&game.text()));
        assert!(!text.contains(&home_text));
        assert!(!text.contains(&launcher_text));
        assert!(!text.to_lowercase().contains("users\\tester"));
        assert!(!text.contains("/home/tester"));
    }
    let full = format_full_report(&report);
    assert!(full.contains("could not open <userHome>\\Documents\\save.dat"));
    assert!(full.contains("bad file C:\\Users\\<user>\\Desktop\\mod.zip"));
    assert!(full.contains("module <gameDir>\\nams\\mods\\x.dll"));
    assert!(full.contains("user dir /home/<user>/.cache"));
    assert!(full.contains("cd /d \"<launcherDir>\""));
    assert!(
        full.contains("\"<launcherDir>\\NAMS.exe\" run --nier-path \"<gameDir>\"")
            || full.contains("\"<launcherDir>/NAMS.exe\" run --nier-path \"<gameDir>\"")
    );
}
