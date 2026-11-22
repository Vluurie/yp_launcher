# How to make voice lines

Voice lines come in two parts: a sound bank file (`.bnk`) per voice
language, which NAMS loads into Wwise, the sound library the banks are
built for, and `entities\voice_bank_<name>.toml`, which lists the
bank's events and how long each one is. An **event** is a sound in the
bank, found by its name.

Replace every `<…>` with your own value:

```toml
[clips]
"<event name>" = <seconds>
```

## Which name does the bank get?

The part of the file name after `voice_bank_` is the bank's name. A
quest names the bank with `voice_bank = "<name>"`, see
[How to make a quest](07_quest.md), and the bank file is
`vo_<name>.bnk`. A name made only of digits does not load; write
`name = "<name>"` into the file then.

## Where does the `.bnk` go?

`data\sound\<voice language>\vo_<name>.bnk` in your mod, where
`<voice language>` is the folder name of the voice language the player
has set. NAMS looks for the same path in NieR:Automata's own `data`
folder first, then in the `data` folders of the mods.

Voice banks are loaded per language: NAMS loads only the file of the
voice language that is set and does not fall back to another one. Ship
the bank for every voice language, the same file if you recorded only
one.

## When is the bank loaded?

NAMS loads the bank when a quest that names it reaches `REQ` or a
`RUN_N` step, and unloads it when that quest reaches `CLR`, `DONE`,
`END`, `FIN` or `BREAK`. A bank that no quest names is never loaded.
While the bank is loaded, NAMS finds each event of `[clips]` by its name
and reports the length given there.

## Which fields does the file have?

| Field | Default | What it does |
|---|---|---|
| `name` | text after `voice_bank_` in the file name | The bank's name. |
| `[clips]` | empty | Event name = length in seconds. Each key must be the name of an event in the bank. |

## Why is there no voice?

**Nothing in the log.** The `.bnk` is not at
`data\sound\<voice language>\vo_<name>.bnk` for the voice language that
is set, or no quest that names the bank is at `REQ` or a `RUN_N` step.

```
[CustomAudio] Failed to read "<path>": <reason>
```

The file was found but could not be read.

```
[CustomAudio] LoadBankMemoryView failed: result=<n>
```

Wwise did not accept the file as a sound bank.

```
[Native] <file> [voice_bank_bundle]: parse voice_bank_bundle: <reason>
```

A `[clips]` value is not a number, or the name is only digits.

```
[NAMS] Skipping quest_bundle '<id>' from mod '<mod id>' because the voice_bank '<name>' it depends on (from mod '<unstaged>') is not loaded.
```

No loaded mod has a bank with this name, so the quest is skipped.

**Banks and their quests are missing and the log is clean.**
`custom_voice_lines` is `false` in the `[content]` table of
[nams.toml](13_nams_toml.md). It skips the banks and every quest that
names one.
