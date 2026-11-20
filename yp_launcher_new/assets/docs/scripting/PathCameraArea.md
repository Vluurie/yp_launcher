# PathCameraArea

A [PathCameraAction](PathCameraAction.md) with a region around it, and a way to blend in and
out.

This is the one you place. The plain action has no idea when it should start; this one watches
a region and switches itself on when the player walks in, off when they leave, and eases the
camera across at both ends.

## Start here

For a basic side-view stretch:

1. Place a PathCameraArea and draw `area` around the stretch it should cover.
2. Draw `playerPath` along the way the player should walk.
3. Leave `route` empty.
4. Set `offset` to `0 4 -20`: four up, twenty out to the side.
5. Switch `useFovy_` on and set `fovy_` to `35`.
6. Set `interRate` to `2` and leave `endInter` at `-1`, so leaving eases back just as
   slowly as arriving.
7. Leave everything else alone until you need it.

Steps 2 to 5 are settings of the [plain action](PathCameraAction.md) and are explained there.

## The cycle

```
   WAIT  ──────────►  ACTIVE  ⇄  STANDBY
    ▲    player enters              │
    │                               │
    └───────────────────────────────┘
              player leaves
```

**WAIT** is where it sits while the player is outside the region. The camera is untouched and
nothing is being pulled.

**ACTIVE** and **STANDBY** are the [plain action's](PathCameraAction.md) two states, and once
the player is inside the region the action underneath takes over and reports them.

`WAIT` is the useful one to watch for: it means the player is not in this stretch.

The switch in either direction fires the blending below. Crossing in sets the entering values
and crossing out sets the leaving ones, each once at the moment of crossing rather than every
frame.

## area

The region. Drawn the same way as on any other action, and described under
[areas](_areas.md).

```
   ┌─ area ────────────────────────────┐
   │                                   │   inside:  the path camera runs
   │             🯅                     │
   └───────────────────────────────────┘   outside: WAIT, camera untouched
```

Do not overlap two of these. Only one path camera can drive the camera at a time, and which
one wins is unpredictable. Put them end to end instead, so the player leaves one as they
enter the next.

## interRate and endInter

How long the camera takes to ease across, in seconds.

`interRate` is used on the way in, `endInter` on the way out. At `-1`, which is what most of
them use, `endInter` is not used and the entering time is applied again on the way out.

```
   player enters                     player leaves
        │                                 │
        ▼                                 ▼
   interRate  2.0                    endInter  3.0
   ease in over two seconds          ease out over three
```

Common values run from `0.1` for a near-instant snap up to `10` for a long, slow drift. `2`
and `3` are the ones the game uses most.

## useStartEndInter_

The switch in front of both of the above. With it off, no easing time is sent at all and the
camera picks up the new framing.

The XML field is `useStartEndInter_`, one word longer than you might guess from the two
settings it controls.

## accType and endAccType

The shape of the ease, entering and leaving.

```
   0  steady            1  slow start        2  slow finish      3  slow both ends

   │        ╱           │       ╱            │  ╱                │      ╱
   │      ╱             │     ╱              │ ╱                 │    ╱
   │    ╱               │   ╱                │╱                  │  ╱
   │  ╱                 │ ╱                  │                   │ ╱
   └──────────          └─────────           └────────           └────────
   time ──►
```

| Value | Movement |
|---|---|
| 0 | the same amount every moment |
| 1 | slow at the start, fast at the end |
| 2 | fast at the start, slow at the end |
| 3 | slow at both ends, fast in the middle |

`0` is what almost every one of them uses. `3` is the next most common; use it when the entry
should feel deliberate.

## cameraInterpEnable

Whether the camera is allowed to ease at all while this area holds it. On in nearly every one
of them; switching it off makes the camera jump straight to its framing.

## enemyPath

*Advanced: not needed for a basic camera.*

A fourth line, alongside the three the [plain action](PathCameraAction.md) has. It is built
and kept up to date exactly like the others, but nothing in this action reads it: it is here
for enemies to follow through the stretch, the way `playerPath` is there for the player.

Most areas leave it empty. The ones that use it are the stretches where enemies come at you
along a fixed route.

## The interpolation times

*Advanced: not needed for a basic camera.*

Ten settings that give each part of the camera its own easing time, instead of one time for
everything. Five for entering, five for leaving:

| Entering | Leaving | Controls |
|---|---|---|
| `interpTimeDistance_` | `endInterpTimeDistance_` | how far back the camera sits |
| `interpTimeAngle_` | `endInterpTimeAngle_` | the angle |
| `interpTimeFov_` | `endInterpTimeFov_` | the view width |
| `interpTimePosition_` | `endInterpTimePosition_` | where the camera sits |
| `interpTimeTarget_` | `endInterpTimeTarget_` | the spot it looks at |

Each has an acceleration type of its own (`interpTimeAccTypeDistance_` and so on, ten more
fields), taking the same values as `accType` above.

A time of `-1` means the setting is off and `interRate` handles that part instead. **Every one
of these is `-1` in every camera in the original game**, so the whole block is unused. It
works, but there is no shipped example of it to copy from.

Use them when one part of the move should take longer than the rest: a camera that swings
round quickly while pulling back slowly.

## Everything else

`route`, `playerPath`, `offset`, `fovy_`, the slide limits and all the rest belong to the
[plain action](PathCameraAction.md) and are described there. This one carries every one of
them.

## A real one from the game

```xml
<action>
	<code str="PathCameraArea">0xf0213c66</code>
	<name eng="Pass camera area - first room 1st floor">パスカメラエリア-最初の部屋1階</name>
	<id>0x59ac7258</id>
	<attribute>0x0</attribute>
	<route>
		<attribute>0x0</attribute>
		<nodes>
			<size>0</size>
		</nodes>
		<resolution>8</resolution>
	</route>
	<playerPath>
		<attribute>0x0</attribute>
		<nodes>
			<size>2</size>
			<value>
				<point>908.441772 -30.21022 -323.869843</point>
			</value>
			<value>
				<point>938.304565 -30.21022 -340.968231</point>
			</value>
		</nodes>
		<resolution>8</resolution>
	</playerPath>
	<offset>0 4 -20</offset>
	<fixHeightUse_>1</fixHeightUse_>
	<fixHeight_>-23.459999</fixHeight_>
	<playerY>0</playerY>
	<branchDist>2</branchDist>
	<excludeArea>
		<size>0</size>
	</excludeArea>
	<useOffsetPlDir>0</useOffsetPlDir>
	<useOffsetPlDirRate_>4.5</useOffsetPlDirRate_>
	<sideViewCameraPathDirOffsetEnable_>0</sideViewCameraPathDirOffsetEnable_>
	<sideViewCameraPathDirOffsetLengthSlow_>0</sideViewCameraPathDirOffsetLengthSlow_>
	<sideViewCameraPathDirOffsetLengthMiddle_>0</sideViewCameraPathDirOffsetLengthMiddle_>
	<sideViewCameraPathDirOffsetLengthHigh_>0</sideViewCameraPathDirOffsetLengthHigh_>
	<sideViewCameraPathDirOffsetInterp_>0.1</sideViewCameraPathDirOffsetInterp_>
	<sideViewCameraPathDirOffsetInterpStop_>0.1</sideViewCameraPathDirOffsetInterpStop_>
	<sideViewCameraInterpSeparate_>0</sideViewCameraInterpSeparate_>
	<sideViewCameraInterpSeparateXZ_>0.1</sideViewCameraInterpSeparateXZ_>
	<sideViewCameraInterpSeparateY_>0.1</sideViewCameraInterpSeparateY_>
	<useFovy_>1</useFovy_>
	<fovy_>40</fovy_>
	<viewOffsetInterp_>0.1</viewOffsetInterp_>
	<viewOffsetGround_>0 0 0</viewOffsetGround_>
	<viewOffsetAir_>0 0 0</viewOffsetAir_>
	<usePos>1</usePos>
	<useTarget>1</useTarget>
	<pos>
		<position>937.350525 -25.635801 -310.449829</position>
		<rotation>0 0.523599 0</rotation>
	</pos>
	<tar>
		<position>922.225159 -28.83931 -336.866943</position>
		<rotation>0 0.523599 0</rotation>
	</tar>
	<slideUp_>0.5</slideUp_>
	<slideDown_>0.25</slideDown_>
	<slideX_>2</slideX_>
	<forceAdjust_>0</forceAdjust_>
	<useInterpLimit_>0</useInterpLimit_>
	<interpLimit_>0.01</interpLimit_>
	<bInterpRateRefHackRate_>0</bInterpRateRefHackRate_>
	<firstOffset_>0 0 0</firstOffset_>
	<area>
		<size>1</size>
		<value>
			<code str="app::area::BoxArea">0x18cffd98</code>
			<position>923.752258 -33.107903 -332.918091</position>
			<rotation>0 0.523599 0</rotation>
			<scale>15.5277 8.033354 2.208687</scale>
			<points>-1 -1 1 -1 1 1 -1 1</points>
			<height>1</height>
		</value>
	</area>
	<cameraInterpEnable>1</cameraInterpEnable>
	<useStartEndInter_>1</useStartEndInter_>
	<interRate>2</interRate>
	<accType>0</accType>
	<endInter>2</endInter>
	<endAccType>0</endAccType>
	<interpTimeDistance_>-1</interpTimeDistance_>
	<interpTimeAngle_>-1</interpTimeAngle_>
	<interpTimeFov_>-1</interpTimeFov_>
	<interpTimePosition_>-1</interpTimePosition_>
	<interpTimeTarget_>-1</interpTimeTarget_>
	<interpTimeAccTypeDistance_>0</interpTimeAccTypeDistance_>
	<interpTimeAccTypeAngle_>0</interpTimeAccTypeAngle_>
	<interpTimeAccTypeFov_>0</interpTimeAccTypeFov_>
	<interpTimeAccTypePosition_>0</interpTimeAccTypePosition_>
	<interpTimeAccTypeTarget_>0</interpTimeAccTypeTarget_>
	<endInterpTimeDistance_>-1</endInterpTimeDistance_>
	<endInterpTimeAngle_>-1</endInterpTimeAngle_>
	<endInterpTimeFov_>-1</endInterpTimeFov_>
	<endInterpTimePosition_>-1</endInterpTimePosition_>
	<endInterpTimeTarget_>-1</endInterpTimeTarget_>
	<endInterpTimeAccTypeDistance_>0</endInterpTimeAccTypeDistance_>
	<endInterpTimeAccTypeAngle_>0</endInterpTimeAccTypeAngle_>
	<endInterpTimeAccTypeFov_>0</endInterpTimeAccTypeFov_>
	<endInterpTimeAccTypePosition_>0</endInterpTimeAccTypePosition_>
	<endInterpTimeAccTypeTarget_>0</endInterpTimeAccTypeTarget_>
</action>
```

Left out here: `enemyPath`. It is optional, and leaving it out keeps the default.
