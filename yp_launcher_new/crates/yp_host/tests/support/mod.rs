#![allow(dead_code)]

use std::path::Path;

use yp_core::wine_paths::{posix, to_posix};
use yp_core::{HostEnv, Os};
use yp_host::wine::WineContext;

pub fn write(path: &str, contents: &str) -> String {
    let parent = posix::dirname(path);
    std::fs::create_dir_all(&parent).unwrap();
    std::fs::write(path, contents).unwrap();
    path.to_owned()
}

pub fn mkdirs(path: &str) -> String {
    std::fs::create_dir_all(path).unwrap();
    path.to_owned()
}

pub fn without_drive(path: &str) -> String {
    let mut chars = path.chars();
    match (chars.next(), chars.next()) {
        (Some(letter), Some(':')) if letter.is_ascii_alphabetic() => path[2..].to_owned(),
        _ => path.to_owned(),
    }
}

pub fn linux_host(home: &str) -> HostEnv {
    HostEnv::for_test(Os::Linux, &[("HOME", home)], Path::new(home))
}

pub fn host_for(os: Os, vars: &[(&str, &str)], root: &str) -> HostEnv {
    HostEnv::for_test(os, vars, Path::new(root))
}

pub struct FakeSteamTree {
    _dir: tempfile::TempDir,
    pub home: String,
}

impl FakeSteamTree {
    pub fn create() -> Self {
        let dir = tempfile::Builder::new().prefix("yp_steam_").tempdir().unwrap();
        let home = to_posix(dir.path());
        Self { _dir: dir, home }
    }

    pub fn steam_root(&self) -> String {
        posix::join_all(&self.home, &[".local", "share", "Steam"])
    }

    pub fn add_steam_root(&self) -> String {
        mkdirs(&posix::join(&self.steam_root(), "steamapps"));
        self.steam_root()
    }

    pub fn add_nier(&self, library: Option<&str>) -> String {
        let root = library.map(str::to_owned).unwrap_or_else(|| self.steam_root());
        let dir = posix::join_all(&root, &["steamapps", "common", "NieRAutomata"]);
        write(&posix::join(&dir, "NieRAutomata.exe"), "PRJ_028");
        dir
    }

    pub fn add_library_folders(&self, contents: &str) -> String {
        write(&posix::join_all(&self.steam_root(), &["steamapps", "libraryfolders.vdf"]), contents)
    }

    pub fn add_steam_client(&self) -> String {
        write(&posix::join_all(&self.home, &[".steam", "sdk64", "steamclient.so"]), "")
    }

    pub fn add_proton(&self, name: &str) -> String {
        write(&posix::join_all(&self.steam_root(), &["compatibilitytools.d", name, "proton"]), "#!/bin/sh\n")
    }

    pub fn add_steam_config(&self, contents: &str) -> String {
        write(&posix::join_all(&self.steam_root(), &["config", "config.vdf"]), contents)
    }

    pub fn add_official_proton(&self, dir_name: &str) -> String {
        write(&posix::join_all(&self.steam_root(), &["steamapps", "common", dir_name, "proton"]), "#!/bin/sh\n")
    }

    pub fn add_compat_tool(&self, dir_name: &str, manifest: &str) -> String {
        let proton = self.add_proton(dir_name);
        write(&posix::join(&posix::dirname(&proton), "compatibilitytool.vdf"), manifest);
        proton
    }

    pub fn add_compat_prefix(&self) -> String {
        let pfx = posix::join_all(&self.steam_root(), &["steamapps", "compatdata", "524220", "pfx"]);
        mkdirs(&posix::join_all(&pfx, &["drive_c", "users", "steamuser", "AppData", "Roaming"]));
        mkdirs(&posix::join_all(&pfx, &["dosdevices", "z:"]));
        pfx
    }

    pub fn host(&self) -> HostEnv {
        linux_host(&self.home)
    }

    pub fn plain(&self) -> WineContext {
        WineContext::isolated(self.host())
    }

    pub fn native(&self) -> WineContext {
        self.plain().with_steam_roots(vec![self.steam_root()])
    }
}

pub struct FakeBottleTree {
    _dir: tempfile::TempDir,
    pub root: String,
}

impl FakeBottleTree {
    pub fn create() -> Self {
        let dir = tempfile::Builder::new().prefix("yp_bottles_").tempdir().unwrap();
        let root = to_posix(dir.path());
        Self { _dir: dir, root }
    }

    pub fn add_bottle(&self, name: &str) -> String {
        self.add_bottle_for(name, "crossover")
    }

    pub fn add_bottle_for(&self, name: &str, user: &str) -> String {
        let bottle = posix::join(&self.root, name);
        mkdirs(&posix::join_all(&bottle, &["drive_c", "users", user, "AppData", "Roaming"]));
        mkdirs(&posix::join_all(&bottle, &["drive_c", "users", "Public"]));
        bottle
    }

    pub fn add_nier(&self, bottle_name: &str) -> String {
        write(
            &posix::join_all(
                &self.root,
                &[
                    bottle_name,
                    "drive_c",
                    "Program Files (x86)",
                    "Steam",
                    "steamapps",
                    "common",
                    "NieRAutomata",
                    "NieRAutomata.exe",
                ],
            ),
            "PRJ_028",
        )
    }

    pub fn add_library_folders(&self, bottle_name: &str, contents: &str) -> String {
        write(
            &posix::join_all(
                &self.root,
                &[bottle_name, "drive_c", "Program Files (x86)", "Steam", "steamapps", "libraryfolders.vdf"],
            ),
            contents,
        )
    }

    pub fn add_wine_script(&self) -> String {
        write(&posix::join(&self.root, "fake-wine"), "#!/bin/sh\nprintf \"%s\\n\" \"$@\"\n")
    }

    pub fn ctx(&self, os: Os, wine_command: Option<String>) -> WineContext {
        WineContext::isolated(host_for(os, &[("HOME", &posix::join(&self.root, "home"))], &self.root))
            .with_bottle_roots(vec![self.root.clone()])
            .with_wine_command(wine_command)
    }
}
