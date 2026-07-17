# YoRHa Protocol Launcher

A friendly launcher for NieR:Automata mods. It installs and manages your mods,
textures and cutscenes, lets you tweak in-game settings, and starts the game for
you, all from one window.

Under the hood it runs **NAMS**, the mod system that hosts the game, so you don't
have to touch any config files or command lines yourself.

## Download

Grab the latest build for your system from the
[**Releases page**](https://github.com/Vluurie/yp_launcher/releases):

- **Windows**: `YoRHa Protocol Launcher.zip`. Unzip it anywhere and run
  `YoRHa Protocol Launcher.exe`. It's portable, no installer needed.
- **macOS**: `YP-Launcher.dmg`. Open it and drag the app to Applications.
  The first time, **right-click the app and choose Open** (it isn't notarized, so
  a plain double-click will be blocked by Gatekeeper).
- **Linux**: the `.AppImage` (make it executable and run it) or the `.tar.gz`
  (unpack and run `yp_launcher`).

## Getting started

Select your game, press play, done.

1. Open the launcher. On the very first run a short setup wizard walks you
   through picking your game.
2. Point it at your NieR:Automata install: click **AUTO** to let it find the game
   for you, or **SELECT** to pick your `NieRAutomata.exe` by hand.
3. Click **PLAY** to start the game with your mods. The same button turns into
   **STOP** while the game is running.

That's it. From the sidebar you can drag mod archives in to install them, manage
textures and cutscenes, and adjust settings from the config panels.

### Running on macOS and Linux

The game itself is a Windows program, so on macOS and Linux it runs through a
compatibility layer:

- **macOS**: needs [CrossOver](https://www.codeweavers.com/crossover). Install
  CrossOver and NieR:Automata into a bottle, then point the launcher at the
  `NieRAutomata.exe` *inside* that bottle. Everything else is figured out for you.
- **Linux**: runs the game through Steam Proton (or CrossOver/Wine). Just point
  the launcher at your installed `NieRAutomata.exe`.

Installing mods, textures and configs works the same everywhere.

## Where your files go

Everything the launcher manages lives in a `nams/` folder next to your game:
installed mods, texture packs, cutscenes and all the settings. Nothing is
scattered across your system, so your setup stays with the game folder.

## Links

- [Guide / Documentation](https://gitlab.yasupa.de/nams/yp-docs/-/blob/master/YoRHa_Protocol_Documentation.md)
- [Discord](https://discord.gg/Z5spWtF8qs)
- [Source Code](https://github.com/Vluurie/yp_launcher)
- [NAO Launcher (alternative)](https://www.nexusmods.com/nierautomata/mods/772?tab=files)

---

## Building from source

Most people should just download a release above. This section is for developers.

You'll need the Flutter SDK (Dart `^3.9.2`). Then:

```bash
flutter pub get
dart run build_runner build --delete-conflicting-outputs   # re-run after changing a @Riverpod provider
```

The `assets/bins/` folder holds the runtime binaries. `NAMS.exe` and the
`yorha_protocol.dll` plugin are committed; only the 7-Zip binaries are fetched
separately (`7z.exe`/`7z.dll` on Windows, `7zz` on macOS/Linux) from the
[official 7-Zip 26.02 release](https://github.com/ip7z/7zip/releases). On macOS
the `7zz` binary must be ad-hoc signed (`codesign --force --sign - 7zz`).

Build commands:

```bash
flutter build windows                # Windows
./scripts/build_macos_dmg.sh         # macOS  -> build/YP-Launcher-<version>.dmg
./scripts/build_linux_tarball.sh     # Linux  -> tarball
./scripts/build_linux_appimage.sh    # Linux  -> AppImage
```
