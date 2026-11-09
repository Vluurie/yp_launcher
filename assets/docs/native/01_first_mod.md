# How to make your first mod in 5 minutes

Add one item and check that NAMS loads it.

<!-- yp:first-mod -->

## Step 1 - Make a folder

In your game directory:

```
NieRAutomata\nams\mods\my_first_mod\
```

## Step 2 - Create `mod.toml`

```toml
id = "my_first_mod"
display_name = "My First Mod"
author = "Your Name"
version = "1.0.0"
```

What each field does is in
[how to lay out your mod folder](02_mod_folder_layout.md).

## Step 3 - Make an `entities\` folder

```
my_first_mod\
├── mod.toml
└── entities\
```

## Step 4 - Add an item

Inside `entities\`, create `item_9999.toml`:

```toml
name = "item_test_pebble"   # the item's internal name
id_type = 0                 # 0 = consumables, materials and key items
buy = -1
sell = -1
no_trash = true

[auto_give]
quantity = 1                # how many the player is handed

[text.name]
_us = "Test Pebble"

[text.help]
_us = "A small stone. Proof your mod loaded."
```

The number in the filename (`9999`) is the item's id - you don't write
it inside the file. If two mods claim
the same number, NAMS keeps one and skips the other, with a log line
saying so.

`_us` means English. The other language keys are `_de`,
`_fr`, `_es`, `_it`, `_cn`, `_tw`, `_kr`, and the empty key `""` for
Japanese. NAMS shows the key of the current language; only Chinese and
Korean fall back to `_us` when their own key is missing. Every field is
explained in [how to make an item](03_item.md).

## Step 5 - Load it

Start NieR:Automata and load your save. `[auto_give]` hands the item
over when a save finishes loading, so the Test Pebble appears once you
are in your save, not on the title screen.

Then check the log for this line:

```
[Native] my_first_mod: staged 1 entities
```

The log is `logs\nams.log` in the folder of `NAMS.exe`. With the
launcher on Windows that is
`data\flutter_assets\assets\bins\logs\nams.log` inside the launcher
folder - or press **LOGS** in the launcher to open the log viewer. If
the number is 0 or the line is missing, the file was not accepted; the
`[Native]` lines right above say why. More in
[how to debug](17_debugging.md).

## Changing things afterwards

Entity files are read again the moment you save them while
NieR:Automata runs (log: `[NAMS] hot register ...`) - items included,
so you can edit the pebble's text and watch it change. Some kinds need
a restart; the [cheat sheet](18_cheat_sheet.md) marks them. `.json`
entity files always need one. If a change does not show up, restart.
