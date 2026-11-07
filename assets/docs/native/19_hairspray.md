# How to make a hairspray

A hairspray entry marks an inventory item as a hair spray, so the
inventory lets 2B, 9S and A2 use it.

Create a container file such as `entities\hairspray.toml` with one
`[[hairspray]]` block per hair spray, and make the matching item as in
[How to make an item](03_item.md).

## What happens when the player uses it?

Using the item sets the hair spray of the character in control to `0`,
the number of the base White Hair spray (item `880`). The item is not
used up.

The colour values of the entry are not applied to the hair.

## When it does not load

**`` [Native] <file> [hairspray]: [hairspray][<n>] parse hairspray: missing field `<field>` ``**
Block number `<n>` (counting from `0`) lacks a required field.

**`[NAMS] Skipping hairspray '<id>' from mod '<mod>' because the item '<item_id>' it depends on (from mod '<unstaged>') is not loaded.`**
No loaded mod provides `item_<item_id>.toml`, and `item_id` is not a
base item.

**`[NAMS] Two mods try to use the same hairspray number '<id>'.`** or
**`[NAMS] Mod '<mod>' lists the same hairspray number '<id>' twice.`**
Give every hair spray its own `id`.

`hairspray_<id>.toml` is not a bundle shortcut because there is no
`hairspray_bundle` schema. Use a container file such as
`hairspray.toml` with `[[hairspray]]`. Without that table, every field
of the file is read as a kind of its own and fails with
`[Native] <file> [<field>]: unknown entity kind '<field>'`.

## Fields

| Field | Type | Required | Notes |
|---|---|---:|---|
| `id` | whole number from `0` to `255` | yes | Number of the hair spray. `0` to `18` are the base hair sprays. |
| `item_id` | whole number | yes | Item used from the inventory. |
| `color_r` | number | yes | Red channel, stored with the entry. |
| `color_g` | number | yes | Green channel, stored with the entry. |
| `color_b` | number | yes | Blue channel, stored with the entry. |
| `color_a` | number | yes | Alpha channel, stored with the entry. |

An outfit names the meshes a hair spray colours with
`hairspray_meshes`, see [How to make an outfit](05_outfit.md). Without
it, `Hair` is coloured.
