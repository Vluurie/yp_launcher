# The NAMS save file

NAMS keeps the mod content of a save in a file of its own. When you
save, NAMS writes the parts listed below into that file and takes the
mod content out of the NieR:Automata save; when you load, it puts the
mod content back.

## Where is the file?

In the folder NieR:Automata is started from:

```
nams\_internal\custom_save\slot_<number>.bin
```

There is one file per save slot. `<number>` is the slot's number.

## When does NAMS write and read it?

- **When you save:** before the slot is written, NAMS takes the mod
  content out of the save and writes it into `slot_<number>.bin`.
- **When you load:** once the slot has been read, NAMS puts the mod
  content back into it.

You never have to do anything for this.

## What does NAMS keep in it?

| Part | What NAMS moves into the file |
|---|---|
| Items | items from mods, with their count |
| Weapons | weapons from mods with their level and "new" marker, and which of them are equipped in either weapon set |
| Outfits | outfits and hair sprays from mods that 2B, 9S or A2 wear |
| Accessories | which accessories each character wears, the ones from mods included |
| Place | where you saved, when that was in a subphase from a mod |
| States | scenario flags and scenario variables from mods, and the progress of quests from mods |

NAMS plugins can keep data of their own in the same file; NAMS leaves
it there on every save.

**Place.** When you save in a subphase from a mod (see
[How to add a phase or a subphase](25_phases.md)), NAMS writes a place
outside the mod subphases into the NieR:Automata save, when it has one,
and keeps the real place in its file. Loading puts the real place back.
For what the save slot shows, see `save_name` and `save_image` on the
same page.

**States.** The scenario flags and scenario variables a mod declares
(see [Counters and switches for your quest](23_scenario_variables.md))
are moved here. This also covers the flags NAMS creates for
[intel entries](42_intel.md), [novels](40_novel.md) and
[archives](41_archive.md): which of them are unlocked and which the
player has already looked at.

## What happens when I remove a mod?

When you load, NAMS puts back only what an installed mod still defines.
Everything else stays in the file.

When you save again, NAMS keeps the content of the removed mod in the
file: its items, weapons, accessories, scenario flags, scenario
variables and quest progress. Install the mod again and they come back
on the next load.

Not kept are:

- outfits and hair sprays of the removed mod: the character wears what
  the NieR:Automata save holds,
- which weapon of the removed mod was equipped,
- the place, if the subphase is gone: you load at the place written
  into the NieR:Automata save.

## What happens when I switch NAMS content off?

With `disable_content_features = true` (see
[How to configure NAMS](13_nams_toml.md)), NAMS moves nothing into or
out of this file. A save then loads without the mod content of the
file, and the file stays as it is: switch the content on again, and the
next load puts the mod content back.

A single feature switched off under `[content]` works like removing
that kind of content from every mod, see
[What happens when I remove a mod?](#what-happens-when-i-remove-a-mod).

## What about `nams\custom_inventory.json`?

A slot without its own `slot_<number>.bin` loads its items, weapons,
outfits, hair sprays and accessories from `nams\custom_inventory.json`
when that file has an entry for the slot. The next save writes the slot
into its own `slot_<number>.bin`.

## How do I back up or move a save?

Copy `slot_<number>.bin` together with the NieR:Automata save of the
same slot, and keep the slot number the same.

NAMS pairs the two files only by the slot number. A save in a slot
without `slot_<number>.bin` loads without the mod content, unless
`nams\custom_inventory.json` has an entry for that slot. A save copied
into a slot that already has a `slot_<number>.bin` from another save
gets the mod content of that file.
