# ShootingEnemyCurveAction

A flight path for an enemy in the shooting sections, with its movement and shooting written
along the way.

The original name is 敵パスカーブ, "enemy path curve". It belongs to the parts of the game
that turn into a shoot-em-up: an enemy follows the curve this action describes, and the
settings decide how it moves and fires while doing so.

It is by far the largest action in the game.

## Start here

For a basic flight path:

1. Draw the curve in `bezier_`.
2. Set `speed_` for how fast it is flown.
3. Set `shotType_` to `1` for straight shots, and give the rhythm with `shotWait_`,
   `shotTime_`, `shotInterval_` and `shotReload_`.
4. Mark one node with `startAttack_` and a later one with `endAttack_`, so the enemy
   fires only between them.
5. Set `loopNum_` to how many times the path is flown.
6. Leave everything else alone until you need it.

## It does not run itself

Unlike every other action here, this one performs nothing. It is a description that an enemy
reads while flying.

```
   ┌─ the action ────────┐            ┌─ the enemy ─────────┐
   │  the curve          │  reads ◄───│  flies along it     │
   │  the settings       │            │  shoots on the way  │
   └─────────────────────┘            └─────────────────────┘
```

Its own state only says whether anyone is currently using it:

```
   WAIT  ◄────────►  ACTIVE
```

An enemy on the curve keeps it in `ACTIVE`. Two frames after the last one stops, it falls back
to `WAIT`. It needs no blocking mark and holds nothing up.

## The path moves with the player

Easy to get wrong: **the curve is not fixed in the world.**

Every frame the offsets are transformed through the player's camera and added to the player's
position, so the whole shape is carried along in front of them.

```
        player flies right ──►              player flies left ──►

            ╭─────╮                              ╭─────╮
       ►►  │  ○   │                              │   ○  │  ◄◄
            ╰─────╯                              ╰─────╯
         the path stays                      and stays in front
         in front of them                    when they turn
```

That is why the numbers in these fields are small: they are distances from the player, not
positions in the world.

```
   offsetStart_ ──┐
   offsetLoop_  ──┼── through the player's camera ──► the path
   offsetEnd_   ──┘
```

`locationStart_`, `locationLoop_` and `locationEnd_` hold the result. You do not fill them in;
the game overwrites them every frame.

## The curve

The shape is a **bezier curve**: a line through a series of nodes, each with two handles that
bend it.

```
              handle
                 ╲
        ●─────────╳         ● node
       ╱           ╲        ╳ handle
      ╱             ●───────────╲
     ●                           ╲
   start                          ●  end
```

`bezier_` holds those nodes and their control points.

`bezierCreatedCenter_`, `bezierCreatedFront_` and `bezierCreatedUp_` record which way the
player was facing when the curve was drawn, so the shape can be lifted out of that original
frame and placed in the current one. That is what keeps a curve looking the same however the
player is oriented.

`reverse_` flies the path backwards. `loopNum_` is how many times to fly it; each run counts
down, and at zero the enemy leaves.

## Node values override the action

Almost every setting exists twice: once on the action, once inside each node.

```
   action    shotType_ 1  ──── the default for the whole path
                │
   ┌────────────┴────────────┬─────────────────────┐
   │                         │                     │
  node 1                   node 2                node 3
  shotType_ -1             shotType_ 4           shotType_ -1
  → uses 1                 → uses 4              → uses 1
```

A node with nothing to say leaves **-1** there. That is how an enemy can approach quietly and
only open fire halfway along.

A value that neither the node nor the action sets leaves the enemy's current value alone
rather than resetting it.

## Movement

`speed_` is how fast the enemy travels the curve. Left unset it uses `nodeComeSpeed_` on the
first node and `nodeLeaveSpeed_` after that. `stayTime_` holds it at a node before moving on.

**`moveType_` adds a weave across the direction of travel**, on top of the path:

```
   moveType_ 0            moveType_ 1            moveType_ 2
   no weave               side to side           up and down

   ●────────────►         ●─╮  ╭─╮  ╭─►              ╭─╮  ╭─╮
                            ╰──╯ ╰──╯            ●───╯ ╰──╯ ╰──►

                          ├────┤                     ├────┤
                        moveRadius_               moveRadius_
```

The enemy drifts off the line until it reaches `moveRadius_`, then swings back, and keeps
swinging. `moveSpeed_` is how fast it drifts. Values 3 and 4 are the same two shapes, just
starting in the other direction.

## Facing

`dirType_` decides where the enemy looks, which is separate from where it flies:

```
   dirType_ 1                      dirType_ 3
   always at the player            turning steadily

        ●  enemy                        ●  ↻
        │                               
        ▼                            it spins on its
        ☺  player                    own as it flies
```

| Value | What it looks at |
|---|---|
| 0 | a fixed angle |
| 1 | **the player**, followed continuously |
| 2 | an angle that swings back and forth |
| 3 | turning steadily |
| 4 | like 0, but facing the opposite way |

`rotRate_` is how fast it turns to face that way. Left unset it uses a slow default.

## Shooting

`shotType_` picks the pattern. The names come from the game's own editor labels:

```
   1  one way          2  three way        3  six way

        │                 ╲ │ ╱             ╲╲ │ ╱╱
        │                  ╲│╱               ╲╲│╱╱
        ●                   ●                  ●

   4  scatter           5  heavy            7  laser

        ╱                   ▓                  ║
       │  random            ▓                  ║
        ●  each shot        ●                  ●
```

`shotShakeAngle_` is the spread those use: the gap between the three-way shots, the widening
of the six, and the range the random one picks from.

`shotShakeRadius_` does something different. It moves the **starting point** sideways rather
than the direction:

```
   shakeAngle_ turns the shot      shakeRadius_ moves where it starts

        ╲ │ ╱                           │   │   │
         ╲│╱                          ● ─► ● ◄─ ●
          ●                            the muzzle shifts
```

Four values set the rhythm:

```
   shotWait_    shotTime_                    shotReload_    shotTime_
   ├────────┤   ├──────────────────────┤     ├─────────┤    ├──────────
                │  │  │  │  │  │  │  │                      │  │  │  │
                └──┴──┴──┴──┴──┴──┴──┘                      └──┴──┴──┘
                 shotInterval_ between
                    each of these
```

Each of those is rolled again every time the cycle turns, so the firing never falls into a
perfectly even rhythm.

`shotSpeedScale_` scales how fast the shots travel, `shotStrongRate_` how often a stronger one
goes out.

`startAttack_` and `endAttack_` are a **pair of switches**, not single events:

```
   node 1        node 2            node 3         node 4
   startAttack_                    endAttack_
      │                               │
      ▼                               ▼
   ●──────────────────────────────────●─────────────►
      └──────── firing here ──────────┘
```

The enemy also turns to face the player at the moment the stretch opens.

## parameter_: four separate guns

*Advanced: not needed for a basic flight path.*

The long row of numbers is **four guns**, ten values each, and each one fires on its own
schedule with its own pattern.

```
   ┌─ gun 1 ─┬─ gun 2 ─┬─ gun 3 ─┬─ gun 4 ─┐
   │ 10 vals │ 10 vals │ 10 vals │ 10 vals │
   └─────────┴─────────┴─────────┴─────────┘

   start  end  type  wait  duration  reload  interval  shakeRadius  shakeAngle  strongRate
```

So one enemy can carry a fast forward gun and a slow scattering one at the same time:

```
   gun 1  │ ││ ││ ││ ││ ││ ││ ││ │      fast, one way
   gun 2  │   ╲│╱      ╲│╱     ╲│╱      slow, three way
          └──────────── time ──────────►
```

The values mean the same as the shooting settings above, and the developers' labels survive in
the game: 射撃開始 (fire starts), 射撃終了 (fire ends), 射撃時間 (fire duration), 射撃間隔
(gap between shots), 横ブレ (sideways wobble), 角度ブレ (angle wobble), 強弾割合 (share of
strong shots).

A `-1` means the same as everywhere else: not set.

## debug_

Switches on a path visualiser. **It draws nothing in the retail game**: the drawing code was
removed.

## A real one from the game

```xml
<action>
	<code str="ShootingEnemyCurveAction">0x4d5c2719</code>
	<name eng="&quot; enemy pass curve enclosure &quot;">"敵パスカーブ 囲み"</name>
	<id>0x88873d1c</id>
	<attribute>0x0</attribute>
	<bezier_>
		<attribute>0xc630ad6f</attribute>
		<controls>
			<size>0</size>
		</controls>
		<nodes>
			<size>0</size>
		</nodes>
	</bezier_>
	<bezierCreatedCenter_>0 0 0</bezierCreatedCenter_>
	<bezierCreatedFront_>0 0 1</bezierCreatedFront_>
	<bezierCreatedUp_>0.494995 0.868896 0</bezierCreatedUp_>
	<locationStart_>
		<position>50 0 -17.087637</position>
		<rotation>0 4.303186 0</rotation>
	</locationStart_>
	<locationLoop_>
		<position>16 0 -7.087637</position>
		<rotation>0 4.303186 0</rotation>
	</locationLoop_>
	<locationEnd_>
		<position>50 0 -7.087637</position>
		<rotation>0 4.303186 0</rotation>
	</locationEnd_>
	<offsetStart_>50 0 -17.087637</offsetStart_>
	<offsetLoop_>16 0 -7.087637</offsetLoop_>
	<offsetEnd_>50 0 -7.087637</offsetEnd_>
	<loopNum_>0</loopNum_>
	<reverse_>0</reverse_>
	<startAttack_>0</startAttack_>
	<endAttack_>0</endAttack_>
	<stayTime_>15</stayTime_>
	<rotRate_>0</rotRate_>
	<speed_>0</speed_>
	<dirType_>1</dirType_>
	<moveType_>1</moveType_>
	<moveSpeed_>0.01</moveSpeed_>
	<moveRadius_>2</moveRadius_>
	<shotType_>1</shotType_>
	<shotTime_>1</shotTime_>
	<shotInterval_>0.4</shotInterval_>
	<shotReload_>0.75</shotReload_>
	<shotWait_>1.5</shotWait_>
	<shotSpeedScale_>0.7</shotSpeedScale_>
	<shotShakeRadius_>1</shotShakeRadius_>
	<shotShakeAngle_>25</shotShakeAngle_>
	<shotStrongRate_>0.2</shotStrongRate_>
	<nodeComeSpeed_>0.04</nodeComeSpeed_>
	<nodeLeaveSpeed_>0.6</nodeLeaveSpeed_>
	<parameter_>0 0 -1 -1 -1 -1 -1 -1 -1 0 0 1 -1 -1 -1 -1 -1 -1 0 0 1 1 -1 -1 -1 -1 -1 0 0 -1 1 1 -1 -1 -1 -1 0 0 0 0</parameter_>
	<debug_>0</debug_>
</action>
```

That is every field this action has.
