# ReShade, 3DMigoto and DLL mods

ReShade, 3DMigoto shader mods and DLL mods like speedrun timers or
trainers are made for the plain game. They expect to sit next to
`NieRAutomata.exe` and to be loaded by it. With NAMS the game runs
inside `NAMS.exe` instead, so a copy in your NieRAutomata folder is
never loaded. NAMS loads them from its own folder in your game install,
and the launcher's **Third Party** tab puts them there for you.

These tools are made by other people. The NAMS team can't test or fix
them, so if one misbehaves, its developer is the one to ask.

**KEEP IN MIND!** ReShade add-ons and Direct3D wrappers that load extra
DLLs from the folder of the game's exe don't work under NAMS. Under NAMS
that folder is the one holding `NAMS.exe`, so the mod looks there and
doesn't find its files. Don't copy them next to `NAMS.exe` either. If
you use the launcher, it replaces that folder with every update.

## How do I get ReShade working with NAMS?

Install ReShade into your NieRAutomata folder as usual (select
`NieRAutomata.exe`, pick **DirectX 10/11/12**), then open the
**Third Party** tab. The launcher copies ReShade into NAMS by itself.
Your NieRAutomata folder stays as it is, so ReShade keeps working
without NAMS too.

If the ReShade card says *Found in your game folder*, ReShade is turned
off in NAMS. Turn **Disable ReShade Loading** off in the **NAMS** tab
and open the **Third Party** tab again.

It worked when the card says **Active** and the ReShade overlay shows
up in the game.

If the card says *No effects installed yet*, ReShade has no shaders to
run. Drop a preset or shader pack (`.zip`, `.7z` or `.rar`) onto the box
at the top of the tab.

Only one ReShade loads at a time. On Linux (Wine, Proton), ReShade also
needs a Windows `d3dcompiler_47.dll` in its folder; the card warns you
when it's missing.

## How do I add a 3DMigoto mod?

Drop the mod archive onto the box at the top of the **Third Party** tab.

3DMigoto only changes a game when `d3dx.ini` names that game's exe under
`[Loader]`. Under NAMS that exe is `NAMS.exe`, so the launcher writes
`target = NAMS.exe` there for you. If you edit `d3dx.ini` yourself, keep
that line.

Shader edits made for Special K (they come with a `d3d11_shaders.ini`)
don't work with NAMS. The launcher tells you when you drop one.

## How do I add a DLL mod made for the plain game?

Drop the `.dll`, or the archive it came in, onto the box. It shows up
under **Game mods**.

These mods find the game by asking which program is running. Under NAMS
the honest answer would be `NAMS.exe`, so NAMS answers for them as if
they ran in `NieRAutomata.exe`. Most of them work that way, but the
launcher can't check what a mod does, and any of them can crash the
game. Only add files you trust.

Each entry has its own settings:

| Setting | What it does |
| --- | --- |
| Show game.bin as the main module | Lets a mod that reads the game's memory find the game. Leave it on unless the mod is made to work with `NAMS.exe`. |
| Show NieRAutomata.exe as the main exe | Gives mods that check the exe name or the exe file the `NieRAutomata.exe` of your install. |
| Game version the mod sees | Pick the game version the mod was made for. A mod made for the 2021 version reads the wrong memory on the 2026 update unless the mod supports it. |
| Log the game code this mod changes | Writes every place in the game code the mod changed to `nams.log`. |

The settings are saved in `nams\thirdparty\game\game.toml`.

If the mod replaces or hooks game functions that NAMS hooks too, the
two can get in each other's way. Turn off the NAMS feature that covers
the same part of the game in the **NAMS** tab, or adjust the mod's own
settings so they work together. **Log the game code this mod changes**
shows you which parts of the game code the mod touches.

## What about other d3d11.dll or dxgi.dll mods?

A `d3d11.dll` or `dxgi.dll` that isn't ReShade or 3DMigoto wraps the
game's Direct3D rendering. Drop it onto the box like everything else;
it shows up under **Graphic proxies**, and NAMS sends the game's
Direct3D calls through it.

Two mods with the same DLL name can't run together. NAMS loads the
first one by folder name and skips the other.

If the launcher puts a DLL into the wrong section, use the move button
on its entry.

## How do I turn a mod off?

| What | Where |
| --- | --- |
| One DLL mod or Direct3D wrapper | Its switch in the **Third Party** tab |
| ReShade | **NAMS** tab → **Disable ReShade Loading** |
| 3DMigoto | **NAMS** tab → **Disable 3DMigoto Loading** |
| All Direct3D wrappers | **NAMS** tab → **Disable Graphics Proxy Loading** |
| All DLL mods | **NAMS** tab → **Disable Game Mod Loading** |

Changes apply the next time you start the game.

## Where do the files go?

```
<your NieRAutomata folder>\nams\thirdparty\
├── reshade\<mod>\
├── 3dmigoto\<mod>\
├── graphic\<mod>\
└── game\<mod>\
```

Each mod has its own folder inside the matching one. The folder button
on each card in the tab opens it.

To add a mod by hand, make a new folder for it inside the matching one
and put its DLLs directly in there. NAMS doesn't look into subfolders.

Older launcher versions kept everything in a `thirdparty` folder next to
`NAMS.exe`. The launcher moves it over by itself.

## The game crashes or acts strange

Turn the mod you added last off and start again. If the game is fine
without it, the mod is the cause; ask its developer. Before you report
a crash to the NAMS team, try once without anything from the
**Third Party** tab.

A mod that works in the plain game but not with NAMS has to support
running inside `NAMS.exe`. Only its developer can change that.

## A mod doesn't load

Open the **Troubleshooting** tab, go to the logs and look in `nams.log`
for lines that start with `[ReShade]`, `[3DMigoto]`, `[Graphic]` or
`[GameMods]`.

| The log says | What to do |
| --- | --- |
| `no runtime DLL` | NAMS found no ReShade or 3DMigoto DLL. It has to lie directly in the mod's folder and be named `dxgi.dll`, `ReShade64.dll` or `ReShade64_Addon.dll` for ReShade, `d3d11.dll` or `3dmigoto.dll` for 3DMigoto. |
| `a dependency DLL beside it is missing` | Some of the mod's files are missing. Copy all of them into the same folder. |
| `wrong architecture (needs a 64-bit DLL)` | That's the 32-bit version. Get the 64-bit one. |
| `has no [Loader] target` or `[Loader] target is '...'` | Put `target = NAMS.exe` under `[Loader]` in `d3dx.ini`. |
| `skipped, the runtime is loaded from` | A second ReShade or 3DMigoto. Only the first one by folder name loads. |
| `skipped, a DLL named ... is already loaded` | Two mods use the same DLL name. Only the first one by folder name loads. |
| `no matching imports found` | The mod finds the game in a way NAMS can't answer for. It won't work under NAMS until its developer adds support. |

## What's planned

Loading ReShade, 3DMigoto and DLL mods moves out of NAMS into a separate
compatibility DLL, offered as an extra download. NAMS itself then loads
no third-party mods. Until then, everything on this page works as
described.
