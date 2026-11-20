# Token categories

Tokens are what a [group](_groups.md) watches. Each token is two values: a `code` saying
what kind of thing to watch, and an `id` saying which one.

```xml
<value>
	<code str="GlobalRoom">0x5cda8ece</code>
	<id>0x110</id>
</value>
```

This one watches whether the player is in the resistance camp.

## The kinds

| code | The token is "there" while |
|---|---|
| `@SceneState` | that [scene state](00_hap_system.md) is set |
| `GlobalRoom` | the player is in that part of the world |
| `SubPhase` | that section of the phase is the current one |
| `GlobalPhase` | that phase is the current one |
| `Hacking` | with id `0x1`: the hacking minigame is running. With id `0x0`: it is not |
| `Event` | that event is the current one |
| `Scenario` | that scenario is the current one |

`@SceneState` is by far the most used, with `GlobalRoom` and `SubPhase` behind it. `Event`
and `Scenario` each appear once in the original game.

The `id` for a `@SceneState` token is the hashed name of the scene state, and for a
`GlobalRoom` token the room's number: `0x110` is the resistance camp, `0x550` the bunker.
An editor fills these in from the readable name.

Both numbers are the plain [CRC32](_files.md#where-the-numbers-come-from) of the name, not the
case-insensitive variant. One real token, exactly as the game writes it:

```xml
<tokens>
	<size>1</size>
	<value>
		<code str="@SceneState">0x35c92545</code>
		<id str="q020_1_2/REQ">0xc4753e3</id>
	</value>
</tokens>
```

`0x35c92545` is `CRC32("@SceneState")` and `0xc4753e3` is `CRC32("q020_1_2/REQ")`. Get the
hash wrong and nothing complains: the group never switches on.

## Several tokens

Within one kind, any of them is enough: a group listing six rooms is "there" in each of the
six.

Across kinds they all have to agree. A group watching a scene state **and** a room is only
"there" while the state is set and the player is in the room.

## What "there" does

Nothing on its own. The group's attribute decides how to react: off while there, off while
absent, or paused. See [groups](_groups.md).

## Codes you will see but never write

`Phase`, `Quest` and `Room` tokens exist in memory while the game runs: the game attaches
them to groups itself as files load, and they drive the `0x40` loading described under
[groups](_groups.md). They never appear in a `<tokens>` list you write.

`Global`, `Debug` and `Test` categories exist in the game but no group in the original
content uses them.
