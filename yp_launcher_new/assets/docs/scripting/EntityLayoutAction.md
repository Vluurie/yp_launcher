# EntityLayoutAction

Puts entities into the world at fixed positions: robots, animals, machines, people, and
objects.

Each entity it places is described by a [layout](_layouts.md), and one action can carry
several of them, so a whole camp comes from a single line in the list.

```
   ┌─ EntityLayoutAction ────────────────────────┐
   │                                             │
   │   layout 1   a small robot, on the road     │
   │   layout 2   a big one, behind it           │
   │   layout 3   a crate, off to the side       │
   │                                             │
   └─────────────────────────────────────────────┘
```

Everything about *what* gets placed and *how it behaves* is part of the layout and is
described there.

## The cycle

The action reports one of three states, and it moves between them on its own as the
entities appear and are killed.

```
   CREATING   not everything is standing yet
     │        the number is how many are still appearing
     │
     │  the last one has appeared
     ▼
   CREATED    everything is standing
     │        the number is how many are still alive
     │
     │  the last one is gone
     ▼
   DONE       all placed, none left alive
```

A state is a name plus a number.

**CREATING** is the build-up. A layout can carry a delay, so entities do not have to
appear all at once, and until the last one has appeared the action stays here. The number
next to the state is how many are still on their way in.

**CREATED** means everything has been placed. The number now counts the living, and only
goes down.

```
   3 placed, none killed     CREATED 3
   one killed                CREATED 2
   all killed                DONE 0
```

**DONE** is reached when everything has been placed and nothing placed is alive any more.
That is what lets a [ConditionBlock](ConditionBlock.md) wait for "the whole camp is dead".

An action that places a crate rather than an enemy works the same way. The crate is not
killed, so the action stays at `CREATED` for as long as the crate exists.

A [ConditionBlock](ConditionBlock.md) watching a crate for `DONE` therefore waits forever.
For anything that is not expected to die, wait for `CREATED` instead.

## Attribute 0x400

A blocking action waits for `DONE`; add `0x400` and it waits for `CREATED` instead.

## Commands you can send it

| Command | What it does |
|---|---|
| `layoutIsDone` | answers whether everything has been placed |

## bForwardState

*Advanced: not needed for a basic layout.*

With this switched on, the action stops counting. Instead of `CREATING` and `CREATED`, it
mirrors the state of the entity it placed, whatever that entity happens to be doing.

With several entities in one action, the **first one that has a state of its own** is
used. The rest are not consulted, even if they have states too.

The original game leaves it switched off everywhere.

## normal, hard and extream

The action carries three lists of layouts, one per name.

Only **normal** is ever used. Every part of the action reads that one: the counting, the
state, the commands, and the state forwarding. The other two are read from the file and then
never looked at again, and the original game keeps them switched off everywhere.

They are leftovers, not a difficulty setting.

## param

Not the same `param` as the one inside a layout. The layout one holds the named settings
such as `Lv` or `HP_MIN`, and is described under [layouts](_layouts.md). This one sits on
the action itself, next to `bForwardState`.

The original game never fills it in.

## A real one from the game

```xml
<action>
	<code str="EntityLayoutAction">0xda948617</code>
	<name eng="Object arrangement">オブジェ配置</name>
	<id>0xcfdb7c41</id>
	<attribute>0x8</attribute>
	<layouts>
		<normal>
			<id>0x46f92bd3</id>
			<flags>0x0</flags>
			<parent>
				<id>
					<id>
						<code>0x0</code>
						<id>0x0</id>
					</id>
				</id>
			</parent>
			<layouts>
				<size>1</size>
				<value>
					<id>0x384c22e</id>
					<location/>
					<objId>et002a</objId>
					<setType>1</setType>
					<setFlag>0x4</setFlag>
				</value>
			</layouts>
		</normal>
		<hard>
			<id>0xc5424dbc</id>
			<flags>0x1</flags>
			<parent>
				<id>
					<id>
						<code>0x0</code>
						<id>0x0</id>
					</id>
				</id>
			</parent>
			<layouts>
				<size>0</size>
			</layouts>
		</hard>
		<extream>
			<id>0x300dd4df</id>
			<flags>0x1</flags>
			<parent>
				<id>
					<id>
						<code>0x0</code>
						<id>0x0</id>
					</id>
				</id>
			</parent>
			<layouts>
				<size>0</size>
			</layouts>
		</extream>
	</layouts>
	<bForwardState>0</bForwardState>
</action>
```

That is every field this action has.
