# EnemySetAction

Places a group of enemies and reports how many are still alive.

It sits between the two other placement actions.
[EntityLayoutAction](EntityLayoutAction.md) puts entities down and counts them, but knows
nothing about a fight. [EnemyGenerator](EnemyGenerator.md) runs a spawner that keeps
refilling. This one places a fixed set, like the first, but tracks it as a battle, like the
second.

What gets placed is described under [layouts](_layouts.md), the same as for the other
placement actions.

## Start here

For a basic fixed fight:

1. Add one layout per enemy.
2. Leave `condition` empty and `delay` at zero, so they appear at once.
3. Leave `max` at `0` to fight all of them together, or set a small number to let
   them come at you a few at a time.
4. Wait for `DONE` with a [ConditionBlock](ConditionBlock.md) to notice the end.

## The cycle

```
   WAIT       nothing placed, waiting for the condition
     │
     │  the condition passes
     ▼
   DELAY      counting down, the number is the time left
     │
     ▼
   BATTLE     enemies are out, the number is how many are alive
     │
     │  the last one dies
     ▼
   DONE
```

**WAIT** is where it starts. With no condition set it moves on immediately.

**DELAY** is the pause before the enemies appear. With `delay` at zero it passes straight
through, and you never see it.

**BATTLE** carries the count of living enemies, and it drops as they are killed:

```
   BATTLE 3      three alive
   BATTLE 1      two killed
   DONE          the last one is gone
```

A [ConditionBlock](ConditionBlock.md) can wait on that count: hold a door shut until `DONE`,
or react at `BATTLE` with a given number still left.

**DONE** is reached when nothing placed is alive any more, and what was placed is cleaned up
unless the action is marked to keep it.

## condition

What has to be true before the enemies appear. One [condition](_conditions.md), written the
same way as on every other action that carries one: a `puid` for who to watch, a `state` with
name and number, and a `pred` for how to compare.

Leave it out and the enemies appear as soon as the action runs.

## delay

Seconds between the condition passing and the enemies appearing. Zero means at once.

## max

How many enemies may be alive at the same time.

At `0` there is no limit: every layout is placed at once, and you fight all of them together.

```
   max 0, six layouts

   ┌──────────────────────────────────────┐
   │  1  2  3  4  5  6   all six at once  │
   └──────────────────────────────────────┘
```

Set to a number, the rest wait their turn. The action counts how many are alive, and whenever
that drops below the number, it lets the next one out.

```
   max 2, six layouts

   1  2                      waiting: 3 4 5 6
   ─  2   one killed         3 comes out
   3  2                      waiting: 4 5 6
   3  ─   one killed         4 comes out
   3  4                      waiting: 5 6
```

The fight is always two enemies wide, but it lasts until all six are dead. The arena quests
are built this way: one action holds every enemy of the round, and a small `max` lets them
out a few at a time.

## Ending a fight early

Sending `PULLOUT` while the action is in `BATTLE` tells the enemies to leave rather than
killing them, and puts the action back to `WAIT` so it can run again later.

Any other command is passed on to the placed enemies instead, but only once they are all
standing.

## Attributes it reads of its own

**`0x800`** keeps the enemy models loaded after the fight instead of freeing them.

**`0x400`** makes a blocking action wait for the enemies to appear instead of for `DONE`.

## param

Named settings for the action as a whole, in the same form as the ones inside a layout, which
are described under [layouts](_layouts.md).

## A real one from the game

```xml
<action>
	<code str="EnemySetAction">0xfb085793</code>
	<name eng="enemy set">敵セット</name>
	<id>0x1619457e</id>
	<attribute>0x1</attribute>
	<layouts>
		<normal>
			<id>0x93ac5ea2</id>
			<flags>0x0</flags>
			<parent>
				<id>
					<id>
						<code str="app::EntityLayout">0xd8efbb45</code>
						<id>0xba9d8bba</id>
					</id>
				</id>
			</parent>
			<layouts>
				<size>0</size>
			</layouts>
		</normal>
		<hard>
			<id>0x9d6642be</id>
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
			<id>0x163ca671</id>
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
</action>
```

Left out here: `param`. It is optional, and leaving it out keeps the default.
