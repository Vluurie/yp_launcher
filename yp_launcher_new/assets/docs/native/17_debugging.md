# How to debug when something goes wrong

## Where the logs are

Everything is written to the `logs\` folder next to `NAMS.exe`:

| File | What is in it |
|---|---|
| `nams.log` | NAMS and the content loader. Every mod that was loaded, every error while loading. NAMS appends to it; the launcher empties it before each launch. |
| `<plugin>.log` | One file per plugin built on `nams_sdk`, emptied at every start. |
| `heartbeat.log` | Written only when NieR:Automata froze, see below. |
| `crashdump_<time>.txt` | Written only after a crash, see below. |

In the launcher, **LOGS** opens the log viewer for `nams.log`. The
**Troubleshooting** tab has **Generate diagnostics**, which collects a
report of your install to copy (**Copy summary**) or save
(**Save full report**) when asking for help. **Copy command** on the
same tab copies the NAMS command line; started from a terminal, NAMS
prints its log there as well.

## Reading the boot log

Search `nams.log` for `[Native]` and `[NAMS]`. Every mod with an
`entities\` folder prints

```
[Native] <mod id>: staged <n> entities
```

and after all mods are through, the totals:

```
[NAMS] Native loader: <n> entities staged
[NAMS] Native loader: <n> registered, <n> failed, <n> skipped (FK)
```

A `<prefix>_<id>.toml` file is one entity, and every table or `[[…]]`
block of a list file is one. If your mod's number is lower than that,
or the totals show `failed` above zero, the reason is in the lines in
between. Entities dropped by a conflict between mods are counted in
`[NAMS] <n> items skipped due to mod conflicts (see messages above)`.

## Common errors

**`[Native] Failed to parse <file>: <message>`**
The TOML itself is broken: a missing quote, a bracket, a `=` too many.
The message starts with `TOML parse error at line <line>, column <column>`.
Nothing from that file is loaded.

**`[Native] <file> is empty`**
The file has no fields or tables.

**`[Native] <file> [<kind>]: <message>`**
The TOML is valid but a field is wrong for that kind of entity: a
required field is missing or a value has the wrong type. In a list
file, `unknown entity kind '<name>'` means a top-level table is not a
kind, and `[<kind>][<n>]` at the start of the message names the entry;
none of that file's `[[<kind>]]` entries load then. A misspelled
optional field is not reported: NAMS ignores fields it does not know.
Compare with the walkthrough for that kind.

**``[Native] <file> has no `kind` field at top level — skipping``**
A `.json` entity file needs a top-level `kind` field.

**`[Native] Invalid <path>: <message>`**
Your `mod.toml` is broken or has a field NAMS does not know, see
[how to lay out your mod folder](02_mod_folder_layout.md). The whole
mod is skipped.

**`[NAMS] Mod '<mod>' requires plugin '<plugin>', but it is not loaded.`**
Your `mod.toml` lists a plugin that is not installed, or
`disable_plugin_loading` is on. Install it or remove the requirement.

**`[NAMS] Two mods try to use the same <kind> number '<id>'.`**
Two mods claim the same id. The mod whose folder sorts first keeps it;
the other entity is skipped. Disable one or change ids.

**`[NAMS] Mod '<mod>' lists the same <kind> number '<id>' twice.`**
Two entities inside the same mod claim the same id. The first copy is
kept. Change one.

**`[NAMS] Skipping <kind> '<id>' from mod '<mod>' because the <kind> '<id>' it depends on (from mod '<mod>') is not loaded.`**
Cascade drop. The thing you depend on was rejected, so yours dropped
too. Fix the upstream issue first. When the other mod reads
`'<unstaged>'`, no loaded mod defines what you reference: for example
a quest whose `reward_id` names a reward that no loaded mod ships. Fix
the reference or ship the missing entity. References to the ids in
[base item ids](resources/vanilla_item_ids.md),
[base weapon ids](resources/vanilla_weapon_ids.md) and
[base shop names](resources/vanilla_shop_names.md) are not affected.

**`[NAMS] Register failed for <kind> from mod '<mod>': <message>`**
The entity was accepted but could not be installed. The message says
why. Effect rules and mesh overrides end up here:

| Message | Fix |
|---|---|
| `unknown path '<path>'` | The rule's `path` is misspelled or not a known path, see [How to make special effects](11_special_effects.md). |
| ``rule '<name>': exactly one of `value` or `levels` must be set`` | You have both `value` and `levels` (or neither). Pick one. |
| `level key '<key>' must be 1/2/3/4` | Bad key in `levels = { ... }`. Use string keys `"1"`, `"2"`, `"3"`, `"4"`. |
| `unknown op '<op>' (expected mul/add/set)` | Bad `op`. |
| `unknown player '<value>' (expected 2B/9S/A2/not_2b/not_9s/not_a2)` | Bad value in `when = { player = ... }`. |
| `unknown model '<value>' (expected pl2b/pl9s/pla2/pla2_wig)` | Bad `model` in a mesh override. |
| `unknown force '<value>' (expected show/hide)` | Bad `force` in a mesh override. |

**`[NAMS] skipping disabled mod: "<folder>"`**
The mod is listed in `nams\disabled_mods.toml`, see
[How to turn a mod off without deleting it](14_disabled_mods.md).

## Changing files while NieR:Automata runs

- **`entities\*.toml`** are read again the moment you save them, while
  `disable_content_features` is `false`. The entities of that file are
  removed and registered again; the log says
  `[NAMS] hot register <file>: <n> registered, <n> failed, <n> purged`.
  A save with a TOML or field error logs the same `[Native]` errors as
  at boot plus `[NAMS] hot register: nothing staged from <file>`, and
  the previous version stays. A save that fails while registering logs
  `[NAMS] hot register failed for <kind> '<id>': <message>`, and that
  entity stays out until you save a good version. Deleting the file
  removes what it registered. The kinds marked in the
  [cheat sheet](18_cheat_sheet.md) need a restart.
- **`entities\*.json`** are not read again. Restart.
- **`nams.toml`** is re-read on save; which fields apply live is listed in
  [How to configure NAMS](13_nams_toml.md).
- **`disabled_mods.toml`, `mod_load_order.toml`, `default_mods.toml`**
  are re-read on save, but mods already loaded stay loaded. Restart for
  the lists to fully apply.
- **Texture files (`.dds`)** are re-read on save while `hot_reload` in
  `texture_injection.toml` is on, see
  [How to replace textures](33_texture_injection.md).
- **Files under `data\`** are mounted at startup. Restart after
  changing them.

## Finding which mod breaks it

1. Read the totals line and the conflict line. `failed` above zero or
   a conflict count means something was rejected; the error lines
   above name the mod or the file.
2. Disable mods one at a time with `nams\disabled_mods.toml` and
   restart.
3. Keep NieR:Automata running for entity edits (previous section);
   restart for everything under `data\`.

## Freezes and crashes

**The picture freezes and a message box appears.** NAMS checks every
5 seconds whether new frames are drawn. When none came for three checks
in a row, a report is written to `logs\heartbeat.log`, the box
**NieR:Automata - Heartbeat Flatline** shows its path, and
NieR:Automata closes when you press OK. The same happens when no frame
is drawn at all within 90 seconds after start. If the box mentions
NVIDIA Smooth Motion (`NvPresent64.dll`), turn Smooth Motion off for
NieR:Automata in the NVIDIA App; with it detected, that start limit is
30 seconds. The check pauses while the window is inactive, minimized
or behind the Steam overlay, and it is off during the first 30 seconds
after the first frame.

**NieR:Automata closes with a crash box.** The box
**NieR:Automata has crashed** says where the crash reports were saved:
`logs\crashdump_<time>.txt` holds the stack trace and the calls that
were still running. When the crash is traced to a mod's plugin, the
box is titled **Plugin "<plugin>" crashed** and names the mod and
where to report it. Attach the file when reporting.

**NAMS itself hits an internal error.** It writes
`logs\crashdump_<time>.txt` with the error message, the backtrace and
the calls that were still running, and logs
`Panic crash report written to <path>` in `nams.log`. Attach that
file when reporting.
