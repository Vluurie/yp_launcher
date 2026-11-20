# Save state

You were probably curious about the dynamic text values in the save file and what user0
and all that means? Well, here's the answer.

When the game saves, it does not only write the scene states a quest has reached. It also
writes a small block per action, recording what that action has already done. That block is
what keeps a looted chest empty after loading, and it looks like this:

```
version v0.0.2
Scene
 Action
  3748541c
   ch -1
   user0
    size 10
    value 0x1a0b123a
    value 0x1b1cda65
    ...
```

`version v0.0.2` is written by the game itself and is always that string. Every entry below
`Scene/Action` is one action, named by its own id in hex.

## The fields

| Field | Appears on | Meaning |
|---|---|---|
| `ch` | actions with a top byte | see below |
| `done` | any action | the action has finished |
| `disabled` | any action | the action was switched off |
| `user0` | layout actions | which entities are already looted / cleared |
| `user1` | layout actions | which entities had their item table evaluated |
| `layout` | layout actions | which entities are dead or terminated |
| `layout_enable` | layout actions | which entities were switched away from their default |

The four list fields all store the same thing: a sorted list of layout ids. Membership *is*
the value. An entity whose id is in `user0` has that mark and one that is not does not. An
empty list means the field is left out of the file entirely.

## Turning saving on

Whether an action is saved at all is decided by its [attribute](02_action.md), in the content
file. Two parts of that number matter, and they do different things.

**`0x10` is the saved switch.** An action with it writes a block; an action without it never
appears in the save and starts fresh every time the quest runs.

**The top byte adds the entity marks.** Written as `0xff000010`, the action also records
which of its entities have been looted or cleared — the `ch`, `user0`, `user1` and `layout`
fields. Without the top byte the block still exists, but only carries the small fields such
as `done`.

```
   attribute       0x10        done                    saved, nothing entity-specific
   attribute 0xff000010        ch -1, user0, user1     saved with entity marks
```

A chest that should stay empty after loading is written as `0xff000010`.

## `ch`

`ch` is the top byte of the attribute, written out as a signed number. It is only written at
all when that byte is not zero, which is why blocks holding just a `done` or a `state` have
no `ch` line.

```
   attribute 0xff000010   →   ch -1
   attribute 0x01000010   →   ch  1
   attribute 0x06000010   →   ch  6
```

`0xff` is what nearly all content uses, and it reads back as `-1` because the byte is signed.
A few actions use other values; `1` and `6` both occur in the original game.

The value also decides whether the block survives the cleanup pass that runs before the file
is written: a block is kept only if its `ch` is negative and it carries at least one real
field beside it. So an action marked this way but with nothing to remember is dropped, and a
block never contains just a `ch` on its own.

Nothing reads the number back on load — the slot the loader would use for it is empty in this
archive, so only the sign matters, not the value. Use `0xff`.

## `user0` and `user1`

These two only ever appear on [`EntityLayoutAction`](EntityLayoutAction.md) and
[`EntityLayoutArea`](EntityLayoutArea.md).

Reading a real save back against the original content, the entities marked in `user0` are:

| Count | Entity | |
|---|---|---|
| 63 | `ba2014` | container |
| 33 | `ba2015` | container |
| 22 | `ba2016` | container |
| 8 | `it0000` | item pickup |
| 3 | `bg0801`–`bg0803` | world objects |

120 of those 129 carry an `ItemTable` setting. They are the lootable ones.

**`user0` means "this one has been emptied".** When the entity is placed, the game asks
whether its id is in `user0`; if it is, the entity is made invisible and put straight into
its end state instead of standing there full again. When the player takes the contents, the
id is added.

**`user1` means "the item table has already been looked at".** The game reads the entity's
`ItemTable`, and if the player is still missing something from it, `user0` is cleared again
so the entity comes back. That is why the two lists usually hold the same ids.

You cannot set these from the file. They are runtime marks only — across all 4264 content
files there is not a single layout that has one of these bits written by hand.

## `layout` and `layout_enable`

`layout` holds the entities that are dead or terminated. On load they are reset and skipped,
so a camp you already cleared does not repopulate.

`layout_enable` holds only the entities whose enabled state was changed away from what the
file says. It is rare — one action in the whole save studied here.

## Fields on other actions

Actions that carry no layouts still save something small:

| Action | Field |
|---|---|
| `ConditionCommands` | `done` |
| `SendCommand` | `done` |
| `HapMultiMarkLocationAction` | `disabled` |
| individual layouts | `state`, `bLock` |

## Building a lootable container

Nothing has to be done for the persistence itself. Place the entity, give it an item table,
and the marks follow:

```xml
<value>
	<id>0x96403d63</id>
	<location>
		<position>220.18 -1.11 -383.09</position>
	</location>
	<objId>ba2014</objId>
	<param>
		<count>0x1</count>
		<value>
			<name>ItemTable</name>
			<code str="unsigned int">0x562ef932</code>
			<body>0xa03</body>
		</value>
	</param>
</value>
```

`ba2014`, `ba2015` and `ba2016` are the containers, `it0000` is a loose pickup. The
`ItemTable` body is an index into the drop tables; the values used by the original content
run from `0xa00` to `0xa53`.

## What is not established

Before a layout may hold any of these marks, the game checks one bit on the layout's parent
variant. That bit is not any of the `<flags>` written in the content files — across all
content there is not a single layout with it set by hand — so where it comes from could not
be established.

The meaning of the individual `<flags>` values on a layout (`0x1`, `0x2`, `0x3`, `0x8`) is
also unread here. The outer `<flags>` next to `normal` / `hard` / `extream` belongs to those
three lists; see [normal, hard and extream](EntityLayoutAction.md#normal-hard-and-extream).

Whether `user2` and `user3` are ever used is unknown. The four slots exist and the game can
read and write all of them, but no code was found that sets anything beyond `user0` and
`user1`, and neither appears in the save studied here.
