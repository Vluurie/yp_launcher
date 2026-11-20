# How to make a reward

`entities\reward_<id>.toml`. A reward is gold, experience points (exp)
and items under one id. When reward `<id>` is requested, NAMS adds its
items, then its gold and exp. If a vanilla reward has the same id, NAMS
hands out yours instead.

Replace every `<…>` with your own value:

```toml
gold = <amount>
exp = <amount>

[[items]]
item_id = 530
quantity = <count>

[[items]]
item_id = 0
quantity = <count>
```

530 is Rusted Clump and 0 is Small Recovery, see
[vanilla item ids](resources/vanilla_item_ids.md).

## How does a quest give it?

Write `reward_id = <id>` in the quest file, see
[How to make a quest](07_quest.md). Several quests may point at the
same reward. Nothing pays the reward out when a quest finishes; it is
paid only when reward `<id>` is requested. A quest whose reward no
loaded mod has is skipped.

## Which fields does a reward have?

| Field | Default | What it does |
|---|---|---|
| `id` | number in the file name | The reward's id. |
| `gold` | `0` | Gold added. Only a value above 0 is added. |
| `exp` | `0` | Exp added. Only a value above 0 is added. |
| `[[items]]` | none | One block per item, with `item_id` and `quantity`, both required. |

Each unit of `quantity` takes one of 20 item slots, so a reward hands
out at most 20 items in total; the rest is left out. List each
`item_id` once. `item_id` must be a vanilla item or an item from a
loaded mod.

## Why is my reward missing?

```
[NAMS] Skipping reward_bundle '<id>' from mod '<mod id>' because the item '<item id>' it depends on (from mod '<unstaged>') is not loaded.
```

The item is neither a vanilla item nor an item from a loaded mod.
Quests that point at this reward are skipped too.

```
[NAMS] Register failed for reward_bundle from mod '<mod id>': duplicate reward_item for (<id>, <item id>)
```

The same `item_id` is listed twice. That block and the ones after it
are not registered; put the total into one block.

```
[NAMS] Two mods try to use the same reward_bundle number '<id>'.
```

Another mod has a reward with this id. The lines below it name the mod
that keeps it.

**Fewer items arrive than listed.** The `quantity` values add up to
more than 20.

**Nothing of the reward loads and the log is clean.** `custom_quests`
is `false` in the `[content]` table of [nams.toml](13_nams_toml.md);
it covers rewards too.
