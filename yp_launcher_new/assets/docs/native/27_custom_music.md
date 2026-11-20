# How to add your own music

Put a `.wem` audio file in your mod, write a few lines of TOML, and NAMS
plays it as background music. The track loops, fades in and out, and
starts and stops depending on where the player is. NAMS builds the
track in memory; no installed file is changed.

## The short version

```toml
[[music]]
name = "<track name>"
wem  = "<file name>"
room = 0x110
```

Put this block into any `.toml` file in your mod's `entities\` folder,
and the audio file at `data\sound\stream\<file name>.wem` in the same
mod. [How to lay out your mod folder](02_mod_folder_layout.md) shows
where these folders sit. The parts in angle brackets are your own names.

Walk into the Resistance Camp, room `0x110`, and your track plays. Walk
out and it stops.

## Which audio file works

The file has to be a Wwise `.wem`. Wwise is the audio system
NieR:Automata plays its sound with, and `.wem` is its audio file format.
NAMS sets every `[[music]]` file up as Vorbis-encoded audio, so encode
the `.wem` with Vorbis, a compressed audio format.

`wem` is the file name without `.wem`. NAMS turns that name into the
Wwise id of the file, so you never pick a number yourself.

The track holds no copy of the audio: Wwise streams the file from
`data\sound\stream` while it plays.

## What goes into `[[music]]`

Every track is one `[[music]]` block.

| Key | Default | What it does |
|---|---|---|
| `name` | required | A unique name for the track. ASCII letters, digits and `_` only. Two tracks can use the very same audio file as long as their names are different. |
| `wem` | | The audio file in `data\sound\stream`, without `.wem`. Use either `wem` or `bank` in a block, never both. |
| `bank` | | A Wwise sound bank (`.bnk`) you built yourself, as a path inside your mod folder. NAMS starts the track with the event `BGM_Set_St_<name>` and stops it with `BGM_Set_St_<name>_Stop`, so the bank has to contain both. An event is the name Wwise plays a sound by. |
| `music_effects` | `true` | Picks the Wwise bus the track plays on. A bus is a channel of the Wwise mix. `true` is the bus of the vanilla music, `false` a different one. No effect with `bank`. |
| `loop` | `true` | Whether the track repeats. No effect with `bank`. |
| `fade_ms` | `3000` | How long the track takes to fade in and to fade out, in milliseconds. `0` cuts in and out at once. No effect with `bank`. |
| `loop_fade_ms` | `0` | Softens the moment the track loops: the ending pass fades out while the new pass fades in over this many milliseconds. `0` cuts straight back to the beginning. At most `10000`. Needs `loop = true`. Refused with `bank`. |
| `start_ms` | `0` | Where the first pass begins, in milliseconds into the file. `0` or more. Refused with `bank`. |
| `loop_start_ms` | `start_ms` | Where every repeat begins, in milliseconds into the file. Needs `loop = true`. Refused with `bank`. |
| `end_ms` | end of the file | Where each pass ends and the track jumps back. It has to lie within the file, and both start values have to lie before it. Refused with `bank`. |
| `volume` | `0` | Makes the track quieter or louder, in decibels. Negative is quieter. `0` plays the file at its own level. From `-48` up to `6`. Refused with `bank`. |
| `effects` | none | Up to four effects, see [Adding effects](#adding-effects). Refused with `bank`. |
| `room` | | The room the track belongs to. |
| `zone` | | One box or a list of boxes inside the room. Needs `room`. |
| `exclude` | none | Boxes in which the track does not play. |
| `phase` | | The phase group the track belongs to. |
| `subphases` | none | The subphases the track belongs to. |
| `global` | `false` | Plays the track everywhere. |
| `battle` | `false` | Plays the track only during a fight. |
| `scripted` | `false` | Starts the track only from a script. |

The keys from `room` down are explained under
[Deciding when it plays](#deciding-when-it-plays).

## Deciding when it plays

The keys `room`, `zone`, `phase`, `subphases`, `global`, `battle` and
`scripted` are the triggers. Give every track at least one of them. If
you set more than one, they all have to apply at the same time.

### `room`
`0x110` is the Resistance Camp. With no `zone` next to it the track
plays anywhere in that room and stops as soon as the player leaves.

To find out a room number, open the overlay and look at the Map
Manager. Its header shows the current room as `Grid (Current: 0x…)`.

### `zone`
A box inside the room. The track only plays while the player is inside
the box:

`zone = { pos = [<x>, <y>, <z>], size = [<x>, <y>, <z>], rot_y = <angle> }`

`pos` is the corner where the box begins. `size` is how far it reaches
from that corner along each axis, and every axis has to be above `0`.
It is not a radius, and the box is not centred on `pos`. `rot_y` turns
the box around the upright axis through `pos`, in radians. It is `0`
unless you say otherwise.

When one box does not fit the area, give a list of boxes instead,
`zone = [{ … }, { … }]`. The track plays while the player is inside any
of them.

A zone always needs a `room` alongside it.

### `exclude`
A list of boxes, written like `zone`, in which the track does not play
even though every other condition holds. While the player stands in
such a box, the next track that still applies plays instead. If there is
none, NAMS switches the music off there, the vanilla music included.
Stepping out brings the track back.

### `phase` and `subphases`
Ties the track to how far the story has come. A phase is a chapter of
the story, a subphase a point inside it; see
[How to add a phase or a subphase](25_phases.md). `phase` is the phase
group as a number, `0x200` for the phase `p200`. `subphases` lists
subphase names; upper and lower case do not matter. Without `phase`, a
subphase name counts in every phase.

### `global`
Plays the track everywhere. It cannot be combined with `room`, `zone`,
`phase`, `subphases` or `scripted`.

### `battle`
A second track for fights. Give it the same `room`, `zone`, `phase` or
`subphases` as your normal track and add `battle = true`; the example
under [Only one track at a time](#only-one-track-at-a-time) does this.

A `battle` track plays only while the battle music state is on: during
a fight and for a short hold after the last enemy. When the fight
starts, your normal track fades out over its `fade_ms` and the battle
track fades in over its own. When it ends, the other way round.

On its own or with `global`, `battle` plays in every fight anywhere. It
cannot be combined with `scripted`.

## Starting a track from a script

A Ruby script can start any of your tracks by name, wherever the player
is standing. Use the track's `name` from the TOML:

```ruby
call_bgm("<track name>")
```

The usual `BGM_` prefix works too, so `call_bgm("BGM_<track name>")`
starts the same track. [Patching vanilla scripts](24_ruby_patches.md)
shows how to change a script.

The track starts right at that call and keeps playing. Rooms and zones
are ignored while it does.

A call that puts on a vanilla track ends it, and the normal room and
zone rules take over again. To hand control back yourself, play a
vanilla track:

```ruby
call_bgm("BGM_Set_None_3000mm")
```

A track started this way while a [novel](40_novel.md) plays stops when
the novel closes.

### `scripted`
For a track that only ever starts from a script, set `scripted = true`
and leave the other triggers off. It never starts on its own, no matter
where the player goes, and waits for its `call_bgm`.

`scripted` cannot be combined with `room`, `zone`, `phase`, `subphases`,
`global` or `battle`. A track that does have triggers can still be
started from a script as well, and its own triggers work again once a
vanilla track has taken the music back.

## Only one track at a time

Two of your tracks never play together. When several of your blocks
apply at once, the most specific one wins, in this order:

1. `battle`, during a fight
2. a `zone`
3. a `room`
4. `subphases`
5. a `phase`
6. `global`

Each line beats any mix of the lines below it. Between two equally
specific tracks, the one whose `name` comes first in character order
wins: digits, then capital letters, then `_`, then small letters.

```toml
[[music]]
name = "<camp track>"
wem  = "<file name>"
room = 0x110

[[music]]
name      = "<camp story track>"
wem       = "<file name>"
room      = 0x110
phase     = 0x200
subphases = ["170_00_A_Resistance_Camp_2B"]
effects   = [215895407]

[[music]]
name   = "<camp battle track>"
wem    = "<second file name>"
room   = 0x110
battle = true
```

Anywhere in the camp you hear `<camp track>`. While the story is at
`170_00_A_Resistance_Camp_2B`, a subphase of `p200`,
`<camp story track>` plays instead: the same file with an effect. During
a fight in the camp, `<camp battle track>` wins over both.

## Adding effects

`effects` adds Wwise effects to a track. These ids are available:

```
215895407
1276370221
1483081688
1747525080
2330782849
2398066140
3847677565
943261234
```

Any other id is refused. You can list up to four, each once, and they
go into the track's effect slots in the order you write them:
`effects = [215895407, 943261234]`.

`effects` only works together with `wem`.

## Editing while you play

Change the `.toml` while NieR:Automata runs and save it. Every track in
that file is reloaded straight away: a playing track is cut, its old
audio is released, and your new settings apply.

For example, change `effects = [215895407]` to
`effects = [1747525080]` in a track and save again, and you hear the
difference without restarting.

Replacing the audio file works the same way. Put a new file under the
same name, then save the TOML once more so it is picked up.

Removing a block or renaming a track does not take the old track out:
it stays until you delete the file or restart. A track whose bank Wwise
refused stays silent until you restart, and so does a `bank` track
whose file could not be read.

## When nothing plays

The log is described in
[How to debug when something goes wrong](17_debugging.md).

**The log has `[NAMS] Register failed for music from mod '<mod id>': <reason>`.**
The block was refused, and the reason says what is wrong, for
example `music '<name>': a zone needs a room`.

**The log has `[Native] <file> [music]: [music][<index>] parse music: <message>`.**
The TOML of one block could not be read, and then every `[[music]]`
block in that file is dropped.

**The log has `[CustomMusic] '<name>': cannot read "<path>": <error>`.**
The audio file is not where `wem` says it is. With
`wem = "<file name>"` it has to be at `data\sound\stream\<file name>.wem`.

**The log has `[CustomMusic] '<name>': "<path>" is not a valid wem`.**
NAMS could not read the file's format header: it is not a Wwise `.wem`,
for example a `.wav` that was only renamed. It has to be converted
properly.

**The log has `[Audio] LoadBankMemoryView failed for '<name>': result=<number>`.**
Wwise refused the bank. Fix it and restart.

**A key has no effect.** A key NAMS does not know is skipped without a
message, so check its spelling.

**There is no `[CustomMusic] '<name>' started, transitions locked`
line.** The track never applied:

- The room number does not match. Check it in the Map Manager.
- The zone box does not really cover the player: `pos` is a corner,
  not the middle.
- The phase or the subphase is not the current one.
- **New music** is off. In the launcher it is on the **NAMS** tab,
  **CONTENT FEATURES**, **Audio**; in `nams.toml` it is `custom_music`
  ([How to configure NAMS](13_nams_toml.md)).

When the track loads, the log shows the file and whether looping is on,
and then the bank:

```
[CustomMusic] '<name>': wem <file name> — <sample rate> Hz, <channels> ch, <length> ms, loop=true
[Audio] loaded bank '<name>' (<size> bytes)
```

**A script track stops early with `[CustomMusic] '<name>' cancelled by '<event>'`.**
Something put on a vanilla track, and the room and zone rules took over
again.
