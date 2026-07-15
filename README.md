# YoRHa Protocol Launcher

A Windows launcher for NieR:Automata mods, built with Flutter. It drives
**NAMS** — the mod system that hosts the game — and manages mods, textures,
cutscenes and NAMS configuration. Also supports macOS and Linux via
Wine/CrossOver.

## Requirements

- Flutter SDK (Dart `^3.9.2`)
- NieR:Automata (latest Steam version)
- Windows (native), or macOS/Linux with Wine or CrossOver

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

The launcher ships NAMS and its plugins in `assets/bins/`:

```
assets/bins/
├── NAMS.exe                    # the mod system; hosts and runs the game
├── 7z.exe, 7z.dll              # archive extraction for mod installs
└── plugins/
    └── yorha_protocol.dll      # YoRHa Protocol plugin, loaded by NAMS
```

## Building

```bash
flutter build windows
```

## Usage

1. Launch the application
2. Click **SELECT** to choose your `NieRAutomata.exe` file
3. The launcher validates the executable (checks for the `PRJ_028` signature)
   and remembers the folder it lives in
4. Click **PLAY** to start the game through NAMS
5. Click **STOP** to terminate it

## How It Works

The launcher is portable — nothing is copied into AppData. NAMS and its plugins
stay where they ship, which at runtime is:

```
<launcher exe dir>/data/flutter_assets/assets/bins/
```

That directory is also NAMS's working directory. On **PLAY** the launcher runs:

```
NAMS.exe run --nier-path <game dir>
```

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

The one exception is `%APPDATA%/NAMS/settings.json`, which the mod itself owns.

## Links

- [Source Code](https://github.com/Vluurie/yp_launcher)
- [Guide / Documentation](https://gitlab.yasupa.de/nams/yp-docs/-/blob/master/YoRHa_Protocol_Documentation.md)
- [Discord](https://discord.gg/Z5spWtF8qs)
- [NAO Launcher (alternative)](https://www.nexusmods.com/nierautomata/mods/772?tab=files)
