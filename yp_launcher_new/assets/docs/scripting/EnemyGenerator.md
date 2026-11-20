# EnemyGenerator

Keeps producing enemies for as long as the player stays in the area.

Where [EntityLayoutAction](EntityLayoutAction.md) places a fixed set of entities at fixed
spots, this one runs a spawner: it picks enemies from a pool, puts them at spawn points,
and refills as they die.

It is the most involved action in the game, with 54 different settings.

## Start here

For a basic one-off fight:

1. Draw `area` where the fight should begin.
2. Put one enemy in the pool, with its level range.
3. Add two or three spawn points around the spot.
4. Set `spawnRange` to how many enemies the fight should produce in total, and
   `createRange` to how many may stand at once.
5. Set `respawnTime` to zero, so the fight happens once and ends in `DONE`.
6. Leave everything else alone until you need it.

## The cycle

The generator always reports a state, and it moves between them on its own. Other actions
can wait for those states, which is how a quest notices that a fight is over.

```
        ┌──────────────────────────────────────────────┐
        │                                              │
        ▼                                              │
      WAIT                                             │
        │   player enters the trigger area             │
        ▼                                              │
      BATTLE                                           │
        │   the wave ends                              │
        ├───────────────┐                              │
        ▼               ▼                              │
   respawn time    respawn time                        │
   is zero         is set                              │
        │               │                              │
        ▼               ▼                              │
      DONE         WAIT_RESPAWN ───────────────────────┘
   finished for    counts down, then starts over
   good
```

**WAIT** is the resting state. Nothing has been produced yet and the generator is watching
whether the player comes close.

**BATTLE** means enemies are being produced and refilled. A state carries a number as well
as a name, and here that number is how many enemies are alive right now. It counts down as
they die:

```
   BATTLE 8      eight of them still standing
   BATTLE 5      three killed
   BATTLE 2      only two left
```

To react to a fight being over, wait for `DONE`. The number is for reacting *during* the
fight, before it is over: a [ConditionBlock](ConditionBlock.md) can wait for `BATTLE` with
"2 or fewer left" and use that to send reinforcements, open a gate, or start a line of
dialogue while the fight is still going on.

Every generator in the original game that is watched this way waits for a count still above
zero, never for zero.

**WAIT_RESPAWN** means the wave is over and everything the generator produced has been
removed. Here the number is the time still to wait, counting down on its own. When it
reaches zero, every random value is rolled afresh and the generator goes back to `WAIT`,
ready to run again.

**DONE** means finished for good. The generator ends up here instead of waiting when the
respawn time is zero, and it does not start again.

Once a generator has been through a full round, it remembers that fact for good. That is
what the [second set of settings](#making-an-enemy-different-the-second-time-around) checks
when it decides whether an enemy should appear differently than it did the first time.

## Switching it off and on again

A fourth state, `DEACTIVATED`, sits outside that cycle. The generator only enters or leaves it
when it is sent a command.

Sending `DEACTIVATE` puts it there. Whatever state it was in at that moment is put aside
first, so the generator remembers where it was interrupted. `ACTIVATE` takes it back out
again and restores exactly the state that was put aside, then forgets it. A generator
switched off in the middle of a fight resumes that fight rather than starting over.

While it is switched off, the enemies it already produced stay where they are; only the
production of new ones stops.

Sending the same command twice does nothing the second time. A generator that is already
switched off will not overwrite its remembered state with `DEACTIVATED` itself.

## The areas

Every area can be given in two ways: as a shape drawn in the world, or as a plain distance.
If a shape is present it is used; otherwise the distance is, measured from the generator's
own position. A distance of zero or less means the area is not set at all.

They all measure against **the player's position**. None of them looks at where the enemies
are.

If the generator is attached to something else, the point that distances are measured from
moves along with it. A generator on a moving platform therefore carries its areas with it.

### The two that control the generator

`area` starts it. While the generator is waiting, the player entering this area is what
begins the wave.

`resetArea` ends it. Once it ends, every enemy is removed and the respawn wait begins.

`resetType` decides what "ended" means, and the three settings do not all watch the same
area:

| Setting | What has to happen |
|---|---|
| none | never resets |
| outside | the player has to be **outside `area`**, the one that started it |
| inside | the player has to be **inside `resetArea`** |
| outside previous | the player has to be **outside `resetArea`**, and only once that area has been checked at least once |

The last one is the default.

### The two that only produce markers

*Advanced: not needed for a basic generator.*

`searchArea` and `escapeArea` do not start or stop anything. They record whether the player
is inside them.

The result is not a plain yes or no. Alongside the answer, the game also records whether the
area was configured at all, so an area nobody filled in is kept apart from an area the player
has left.

### The two guard distances

*Advanced: not needed for a basic generator.*

`guardSDist` and `guardLDist`, a short and a long one, exist only as distances and have no
shape form.

They have one detail the others do not: while the player is not yet counted as inside, the
distance is treated as slightly larger than written. Once inside, the written value applies
again. That gap keeps the marker from switching on and off repeatedly when somebody stands
right at the edge.

### What happens with the markers

Everything the areas produce is collected into one set of markers, and that set is written
onto each enemy the generator spawned. It is written once when the enemy appears and then
again on every update, so it stays current while the fight runs.

Each enemy keeps two copies: the current set and the one from before. Keeping the previous
set is what allows the moment of crossing a boundary to be noticed, rather than only the
state of being inside or outside.

What reads those markers afterwards, and what behaviour they produce, is unknown.

## The enemy pool

The generator does not name one enemy but a list of candidates, each with a weight.

```
   ┌─ pool ──────────────────────────────────────┐
   │  em0060   rate 5     level 12 to 15         │
   │  em0061   rate 2     level 14 to 16         │
   │  em0110   rate 1     level 20 to 20         │
   └─────────────────────────────────────────────┘
```

A higher rate makes that enemy more likely. If every rate is zero, the generator falls back
to picking evenly, so a pool with no weights still works.

The pool holds up to 23 entries. Each one can carry its own level range and the same extra
settings a [layout](_layouts.md) can carry.

## Making an enemy different the second time around

*Advanced: not needed for a basic generator.*

A pool entry can hold its settings twice: the normal ones, and a second set called
**type2**. The second set is what the enemy gets when the generator runs again after having
already finished a full round.

The switch that turns it on is **type2type**.

```
   ┌─ pool entry ────────────────────────────────┐
   │  em0060                                     │
   │                                             │
   │  type2type   1        second set is on      │
   │                                             │
   │  normal      the settings for the first run │
   │  type2       the settings for later runs    │
   └─────────────────────────────────────────────┘
```

Despite what its name suggests, `type2type` does not choose between kinds. It is a plain
on or off: zero means off, anything else means on. Every generator in the original game
that uses it has it set to one.

Two things have to be true before the second set is used:

```
   first run             ─►  always the normal settings
   later run, switch off ─►  still the normal settings
   later run, switch on  ─►  type2
```

An enemy that guards a camp can be an ordinary enemy when you first arrive, and something
else entirely when you come back later.

`type2` holds the same kinds of settings as the normal ones, and they are described under
[layouts](_layouts.md), with one exception: the scene an enemy belongs to is always taken from
the normal settings, even on a later run.

Leaving `type2type` out, or setting it to zero, means the enemy is the same every time.

## The spawn points

Where enemies appear is a list of points, up to 30 of them, and they are picked by weight
just like the pool.

```
   ┌─ spawn point ───────────────────────────────┐
   │  point        12.0  0.0  -88.0              │
   │  radius       5.0                           │
   │  rate         3                             │
   │  minDistance  10                            │
   └─────────────────────────────────────────────┘
```

**radius** scatters enemies around the point instead of stacking them on it.

**minDistance** is the space to keep from enemies that are already standing there. The game
tries up to ten times to find a spot that respects it before giving up and using what it
has.

**rate** is the weight, same rule as the pool.

A point can also fix which way the enemy faces when it appears. Left alone, the facing is
random.

## Placing on the ground

`fitGround` traces downwards from the calculated spot and puts the enemy on whatever
surface it hits. It is on by default, which is why enemies do not hang in the air when a
spawn point sits slightly above the terrain.

## Everything is a range

Almost every number here is a pair of a minimum and a maximum rather than a single value.

| Setting | What it controls |
|---|---|
| `respawnTime` | how long the wait before starting over lasts |
| `spawnRange` | how many enemies the whole cycle produces in total |
| `createRange` | how many exist at the same time |
| `levelRange` | the level of each enemy |
| `spawnInterval` | the pause between two batches |
| `amountEachSpawn` | how many appear per batch |

At the start of every cycle the generator rolls a fresh value from each pair, so two visits
to the same generator do not play out identically.

Setting a pair's minimum and maximum to the same value makes it fixed.

## Level, absolute or on top of the player

`relativeLevel` decides how the level is read.

**Off.** The level comes from the pool entry. If that entry has none, the generator's own
range is used instead.

**On.** The player's level, plus a roll from the entry's range, plus a roll from the
generator's range. Both ranges act as a bonus on top of the player rather than as an absolute
level.

## Holding enemies back until nobody is looking

*Advanced: not needed for a basic generator.*

`cameraType` decides whether an enemy is allowed to appear based on whether its spot is
currently on screen.

| Value | Effect |
|---|---|
| 0 | no check, enemies appear regardless |
| 2 | only appear **off** screen |
| anything else | only appear **on** screen |

When the check fails the enemy is not dropped, only held back. It appears as soon as the
camera allows it.

## Commands you can send it

While a quest is running, a [SendCommand](SendCommand.md) aimed at the generator, or a line
of script, can send it any of these seven.

| Command | What it does |
|---|---|
| `activate` | switches it back on, resuming where it left off |
| `deactivate` | switches it off, remembering where it was |
| `stopSpawn` | stops production, leaves the enemies alive |
| `getDeadCount` | answers how many have died so far |
| `setDieRtn` | kills everything alive right now |
| `setDieRtnNoDrop` | the same, but no loot and no experience |
| `setBerserkerLoop` | switches every living enemy into berserk |

**stopSpawn** is the gentle way to end a fight. No more enemies are produced, but the ones
already there stay and can still be fought. Once it is set, it stays set.

**setDieRtn** is the abrupt way: everything alive dies at once, dropping loot and giving
experience as though the player had killed it.

**setDieRtnNoDrop** does the same but suppresses both the loot and the experience. Its name
mentions only the drops; the experience is silently gone too. Use it to clear a fight away
without rewarding it, for instance when a scene takes over.

**getDeadCount** reads back a number instead of changing something. While the fight runs it
answers how many of the produced enemies have died. Once the generator reports `DONE`, it
answers with the full count that the cycle was supposed to produce.

**setBerserkerLoop** marks every living enemy as berserk. What that changes is up to each
enemy and is not part of the generator.

## matrix

Never used.

## A real one from the game

```xml
<action>
	<code str="EnemyGenerator">0x6f0fb5bd</code>
	<name eng="enemy generator">敵発生機</name>
	<id>0x17d4cb70</id>
	<attribute>0x0</attribute>
	<points>
		<attribute>0x0</attribute>
		<nodes>
			<size>1</size>
			<value>
				<point>7.87 0 -4.01</point>
				<radius>1</radius>
				<rate>0</rate>
				<minDistance>0</minDistance>
			</value>
		</nodes>
	</points>
	<items>
		<size>1</size>
		<value>
			<objId>et0015</objId>
			<rate>0</rate>
		</value>
	</items>
	<spawnRange>
		<min>60</min>
		<max>60</max>
	</spawnRange>
	<createRange>
		<min>20</min>
		<max>20</max>
	</createRange>
	<spawnInterval>
		<min>0.5</min>
		<max>0.5</max>
	</spawnInterval>
	<amountEachSpawn>
		<min>1</min>
		<max>1</max>
	</amountEachSpawn>
	<cameraType>-1063256064</cameraType>
	<relativeLevel>0</relativeLevel>
	<area>
		<size>1</size>
		<value>
			<code str="app::area::BoxArea">0x18cffd98</code>
			<position>0 -4.95115 0</position>
			<rotation>0 0 0</rotation>
			<scale>22.096167 9.965706 13.306171</scale>
			<points>-1 -1 1 -1 1 1 -1 1</points>
			<height>1</height>
		</value>
	</area>
	<resetArea>
		<size>0</size>
	</resetArea>
	<resetType>0</resetType>
	<searchArea>
		<size>0</size>
	</searchArea>
	<UNKNOWN id="0x91c37192">
		<size>0</size>
	</UNKNOWN>
</action>
```

Left out here: `dist`, `escapeArea`, `fitGround`, `levelRange`, `respawnTime` and `stateBeforeDeactivated`. They are optional, and leaving them out keeps the defaults.
