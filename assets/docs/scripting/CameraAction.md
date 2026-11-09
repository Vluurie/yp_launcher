# CameraAction

Takes over the camera: where it sits, what it looks at, how far back it is, how wide the view
is, and how long it takes to get there.

Almost every camera that is not the ordinary follow-camera comes from one of these: the angle
when you walk into a room, the fixed shot during a conversation, the pull-back at the start of
a place.

## Start here

For a basic fixed shot:

1. Switch `usePos` on and set the position.
2. Switch `useTarget` on and set the target.
3. Set `interRate` to `0.1` for a quick move, lower for a slower one.
4. Leave the five interpolation times at `-1` and the `end` settings alone until you
   need them.

No area is needed: without one the action runs from the moment its group switches on.

## It asks, it does not command

The action does not move the camera. It fills in a list of wishes and marks which entries it
filled in. Something else reads that list and moves the camera.

```
   ┌─ the action ────────┐        ┌─ the wish list ──────┐        ┌─ the camera ──┐
   │                     │        │  ✓ position          │        │               │
   │  position           │───────►│  ✓ target            │───────►│   moves       │
   │  target             │        │    up vector         │        │               │
   │  distance           │        │  ✓ distance          │        │               │
   └─────────────────────┘        └──────────────────────┘        └───────────────┘
                                     ✓ = this one was filled in
```

An entry nobody marked keeps whatever it had. Leaving a setting alone is not the same as
setting it to zero.

Because of that, two of these can be active at once without conflict, as long as they mark
different entries. One can set the distance while another sets what the camera looks at.

## The switches decide what gets marked

Nothing reaches the list unless a switch says so.

| Switch | Marks |
|---|---|
| `usePos` | the position |
| `useTarget` | what the camera looks at |
| `overwrite` | the same entry, with a stronger mark |
| `rotateOnly` | the three rotation values |
| `distanceOnly` | the distance |

`useTarget` and `overwrite` write the same entry. Only the mark differs: a plain target can be
replaced by something else, an overwriting one cannot.

The offset is always marked and has no switch.

## Position and target can be attached to something

The position and the target are not always plain coordinates. Each can name a thing in the
world and a part of it. The numbers are then an offset from that part.

```
   plain                          attached to something

   (10, 2, 30)                    (0, 2, -3) on the boss, part 5
        ●  fixed in the world              ●  moves with the boss
```

If the named thing is not there, the numbers are used as they are.

## Rotation, distance and view width

```
                   Rotation_Y turns around
                        ↻
                    ╭───────╮
                    │       │
      Rotation_X ↕  │   ☺   │      Distance ├──────────┤
      up and down   │       │      how far back
                    ╰───────╯

      Rotation_Z rolls the picture sideways
```

The rotation values only get through when `rotateOnly` is on. The distance only gets through
when `distanceOnly` is on. Both are on in most of the ones in the game.

`Fovy` is how wide the view is. A small number is narrow, a large one wide and distorted.

## Getting there takes time

A camera that jumps is jarring, so values move across gradually.

`interRate` is the simple way: a share of the remaining gap each frame. At `0.1` it closes a
tenth of what is left every frame, so it is fast at the start and slow at the end.

The five interpolation times are the exact way. Each gives a duration in seconds for one
thing: the distance, the rotation, the view width, the position, and the look-at.

```
   distance   ├──────────────────┤     2 seconds
   rotation   ├─────┤                  0.5 seconds
   view width ├────────────┤           1.2 seconds
```

They are independent, so the camera can turn quickly while the distance changes slowly. At
`-1` the value has no timed move and uses `interRate` instead.

Each move takes the current value as its starting point at the moment it begins. A move
started while another is still running continues from where the camera actually is.

## The shape of the move

A duration says how long. The acceleration type says whether it starts fast or slow.

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
| 0 | the same amount every frame |
| 1 | slow at the start, fast at the end |
| 2 | fast at the start, slow at the end |
| 3 | slow at both ends, fast in the middle |

Each of the five timed moves has its own, and there is one for the plain rate as well. Zero is
the most common by a wide margin.

## Leaving again

When the player walks out, the same set of settings exists a second time with `end` in front
of the name. They decide how the camera hands control back.

```
   player enters                    player leaves
        │                                │
        ▼                                ▼
   ┌─────────────┐                  ┌─────────────┐
   │ the times   │   camera held    │ the end     │   camera released
   │ the shapes  │ ───────────────► │ times and   │ ───────────────►
   └─────────────┘                  │ shapes      │
                                    └─────────────┘
```

`endInter` is the plain rate for leaving. At `-1` the entering rate is used again.

An action only cancels the moves it started itself. Each move records who asked for it, and
the check on the way out compares that.

```
   area A starts a distance move ──┐
   area B starts a target move   ──┼──►  both running
   player leaves area A          ──┘
        │
        ▼
   only A's distance move stops; B's target move carries on
```

Without that, leaving one camera would cancel the work of another.

## Most of them have no area

An action with no area and no link counts as always inside. It runs from the moment its group
switches on until the group switches off, and the group is what controls it. Most camera
actions in the game work this way.

The name the editor fills in reads "camera parameter specification area", and the developers
usually kept it or added to it. It is a label and says nothing about whether the action has an
area.

## Holding, height and following

*Advanced: not needed for a basic camera.*

These change how the camera behaves while it is held, not where it starts.

| Setting | Effect |
|---|---|
| `useHold` | keeps a fixed sideways and forwards distance |
| `fixHeightUse_` | pins the camera to one height |
| `useMoveOffset_` | lets the camera lead the player's movement |
| `overwriteNormalInterRate` | replaces the game's default rate while held |
| `useHandOffInterp_` | the rate used when control is passed on |
| `playerForceIn_` | pushes the camera in towards the player |
| `leavePlayer_` | lets the camera stop following the player |
| `actionCamStop` | stops the automatic action camera |
| `disableCameraHit` | lets the camera pass through walls |
| `noControlTarget` | takes the look-at away from the player's control |
| `newInter` | switches on the alternative move behaviour |

Each of the first five has a number next to it giving the distance, height or rate.

`playerTargetForceInUp_` only does anything while the player stands on one particular object
in the game. Everywhere else it is ignored.

## The up vector

*Advanced: not needed for a basic camera.*

Normally the camera treats up as up. Two settings change that.

`upVecForce` sets a direction to use as up. `upVec` turns the up direction along with the
rotation values, so a rolled camera keeps its own upright.

The previous up direction is stored and restored when the camera is released.

## A time limit

`useLimit` with a limit time gives the action an expiry. When the time runs out the action
finishes and releases the camera, even if the player is still standing there.

```
   camera taken ─────────────────────────► the time runs out
                     the limit                     │
                                                   ▼
                                             releases, done
```

The clock only runs while the action is playing.

## Two settings that do nothing

`disableBattle` and `playerStop` are read from the file and stored, but nothing in the game
reads them afterwards. Setting either has no effect.

They appear on every camera action because the editor wrote out every setting.

## actType

A field of this name exists inside the action. Nothing writes it and no file contains a value
for it, so it is always zero.

It is not the acceleration type described above, which is a real setting.

## A real one from the game

```xml
<action>
	<code str="CameraAction">0xad895807</code>
	<name eng="Camera parameter specification area">カメラパラメータ指定エリア</name>
	<id>0x2b8463d9</id>
	<attribute>0x0</attribute>
	<area>
		<size>0</size>
	</area>
	<upVecForce>0 1 0</upVecForce>
	<Rotation_X>-6</Rotation_X>
	<Rotation_Y>0</Rotation_Y>
	<Fovy>50</Fovy>
	<Distance>4</Distance>
	<pos>
		<position>-104.029778 156.603104 -6.887785</position>
	</pos>
	<tar>
		<position>-110.866959 153.14949 6.007416</position>
	</tar>
	<usePos>1</usePos>
	<useTarget>1</useTarget>
	<interRate>0.1</interRate>
	<overwrite>0</overwrite>
	<offset>0 1.6 0</offset>
	<disableBattle>0</disableBattle>
	<useLimit>0</useLimit>
	<limitTime>10</limitTime>
	<accType>0</accType>
	<playerStop>0</playerStop>
	<actionCamStop>0</actionCamStop>
	<rotateOnly>1</rotateOnly>
	<distanceOnly>1</distanceOnly>
	<disableCameraHit>0</disableCameraHit>
	<newInter>0</newInter>
	<endInter>-1</endInter>
	<endAccType>0</endAccType>
	<Rotation_Z>0</Rotation_Z>
	<noControlTarget>0</noControlTarget>
	<overwriteNormalInterRate>0</overwriteNormalInterRate>
	<normalInterRate>0.05</normalInterRate>
	<useMoveOffset_>0</useMoveOffset_>
	<moveOffsetScale_>10</moveOffsetScale_>
	<moveOffsetInterp_>0.05</moveOffsetInterp_>
	<moveOffsetInterpStop_>0.01</moveOffsetInterpStop_>
	<useHold>0</useHold>
	<holdDistanceX_>9.9</holdDistanceX_>
	<holdDistanceZ_>4</holdDistanceZ_>
	<useHandOffInterp_>0</useHandOffInterp_>
	<handOffInterpRate_>0.01</handOffInterpRate_>
	<fixHeightUse_>0</fixHeightUse_>
	<fixHeight_>0</fixHeight_>
	<playerForceIn_>0</playerForceIn_>
	<leavePlayer_>0</leavePlayer_>
	<playerTargetForceInUp_>0</playerTargetForceInUp_>
	<playerTargetForceInUpDist_>0</playerTargetForceInUpDist_>
	<interpTimeDistance_>8</interpTimeDistance_>
	<interpTimeAngle_>8</interpTimeAngle_>
	<interpTimeFov_>8</interpTimeFov_>
	<interpTimePosition_>8</interpTimePosition_>
	<interpTimeTarget_>8</interpTimeTarget_>
	<interpTimeAccTypeDistance_>1</interpTimeAccTypeDistance_>
	<interpTimeAccTypeAngle_>1</interpTimeAccTypeAngle_>
	<interpTimeAccTypeFov_>1</interpTimeAccTypeFov_>
	<interpTimeAccTypePosition_>1</interpTimeAccTypePosition_>
	<interpTimeAccTypeTarget_>1</interpTimeAccTypeTarget_>
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
	<forceInterTimer>0</forceInterTimer>
	<forceInterRate>0.1</forceInterRate>
</action>
```

Left out here: `forceDis`, `forceDisInter`, `forceDisInterRate`, `forcePosInter`, `forcePosInterRate` and `upVec`. They are optional, and leaving them out keeps the defaults.
