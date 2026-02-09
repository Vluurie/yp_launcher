# YoRHa Protocol Launcher

A Windows launcher for the YoRHa Protocol mod for NieR:Automata, built with Flutter. Also supports macOS and Linux via Wine/CrossOver.

## Requirements

- Flutter SDK ^3.9.2
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

### 3. Add Required Files

Place the following files in `assets/bins/`:

- `launch_nier.exe` - Launcher executable
- `modloader.dll` - Modloader library
- `yorha_protocol.dll` - YoRHa Protocol mod library

## Building

```bash
flutter build windows
```

## Usage

1. Launch the application
2. Click **SELECT** to choose your `NieR:Automata.exe` file
3. The launcher validates the executable (checks for the `PRJ_028` signature)
4. Click **PLAY** to start NieR:Automata with the YoRHa Protocol mod
5. Click **STOP** to terminate the game

## How It Works

On first launch, mod files are copied from bundled assets to a launcher directory:

- **Windows:** `%APPDATA%\YoRHaProtocolLauncher`
- **macOS (CrossOver):** `~/Library/Application Support/CrossOver/Bottles/<bottle>/drive_c/users/<user>/AppData/Roaming/YoRHaProtocolLauncher`
- **Linux (Wine):** `~/.wine/drive_c/users/<user>/AppData/Roaming/YoRHaProtocolLauncher`
- **macOS (native):** `~/Library/Application Support/YoRHaProtocolLauncher`
- **Linux (native):** `~/.local/share/YoRHaProtocolLauncher`

When you click PLAY, the launcher executes:

```
launch_nier.exe --modloader-dll <path> --mod-dll <path>
```

The working directory is set to your NieR:Automata installation folder.

## Links

- [Source Code](https://github.com/Vluurie/yp_launcher)
- [Guide / Documentation](https://gitlab.yasupa.de/nams/yp-docs/-/blob/master/YoRHa_Protocol_Documentation.md)
- [Discord](https://discord.gg/Z5spWtF8qs)
- [NAO Launcher (alternative)](https://www.nexusmods.com/nierautomata/mods/772?tab=files)
