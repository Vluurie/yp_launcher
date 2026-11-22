# Counters and switches for your quest

Scene states can only say yes or no. When your quest needs to count
something or remember several things at once, use a **variable** (a
number) or a **flag** (on/off).

You declare them in a `scenario.toml` next to your quest, and your ruby
script reads and writes them by name. NAMS keeps their values with the
save slot in its own save file, see
[the NAMS save file](43_nams_save_file.md).

```toml
[[scenario_variable]]
name = "var_myquest_kills"
value = 0
comment = "Machines killed for the hunt quest"

[[scenario_flag]]
name = "flag_myquest_met_hermit"
comment = "Player has talked to the hermit"
```

Start every name with something unique to your quest, like the quest
name. Names are shared by every mod, so `kills` would clash with
another mod, while `var_myquest_kills` won't.

The filename must not start with one of the prefixes that name an
entity kind, like `quest_` or `item_`: such a file is read as that kind,
see [How to lay out your mod folder](02_mod_folder_layout.md).

### `[[scenario_variable]]`

| Field | Default | What it does |
|---|---|---|
| `name` | required | What your script calls it. |
| `value` | `0` | What it starts at. |
| `comment` | none | A note to yourself. |
| `var_type` | `"int"` | The type NAMS writes for the variable. |
| `path` | none | Binds the variable to a number NAMS keeps. The only one is `player.flashlight.charge`. |
| `max` | none | With `path`: the highest value NAMS writes into the variable. |

With `path = "player.flashlight.charge"`, the rules of used-up items on
`player.flashlight.charge` change this variable, see
[How to make special effects](11_special_effects.md). Only one
variable can hold a path.

### `[[scenario_flag]]`

`name`, plus an optional `comment`. Add `value = true` if the flag
should start switched on.

A flag can also carry effect rules that hold while it is on, see
[How to make special effects](11_special_effects.md).

## Seeing them while playing

In the YP Devkit, open the HAP tree and press **View Scenario Flags &
Variables**. The tabs **Flags** and **Variables** list every flag and
variable with its current value. **Only entries declared by mods** hides
the rest, and yours carry the mark **Declared by a mod**.

A flag has a switch and a variable a number field, so you can change
them while you test. **Reset to start value** puts one back to the
value from your file.

## When something is wrong

The log says why a block is refused:

| The log says | Fix |
|---|---|
| `Two mods try to use the same scenario_variable number 'var_myquest_kills'` | give the variable a name of its own |
| `lists the same scenario_variable number 'var_myquest_kills' twice` | remove the second block |
| `'…' is not a number NAMS keeps, expected one of ["player.flashlight.charge"]` | fix `path` |
| `'player.flashlight.charge' is already kept in '…'` | keep the path in one variable only |

Two flags with the same name give the first two lines with
`scenario_flag`.

A changed block takes effect after a restart. Saving the file while
playing logs `duplicate scenario_variable` or `duplicate
scenario_flag` and keeps the old block.
