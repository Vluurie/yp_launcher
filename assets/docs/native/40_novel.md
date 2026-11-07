# How to make a novel

A novel is a story in the **Novels** list of the Data menu, which the
player opens from the pause menu. The player picks it, answers the
question that follows with **Yes**, and NAMS plays your text file.

A novel needs an entity file that puts it into the Novels list, and a
text file with the story, one per language.

## How do I get my novel into the Novels list?

Create `entities\novel_<name>.toml`, where `<name>` is the name of your
novel (see [How to lay out your mod folder](02_mod_folder_layout.md)).
The name may contain `_`.

```toml
unlocked = true

[text.title]
_us = "<title>"

[text.description]
_us = "<description>"
```

| Field | Default | What it does |
|---|---|---|
| `unlocked` | `false` | `true` puts the novel into the list from the start. With `false` it stays hidden until you unlock it, see [How do I unlock the novel from my quest?](#how-do-i-unlock-the-novel-from-my-quest) |
| `[text.title]` | required | The title in the list and in the detail panel (the panel that shows the selected entry). The question asked before the novel starts is cut off after 63 characters, title included, so keep the title short. |
| `[text.description]` | empty | The text in the detail panel while the novel is selected. It is cut off after 510 characters. |
| `name` | `<name>` from the file name | The name of the novel. Write it here only when the name is made of digits alone; NAMS does not take such a name from the file name. |

Both texts take the language codes from
[How to translate your mod](12_translate.md).

The name may be at most 33 characters long; a character outside ASCII
counts as two to four. It must not be the file name of a novel that
ships with NieR:Automata. When two mods have a novel of the same name,
only one of them loads, so start the name with something unique to
your mod.

To check it, open the pause menu, go to the Data menu and pick
**Novels**. Your title is in the list with the "new" marker,
and your description shows in the detail panel when you select it.
**Novels** appears in the Data menu once any novel is unlocked, yours
included.
<!-- screenshot: Data menu → Novels with the custom novel selected -->

## Where does the story go?

The story is a text file in the `data\novel\` folder of your mod. NAMS
opens the file of the current language:

| Language | File |
|---|---|
| English (US) | `data\novel\<name>_ENG.txt` |
| German | `data\novel\<name>_GER.txt` |
| French | `data\novel\<name>_FRA.txt` |
| Spanish | `data\novel\<name>_ESP.txt` |
| Italian | `data\novel\<name>_ITA.txt` |
| Chinese (Simplified) | `data\novel\<name>_CHS.txt` |
| Chinese (Traditional) | `data\novel\<name>_CHT.txt` |
| Korean | `data\novel\<name>_KOR.txt` |
| Japanese, English (UK) and every other language | `data\novel\<name>.txt` |

NAMS does not fall back to another language. If you write one language
only, copy that file to every name in the table.

NAMS hands the whole file unchanged to NieR:Automata's novel player, so
the story is written in that player's own text format.

To test it, pick the novel and answer the question with **Yes**: NAMS
loads the file of the current language and starts
it.

## How do I unlock the novel from my quest?

Leave `unlocked = false`. NAMS creates a scenario flag (a named on/off
switch kept with the save, see
[Counters and switches for your quest](23_scenario_variables.md)) with
the same name as your novel. The novel shows up in the list once that
flag is on. Switch it on from your quest script like a flag you
declared yourself.

The "new" marker stays on the novel until the cursor has been on it and
moves on, or until the player opens it with **Yes**. NAMS keeps that in
a second flag, `<name>_read`. When you save, both flags go into the
NAMS save file, see [The NAMS save file](43_nams_save_file.md). Do not
declare flags with either name yourself: one of the two declarations
then fails to register.

## Something does not work

Errors are written to the NAMS log, see
[How to debug when something goes wrong](17_debugging.md).

**My novel is not in the list.** Look for a line that starts with
`[NAMS] Register failed for novel_bundle from mod '<mod id>': ` (or with
`[NAMS] hot register failed for novel_bundle '<name>': ` after you saved
the file while NieR:Automata runs) and ends with one of these:

- `novel '<name>' is a vanilla novel` – the name is the file name of a
  novel that ships with NieR:Automata. Pick another one.
- `novel '<name>' name is longer than 33 characters` – shorten the name.
- `novel '<name>' has no [text.title]` – add a `[text.title]` block.
- `novel '<name>': a scenario_flag of the same name already exists` or
  `novel '<name>': a scenario_flag named '<name>_read' already exists` –
  another mod or your own `scenario.toml` already declares that flag
  name.

Two other lines also keep a novel out:

- `[Native] <file> [novel_bundle]: parse novel_bundle: <reason>` – NAMS
  could not read the file; `<reason>` says why. A name made of digits
  alone without a `name` line ends here too.
- `[NAMS] Two mods try to use the same novel_bundle number '<name>'.` –
  another mod has a novel of the same name. The mod named after
  `• Skipped:` does not get its novel. Rename yours.

Without any of these lines, check that `novels` under `[content]` in
`nams.toml` is not `false` and `disable_content_features` is not `true`
(see [How to configure NAMS](13_nams_toml.md)). Then check `unlocked`,
or whether your script has switched the flag on yet.
