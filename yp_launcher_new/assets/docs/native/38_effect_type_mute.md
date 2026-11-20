# How to switch an effect type off

An effect is built from records, and every record has a class: a number
that says what kind of part it makes. That number is the **type**.
`[[effect_type_mute]]` switches one type off: while the rule applies, no
part of that type is created, whichever effect asks for one.

## The short version

The whole rule, in any TOML file of your mod's `entities` folder:

```toml
[[effect_type_mute]]
name = "no_class_18"
type = 18
```

Save, and no new part of type 18 is created. One that is on screen at
that moment is not removed.

## Finding the number

Open the effect in the **EFFECT EDITOR**. Every record shows its class
as `<group> · class <number>`, and that number is the type. The editor
names class 18 **Lens flare**.

These types can be switched off:

```
0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 23 24 25 26 27
28 29 30 31 32 33 34 35 36 37 38 39 40 41 42 43 44 45 47 48 49 50 51 52
53 55 70 71 80 81 90
```

Any other number is refused when the file loads, and the log prints this
same list.

## `[[effect_type_mute]]`

| Field | Default | What it does |
|---|---|---|
| `name` | needed | A unique name for the rule. |
| `type` | needed | The type number from the list above. |
| `room` | every room | Only while the player is in this room. |
| `phase` | every phase | Only in this story phase, as a number: `0x200` for `p200`. |
| `subphases` | the whole phase | Only in these subphases. Needs a `phase`. |
| `not_subphases` | none | Not in these subphases. Needs a `phase`. |

```toml
[[effect_type_mute]]
name      = "no_class_18"
type      = 18
room      = 0x110
phase     = 0x200
subphases = ["170_00_A_Resistance_Camp_2B"]
```

A rule with only `name` and `type` applies everywhere, always.

## Editing while you play

Save the TOML and the rule is live. Nothing has to reload, because the
switch is checked every time a part of that type would be created.

## When the effect is still there

- The log refused the rule:
  `[NAMS] Register failed for effect_type_mute from mod '<mod id>': effect_type_mute '<name>': type <number> cannot be switched off, known types are [<the list above>]`.
  Pick a number from the list.
- `room` or `phase` do not match where you stand. Take them out for a
  moment to check whether the rule works at all.
- The effect is made of other types. Look at the classes of its records
  in the **EFFECT EDITOR**.
- The part was created before the rule applied. It is not removed.
- `effect_type_mute = false` under `[content]` in `nams.toml` switches
  every rule off, see [How to configure NAMS](13_nams_toml.md).
