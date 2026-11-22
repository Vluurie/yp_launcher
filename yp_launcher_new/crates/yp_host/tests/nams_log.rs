use std::io::Write;

use yp_core::AppPaths;
use yp_host::nams_log::panel::{
    append_capped, is_modloader_module, LogPanel, LogTab, DEFAULT_HIDDEN_LEVELS, LOG_LEVELS, MAX_ENTRIES_PER_TAB,
};
use yp_host::nams_log::{clear, format_timestamp, parse_line, read, LogEntry, LogLevel, LogTail, TailUpdate, TAIL_BYTES};

fn entry(message: &str) -> LogEntry {
    LogEntry {
        timestamp: "00:00:00".to_owned(),
        level: LogLevel::Info,
        module: "test".to_owned(),
        message: message.to_owned(),
    }
}

fn entries(from: usize, count: usize) -> Vec<LogEntry> {
    (0..count).map(|index| entry(&(from + index).to_string())).collect()
}

#[test]
fn keeps_everything_while_below_the_cap() {
    let mut list = entries(0, 10);
    append_capped(&mut list, entries(10, 5));
    assert_eq!(list.len(), 15);
    assert_eq!(list.first().unwrap().message, "0");
    assert_eq!(list.last().unwrap().message, "14");
}

#[test]
fn drops_the_oldest_entries_once_the_cap_is_passed() {
    let mut list = entries(0, MAX_ENTRIES_PER_TAB);
    append_capped(&mut list, entries(0, 3));
    assert_eq!(list.len(), MAX_ENTRIES_PER_TAB);
    assert_eq!(list.first().unwrap().message, "3");
    assert_eq!(list.last().unwrap().message, "2");
}

#[test]
fn keeps_only_the_newest_when_a_single_batch_exceeds_the_cap() {
    let mut list = entries(0, 10);
    append_capped(&mut list, entries(0, MAX_ENTRIES_PER_TAB + 500));
    assert_eq!(list.len(), MAX_ENTRIES_PER_TAB);
    assert_eq!(list.last().unwrap().message, (MAX_ENTRIES_PER_TAB + 499).to_string());
    assert_eq!(list.first().unwrap().message, "500");
}

#[test]
fn an_empty_batch_leaves_the_list_untouched() {
    let mut list = entries(0, 5);
    let before = list.as_ptr();
    append_capped(&mut list, Vec::new());
    assert_eq!(list.as_ptr(), before);
    assert_eq!(list.len(), 5);
}

#[test]
fn never_exceeds_the_cap_across_repeated_appends() {
    let mut list = Vec::new();
    for round in 0..20 {
        append_capped(&mut list, entries(round * 400, 400));
        assert!(list.len() <= MAX_ENTRIES_PER_TAB);
    }
    assert_eq!(list.last().unwrap().message, (20 * 400 - 1).to_string());
}

#[test]
fn parses_tracing_lines_with_ansi_colours() {
    let raw = "\u{1b}[2m2026-09-10T21:30:45.867300Z\u{1b}[0m \u{1b}[32m INFO\u{1b}[0m \u{1b}[2mnier_rs::content::mod_load_order\u{1b}[0m\u{1b}[2m:\u{1b}[0m [NAMS] mod_load_order.toml loaded (0 entries): []\r";
    let parsed = parse_line(raw).unwrap();
    assert_eq!(parsed.timestamp, "21:30:45.867");
    assert_eq!(parsed.level, LogLevel::Info);
    assert_eq!(parsed.module, "nier_rs::content::mod_load_order");
    assert_eq!(parsed.message, "[NAMS] mod_load_order.toml loaded (0 entries): []");
}

#[test]
fn parses_plain_lines_and_rejects_malformed_ones() {
    let plain = parse_line("2026-02-14T12:46:29.398602Z  WARN nams: Startup: done").unwrap();
    assert_eq!((plain.level, plain.module.as_str(), plain.message.as_str()), (LogLevel::Warn, "nams", "Startup: done"));
    let glued = parse_line("2026-02-14T12:46:29.3Z ERROR mod:message").unwrap();
    assert_eq!((glued.module.as_str(), glued.message.as_str(), glued.timestamp.as_str()), ("mod", "message", "12:46:29.3"));
    assert_eq!(parse_line("2026-02-14T12:46:29.398602Z  NOTICE nams: x"), None);
    assert_eq!(parse_line("2026-02-14T12:46:29Z  INFO nams: x"), None);
    assert_eq!(parse_line("2026-02-14 12:46:29.1Z  INFO nams: x"), None);
    let bare = parse_line("2026-02-14T12:46:29.1Z INFO LodMod shadow resolution: live recreate").unwrap();
    assert_eq!((bare.module.as_str(), bare.message.as_str()), ("", "LodMod shadow resolution: live recreate"));
    assert_eq!(parse_line("2026-02-14T12:46:29.1ZINFO nams: x"), None);
    assert_eq!(parse_line("   at some stack frame"), None);
    let empty = parse_line("2026-02-14T12:46:29.1Z TRACE nams:").unwrap();
    assert_eq!((empty.level, empty.message.as_str()), (LogLevel::Trace, ""));
}

#[test]
fn timestamps_are_trimmed_to_milliseconds() {
    assert_eq!(format_timestamp("2026-02-14T12:46:29.398602Z"), "12:46:29.398");
    assert_eq!(format_timestamp("2026-02-14T12:46:29.39Z"), "12:46:29.39");
    assert_eq!(format_timestamp("no-t-here"), "no-t-here");
}

fn log_line(level: &str, module: &str, message: &str) -> String {
    format!("2026-02-14T12:46:29.398602Z {level:>5} {module}: {message}\n")
}

#[test]
fn reads_and_clears_the_nams_log() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::for_test(dir.path());
    assert!(read(&paths).is_empty());
    assert!(!clear(&paths));
    std::fs::create_dir_all(&paths.logs_dir).unwrap();
    let text = format!("{}garbage\n\n{}", log_line("INFO", "nams", "a"), log_line("ERROR", "yorha_protocol", "b"));
    std::fs::write(paths.nams_log(), text).unwrap();
    let entries = read(&paths);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[1].level, LogLevel::Error);
    assert!(clear(&paths));
    assert_eq!(std::fs::read(paths.nams_log()).unwrap(), Vec::<u8>::new());
}

#[test]
fn tail_streams_appends_holds_partial_lines_and_resets() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nams.log");
    let mut tail = LogTail::new(path.clone());
    assert_eq!(tail.poll(), TailUpdate::Unchanged);
    std::fs::write(&path, format!("{}2026-02-14T12:46:29.1Z  INFO nams: part", log_line("INFO", "nams", "first"))).unwrap();
    let TailUpdate::Appended(first) = tail.poll() else {
        panic!("expected entries");
    };
    assert_eq!(first.len(), 1);
    assert_eq!(tail.poll(), TailUpdate::Unchanged);
    let mut file = std::fs::OpenOptions::new().append(true).open(&path).unwrap();
    file.write_all("ial äö\n".as_bytes()).unwrap();
    drop(file);
    let TailUpdate::Appended(second) = tail.poll() else {
        panic!("expected the completed line");
    };
    assert_eq!(second[0].message, "partial äö");
    std::fs::write(&path, log_line("WARN", "nams", "after truncation")).unwrap();
    let TailUpdate::Appended(third) = tail.poll() else {
        panic!("expected entries after truncation");
    };
    assert_eq!(third[0].message, "after truncation");
    std::fs::remove_file(&path).unwrap();
    assert_eq!(tail.poll(), TailUpdate::Reset);
    assert_eq!(tail.poll(), TailUpdate::Unchanged);
}

#[test]
fn tail_skips_old_backlog_on_the_first_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nams.log");
    let line = log_line("INFO", "nams", "backlog");
    let repeats = (TAIL_BYTES as usize / line.len()) + 100;
    let mut text = line.repeat(repeats);
    text.push_str(&log_line("ERROR", "nams", "newest"));
    std::fs::write(&path, text).unwrap();
    let TailUpdate::Appended(entries) = LogTail::new(path).poll() else {
        panic!("expected entries");
    };
    assert!(entries.len() < repeats);
    assert_eq!(entries.last().unwrap().message, "newest");
    assert!(entries.iter().all(|entry| entry.timestamp == "12:46:29.398"));
}

#[test]
fn panel_routes_modules_filters_and_counts() {
    assert!(is_modloader_module("Nier_rs::engine"));
    assert!(is_modloader_module("wgpu_core"));
    assert!(!is_modloader_module("yorha_protocol"));
    let mut panel = LogPanel::default();
    assert_eq!(panel.active_tab, LogTab::Modloader);
    assert_eq!(panel.hidden_levels.iter().copied().collect::<Vec<_>>(), DEFAULT_HIDDEN_LEVELS);
    assert_eq!(LOG_LEVELS[0], LogLevel::Error);
    let make = |level: LogLevel, module: &str, message: &str| LogEntry {
        timestamp: "t".into(),
        level,
        module: module.into(),
        message: message.into(),
    };
    panel.apply(TailUpdate::Appended(vec![
        make(LogLevel::Info, "nams", "startup"),
        make(LogLevel::Error, "nams", "Broken Thing"),
        make(LogLevel::Warn, "yorha_protocol", "overlay"),
    ]));
    assert_eq!(panel.modloader_entries.len(), 2);
    assert_eq!(panel.yorha_entries.len(), 1);
    assert_eq!(panel.filtered_entries().len(), 1);
    assert_eq!(panel.level_counts()[&LogLevel::Info], 1);
    panel.toggle_level(LogLevel::Info);
    assert!(panel.is_level_shown(LogLevel::Info));
    assert_eq!(panel.filtered_entries().len(), 2);
    panel.set_search_query("  broken ");
    assert_eq!(panel.filtered_entries()[0].message, "Broken Thing");
    panel.set_search_query("error");
    assert_eq!(panel.filtered_entries().len(), 1);
    panel.set_active_tab(LogTab::Yorha);
    assert_eq!(LogTab::Yorha.id(), "yorha");
    panel.set_search_query("");
    assert_eq!(panel.filtered_entries().len(), 1);
    panel.apply(TailUpdate::Reset);
    assert!(panel.active_entries().is_empty() && panel.modloader_entries.is_empty());
}
