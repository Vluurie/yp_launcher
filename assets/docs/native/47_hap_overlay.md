# How to add to the HAP content of a room, phase or quest

A room, a phase and a quest can each bring a **HAP container**: the
archive with their groups, their lists and the compiled scripts. Groups
and lists are described on the [HAP groups page](31_hap_groups.md),
ScriptActions on the [ruby patches page](24_ruby_patches.md).

A `[[hap_overlay]]` adds your own groups, lists and scripts to one
container. A `[[hap_patch]]` changes one list that is already in it: it
puts actions in or takes actions out. NAMS builds the changed container
when it is loaded, and keeps it until your entity files change.

Both go into a file in your mod's `entities` folder, see
[How to lay out your mod folder](02_mod_folder_layout.md).

## Which container does a block change?

Every block names exactly one:

| Field | Container |
|---|---|
| `room` | the room's, like `0x110` for the Resistance Camp |
| `phase` | the phase's, like `0x100` for `p100` |
| `quest` | the quest's, the number after the `q`: `20` for `q020` |
| `global = true` | `core_hap.pak`, the container that belongs to no room, phase or quest |

## `[[hap_overlay]]`

| Field | What it does |
|---|---|
| `name` | Required. Your name for the block, shown in the log. Each name can be used once. |
| `groups` | Files with `<group>` blocks inside `<root>`. Every `<group>` block is added to the container's groups. |
| `lists` | Files with one list each, inside `<root>`. Each list needs its `<group>`. |
| `scripts` | Entries `{ id = …, file = "…" }`: a compiled ruby script, stored under `id`, the id a ScriptAction's `<script>` names. NAMS checks that the file starts with `RITE`. |
| `parent` | A group id. Every group from `groups` whose `<parent>` is not another group of the same block goes under it. Without `parent` they go under the top group of the room, phase or quest. With `global = true` and `groups`, `parent` is required. |

A block needs at least one of `groups`, `lists` and `scripts`. File
paths start at your mod folder.

A `<parent>` that names a group outside the block is replaced as well.
To put your groups under an existing group, write its id in `parent`.

## `[[hap_patch]]`

| Field | What it does |
|---|---|
| `name` | Required. Your name for the block, shown in the log. Each name can be used once. |
| `list` | Required. The id of the list to change. In the YP Devkit, select the list in the HAP tree: **ID**. |
| `action` | A file with the `<action>` blocks to put in, in that order. |
| `after` | Id of the action in the list that the new actions go after. In the YP Devkit: **Action ID**. |
| `before` | Id of the action in the list that the new actions go before. |
| `remove` | Ids of the actions to take out of the list. |

With `action`, give exactly one of `after` and `before`. A patch needs
`action`, `remove` or both. NAMS takes the `remove` actions out first,
then puts the new ones in.

If the list, an id in `remove`, or the `after` or `before` action is not
in the container, NAMS skips the whole patch.

## How do I see that it worked?

When NAMS builds a container with your content, the log shows one line
for it:

```
[HapOverlay] room 0x110: … group(s), … list(s), … patch(es), … script(s)
```

The container is named `room 0x110`, `phase 0x100`, `quest 20` or
`global`.

## Why did nothing change?

**No `[HapOverlay]` line for that container.** The **Script changes**
switch (`script_overlays` in `nams.toml`; in the launcher, tab **NAMS**,
card **CONTENT FEATURES**, group **Scripts**) is off, and turning it on
needs a restart. Or the container has not been loaded yet.

**A block was refused when the mod loaded.** The log names the block
and the reason:

| The log says | Fix |
|---|---|
| `needs one of room, phase, quest or global = true` | give the block one container |
| `room, phase, quest and global exclude each other` | keep only one of them |
| `needs groups, lists or scripts` | add one of them |
| `groups under global need a parent` | add `parent` |
| `needs an id` | give the script its `id` |
| `has no <root> element` | wrap the file in `<root>` |
| `is not compiled mruby (no RITE header)` | point `file` at the compiled script |
| `needs a list id` | add `list` |
| `needs an action to put in or ids to remove` | add `action` or `remove` |
| `after and before need an action` | add `action`, or remove `after` and `before` |
| `an action needs exactly one of after or before` | keep one of them |
| `has no <action> block` | put the action inside `<action>` |
| a file path, then an error while reading it | check the path, it starts at your mod folder |
| `Two mods try to use the same hap_overlay number '…'` (or `hap_patch`) | give the block a name of its own |

A block that cannot be read stops every block of the same kind in that
file, so fix it before you look for other errors.

**The container was loaded, but a part is missing.**

| The log says | What happened |
|---|---|
| `[HapOverlay] hap_patch '…': … has no list 0x…` | the list is not in that container; check `list` and the container field |
| `[HapOverlay] hap_patch '…': list has no action 0x…` | an id in `remove`, `after` or `before` is not in the list |
| `a list has no <group>` | add `<group>` to the list; until then no block goes into that container |
| `container has no _hap.pak` | that room, phase or quest has no HAP container |

## Changing it while playing

A changed TOML file takes effect the next time its container is loaded.
NAMS reads the XML and script files only together with their TOML file,
so after editing one of them, save the TOML file again or restart.
