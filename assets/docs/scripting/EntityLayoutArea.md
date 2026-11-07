# EntityLayoutArea

Places entities like [EntityLayoutAction](EntityLayoutAction.md) does, but only while the
player is in a particular part of the world.

It can do everything the plain action can and carries the same layouts, described under
[layouts](_layouts.md). On top of that it is tied to a region.

```
   ┌─ EntityLayoutArea ──────────────────────────┐
   │                                             │
   │   layouts   enemies                         │
   │   area      the market                      │
   │                                             │
   └─────────────────────────────────────────────┘
```

## The cycle

```
   WAIT       nothing placed, watching for the player
     │
     │  player enters the area
     ▼
   CREATING   entities are appearing
     │
     ▼
   CREATED    everything is standing
     │
     │  the reset condition is met
     ▼
   back to WAIT, everything removed
```

`WAIT` is the state the plain action never has. It is what makes this one repeatable: after
the player leaves, everything placed is removed and it waits to run again from the start.

While it is in `WAIT` nothing exists in the world, so nothing is costing anything.

`CREATING`, `CREATED` and `DONE` mean the same as in the plain action, including the numbers
that go with them.

## What puts it back to WAIT

`resetType` decides that. It takes the same four values as on the
[EnemyGenerator](EnemyGenerator.md), with the same meaning.

| Value | Name | What has to happen |
|---|---|---|
| 0 | none | never resets, the placement stays for good |
| 1 | outside | the player is **outside `area`**, the region that started it |
| 2 | inside | the player is **inside `resetArea`**, a second region |
| 3 | outside previous | the player is **outside `resetArea`**, but only once a region has actually been entered |

With **outside**, one region does both jobs: entering starts it and leaving ends it.

With **inside**, the region that ends it is separate from the one that starts it and can be
somewhere else entirely. Leaving the first one is then not enough; the player has to reach
the second.

**Outside previous** is the usual choice when several regions share one action. It waits
until one of them has been entered, and only then does leaving count.

Each of them works off a shape if one is drawn, and falls back to a plain distance if not:
outside uses `areaDist`, the other two use `resetDist`.

## Commands you can send it

| Command | What it does |
|---|---|
| `layoutIsDone` | answers whether everything has been placed |

## Attribute 0x400

A blocking action waits for `DONE`; add `0x400` and it waits for `CREATED` instead.

## Everything else

`bForwardState`, the three difficulty lists, and `param` work exactly as they do on the
plain action, and are described there.

## A real one from the game

```xml
<action>
	<code str="EntityLayoutArea">0xcf775473</code>
	<name eng="Object placement area - lights out">オブジェ配置エリア-消灯</name>
	<id>0xcf3c0130</id>
	<attribute>0x0</attribute>
	<layouts>
		<normal>
			<id>0x350488ec</id>
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
					<id>0xe2825df0</id>
					<location>
						<position>-600 -300 570</position>
					</location>
					<objId>bg0260</objId>
					<setRtn>5</setRtn>
				</value>
			</layouts>
		</normal>
		<hard>
			<id>0x86a1d789</id>
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
			<id>0x26f48d25</id>
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
	<area>
		<size>0</size>
	</area>
	<resetArea>
		<size>0</size>
	</resetArea>
	<resetType>1</resetType>
</action>
```

That is every field this action has.
