# PathCameraAction

Holds the camera at a fixed angle, and can guide the player along a drawn line while it does.

This is what the 2D sections are made of. The camera watches from the side or from above and
stays put instead of swinging around behind the player.

Place [PathCameraArea](PathCameraArea.md) rather than this one. It is this same action with a
region added, and the region is what switches it on when the player walks in. Everything on
this page is on that one too.

```
   ┌─ PathCameraAction ──────────────────────────┐
   │                                             │
   │   playerPath   the line the player walks    │
   │   offset       0 4 -20                      │
   │   fovy_        35                           │
   │                                             │
   └─────────────────────────────────────────────┘
```

Every frame the action hands the camera a position and a spot to look at, and the camera
travels towards both. If you drew a `playerPath` it also pulls the player onto that line. Leave
`playerPath` empty and the player is never touched; only the camera changes.

**The steps for building one are on the [PathCameraArea](PathCameraArea.md) page.** Both
share the settings below.

`offset` is the setting you will change most, and what its numbers mean depends on whether
`route` is drawn.

Speeds on this page are written like `0.1`. That is not metres: the camera closes a tenth of
the remaining distance each frame. Bigger is faster and tighter; smaller is slower and
smoother.

## The cycle

```
   STANDBY  ─────►  ACTIVE
```

**STANDBY** while it waits, **ACTIVE** while it is driving the camera.

Only one path camera can drive the camera at a time. If two overlap, one of them wins and the
other reports `STANDBY` and does nothing. Which one wins is unpredictable, so do not overlap
them. To move from one shot to the next, place two
[PathCameraAreas](PathCameraArea.md) end to end.

## playerPath

The line the player walks along. Leave it empty and the player is left alone entirely.

Every frame the point on the line nearest the player is found, and the player is pulled towards
it. That is what keeps them on a fixed track through a 2D section. How hard they are pulled is
set by `forceAdjust_` below.

The search for the nearest point ignores height, so a line drawn above or below the player
still catches them.

```
   ●───────●───────●───────●     playerPath
              🯅
        the player is pulled onto the line
```

The player is also told which line they are on, which their own movement code reads. That
message expires after 0.2 seconds, so the action has to keep sending it every frame.

Even with a line drawn, the player is left alone while they are on a ladder, and while they are
frozen, unless a shooting section is running, in which case they are moved anyway. Standing
inside an `excludeArea`, or close enough to a `playerPath2` to trigger the branch, releases them
as well.

## forceAdjust\_

How firmly the player is held to the line.

Off, they are eased towards it: each frame they are moved two percent of the way there, and
only if they moved since the last frame. Moved means their spot in the world changed by more
than a centimetre, whatever caused it: the stick, a knockback, or the floor moving under them.
Standing still leaves them where they are.

On, they are placed straight onto the line every frame.

```
   forceAdjust_ = 0                 forceAdjust_ = 1

   🯅 ····┐                          🯅
         ┊ eased 2% each frame       ┃ placed exactly
   ●─────▼───────●            ●──────▼──────●
```

Off is the softer option and lets the player drift slightly. On removes that entirely.

## playerPath2 and branchDist

*Advanced: not needed for a basic camera.*

Use these only when the player can choose between two ways forward. `playerPath2` is a second
line, for a fork in the path.

`playerPath2` is ignored until the player comes within `branchDist` metres of it. From that
point both lines are measured and the shot works from whichever player line is nearer, so the
view goes with the branch the player walked towards. The camera's own `route`, if one is
drawn, is a separate thing and stays as it is.

```
   ●────────●────────●────────●────────●    playerPath
                     │
                     └──●────────●          playerPath2
                        ↑
                  within branchDist,
                  the nearer line drives the camera
```

The `branchDist` check measures from the player to the nearest point on `playerPath2`, in all
three directions, so height counts towards it: a branch running above the first line is further
away than its distance on the ground suggests. The comparison that then picks the winner ignores
height and measures along the ground only.

Winning is about the shot, not about pulling. The whole time the player is within
`branchDist` of the second line, the pulling pauses and they steer for themselves, which is
what lets them choose the fork. Meanwhile the shot works from whichever player line is nearer,
switching sides as they commit. The player is never pulled onto the second line at all: walk
back to the first line, out of the branch's reach, and the pulling takes hold again there.

## route

A separate line, this one for the camera. It changes what `offset` means.

**With `route` drawn**, the camera is put on it, at the point nearest the player, and `offset`
is added to that point. Height counts towards which point is nearest here, unlike the player's
line, so a camera line running above the player is measured as being that much further away.

```
                 ┌───┐
                 │ ▣ │   the camera
                 └───┘
                   ▲
                   │ offset
                   │
   ●───────●───────●───────●     route
                   ↑
            the point nearest the player
```

**With `route` empty**, the camera is placed from the direction of the player's own path
instead. That direction is turned ninety degrees and stepped away by the last number of
`offset`, which puts the camera out to the side of the way the player is walking. This is what
produces the side-on view, and it is how most of these are built.

```
        🯅 ────────►   the player walks this way

              ┌───┐
              │ ▣ │   the camera stands off to the side
              └───┘
```

With no `playerPath` either, there is no walking direction to work from, and straight up is
used in its place: the last number of `offset` then raises the camera directly above the
player instead of stepping it out to the side. For a still shot with no lines at all, `usePos`
and `useTarget` below are the better tool.

## resolution

Every line (`route`, `playerPath` and `playerPath2`) carries its own `resolution`. It is the
number of steps each stretch between two of your points is divided into when the line is
rounded into a curve: at `8` every stretch becomes eight small steps along the bend; at `1`
the line runs straight from point to point. Leave it at `8`.

## offset

Where the camera sits. Three numbers in metres.

With `route` drawn, all three are added to the point on the line along the map's own fixed
directions, which are the same everywhere no matter which way the line runs. `0 2 7` puts
the camera two metres higher and seven further along the map's forward direction, not seven
along the line.

With `route` empty the three numbers do different jobs. The last one sets how far out to the
side the camera stands. The middle one raises it, measured in the player's own upright
direction, so for a player standing on the ground it is simply height.

```
              ┌───┐
              │ ▣ │
              └───┘
                ▲
                │  middle number: height
                │
   🯅 ───────────┘
        last number: distance out to the side
```

The first number runs along the player's left-right direction, but only the up-and-down part
of that direction reaches the final camera position. For a player standing upright, left-right
has no up-and-down part, so it does nothing. Leave it at `0`.

## firstOffset\_ and bInterpRateRefHackRate\_

*Advanced: not needed for a basic camera.*

A second offset and the switch that brings it in. Off, `offset` is used as written.

On, the camera sits between the two, and the player decides where: as they hack or
perfect-evade, the camera travels from `firstOffset_` across to `offset` while that move plays
out.

```
   firstOffset_ ●─────────────────────● offset
                └──── the blend ──────┘
```

You set the two ends and nothing else. There is no speed setting, because the player's move
supplies the timing.

## excludeArea

Boxes where the player gets their own steering back.

```
   ┌───────────────────────────────────┐
   │  excludeArea                      │   inside:  the player steers
   │                                   │
   │             🯅                     │
   └───────────────────────────────────┘   outside: held to the line
```

Inside one of these the player is not pulled onto the line and their normal controls are handed
back. Put one over a doorway or a ledge where being locked to the line would trap them.

The camera keeps working either way. This releases the player, not the shot.

## playerY

Makes the camera take its height from the player rather than from the line, so it rises and
falls as they do. It only does anything when `route` is drawn, because without a camera line
there is no line height to replace.

The middle number of `offset` is added on top here as everywhere, so the camera sits that far
above the player.

## fixHeightUse\_ and fixHeight\_

Pins the camera to one height. `fixHeightUse_` switches it on and `fixHeight_` is the height.

This overrides everything else about height, including `playerY`, and it holds for as long as
the action runs. The camera stops rising and falling and stays on one level while the player
moves up and down beneath it.

`fixHeight_` is a height in the world, not a height above the player. To fill it in, read the
height off the floor you want the camera level with and write that number down. In the game
these come out as values like `-23.46`, because that is where that particular floor sits.

The middle number of `offset` is added on top, so leave it at `0` unless you want the camera
sitting that far above the height you asked for.

## viewOffsetGround\_, viewOffsetAir\_ and viewOffsetInterp\_

Raise or lower the spot the camera looks at, with one value for a player standing on the ground
and another for a player in the air.

```
   on the ground  ──►  viewOffsetGround_  ┐
                                          ├─ viewOffsetInterp_ moves between them
   in the air     ──►  viewOffsetAir_     ┘
```

Both are three numbers, and the shift is applied from the camera's own point of view rather than
the map's: the first number moves the aim across the screen, the middle one up, the last one
towards or away from the camera. Every one of these in the original game uses the middle number
only.

`0 1 0` on the ground raises the aim by a metre and shows more of the floor ahead. `0 3 0` in
the air raises it by three, which keeps a jumping player from sliding down to the bottom of the
screen.

`viewOffsetInterp_` is how fast the aim travels between the two when the player takes off or
lands. At `0.1` it closes a tenth of the remaining gap each frame; `0.05` makes the drift
visibly slower.

## Running ahead of the player

`sideViewCameraPathDirOffsetEnable_` pushes the shot ahead of the player in the direction they
are moving, so more of what is coming is on screen. The camera and the spot it watches both
slide forward by the same amount, so the player ends up behind the middle of the screen rather
than in it.

```
   🯅 ───►                    ┌───┐
                             │ ▣ │
        └─── the length ─────┘
```

Three lengths, in metres, and what the player is doing picks between them:

| Setting            | Used when                                                       |
| ------------------ | --------------------------------------------------------------- |
| `...LengthHigh_`   | sprinting, dashing, sliding, and jumps launched out of a sprint |
| `...LengthMiddle_` | running, and jumps launched out of a run                        |
| `...LengthSlow_`   | everything else, including walking and standing                 |

Putting the same number in all three gives the same lead whatever they do.

A jump counts as a sprinting jump if the player was above sprinting speed when they left the
ground, and it keeps that status in the air. Dropping well below running speed in the first
moments of the jump turns it back into an ordinary one.

Two more settings decide how fast the camera takes that lead up, and the player's speed picks
between them:

| Situation                              | Setting used                             |
| -------------------------------------- | ---------------------------------------- |
| moving quickly                         | `sideViewCameraPathDirOffsetInterp_`     |
| slow, standing still, or in a cutscene | `sideViewCameraPathDirOffsetInterpStop_` |

Both are a share of the remaining gap per frame. Setting `...InterpStop_` to `0` freezes the
lead where it was when the player stopped, instead of letting it slide back to centre.

## sideViewCameraInterpSeparate\_

The camera normally catches up at a single speed. This switch splits it into a sideways speed
and an up-and-down speed.

```
   off                                 on

   one speed for everything            ↕  sideViewCameraInterpSeparateY_
                                       ↔  sideViewCameraInterpSeparateXZ_
```

Pairing `0.2` sideways with `0.08` vertical follows sideways movement tightly while lagging
behind vertical movement, which stops the picture bobbing every time the player jumps.

Both are a share of the remaining gap per frame. Larger snaps the camera to the player; smaller
lets it trail.

The split covers where the camera sits and what it looks at. How fast the distance and the view
width change is not affected.

## useInterpLimit\_ and interpLimit\_

*Advanced: not needed for a basic camera.*

Sets the lowest speed the camera may follow at. The number is the same kind as every other
speed on this page: a share of the remaining gap closed each frame. The camera uses whichever
is faster, its own speed or this one, so a number below its normal speed changes nothing. Use
it to guarantee the camera keeps up on a stretch where it would otherwise fall behind.

## useOffsetPlDir and useOffsetPlDirRate\_

*Advanced: not needed for a basic camera.*

Shifts the camera towards the direction the player is facing. As with running ahead, the whole
shot moves: the spot the camera watches shifts towards the facing direction and the camera
goes with it, keeping its distance.

The shift is a slice of how far back the camera sits, and the rate is what that slice is
divided by, so a larger number gives a smaller shift:

| `useOffsetPlDirRate_` | Result                                       |
| --------------------- | -------------------------------------------- |
| `1`                   | a large shift                                |
| `4.5`                 | a moderate one, and what the game returns to |
| `6`                   | a small one                                  |

How far the camera shifts also depends on what the player is doing: walking gives the smallest
shift, running half again as much, and sprinting or dashing twice the walking amount. At the
normal `4.5` that comes out as roughly a fifth of the camera's distance while walking and
nearly half of it at a sprint. Below a slow walking pace there is no shift at all, so the
camera stays centred while the player stands or shuffles.

The rate is read once when the action starts, so changing it mid-run has no effect.

## usePos and useTarget

| Setting     | Together with | What it does                         |
| ----------- | ------------- | ------------------------------------ |
| `usePos`    | `pos`         | the camera stands at one fixed place |
| `useTarget` | `tar`         | the camera looks at one fixed place  |

Switch both on for a shot that does not move: the camera stands at `pos` and stares at `tar`
while the player walks around inside the picture. Aim one down at a room from above and you have
a top-down section without drawing a camera line at all.

Neither of these reaches the camera during a cutscene. The view width below still does, so a
cutscene keeps your field of view but takes back the position.

`pos` and `tar` each hold a position, and each can instead name an entity placed by an
[EntityLayoutAction](EntityLayoutAction.md), such as a lift. The position is then measured from
that lift and the shot rides along with it.

## useFovy\_ and fovy\_

`fovy_` is how wide the view is, in degrees, and `useFovy_` switches it on.

```
   small number          large number
   narrow and flat       wide, and more distorted

        ╱▔▔╲                ╱▔▔▔▔▔▔╲
       ╱    ╲              ╱        ╲
      ▣                   ▣
```

`50` is what the game uses when nothing changes it. `35` flattens the picture and is what makes
a side-on stretch read as 2D. `60` widens it and exaggerates depth. The game stays between 10
and 60.

## slideUp\_, slideDown\_ and slideX\_

How far the player may push the camera away from your framing with the right stick.

```
              ↑ slideUp_
              │
     ◄────────┼────────►  slideX_
              │
              ↓ slideDown_
```

These are distances, not speeds. Holding the stick pushes the camera further until it reaches
these limits, and letting go lets it drift back.

`0.5` up, `0.25` down and `2` sideways give a small look-around without letting the player break
the shot. All three at `0` means the camera cannot be moved at all.

## Attribute 0x400

Without it the camera stays exactly on your line, walls included: in a closed room the view can
end up inside the wall. With it the camera is pushed back out whenever something comes between
it and the player, the way the ordinary camera behaves.

## A real one from the game

```xml
<action>
	<code str="PathCameraAction">0xae48b63f</code>
	<name eng="pass camera">パスカメラ</name>
	<id>0x49b1dbd6</id>
	<attribute>0x2</attribute>
	<route>
		<attribute>0x62503c0</attribute>
		<nodes>
			<size>2</size>
			<value>
				<point>490.965088 10.477245 -31.003338</point>
			</value>
			<value>
				<point>514.272278 10.477245 -29.340782</point>
			</value>
		</nodes>
		<resolution>8</resolution>
	</route>
	<offset>0 0 0</offset>
	<fixHeightUse_>0</fixHeightUse_>
	<fixHeight_>0</fixHeight_>
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
	<useFovy_>0</useFovy_>
	<fovy_>50</fovy_>
	<viewOffsetInterp_>0.1</viewOffsetInterp_>
	<viewOffsetGround_>0 0 0</viewOffsetGround_>
	<viewOffsetAir_>0 0 0</viewOffsetAir_>
	<usePos>0</usePos>
	<useTarget>0</useTarget>
	<pos>
		<position>494.166077 10.477215 -30.593132</position>
	</pos>
	<tar>
		<position>494.166077 10.477215 -30.593132</position>
	</tar>
	<slideUp_>0.5</slideUp_>
	<slideDown_>0.25</slideDown_>
	<slideX_>2</slideX_>
	<forceAdjust_>0</forceAdjust_>
	<useInterpLimit_>0</useInterpLimit_>
	<interpLimit_>0.01</interpLimit_>
	<bInterpRateRefHackRate_>0</bInterpRateRefHackRate_>
	<firstOffset_>0 0 0</firstOffset_>
</action>
```

That is every field this action has.
