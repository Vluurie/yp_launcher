# How to make an archive

An archive is a document in the **Archives** list of the Data menu,
which the player opens from the pause menu. The list shows categories;
the player opens a category and picks an entry, and its text appears in
the detail panel (the panel that shows the selected entry).

Your archives go into categories of your own, listed after the
categories that ship with NieR:Automata.

## How do I get my archive into the Archives list?

Create `entities\archive_<name>.toml`, where `<name>` is the name of your
archive (see [How to lay out your mod folder](02_mod_folder_layout.md)).
The name may contain `_`.

```toml
category = "<category>"
unlocked = true

[text.category]
_us = "<category header>"

[text.title]
_us = "<title>"

[text.description]
_us = "<text>"

[icon]
misctex_name = "<texture name>"
```

| Field | Default | What it does |
|---|---|---|
| `category` | required | The name of the category this archive is listed under. Every archive with the same `category` lands under the same header. |
| `unlocked` | `false` | `true` puts the archive into its category from the start. With `false` it stays hidden until you unlock it, see [How do I unlock the archive from my quest?](#how-do-i-unlock-the-archive-from-my-quest) |
| `[text.category]` | see below | The header text of the category. |
| `[text.title]` | required | The entry in the list and the title in the detail panel. |
| `[text.description]` | empty | The text in the detail panel. |
| `[icon] misctex_name` | none | A picture in the detail panel, see [How do I show a picture with my archive?](#how-do-i-show-a-picture-with-my-archive) |
| `name` | `<name>` from the file name | The name of the archive. Write it here only when the name is made of digits alone; NAMS does not take such a name from the file name. |

All texts take the language codes from
[How to translate your mod](12_translate.md).

An archive without `[text.category]` is only accepted when an archive of
the same category that has the block was loaded before it. Give the
block to every archive of the category, then the order does not matter.

The name may be at most 33 characters long; a character outside ASCII
counts as two to four. When two mods have an archive of the same name,
only one of them loads, so start the name with something unique to
your mod. A category is listed only while at least one of its archives
is unlocked.

To check it, open the pause menu, go to the Data menu and pick
**Archives**. Your category comes after the categories that
ship with NieR:Automata and carries the "new" marker. Confirm it, and
your entries unfold below it; select one and its description shows in
the detail panel. **Archives** appears in the Data menu once any archive
is unlocked, yours included.

<!-- screenshot: Data menu → Archives with a custom category opened -->

## How do I show a picture with my archive?

Ship the picture as a `.dat`/`.dtt` texture pair in the `data\misctex\`
folder of your mod:

```
data\misctex\misctex_<texture name>.dat
data\misctex\misctex_<texture name>.dtt
```

Write `<texture name>` into `[icon] misctex_name`, without the
`misctex_` prefix and without the extension. An item icon is shipped and
named the same way, see [How to make an item](03_item.md).

If NAMS cannot load the texture, a stand-in picture shows instead.
Without `[icon]`, or with an empty `misctex_name`, the archive shows
text only.

## How do I unlock the archive from my quest?

Leave `unlocked = false`. NAMS creates a scenario flag (a named on/off
switch kept with the save, see
[Counters and switches for your quest](23_scenario_variables.md)) with
the same name as your archive. The archive shows up in its category
once that flag is on, and goes away again when it is switched off.
Switch it from your quest script like a flag you declared yourself.

The "new" marker stays on the entry until the cursor has been on it and
moves on, or until the player backs out of it. The category keeps its
marker while any of its unlocked entries is still new. NAMS keeps the
read state in a second flag, `<name>_read`. When you save, both flags go
into the NAMS save file, see [The NAMS save file](43_nams_save_file.md).
Do not declare flags with either name yourself: one of the two
declarations then fails to register.

## Something does not work

Errors are written to the NAMS log, see
[How to debug when something goes wrong](17_debugging.md).

**My archive is not in the list.** Look for a line that starts with
`[NAMS] Register failed for archive_bundle from mod '<mod id>': ` (or
with `[NAMS] hot register failed for archive_bundle '<name>': ` after you
saved the file while NieR:Automata runs) and ends with one of these:

- `archive '<name>' name is longer than 33 characters` – shorten the name.
- `archive '<name>' has no category` – `category` is empty. Fill it in.
- `archive '<name>' has no [text.title]` – add a `[text.title]` block.
- `archive '<name>': category '<category>' has no [text.category] yet` –
  no archive of that category with a `[text.category]` block was loaded
  before this one. Add the block to this archive.
- `archive '<name>': a scenario_flag of the same name already exists` or
  `archive '<name>': a scenario_flag named '<name>_read' already exists` –
  another mod or your own `scenario.toml` already declares that flag
  name.

Two other lines also keep an archive out:

- `[Native] <file> [archive_bundle]: parse archive_bundle: <reason>` –
  NAMS could not read the file; `<reason>` says why. A file without a
  `category` line ends here, and so does a name made of digits alone
  without a `name` line.
- `[NAMS] Two mods try to use the same archive_bundle number '<name>'.` –
  another mod has an archive of the same name. The mod named after
  `• Skipped:` does not get its archive. Rename yours.

Without any of these lines, check that `archives` under `[content]` in
`nams.toml` is not `false` and `disable_content_features` is not `true`
(see [How to configure NAMS](13_nams_toml.md)). Then check `unlocked`,
or whether your script has switched the flag on yet.

**The detail panel shows a stand-in picture.** NAMS could not load the
texture named in `misctex_name`. Check that the name has no `misctex_`
prefix and that both files are in `data\misctex\` as
`misctex_<texture name>.dat` and `misctex_<texture name>.dtt`.
