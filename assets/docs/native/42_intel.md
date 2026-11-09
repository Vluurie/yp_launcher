# How to show your enemy in Unit Data

Unit Data is the enemy list in the data menu of the pause menu. The list
shows categories; the player opens a category and picks an entry, and
its title and text appear in the detail panel. Confirming the entry
opens the 3D view of the model.

Your entries go into categories of your own, listed after the built-in
ones.

## How do I get my enemy into the Unit Data list?

Add an `[intel]` block to the entity file of your object,
`entities\entity_<name>.toml` (see
[How to add a new game object](34_custom_entities.md)). The object has
to be a new one: an entity file for an existing object id is refused.
Replace each `<…>` with your own text.

```toml
[intel]
category = "<category name>"
unlocked = true

[intel.text.category]
_us = "<header text of the category>"

[intel.text.title]
_us = "<entry text in the list>"

[intel.text.description]
_us = "<text in the detail panel>"
```

- **`category`** (required): the name of the category this entry is
  listed under. Every entry with the same `category` lands under the
  same header.
- **`unlocked`** (default `false`): `true` puts the entry into its
  category from the start. With `false` it stays hidden until it unlocks,
  see [How does my entry get unlocked?](#how-does-my-entry-get-unlocked)
- **`die`**, **`talk`** (default `false`) and **`quest`** (no default):
  how the entry unlocks, see
  [How does my entry get unlocked?](#how-does-my-entry-get-unlocked)
- **`set_type`** and **`set_flag`** (default `0`): two values an object
  is spawned with. The model in the 3D view is spawned with these.
  `die` and `talk` compare them against the object that triggers the
  check.
- **`[intel.text.category]`**: the header text of the category. An entry
  without it is refused unless an entry of the same category that has it
  was loaded before. Put it into every entry of the category, then it
  does not depend on load order.
- **`[intel.text.title]`** (required): the entry text in the list.
- **`[intel.text.description]`**: the text in the detail panel.

All texts take the language keys from
[How to translate your mod](12_translate.md).

A category without any unlocked entry is not listed. Your entries count
towards the completion percentage of Unit Data.

To check it, open Unit Data in the pause menu. Your
category is at the end of the list, with the "new" marker while one of
its unlocked entries is unread. Confirm it, and your entry unfolds below
it; select the entry and the detail panel shows its title and
description.

## How does the 3D view show my model?

Confirm your entry. NAMS loads the files of your object and shows its
model in the 3D view. When the description is longer than the detail
panel shows, confirming opens the description for reading instead.

The 3D view runs the model as a second object. Its name is your `<name>`
with the first two letters replaced by `um`, and it gets the same
template, lane and class as your object. NAMS creates that object for
you; no other mod and no built-in object may use that name.

## How does my entry get unlocked?

Leave `unlocked = false` and set when the entry unlocks:

- **`die = true`**: the entry unlocks through the `die` check, see
  below.
- **`talk = true`**: the entry unlocks through the `talk` check, see
  below.
- **`quest = <id>`**: the entry unlocks when that quest ends.

An entry uses either `die` or `talk`, not both. `quest` works on its own
or together with one of them.

### Which object unlocks a `die` entry?

The `die` check is handed an object with its set type and set flag. It
unlocks your entry when all of these hold:

- the object is your object,
- its set type equals `set_type`,
- its set flag has every bit of `set_flag` set. `set_flag = 0` accepts
  any set flag.

### Which object unlocks a `talk` entry?

The `talk` check is handed an object with its set type and set flag. It
unlocks the first of your entries, in load order, for which all of these
hold:

- the object is your object,
- its set type equals `set_type`,
- its set flag shares at least one bit with `set_flag`. `set_flag = 0`
  accepts any set flag.

### When does `quest` unlock the entry?

`quest` takes the `id` of your quest from its `quest_<id>.toml` (see
[How to make a quest](07_quest.md)) or the id of a built-in quest.

For your own quest the entry unlocks when the quest switches to its
`DONE` or `FIN` state, so to `<quest name>/DONE` or `<quest name>/FIN`.
For a built-in quest the entry unlocks together with the built-in Unit
Data entries of that quest.

### What happens when the entry unlocks?

- The entry and its category get the "new" marker. Confirming the entry
  marks it as read.
- NAMS sends the signal `EM_LIST_UNLOCK` to the global group.

None of your entries unlocks while the current phase group is `0x400`
or while the player stands in one of the rooms `0x5a8` to `0x5aa`.

NAMS keeps the unlock in a flag with the same name as your object and
the read state in a second flag, `<name>_read`. Both go into the NAMS
save file, not into the base save, see
[The NAMS save file](43_nams_save_file.md). Do not declare flags with
either name yourself.

## What if my entry is not in the list?

Errors are written to the NAMS log, see
[How to debug when something goes wrong](17_debugging.md). The log has a
line starting with
`[NAMS] Register failed for entity_bundle from mod '<mod id>':`,
followed by one of these:

| Message | Fix |
|---|---|
| `entity '<name>': [intel] has no category` | Add `category`. |
| `entity '<name>': [intel] has no [intel.text.title]` | Add an `[intel.text.title]` block. |
| `entity '<name>': [intel] can use only one of die and talk` | Remove one of the two. |
| `entity '<name>': [intel] quest is negative` | Use the `id` of a quest. |
| `entity '<name>': intel category '<category>' has no [intel.text.category] yet` | No entry of that category with an `[intel.text.category]` block was loaded before this one. Add the block to this entry. |
| `entity '<name>': a scenario_flag of the same name already exists` or `entity '<name>': a scenario_flag named '<name>_read' already exists` | Another entry already declares a flag with that name. |
| `entity '<name>': its viewer object 0x<id> is a vanilla object` or `entity '<name>': its viewer object 0x<id> is already registered` | The `um` name of your object is taken, see [How does the 3D view show my model?](#how-does-the-3d-view-show-my-model) Give your object a different name. |
| `entity '<name>': 0x<id> is a vanilla object id` | The object is not a new one, see [How to add a new game object](34_custom_entities.md). |

Without any of these lines, check `unlocked`, and whether the object or
quest that should unlock the entry matches `die`, `talk`, `quest`,
`set_type` and `set_flag`, see
[How does my entry get unlocked?](#how-does-my-entry-get-unlocked)
