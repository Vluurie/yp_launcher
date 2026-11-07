# PhaseEventAction

Plays a cutscene.

## Start here

For a basic cutscene:

1. Set `eventNo` to the cutscene's number and `eventType` to `0`.
2. Draw `area` where playback should begin.
3. Draw `loadArea` around it, large enough that loading finishes before the player
   arrives.
4. Set `startType` to `1`, the usual choice.
5. Leave everything else alone until you need it.

## Two areas

```
   ┌─ loadArea ──────────────────────────┐
   │                                     │
   │      ┌─ area ─────────┐             │
   │      │  starts it     │             │
   │      └────────────────┘             │
   │   loading starts here, early        │
   └─────────────────────────────────────┘
```

`loadArea` is the outer one. Entering it begins loading, so the data is ready before the player
arrives. `area` is the inner one, and entering that starts playback.

Without the outer area the player would stand still while the cutscene loads.

Leaving `loadArea` again releases what was loaded, unless `releaseWhenLeaveLoadArea` is off. Once
playback has been asked for, leaving stops mattering.

## The cycle

```
   WAIT
     │  player enters loadArea
     ▼
   LOADING ──────────────────────────► FAILURE   the data could not be read
     │  everything arrived                 │
     ▼                                     └──► loading can be tried again
   LOADED
     │  player enters area
     │
     ├─ player is busy ──► GETDOWN ──┐   waits for them to be back on their feet
     │                               │
     ├─ a fade is set ───► INNOISE ──┤   black screen and noise
     │                               │
     ▼                               │
   READY  ◄──────────────────────────┘
     │  playback asked for
     ▼
   WAIT_PLAY
     │  the cutscene is ready to show
     ▼
   PLAYING ──► PAUSED ──► PLAYING     only through commands
     │
     ├─ the player skips ──► SKIP ──┐
     │                              │
     ▼                              ▼
   WAIT_DONE ───────────────────► DONE
```

**GETDOWN** is where someone on a ladder or mid-attack ends up until they are standing normally
again.

**INNOISE** is the black fade and the noise effect, described below.

**WAIT_PLAY** is the gap between asking for playback and the first frame appearing.

**PAUSED** is only ever reached by sending a command; nothing gets there on its own.

Switching the action off during `PLAYING` or `PAUSED` ends the cutscene and puts the action back
to `WAIT`.

## startType

This decides when the cutscene gets going.

| Value | What happens the moment the action switches on              |
| ----- | ----------------------------------------------------------- |
| 0     | nothing; it waits for the areas                             |
| 1     | starts loading right away, if no `loadArea` is set          |
| 2     | starts **playing** right away, without waiting for any area |
| 3     | like 1                                                      |
| 4     | like 1, and always goes through the black fade              |

1 is the usual choice. Use 2 when a cutscene has to fire the instant its group comes on.

3 and 4 additionally hand the cutscene a reference to an action: 4 hands it this action, 3 hands
it the one before it in the chain. That reference is used when the cutscene ends.

## Fading in

Two settings cover the moment before playback, both in seconds.

`blackTime_` fades to black first. `noizeTime_` plays the noise effect over it. If either is
set, the action passes through `INNOISE` before `READY`.

```
   blackTime_ 1.0        fade to black over a second
   noizeTime_ 0.5        then half a second of noise
        │
        ▼
   the cutscene starts
```

`startFade` is a separate, gentler fade. It only happens when `fadeSubPhase` matches the
current point in the story, and only when none of the above are in use.

## connectType\_

How the player gets from where they stand to where the cutscene starts.

| Value | What happens                              |
| ----- | ----------------------------------------- |
| 0     | plays immediately                         |
| 1     | the player is lifted off the ground first |
| 2     | the player is sent walking first          |
| 3     | a subtitle is shown, then it plays        |

With 1, the player plays a short animation, an effect goes off, and then they are **raised
upward a little every frame** until they are off the ground or a fifth of a second has passed.

```
        ☺           lifted, frame by frame
        │
        ☺
        │
   ─────☺─────      standing on the ground
```

Once that is done the player reports back and the cutscene starts. Switching the action off
mid-lift puts the camera back and returns the player to standing.

With 2, the player is given a direction and a distance and walks that far before the cutscene
starts. `connectWalkDist_` is the distance, and the direction depends on the shape of the area
they are standing in:

```
   box                          cylinder or sphere

   ┌─────────┐                        ╭───╮
   │    ☺ ──►│  along the box's      │  ☺│      towards the middle
   │         │  own facing           │ ◄─┘      of the area
   └─────────┘                        ╰───╯
```

A box has a rotation to follow; the other two do not.

`connectWalkCaptionWait_` is passed along in both cases.

With 3, `_captionSequenceState` counts through showing the subtitle, waiting, and then starting
playback.

## bStop

Set, the game clears away anything still on screen when the cutscene begins: any open text
window is closed, a running conversation is ended, and every script is told to drop messages it
was holding.

It is counted rather than switched, and the count is undone when the cutscene finishes, so
nested cutscenes do not clear twice.

On by default.

## Which cutscene

`eventNo` picks it. `eventType` decides where it is looked for.

**eventType 0** loads files named directly after the number:

```
   movie/evXXXX.usm     the video
   event/evXXXX.evn
   event/evXXXX.evt
```

**eventType 1** looks inside the data of the phase the game is currently in:

```
   the current phase
   ┌────────────────────────┐
   │   evXXXX.evn           │
   │   evXXXX.evt           │
   └────────────────────────┘
```

Type 1 only works while that phase is loaded, and the same `eventNo` therefore means a
different cutscene in a different phase. Type 0 does not depend on where the game is.

With type 0, part of the number can also be rewritten before loading: if scene state `Route/A`
or `Route/B` is set, one action loads a different version per route. Attribute `0x2000` switches
that off.

`startSpecificSubPhase_` with `subPhase` ties the action to one point in the story. If the game
is anywhere else, the action goes straight to `DONE` without doing anything.

## Loading extra data

_Advanced, skip this for a basic cutscene._

Three more slots, `readEventNo_00` to `_02`, load further cutscenes alongside the main one. Each
has its own `readEventType_`, taking the same values as `eventType` above.

They are loaded, never played. Playing them is the job of **another PhaseEventAction** whose
`eventNo` is that number. The slots only make sure the data is already there when that second
action starts, so nothing has to be fetched at the moment of the switch.

This is how the game chains cutscenes together:

```
   ┌─ action A ────────────┐
   │  eventNo         0xe055     plays this
   │  readEventNo_00  144        gets 0x90 ready
   │  readEventNo_01  145        gets 0x91 ready
   └───────────────────────┘
              │
   ┌─ action B ────────────┐
   │  eventNo         0x90       plays it, already loaded
   │  readEventNo_00  145        keeps 0x91 ready
   └───────────────────────┘
              │
   ┌─ action C ────────────┐
   │  eventNo         0x91       plays it, already loaded
   └───────────────────────┘
```

Watch the numbers: `eventNo` is written in hex, the slots in decimal. 144 is `0x90` and 145 is
`0x91`.

All of them go out in the same frame, and the action waits for every one of them before it
reaches `LOADED`. The action takes longer to get going, but the next one in the chain starts
instantly.

If any one of them fails to load, the whole action goes to `FAILURE`.

A slot is switched off through its `readEventType_`, not through its number: **2 means off**, and
that is what the game fills in when the field is left out. Most actions leave all three off.

## Commands

| Command  | What it does                                      |
| -------- | ------------------------------------------------- |
| `LOAD`   | starts loading without waiting for the player     |
| `PLAY`   | starts playback, or forces it while still loading |
| `PAUSE`  | holds a running cutscene                          |
| `RESUME` | continues a held one                              |
| `STOP`   | skips, exactly as the player skipping does        |

`PLAY` during loading is the useful one: it marks the cutscene as wanted, and from then on the
player leaving the area no longer cancels it.

## Attributes

Beyond the usual ones, this action reads four of its own.

`0x400` makes a blocking action release at `PLAYING` instead of `DONE`, so the list carries on
while the cutscene is still running.

`0x800` skips `GETDOWN`: the cutscene starts even if the player is on a ladder or mid-attack,
instead of waiting for them to be standing normally.

`0x1000` keeps the cutscene data until it has really finished. Without it the data is released
shortly before the end, while the last moment is still playing.

`0x10000` starts pulling the cutscene into memory ahead of time, so it is ready when the
moment comes instead of loading then. It happens once and the mark clears itself afterwards.

## A real one from the game

```xml
<action>
	<code str="PhaseEventAction">0x69108577</code>
	<name eng="replay event">イベントの再生</name>
	<id>0xc205c81c</id>
	<attribute>0x10</attribute>
	<eventNo>0x910</eventNo>
	<subPhase>"invalid value"</subPhase>
	<area>
		<size>1</size>
		<value>
			<code str="app::area::BoxArea">0x18cffd98</code>
			<position>805.695679 -36.398117 -264.304321</position>
			<rotation>0 -1.04 0</rotation>
			<scale>6.776062 23.028837 5.491165</scale>
			<points>-1 -1 1 -1 1 1 -1 1</points>
			<height>1</height>
		</value>
	</area>
	<loadArea>
		<size>2</size>
		<value>
			<code str="app::area::BoxArea">0x18cffd98</code>
			<position>805.695679 -38.085182 -264.304321</position>
			<rotation>0 -1.04 0</rotation>
			<scale>30.067209 26.927139 28.24596</scale>
			<points>-1 -1 1 -1 1 1 -1 1</points>
			<height>1</height>
		</value>
		<value>
			<code str="app::area::BoxArea">0x18cffd98</code>
			<position>820.426453 -84.428772 -213.057144</position>
			<rotation>0 -0.78 0</rotation>
			<scale>38.274658 123.035133 45.442562</scale>
			<points>-1 -1 1 -1 1 1 -1 1</points>
			<height>1</height>
		</value>
	</loadArea>
	<connectType_>2</connectType_>
	<connectWalkDist_>10</connectWalkDist_>
	<connectWalkCaptionWait_>1</connectWalkCaptionWait_>
	<bStop>1</bStop>
	<fadePhase>0x300</fadePhase>
	<fadeSubPhase>""</fadeSubPhase>
</action>
```

Left out here: `UNKNOWN`, `blackTime_`, `eventType`, `location`, `noizeTime_`, `readEventNo_00`, `readEventNo_01`, `readEventNo_02`, `readEventType_00`, `readEventType_01`, `readEventType_02`, `releaseWhenLeaveLoadArea`, `startFade`, `startSpecificSubPhase_` and `startType`. They are optional, and leaving them out keeps the defaults.
