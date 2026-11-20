# How to switch vanilla content off in your own phase

Your phase starts and the area still runs its vanilla content. That
content runs in lists, and every list sits in a **group**, a switch
that turns all of its lists on and off. A `[[hap_group]]` block turns
groups off while your phase is on. Put it in a file in your mod's
`entities` folder, like `hap_groups.toml`, see
[How to lay out your mod folder](02_mod_folder_layout.md).

```toml
[[hap_group]]
name   = "bunker_quiet_in_my_phase"
groups = [0xfcb689f6]   # vanilla group "disabled in bunker"
phase  = 0x100          # phase p100
```

While phase `p100` is on, NAMS switches that group off. When the phase
ends, NAMS stops switching it off.

## `[[hap_group]]`

### `name`
A name you make up, so you can tell your rules apart. It shows up in
the log. Every rule needs its own: a second rule with a name that is
already taken is skipped, and the log says
`Two mods try to use the same hap_group number '…'`, or
`lists the same hap_group number '…' twice` inside one mod.

### `groups`
The groups to switch. In the YP Devkit, open the HAP tree and select a
group: the inspector shows its **ID** with a copy button.

```toml
groups = [0xfcb689f6, 0x3b7fcae0]
```

They all get the same rule, so one rule can cover as many groups as it
takes.

### `phase`
The phase the groups are off in, as a number: `0x100` is `p100`. They
are off while that phase is on.

### `subphases`
Narrows `phase` down to some of its subphases. The groups are then off
only while one of these is the current subphase.

```toml
phase     = 0x100
subphases = ["90_00_AB_Ruined_City"]
```

`subphases` only works together with `phase`.

### `comment`
A note to yourself.

## Switching off one list, one action or one placed object

A group switches everything inside it. To switch off less, name the
part instead of the group:

```toml
[[hap_group]]
name    = "no_object_arrangement_in_my_phase"
actions = [0xcfdb7c41]   # vanilla action "Object arrangement"
phase   = 0x100
```

| Field | What goes off | Where the YP Devkit shows the id |
|---|---|---|
| `haps` | a whole list | select the list in the HAP tree: **ID** |
| `actions` | one action of a list | select the action: **Action ID** |
| `action_layouts` | one placed object of an action that places objects | select that action: each entry of its layouts shows its `0x…` id with a copy button |

A list in `haps` is switched off like a disabled list. An action in
`actions` is skipped like a disabled action, and the rest of its list
goes on without it. A placed object in `action_layouts` is removed from
the world and stays removed while the rule holds.

Without `phase` these stay off all the time. With `phase`, and
`subphases` if you give them, they are off only then.

## `[[hap_group.tokens]]`

Conditions for the groups in `groups`, added to the ones each group
already has. One block each, as many as you like. A group that already
has the same condition does not get it twice. They only work on groups,
not on `haps`, `actions` or `action_layouts`.

```toml
[[hap_group.tokens]]
kind = "GlobalPhase"
id   = 0x100

[[hap_group.tokens]]
kind = "SubPhase"
name = "90_00_AB_Ruined_City"
```

| `kind` | What you write |
|---|---|
| `GlobalPhase` | `id`, a phase number |
| `SubPhase` | `name`, a subphase name |
| `GlobalRoom` | `id`, a room number, like `0x110` for the Resistance Camp |
| `@SceneState` | `name`, a scene state |
| `Quest` | `id` |
| `Grid` | `id`, a grid |
| `Hacking` | `id`, `0` or `1` |
| `Phase` | `id`, a phase number |
| `Room` | `id`, a room number |

`Global`, `Event`, `Scenario`, `Debug` and `Test` are accepted too and
take an `id`.

`@SceneState` and `SubPhase` take a `name`, or the `id` it hashes to.
The rest take an `id`, and a `name` there without an `id` is refused
when the mod loads.

## Turning a group around with `invert`

`invert = true` adds `0x10` to the attribute of each group, unless the
group has it already. The YP Devkit shows the group's attribute as
**Attribute** in the group inspector. To switch a group off in your
phase, use `phase` instead; it changes nothing outside that phase.

`invert` only works on `groups`.

## When it takes effect

NAMS applies the rules to every loaded group each time a scene starts
playing and each time a rule changes. Rules with a `phase` are applied
again each time the phase or subphase changes.

A rule you add or change while playing takes effect without a restart,
also on the groups that are already loaded. `haps`, `actions` and
`action_layouts` follow your change at once.

Nothing takes added conditions or `invert` back off a group that is
already loaded, also when you take them out of the rule; a restart
clears them. A rule you delete from a file that still exists stays
active until you delete the file or restart.

## When nothing changes

A rule with `tokens` or `invert` writes one line per group it changed.
For a rule named `bunker_quiet_in_my_phase` that adds one token to
group `0xfcb689f6`:

```
[HapGroup] 'bunker_quiet_in_my_phase' 0xfcb689f6: 1/1 token(s) added
```

`, inverted` at the end means `invert` was applied too. Rules with only
`phase`, `haps`, `actions` or `action_layouts` write no such line.

A rule NAMS cannot use is refused when the mod loads, and the log says
why:

| The log says | Fix |
|---|---|
| `does nothing for its groups: give it tokens, invert = true, or a phase` | give the rule a `phase`, `tokens` or `invert` |
| `needs at least one group, hap, action or action layout` | fill in `groups`, `haps`, `actions` or `action_layouts` |
| `subphases need a phase` | add `phase` |
| `subphase names cannot be empty` | remove the empty name |
| `tokens and invert only apply to groups` | move them to a rule with `groups` |
| `takes a number, not a name; write id = ...` | write `id` for that kind |
| `needs a name or an id` / `needs an id` | give the token its `name` or `id` |
| `is not a token kind` | use a `kind` from the table above |
| `is 0, which is not an id` | copy the id again from the YP Devkit |
| `hap_group needs a name` | give the rule a `name` |

While playing, `the group refused a … token` means the group could not
take the condition.

No line and no error at all:

- The **Script groups** switch is off (`script_groups` in `nams.toml`;
  in the launcher, tab **NAMS**, card **CONTENT FEATURES**, group
  **Scripts**). Turning it on needs a restart.
- A group id is wrong. Check it in the HAP tree.
- The mod is not loaded. Check the log at startup.

## Related

- [How to add a phase or a subphase](25_phases.md)
- [How to hide or show parts of the world](28_mesh_visibility.md)
