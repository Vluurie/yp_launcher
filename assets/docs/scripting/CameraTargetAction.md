# CameraTargetAction

Turns the camera to look at something.

```
   ┌─ CameraTargetAction ────────────────────────┐
   │   location   the thing to look at           │
   │   time_      5        seconds               │
   │   rate_      0.05     how fast it turns     │
   └─────────────────────────────────────────────┘
```

## The cycle

```
   WAIT
     │  the player enters the area
     ▼
   ACTIVE     the camera keeps turning towards the spot
     │  time_ runs out
     ▼
   held       waits for the player to leave
     │
     ▼
   WAIT       can happen again next time
```

Leaving the area at any point goes straight back to `WAIT`.

## location

The spot to look at. Name an entity instead of coordinates and the camera follows it as it
moves.

## area

Where the player has to be. Without one the gaze starts as soon as the group switches on.

## time_

Seconds the gaze lasts. At `-1` it holds until the player leaves the area.

## rate_

How fast the camera turns. `0.05` is the usual, slow turn; `0.1` is quick; `0.01` is very slow.

## playerForceIn_

Pulls the camera in towards the player during the gaze.

## The player cannot fight it

While the gaze runs, the player's camera control is switched off, and switched back on when it
ends. [Attribute](02_action.md) `0x400` leaves the control with the player instead.

## A real one from the game

```xml
<action>
	<code str="CameraTargetAction">0x4f925d24</code>
	<name eng="Camera gaze_for missile gaze">カメラ注視_ミサイル注視用</name>
	<id>0xc43a0cb3</id>
	<attribute>0x10</attribute>
	<location>
		<position>322.176147 -39.931694 958.190613</position>
	</location>
	<area>
		<size>1</size>
		<value>
			<code str="app::area::BoxArea">0x18cffd98</code>
			<position>430.805115 -85.980217 829.037964</position>
			<rotation>0 -0.76 0</rotation>
			<scale>10.127424 25.671255 5.922655</scale>
			<points>-1 -1 1 -1 1 1 -1 1</points>
			<height>1</height>
		</value>
	</area>
	<time_>5</time_>
	<rate_>0.05</rate_>
	<playerForceIn_>0</playerForceIn_>
</action>
```

`playerForceIn_` is optional and other blocks leave it out, but this one fills it in.
