# MemorySaveAction

Takes a snapshot of the game and sets the point the player comes back to after dying.

It fires once, the moment its group switches on, and reports `DONE` straight away.

## How it decides

```
   the action fires
        │
        ├─ attribute 0x800 set? ──── yes ──────────────────┐
        │                                                  │
        no                                                 │
        │                                                  │
        ├─ attribute 0x400 set? ── yes ──► look up          │
        │                                  pointName        │
        │                                     │             │
        │                          found? ────┤             │
        │                                     │             │
        │                        yes: store   │  no:        │
        │                        the spot     │  nothing    │
        no                                    │             │
        │                                     │             │
        ▼                                     ▼             ▼
   is a return spot set (from now or earlier)?              │
        │                        │                          │
       yes                       no ─────────────────────────┤
        │                                                    │
        ▼                                                    ▼
   when the player dies,                     when the player dies,
   they come back to that                    they come back to the
   spot, facing the same way                 current point in the story
```

## pointName

Names a fast travel point to use as the return spot. The lookup is by name.

Along with the position it records the player's experience and money at that moment, so those
come back too.

Only `0x400` acts on this field. Set without that bit, the name sits there unused.

## Afterwards

A `MEMORY_SAVE` signal goes out once the snapshot is taken, so other actions can react to it.

## A real one from the game

```xml
<action>
	<code str="MemorySaveAction">0xbea4fc64</code>
	<name eng="memory save">メモリセーブ</name>
	<id>0x5cd07a90</id>
	<attribute>0x10</attribute>
	<pointName>""</pointName>
</action>
```

That is every field this action has.
