# Making a change

## What you need first

NieR:Automata has no modding support of its own. It reads its content out of sealed archives
and offers no way in.

**NAMS** is the runtime that adds modding support. A session launched through NAMS runs the game with a
virtual file system in front of it, so files sitting in a mod folder are found in place of
the packed ones. Nothing on disk is modified, and a normal Steam launch stays vanilla.

**YoRHa Protocol** is what you install, and it brings NAMS with it. It is also where the
tools live: the mod manager, the launcher, and the in-game workspace that opens with **F1**,
which holds the quest editor.

```
   Steam                                NAMS
   ┌────────────────────────┐           ┌────────────────────────┐
   │  the game as shipped   │           │  the game plus the     │
   │  no mods               │           │  mods you installed    │
   └────────────────────────┘           └────────────────────────┘
```

Both come from the [YoRHa Protocol download
page](https://www.nexusmods.com/nierautomata/mods/1330). Without them, a quest you author is
a folder of files nothing will ever read.

Older mod pages, written before NAMS existed, tell you to copy files into the game folder or
`/data` directly. Do not follow those steps for content you install through the launcher.

## What a HAP is on disk

An XML file. Everything from the previous pages (the list, its actions and the group it
sits in) is written out as tags:

```xml
<action>
    <name>place the robot</name>
    <id>0x51c116d9</id>
    <attribute>0x8</attribute>
    ...
</action>
```

The original game does not ship these as loose files, and it does not ship them as XML
either. A quest lives in its own `.dat` archive named after it, such as `q900.dat`, and
sits several layers down:

```
   data.cpk
    └─ quest/
        └─ q900.dat
            ├─ q900_hap.pak        the lists and their actions
            ├─ q900_grp.pak        the group tree
            └─ q900_scp.bin        the compiled script
```

Inside those `.pak` files the documents are **YAX**, for Yet Another XML Format. It is XML
packed down to a binary, so opening one in a text editor shows nothing readable. Converting
YAX back to XML is what any of these tools does first.

**You do not have to unpack any of it to work on a quest.** The editor does the whole chain
in memory: it opens the archive, decodes the YAX to XML, hands you that to edit, and builds
everything back up on its own. A finished `.dat` is only written when you save. Unpacking by
hand is for reading the original content outside the editor, not a step on the way to
changing it.

Reading the original files is still worth doing, because they are the best available answer
to "how is this actually used". They are not an answer to "what does this do". A setting that
appears in a thousand files still does not explain itself, and a field's name is not a
description of it: more than one of them does something other than its name suggests.

## Where your own changes go

A mod is a folder under `nams/mods`, and NAMS mounts the `data` inside it over the game's
own files while the game runs.

```
   NieRAutomata/
    └─ nams/
        └─ mods/
            └─ my_quest/
                ├─ mod.toml               what the mod is called
                ├─ data/                  mounted by NAMS over the game's files
                │   └─ quest/
                │       └─ q899.dat       the archive from the previous section
                └─ entities/
                    └─ quest_899.toml     registers the quest
```

Only `data` is mounted, and its layout mirrors the game's own, so `data/quest/q899.dat`
stands in for the `q899.dat` the game would otherwise read out of its `.cpk`.

Everything beside it is read by NAMS rather than by the game. `mod.toml` names the mod, and
the files under `entities` register content: a quest gets its name, its actor and one line of
text per stage from `quest_<id>.toml`. Items, mail, rewards and voice banks are registered
the same way, each as its own file.

Those files have their own documentation, one page per kind, with every field:

| File | Page |
|---|---|
| `mod.toml` and the folder layout | [mod folder layout](../native/02_mod_folder_layout.md) |
| `quest_<id>.toml` | [quest](../native/07_quest.md) |
| `item_<id>.toml` | [item](../native/03_item.md) |
| `reward_<id>.toml` | [reward](../native/09_reward.md) |
| `mail_<id>.toml` | [mail](../native/08_mail.md) |
| voice lines | [voice lines](../native/10_voice_lines.md) |

The quest file is the one to read first: registering a quest is also what creates the scene
states it uses. Its `name` is their prefix, so a quest named `q899` gets `q899/REQ`,
`q899/CLR`, `q899/DONE`, `q899/BREAK`, one `q899/RUN_N` per `[run.N]` text you write, and
the same for `[fail.N]`. Nothing else needs declaring, and a state outside that set does
nothing.

A larger mod fills out the same shape. The published
[UNREGISTERED](https://www.nexusmods.com/nierautomata/mods/1963) quest carries `misctex` and
`sound` next to `quest` under `data`, a `cutscenes` folder, and eight files under `entities`
for its items, mail, reward and voice bank.

Because each mod keeps to its own folder, two of them can both add quests without either
knowing about the other, and removing one is deleting its folder.

## The editor

The in-game workspace that opens with F1 has a quest editor in it. It shows the group tree
on one side and the fields of whatever you select on the other, and it fills in the ids
discussed on the [HAP page](01_hap.md) for you.

Pointing one action at another is done by picking the target from a list; the number behind
it is never typed by hand.

For a quest built from nothing there is also a starting point that writes a working quest to
disk in one step: a group, a list, an action that places an entity, and a script with one
line of dialogue. Changing that is a great deal easier than assembling the first one
yourself.

If you would rather work outside the game, [F-SERVO](https://github.com/ArthurHeitmann/F-SERVO)
is the established tool for it. It opens `.dat`, `.pak` and `.yax`, edits quest XML and the
compiled Ruby scripts that go with them, and covers a great deal besides: textures, audio,
UI text, save files. The XML is the same either way; only the route to it differs.

## Getting a change into the game

Quests can be reloaded while the game runs, so the loop is short:

```
   change a field
        │
        ▼
   reload
        │
        ▼
   look at it in game
```

What you cannot do is verify a quest by reading it. Put the change in, then go and look.

## When nothing happens

Check these first.

**The session is not a NAMS one.** A quest launched through Steam directly does not exist:
its folder is never mounted and nothing registers it.

**The group is off.** Nothing runs when its switch is off, and no error says so. Check the
scene state the group waits for; it can also be waiting for one to be **absent**.

**Waiting for the wrong thing.** A [ConditionBlock](ConditionBlock.md) watching for `DONE`
on something that never dies waits forever. See
[EntityLayoutAction](EntityLayoutAction.md).

**A misspelled command.** [SendCommand](SendCommand.md) reports success for a name nobody
recognises, because an unknown name is treated as a state rather than an error.

**Everything happened at once.** No line was marked blocking, so the whole list ran in a
single instant. See [attribute](02_action.md).

---

Back to [the list of actions](README.md).
