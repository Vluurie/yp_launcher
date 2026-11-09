# EnemySetArea

Places a group of enemies like [EnemySetAction](EnemySetAction.md) does, but tied to a part
of the world.

It does everything the plain action does, with the same layouts, battle count and `max`, and
adds a region of the world on top.

## The cycle

```
   WAIT       waiting for the condition
     │
     ▼
   DELAY      counting down
     │
     ▼
   AREA_WAIT  ready, but nothing placed until the player is in the area
     │
     │  the player arrives
     ▼
   BATTLE     enemies are out, the number is how many are alive
     │
     ├─ the last one dies ──────────────► DONE
     │
     └─ the reset condition is met ─────► back to WAIT, everything removed
```

`AREA_WAIT` is the state the plain action does not have. The enemies exist only once the
player comes near, and cost nothing before that.

Leaving can put it back to `WAIT`, which is what makes it repeatable. The enemies are told to
disappear rather than being killed, so nothing counts as defeated.

## The areas

The regions work exactly as they do on the [EnemyGenerator](EnemyGenerator.md), and are
described there: `area` starts it, `resetArea` and `resetType` decide what ends it, and
`searchArea`, `escapeArea` and the two guard distances feed the enemies information about
where the player is.

Each can be given as a shape or as a plain distance, as described under
[areas](_areas.md).

## Attributes it reads of its own

Besides the usual ones described under [actions](02_action.md), this kind reads four more.

**`0x1000`** keeps the area where you drew it. Without it, the region re-centres itself on the
enemies each time it starts: the game works out the middle of the placed group and measures the
player's distance from there instead of from the shape.

```
   without 0x1000                    with 0x1000

   ┌──────────────┐                  ┌──────────────┐
   │      ● ●     │                  │      ● ●     │
   │    ✛ ● ●     │  centre moves    │  ✛   ● ●     │  centre stays
   │      ●       │  onto the group  │      ●       │  where you put it
   └──────────────┘                  └──────────────┘
```

This is the one you are most likely to want, whenever the enemies sit off to one side of the
region you drew.

**`0x800`** keeps the enemy models loaded after the fight instead of freeing them.

**`0x400`** makes a blocking action wait for the enemies to appear instead of for `DONE`.

**`0x2000`** stops the group being marked finished when the action was already done before it
ran. Only three actions in the game use it.

## Everything else

`condition`, `delay`, `max`, `param` and the commands work as they do on the
[plain action](EnemySetAction.md), and what gets placed is described under
[layouts](_layouts.md).

## A real one from the game

```xml
<action>
	<code str="EnemySetArea">0xe8fefe4b</code>
	<name eng="Enemy Set Area_Reception">敵セットエリア_受付</name>
	<id>0x4e0e986</id>
	<attribute>0x1</attribute>
	<layouts>
		<normal>
			<id>0x1f81b925</id>
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
				<size>0</size>
			</layouts>
		</normal>
		<hard>
			<id>0x9c4aadcb</id>
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
			<id>0xea6ae1c9</id>
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
	<condition>
		<puid>
			<code>0x0</code>
			<id>0x0</id>
		</puid>
		<condition>
			<state/>
			<pred>0</pred>
		</condition>
	</condition>
	<delay>0</delay>
	<max>0</max>
	<area>
		<size>0</size>
	</area>
	<resetArea>
		<size>0</size>
	</resetArea>
	<resetType>3</resetType>
	<searchArea>
		<size>0</size>
	</searchArea>
	<escapeArea>
		<size>0</size>
	</escapeArea>
	<dist/>
</action>
```

Left out here: `UNKNOWN`. It is optional, and leaving it out keeps the default.
