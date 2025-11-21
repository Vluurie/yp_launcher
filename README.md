# YorHa Protocol Launcher

A cross-platform launcher for the YorHa Protocol mod for NieR:Automata.

## Requirements

- Flutter SDK 3.9.2 or higher
- NieR:Automata installation
- Windows, macOS, or Linux

## Setup

### 1. Install Dependencies

```bash
flutter pub get
```

### 2. Generate Code

```bash
dart run build_runner build --delete-conflicting-outputs
```

### 3. Add Required Files

Place the following files in the `assets/bins/` directory:

- `launch_nier.exe` - Launcher executable
- `modloader.dll` - Modloader library
- `yorha_protocol.dll` - YorHa Protocol mod library

Place the YorHa Protocol logo image in `assets/images/`:

- `yp_logo.jpg` - Launcher logo image

## Building

### Windows

```bash
flutter build windows
```

### macOS

```bash
flutter build macos
```

### Linux

```bash
flutter build linux
```

## Usage

1. Launch the application
2. Click SELECT to choose your NieR:Automata.exe file
3. The launcher will validate the executable
4. Click PLAY to start NieR:Automata with the YorHa Protocol mod
5. Click STOP to terminate the game

## Configuration

Click the settings icon in the top right corner to override default mod files with custom versions.

You can override:
- Launcher executable path
- Modloader DLL path
- YorHa Protocol DLL path

## How It Works

The launcher copies the required mod files to a launcher directory:

- Windows: `%APPDATA%\YorHaProtocolLauncher`
- macOS: `~/Library/Application Support/YorHaProtocolLauncher`
- Linux: `~/.local/share/YorHaProtocolLauncher`

When you click PLAY, the launcher executes:

```
launch_nier.exe --modloader-dll <path> --mod-dll <path>
```

The working directory is set to your NieR:Automata installation folder.

## Technical Details

### Dependencies

- automato_theme - UI theme library
- flutter_riverpod - State management
- file_picker - File selection dialogs
- shared_preferences - Settings persistence
- path_provider - Cross-platform paths
- win32 - Windows process management
- ffi - Foreign function interface

### State Management

Uses Riverpod with code generation for type-safe state management.

### Process Management

- Windows: Uses Win32 API for process enumeration and control
- macOS/Linux: Uses pkill and pgrep utilities

### Performance

- File validation runs in background isolates
- File copying operations run in background isolates
- UI remains responsive during all operations

## Troubleshooting

Check the debug console for detailed logging:

- Launcher executable path
- Command line arguments
- Working directory
- Process output and errors

The launcher validates NieR:Automata executables by searching for the PRJ_028 signature.

## License

This project is provided as-is without warranty.
