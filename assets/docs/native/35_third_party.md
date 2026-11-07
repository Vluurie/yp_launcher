# How to use ReShade, 3DMigoto and other tools with NAMS

Put the tool into the matching folder in `thirdparty\`, next to
`NAMS.exe`. NAMS loads it at game start. The launcher keeps `NAMS.exe`
in its own folder; its **Third Party** tab installs into these folders
for you: drop the files onto **Drop ReShade presets, 3DMigoto mods or
game-mod DLLs here**.

```
<folder of NAMS.exe>\
  NAMS.exe
  thirdparty\
    reshade\      ReShade: dxgi.dll, ReShade64.dll or ReShade64_Addon.dll, with its other files
    3dmigoto\     3DMigoto: d3d11.dll or 3dmigoto.dll, d3dx.ini and its other files
    graphic\      mods that replace a Direct3D DLL (d3d11.dll, dxgi.dll), with their files
    game\         DLL mods made for a plain NieRAutomata.exe
```

NAMS loads `graphic\` first, then 3DMigoto, then ReShade. `game\` is
loaded earlier, before NieR:Automata starts running.

For 3DMigoto, `d3dx.ini` needs `target = NAMS.exe` under `[Loader]`.
The launcher writes this line when it installs 3DMigoto. Without it the
log says `[3DMigoto] ... has no [Loader] target; set 'target = NAMS.exe'
or it will not hook`.

To turn a folder off, set its switch to `true` in `nams\nams.toml` or
in the launcher's NAMS tab. It applies at the next start; the log then
says, for example, `[ReShade] Loading disabled via nams.toml`.

| Switch | Launcher NAMS tab | Folder |
|---|---|---|
| `disable_reshade_loading` | Disable ReShade Loading | `thirdparty\reshade\` |
| `disable_3dmigoto_loading` | Disable 3DMigoto Loading | `thirdparty\3dmigoto\` |
| `disable_graphic_mods_loading` | Disable Graphics Proxy Loading | `thirdparty\graphic\` |
| `disable_game_mods_loading` | Disable Game Mod Loading | `thirdparty\game\` |

## How do mods in `thirdparty\graphic\` work?

These mods replace a Direct3D DLL. Copy all files of the mod into the
folder. NAMS loads every `.dll` lying directly in the folder and sends
the calls that `NAMS.exe` and `game.bin` (the module NAMS builds from
`NieRAutomata.exe` and runs) make to a DLL of the same name
through the mod. Files in subfolders are not loaded. Two mods cannot
share one DLL name. The log lists what goes through each mod:
`[Graphic] <dll>: <module> calls <functions> through it`.

## How do mods in `thirdparty\game\` work?

DLL mods in this folder were made for a plain `NieRAutomata.exe`: they
read memory relative to the main module and check the exe to detect the
version. NAMS rewrites the imports of each DLL so that it sees
`game.bin` as its main module, the path of your `NieRAutomata.exe` as
its exe, and the image size of version 1.02. Only the DLL's own imports
change. A file `game.toml` in the folder
adjusts single mods:

```toml
# thirdparty\game\game.toml
["celestial.dll"]
report_patches = true
```

| Field | Default | What it does |
|---|---|---|
| `disabled` | `false` | Skips this DLL. Log: `[GameMods] <dll> disabled via game.toml, skipping`. |
| `report_patches` | `false` | Writes the places in `game.bin` the DLL rewrote to the log (the first 64 of them), after `[GameMods] <dll> rewrote <n> code site(s):`. |
| `fake_module_handle` | `true` | `GetModuleHandle(NULL)` in the DLL returns `game.bin`. With `false` it returns `NAMS.exe`. |
| `fake_exe_path` | `true` | `GetModuleFileName(NULL)` in the DLL returns the path of your `NieRAutomata.exe`. |
| `size_of_image` | `26177536` | Image size of the main module the DLL reads, usually for its version check. The default is the size of version 1.02. `0` passes the real size of `game.bin`. |
| `wait_for_patch_phase` | `false` | NAMS waits until the DLL reports that its patches are done before it goes on. Needs `fake_exe_path = true`. |
| `patch_phase_timeout_ms` | `10000` | Longest wait, in milliseconds. After it NAMS goes on and writes a warning to the log. |

`[defaults]` sets values for every DLL without its own entry. A DLL's
own entry does not take values from `[defaults]`: what it leaves out
has the default in the table. Entry names match the DLL file name in
any case.

## "A tool or mod does not load or does nothing."

- **`[ReShade] no runtime DLL (...) found in ...`** (or `[3DMigoto]`):
  none of the DLL names in the folder tree above lies directly in the
  folder.
- **`[ReShade] failed to load ...: a dependency DLL beside it is
  missing (win32 126)`** (or `[3DMigoto]`): copy the tool's other files
  into the same folder.
- **`[ReShade] failed to load ...: wrong architecture (needs a 64-bit
  DLL)`** (or `[3DMigoto]`): the DLL is not a 64-bit build.
- **`[GameMods] <dll>: no matching imports found`**: the DLL may look
  these functions up itself; then it still reads the wrong base
  address.
