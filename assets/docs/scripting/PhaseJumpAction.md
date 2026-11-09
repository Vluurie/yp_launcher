# PhaseJumpAction

Moves the game to a different part of the story.

It ends one chapter and starts the next, or drops the player into a different area entirely.

## Where to

`phase` names the chapter and `subPhase` the point inside it, written as text like
`80_00_AB_Pascal_Village` or `Ending_O`.

```
   ┌─ PhaseJumpAction ───────────────────────────┐
   │   phase     0x300                           │
   │   subPhase  80_00_AB_Pascal_Village         │
   │   type      1                               │
   └─────────────────────────────────────────────┘
```

## type

Two ways to get there.

| type | What happens |
|---|---|
| 0 | switches over on the spot |
| 1 | goes through a loading screen |

**0** changes the story position without interrupting play. It also clears the flags that
block the player from attacking, jumping, shooting, evading and hacking, so a player who was
locked down for a scene gets control back.

**1** is the real jump, with the loading screen and the fade around it.

Leaving the field out means 0.

## The cycle

```
   WAIT
     │  the jump is triggered
     ▼
   REQUESTED     asked for, not arrived yet
     │  the game reports the new phase and subphase
     ▼
   DONE
```

`REQUESTED` waits until the game actually reports the destination as current. A blocking action
holds the list up until `DONE`, so the lines after it only run once the jump has really happened.

## area

The jump can be tied to a region, and then it only fires when the player is inside it. Most of
them have no region and fire as soon as their group switches on.

## param

Only for `type 1`: what the screen does during the jump.

`type` inside `param` picks the effect:

| Value | Effect |
|---|---|
| 0 | short loading screen |
| 2 | no loading screen |
| 3, 4 | fade for hacking, in and out |
| 5, 6 | hacking glitch, out and in |
| 8 | fade out with a hacking glitch |

`speed` is how long the fade takes: 1 is quick, 2 is slow, anything else is the middle setting.

`color` picks black or white. 0 is black, 1 is white.

`bNoFade` skips the fade and cuts straight over.

## Attribute 0x400

Forces the plain short loading screen and ignores everything in `param`.

## A real one from the game

```xml
<action>
	<code str="PhaseJumpAction">0x227440da</code>
	<name eng="Phase jump - if you run away from an enemy">フェーズジャンプ-敵から逃走した場合</name>
	<id>0xbb8cb203</id>
	<attribute>0x0</attribute>
	<area>
		<size>5</size>
		<value>
			<code str="app::area::BoxArea">0x18cffd98</code>
			<position>594.182495 -48.926048 -204.895599</position>
			<rotation>0 0.525004 0</rotation>
			<scale>12.451462 39.640797 2.170002</scale>
			<points>-1 -1 1 -1 1 1 -1 1</points>
			<height>1</height>
		</value>
		<value>
			<code str="app::area::BoxArea">0x18cffd98</code>
			<position>649.744324 -48.926048 -230.768036</position>
			<rotation>0 0.525004 0</rotation>
			<scale>51.406868 39.640797 4.019037</scale>
			<points>-1 -1 1 -1 1 1 -1 1</points>
			<height>1</height>
		</value>
		<value>
			<code str="app::area::BoxArea">0x18cffd98</code>
			<position>693.103394 -57.351124 -233.602509</position>
			<rotation>0 0.525004 0</rotation>
			<scale>3.538513 39.640797 31.690592</scale>
			<points>-1 -1 1 -1 1 1 -1 1</points>
			<height>1</height>
		</value>
		<value>
			<code str="app::area::BoxArea">0x18cffd98</code>
			<position>728.259338 -52.542961 -174.485748</position>
			<rotation>0 0.525004 0</rotation>
			<scale>7.318348 39.640797 52.917343</scale>
			<points>-1 -1 1 -1 1 1 -1 1</points>
			<height>1</height>
		</value>
		<value>
			<code str="app::area::BoxArea">0x18cffd98</code>
			<position>721.164063 -57.741013 -271.315308</position>
			<rotation>0 0.610004 0</rotation>
			<scale>29.528433 39.640797 5.773804</scale>
			<points>-1 -1 1 -1 1 1 -1 1</points>
			<height>1</height>
		</value>
	</area>
	<phase>0x200</phase>
	<subPhase>60_07_B_King_of_the_forest_ev2</subPhase>
	<type>1</type>
	<param>
		<color>0</color>
		<speed>0</speed>
		<type>5</type>
		<bNoFade>-1845460992</bNoFade>
	</param>
</action>
```

`param` and `type` are optional and other blocks leave them out, but this one fills them in.
