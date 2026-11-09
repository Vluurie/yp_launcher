# How to change a menu or HUD layout

Menus and HUD elements are built from UI layout files (`.uid`), one file
per part, packed in one archive per UI, such as `ui\ui_core.dat`.
`[[ui_override]]` makes one part load from a file in your mod instead.

## How do I make one?

The easiest way is the UI editor in the overlay: open a part, change it,
and save it into your mod with **As override** ticked, which it is by
default. The editor writes both files:

```
nams\mods\my_mod\
  mod.toml
  entities\ui_override_core0371.toml
  ui\override_core0371.uid
```

`entities\ui_override_core0371.toml`:

```toml
[[ui_override]]
name = "my_mod.core0371"
ui = "core"
part = 881
file = "ui/override_core0371.uid"
```

When you set texts on the part's message layers (the layers that show
text), the editor also writes them to
`entities\localized_string_core0371.toml`, in the language you play in.
To add the other languages, see [How to translate your mod](12_translate.md).

## What goes in a row?

| Field | What it holds |
|---|---|
| `name` | A unique name for the row. The editor writes the mod id, a dot, `ui` and the four hex digits of `part`. |
| `ui` | The archive the part belongs to, without `ui_`, the language code and `.dat`: `core` for `ui_core.dat`. |
| `part` | The part's number: the four hex digits in its file name. `core0371.uid` is part `0x371`, which the editor writes as `881`. |
| `file` | The `.uid` in your mod folder, as a path from the mod folder. It has to stay inside the folder. |

All four are required.

## When does my file show?

A part that is created after the override is registered reads your
file. A part already on screen switches to your file as long as the file
has its layers; otherwise it switches the next time it is created.

Your file is only used while it still fits the UI built on the part:
every layer that has an id in the archive's file keeps that id, its
type, its effect and, for a container (a layer that shows another part),
the part it shows; every animation keeps its number. Added layers and
animations are fine. The editor does not save a file that breaks this.

An override only loads if the file reads back cleanly. If it does not,
the log names the row and the reason, and the part keeps its own file.

When two rows replace the same part, the first registered row that fits
is used. While the UI editor edits a part on screen, its version wins
over every row.

Saving the `.toml` again while you play reloads the row and its `.uid`;
saving only the `.uid` does not. `ui_overrides = false` in the
`[content]` table of `nams.toml` turns every override off, see
[How to configure NAMS](13_nams_toml.md).

## Why does my part not change?

**The row does not load.** The log names the file and the reason:

```
[Native] <file> [ui_override]: [ui_override][<n>] <reason>
```

| Reason | What is wrong |
|---|---|
| `parse ui_override: <message>` | A field is missing or has the wrong type. |
| `ui_override needs a name` | `name` is empty. |
| `ui_override '<name>' needs a ui` | `ui` is empty. |
| `ui_override '<name>' needs a file` | `file` is empty. |
| `ui_override '<name>': file must stay inside the mod folder` | `file` leaves the mod folder or is not a path from it. |
| `ui_override '<name>': part <part> is above 0xffff` | `part` is larger than four hex digits. |
| `ui_override '<name>': <path>: <error>` | The file at that path cannot be read. |
| `ui_override '<name>': <error>` | The file does not read back cleanly. |

Row `<n>` is counted from 0. One bad row stops every `[[ui_override]]`
row of that file.

**The row loads, but the part stays as it was.**

```
[UiOverride] not used: NoArchive { name: "<name>" }
[UiOverride] not used: NoSuchPart { name: "<name>" }
[UiOverride] not used: Incompatible { name: "<name>", problems: [<problems>] }
```

`NoArchive`: no UI archive has the name in `ui`. `NoSuchPart`: the
archive has no file with the number in `part`. `Incompatible`: your file
no longer fits the UI built on the part, see
[When does my file show?](#when-does-my-file-show).

**A part on screen keeps its look.**

```
[UiOverride] <ui> part <part> on screen keeps its file: <error>
```

Your file does not have the layers of the part on screen. The part shows
your file the next time it is created.
