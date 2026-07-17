# YoRHa Protocol Launcher

A launcher for NieR:Automata mods, built with Flutter. It drives **NAMS** — the
mod system that hosts the game — and manages mods, textures, cutscenes and NAMS
configuration.

Windows runs it natively. macOS runs the game through CrossOver: install
CrossOver and NieR:Automata into a bottle yourself, then point the launcher at
`NieRAutomata.exe` *inside* that bottle — everything else (bottle, prefix, Wine
binary) is derived from that one path. Mods, textures and configs work with or
without CrossOver; only launching needs it. Linux support is written but
untested on real hardware.

## Requirements

- Flutter SDK (Dart `^3.9.2`)
- NieR:Automata (latest Steam version)
- Windows (native), or macOS 12+ with [CrossOver](https://www.codeweavers.com/crossover)

## Setup

### 1. Install Dependencies

```bash
flutter pub get
```

### 2. Generate Riverpod Code

```bash
dart run build_runner build --delete-conflicting-outputs
```

Re-run this after adding or changing a `@Riverpod` provider.

### 3. Add Required Files

The 7-Zip binaries are gitignored and fetched out of band:

```
assets/bins/
├── NAMS.exe                    # the mod system; hosts and runs the game
├── 7z.exe, 7z.dll              # archive extraction (Windows, gitignored)
├── 7zz                         # archive extraction (macOS/Linux, gitignored)
└── plugins/
    └── yorha_protocol.dll      # YoRHa Protocol plugin, loaded by NAMS
```

`7zz` is the official 7-Zip console build. Fetch and sign it with:

```bash
curl -L -o /tmp/7z.tar.xz https://github.com/ip7z/7zip/releases/download/26.02/7z2602-mac.tar.xz
tar -xf /tmp/7z.tar.xz -C /tmp 7zz License.txt
codesign --force --sign - /tmp/7zz
mv /tmp/7zz assets/bins/7zz
mv /tmp/License.txt assets/bins/7zz-License.txt
```

Expected: 7-Zip 26.02, universal (`lipo -archs` → `x86_64 arm64`),
sha256 `476a8b94dc2c1427368c0f8f211e403cbe7c8adde219d6e0202e75b5a0470697`.
The signature matters: without it `codesign --verify` fails and the extracted
copy cannot be validated. 7-Zip is LGPL + BSD-3-Clause with the unRAR
restriction on the RAR decoder, so its `License.txt` ships alongside it.

## Building

```bash
flutter build windows
```

### macOS

```bash
./scripts/build_macos_dmg.sh
```

Builds, ad-hoc signs and produces `build/YP-Launcher-<version>.dmg`. It is
**not notarized**, so Gatekeeper will refuse a plain double-click — right-click
→ Open, or `xattr -dr com.apple.quarantine "/Applications/YP Launcher.app"`.

### Linux

Works on Linux; the game runs through Proton (or CrossOver/Wine).

```bash
./scripts/build_linux_tarball.sh     # → build/yp_launcher-<version>-linux-x64.tar.gz
./scripts/build_linux_appimage.sh    # → build/yp_launcher-<version>-x86_64.AppImage
```

`assets/bins/7zz` must be the Linux x86-64 build here (no signing needed).

## Usage

1. Launch the application
2. Click **SELECT** to choose your `NieRAutomata.exe` file
3. The launcher validates the executable (checks for the `PRJ_028` signature)
   and remembers the folder it lives in
4. Click **PLAY** to start the game through NAMS
5. Click **STOP** to terminate it

## How It Works

NAMS writes its cache, plugins and logs *next to its own exe*, so that directory
has to be writable. Where it lives depends on the host:

| Host | NAMS runtime directory |
| --- | --- |
| Windows | `<launcher exe dir>/data/flutter_assets/assets/bins/` (portable, as shipped) |
| macOS | `~/Library/Application Support/com.vluurie.yplauncher/bins/` |

On macOS the app bundle is signed and sealed, so the binaries are unpacked once
into Application Support and run from there. They are **not** copied into the
CrossOver bottle — Wine maps the host filesystem to `Z:`, which is how NAMS is
reached from inside the prefix.

On **PLAY** the launcher runs:

```
# Windows
NAMS.exe run --nier-path <game dir>

# macOS
/Applications/CrossOver.app/.../bin/wine --bottle <name> --workdir <runtime dir> \
  <runtime dir>/NAMS.exe run --nier-path 'C:\Program Files (x86)\Steam\...'
```

Note the asymmetry on macOS: the exe is passed as a **host** path (CrossOver's
wine wrapper translates it), while `--nier-path` is a **Windows** path, because
NAMS consumes it Windows-side.

The game runs *inside* the NAMS host process via `game.bin` — there is no
separate `NieRAutomata.exe` process and no DLL injection. Stopping the game
means terminating `NAMS.exe`.

Plugins are enabled by file presence: NAMS loads whatever sits in
`bins/plugins/`. `yorha_protocol.dll` is always present there.

### Configuration and mods

Everything the launcher writes lives under your game folder in `nams/`:

| Path | Purpose |
| --- | --- |
| `nams/nams.toml` | NAMS engine + content settings |
| `nams/lodmod.toml` | LodMod visual settings |
| `nams/texture_injection.toml` | texture pack load order and VRAM budget |
| `nams/disabled_mods.toml` | mods excluded at boot |
| `nams/mod_names.json` | custom mod display names (launcher-only) |
| `nams/mods/` | installed mods |
| `nams/inject/textures/` | texture packs |
| `nams/cutscenes/` | cutscene mods |

The one exception is `settings.json`, which the mod itself owns. It lives where
NAMS looks for it — on Windows `%APPDATA%/NAMS/`, and on macOS inside the
bottle at `<bottle>/drive_c/users/<user>/AppData/Roaming/NAMS/`.

## Links

- [Source Code](https://github.com/Vluurie/yp_launcher)
- [Guide / Documentation](https://gitlab.yasupa.de/nams/yp-docs/-/blob/master/YoRHa_Protocol_Documentation.md)
- [Discord](https://discord.gg/Z5spWtF8qs)
- [NAO Launcher (alternative)](https://www.nexusmods.com/nierautomata/mods/772?tab=files)
