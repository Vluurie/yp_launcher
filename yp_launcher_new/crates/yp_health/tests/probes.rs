mod support;

use std::path::Path;

use support::write;
use yp_core::testing::{FakeResponse, FakeRunner};
use yp_core::{HostEnv, Os, ProcessOutput, ProcessSpec};
use yp_health::probes::{
    HEAD_MAX_LINES, SystemFiles, TAIL_MAX_BYTES, TAIL_MAX_LINES, facts, files_in, first_line, head,
    is_steam_running, linux_context, mount_of, newest_first, os_version_label, tail,
};

fn host(os: Os, vars: &[(&str, &str)]) -> HostEnv {
    HostEnv::for_test(os, vars, Path::new("/yp-test-root"))
}

fn program_is(spec: &ProcessSpec, name: &str) -> bool {
    spec.program.to_string_lossy() == name
}

#[test]
fn tail_keeps_the_last_non_empty_lines_without_colour_codes() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("nams.log");
    write(&file, "one  \r\n\r\n\u{1b}[31mtwo\u{1b}[0m\nthree\n\n");

    let excerpt = tail(&file, 2, TAIL_MAX_BYTES).unwrap();
    assert_eq!(excerpt.name, "nams.log");
    assert_eq!(excerpt.size, 29);
    assert!(excerpt.modified.is_some());
    assert_eq!(excerpt.lines, ["two", "three"]);

    let all = tail(&file, TAIL_MAX_LINES, TAIL_MAX_BYTES).unwrap();
    assert_eq!(all.lines, ["one", "two", "three"]);
    assert_eq!(
        tail(&dir.path().join("missing.log"), 5, TAIL_MAX_BYTES),
        None
    );
    assert_eq!(tail(dir.path(), 5, TAIL_MAX_BYTES), None);
}

#[test]
fn tail_of_a_large_file_drops_the_cut_first_line() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("big.log");
    let mut content = String::new();
    for index in 0..40 {
        content.push_str(&format!("line-{index:03}\n"));
    }
    write(&file, &content);

    let excerpt = tail(&file, 100, 20).unwrap();
    assert_eq!(excerpt.size, 360);
    assert_eq!(excerpt.lines, ["line-038", "line-039"]);
}

#[test]
fn head_keeps_blank_lines_and_stops_at_the_line_limit() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("crashdump_1.txt");
    write(&file, "EXCEPTION\n\nat a\nat b\n");

    assert_eq!(
        head(&file, HEAD_MAX_LINES).unwrap().lines,
        ["EXCEPTION", "", "at a", "at b"]
    );
    assert_eq!(head(&file, 2).unwrap().lines, ["EXCEPTION", ""]);
    assert_eq!(head(&dir.path().join("none.txt"), 2), None);
}

#[test]
fn facts_report_size_and_a_short_sha1_on_request() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("a.txt");
    write(&file, "abc");

    let plain = facts(&file, false).unwrap();
    assert_eq!(plain.name, "a.txt");
    assert_eq!(plain.size, 3);
    assert_eq!(plain.short_sha1, None);
    assert!(plain.modified.is_some());
    let hashed = facts(&file, true).unwrap();
    assert_eq!(hashed.short_sha1.as_deref(), Some("a9993e364706"));
    assert_eq!(facts(&dir.path().join("b.txt"), true), None);
}

#[test]
fn folder_listing_filters_on_lowercase_names_and_sorts_without_case() {
    let dir = tempfile::tempdir().unwrap();
    for name in ["zeta.DLL", "Alpha.dll", "beta.dll", "notes.txt"] {
        write(&dir.path().join(name), name);
    }
    std::fs::create_dir_all(dir.path().join("folder.dll")).unwrap();

    let listed = files_in(dir.path(), &|name| name.ends_with(".dll"), false);
    let names: Vec<&str> = listed.iter().map(|file| file.name.as_str()).collect();
    assert_eq!(names, ["Alpha.dll", "beta.dll", "zeta.DLL"]);
    assert!(files_in(&dir.path().join("missing"), &|_| true, false).is_empty());
}

#[test]
fn newest_file_comes_first() {
    let dir = tempfile::tempdir().unwrap();
    let old = dir.path().join("crashdump_old.txt");
    let new = dir.path().join("crashdump_new.txt");
    write(&old, "old");
    write(&new, "new");
    let earlier = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
    std::fs::File::options()
        .write(true)
        .open(&old)
        .unwrap()
        .set_modified(earlier)
        .unwrap();
    write(&dir.path().join("other.txt"), "x");

    let ordered = newest_first(dir.path(), &|name| name.starts_with("crashdump_"));
    assert_eq!(ordered, [new, old]);
}

#[test]
fn first_line_skips_blank_lines_and_trims() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("version");
    write(&file, "\n   \n  1700000000 proton-9.0-4  \nsecond\n");
    assert_eq!(
        first_line(&file).as_deref(),
        Some("1700000000 proton-9.0-4")
    );
    write(&file, "\n \n");
    assert_eq!(first_line(&file), None);
    assert_eq!(first_line(&dir.path().join("missing")), None);
}

#[test]
fn mount_lookup_asks_findmnt_on_linux_only() {
    let json = "{\"filesystems\": [{\"target\": \"/mnt/win\", \"fstype\": \"ntfs3\", \"options\": \"rw\"}]}";
    let runner = FakeRunner::new().respond_program("findmnt", ProcessOutput::success(json));
    let linux = host(Os::Linux, &[]);

    let mount = mount_of(&linux, &runner, Path::new("/mnt/win/NieRAutomata")).unwrap();
    assert_eq!(mount.target, "/mnt/win");
    assert_eq!(mount.fs_type, "ntfs3");
    assert!(mount.risky());
    let call = &runner.calls()[0];
    assert_eq!(
        call.args_lossy(),
        [
            "-J",
            "-T",
            "/mnt/win/NieRAutomata",
            "-o",
            "TARGET,FSTYPE,OPTIONS"
        ]
    );

    assert_eq!(mount_of(&linux, &runner, Path::new("")), None);
    assert_eq!(
        mount_of(&host(Os::Windows, &[]), &runner, Path::new("C:\\Games")),
        None
    );
    assert_eq!(
        mount_of(&host(Os::MacOs, &[]), &runner, Path::new("/Games")),
        None
    );
    assert_eq!(runner.calls().len(), 1);

    let failing =
        FakeRunner::new().respond_program("findmnt", ProcessOutput::failure(1, "not found"));
    assert_eq!(mount_of(&linux, &failing, Path::new("/x")), None);
    let absent =
        FakeRunner::new().respond(|spec| program_is(spec, "findmnt"), FakeResponse::NotFound);
    assert_eq!(mount_of(&linux, &absent, Path::new("/x")), None);
}

#[test]
fn steam_lookup_uses_pgrep_outside_windows() {
    let running = FakeRunner::new().respond_program("pgrep", ProcessOutput::success("4242\n"));
    assert_eq!(
        is_steam_running(&host(Os::Linux, &[]), &running),
        Some(true)
    );
    assert_eq!(running.calls()[0].args_lossy(), ["-x", "steam"]);
    assert_eq!(
        is_steam_running(&host(Os::MacOs, &[]), &running),
        Some(true)
    );
    assert_eq!(running.calls()[1].args_lossy(), ["-f", "steam.exe"]);

    let stopped = FakeRunner::new().respond_program("pgrep", ProcessOutput::failure(1, ""));
    assert_eq!(
        is_steam_running(&host(Os::Linux, &[]), &stopped),
        Some(false)
    );
    let absent =
        FakeRunner::new().respond(|spec| program_is(spec, "pgrep"), FakeResponse::NotFound);
    assert_eq!(is_steam_running(&host(Os::Linux, &[]), &absent), None);
    let hanging = FakeRunner::new().respond(
        |spec| program_is(spec, "pgrep"),
        FakeResponse::Running {
            output: String::new(),
        },
    );
    assert_eq!(is_steam_running(&host(Os::Linux, &[]), &hanging), None);
}

#[test]
fn linux_context_reads_the_distro_and_session() {
    let dir = tempfile::tempdir().unwrap();
    let files = SystemFiles::under(dir.path());
    write(
        &files.os_release,
        "NAME=\"Arch Linux\"\nPRETTY_NAME=\"Arch Linux (rolling)\"\nID=arch\n",
    );
    let deck = host(
        Os::Linux,
        &[
            ("XDG_SESSION_TYPE", "wayland"),
            ("XDG_CURRENT_DESKTOP", "KDE"),
            ("APPIMAGE", "/tmp/yp.AppImage"),
            ("SteamDeck", "1"),
        ],
    );

    let context = linux_context(&deck, &files);
    let pairs: Vec<(&str, &str)> = context
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();
    assert_eq!(
        pairs,
        [
            ("Distro", "Arch Linux (rolling)"),
            ("Session", "wayland"),
            ("Desktop", "KDE"),
            ("AppImage", "yes"),
            ("Flatpak", "no"),
            ("Steam Deck", "yes"),
        ]
    );

    write(&files.os_release, "NAME=Bazzite\n");
    write(&files.flatpak_info, "[Application]\n");
    let bare = linux_context(&host(Os::Linux, &[]), &files);
    assert_eq!(bare["Distro"], "Bazzite");
    assert_eq!(bare["Session"], "?");
    assert_eq!(bare["Desktop"], "?");
    assert_eq!(bare["AppImage"], "no");
    assert_eq!(bare["Flatpak"], "yes");
    assert_eq!(bare["Steam Deck"], "no");

    let unknown = linux_context(
        &host(Os::Linux, &[("FLATPAK_ID", "x.y.z")]),
        &SystemFiles::under(&dir.path().join("none")),
    );
    assert_eq!(unknown["Distro"], "?");
    assert_eq!(unknown["Flatpak"], "yes");
    assert!(linux_context(&host(Os::Windows, &[]), &files).is_empty());
    assert!(linux_context(&host(Os::MacOs, &[]), &files).is_empty());
}

#[test]
fn os_version_comes_from_the_host_facts_of_each_system() {
    let dir = tempfile::tempdir().unwrap();
    let files = SystemFiles::under(dir.path());
    let runner = FakeRunner::new();

    let windows = host(Os::Windows, &[]).with_windows_build(Some(22_631));
    assert_eq!(
        os_version_label(&windows, &runner, &files),
        "Windows 11 (Build 22631)"
    );

    assert_eq!(
        os_version_label(&host(Os::Linux, &[]), &runner, &files),
        "Linux"
    );
    write(&files.kernel_dir.join("ostype"), "Linux\n");
    write(&files.kernel_dir.join("osrelease"), "6.11.2-arch1-1\n");
    write(
        &files.kernel_dir.join("version"),
        "#1 SMP PREEMPT_DYNAMIC Fri, 04 Oct 2026\n",
    );
    assert_eq!(
        os_version_label(&host(Os::Linux, &[]), &runner, &files),
        "Linux 6.11.2-arch1-1 #1 SMP PREEMPT_DYNAMIC Fri, 04 Oct 2026"
    );

    let mac = FakeRunner::new()
        .respond(
            |spec| program_is(spec, "sw_vers") && spec.args_lossy() == ["-productVersion"],
            FakeResponse::Output(ProcessOutput::success("15.1\n")),
        )
        .respond(
            |spec| program_is(spec, "sw_vers") && spec.args_lossy() == ["-buildVersion"],
            FakeResponse::Output(ProcessOutput::success("24B83\n")),
        );
    assert_eq!(
        os_version_label(&host(Os::MacOs, &[]), &mac, &files),
        "Version 15.1 (Build 24B83)"
    );
    assert_eq!(
        os_version_label(&host(Os::MacOs, &[]), &runner, &files),
        "macOS"
    );
}
