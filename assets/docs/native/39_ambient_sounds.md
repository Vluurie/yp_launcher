# How to add a background sound or silence an existing one

A background sound is a loop that belongs to a room's sound areas. A
sound area is made of layers; each layer holds shape groups, and each
shape group holds shapes in the room. A shape group names the event
that starts its loop when the player enters one of its shapes
(`InEvent`) and the event that stops it when the player leaves
(`OutEvent`). An event is the name a sound is played by in Wwise, the
audio system NieR:Automata plays its sound with.

`[[ambient_mute]]` switches such a loop off by its `InEvent`.
`[[ambient]]` adds a shape group of your own that plays an audio file
from your mod.

## The short version

```toml
[[ambient_mute]]
event = "<InEvent of a loop>"

[[ambient]]
name = "<loop name>"
wem  = "<file name>"
room = 0x110
zone = { pos = [<x>, <y>, <z>], size = [<x>, <y>, <z>] }
```

Put this into any `.toml` file in your mod's `entities\` folder, and the
audio file at `data\sound\stream\<file name>.wem`
([How to lay out your mod folder](02_mod_folder_layout.md)). The parts
in angle brackets are your own values.

The loop with that `InEvent` stays silent. In the Resistance Camp, room
`0x110`, your own loop plays while the player is inside the box.

The audio file is a Vorbis-encoded Wwise `.wem`, the same kind of file
as for music: [Which audio file works](27_custom_music.md#which-audio-file-works).

## Which loop does `event` name?

`event` is the `InEvent` of the shape group that plays the loop, as it
is written in the room's sound-area data. Upper and lower case do not
matter. The mute applies in every loaded sound area that uses that
event.

## `[[ambient_mute]]`

| Key | Default | What it does |
|---|---|---|
| `event` | required | The `InEvent` of the loop. One block per event: a second block with the same `event` is refused. |
| `phase` | none | Mutes only while the story is in this phase group, `0x200` for the phase `p200`. Without `phase` and `subphases` the loop is always muted. |
| `subphases` | none | Narrows the mute to these subphases. Needs `phase`. A name cannot be empty; upper and lower case do not matter. |

A phase is a chapter of the story, a subphase a point inside it; see
[How to add a phase or a subphase](25_phases.md).

NAMS applies the mutes when a room's sound areas load, and again
whenever the phase or subphase changes. A loop that is playing when its
mute starts to apply is stopped. When the mute stops applying, NAMS
lifts it again.

## `[[ambient]]`

| Key | Default | What it does |
|---|---|---|
| `name` | required | A unique name. ASCII letters, digits and `_` only. |
| `wem` | required | The audio file in `data\sound\stream`, without `.wem`. |
| `room` | required | The room the loop belongs to, `0x110` for the Resistance Camp. |
| `zone` | | One box, with the same keys as a `[[music]]` [zone](27_custom_music.md#zone): `pos`, `size` and `rot_y`. Every `size` axis has to be above `0`. |
| `path` | | A line shape instead of a box: `points`, a list of at least two `[x, y, z]` positions, with `edge_radius` (above `0`) and `core_radius` (`0` or more). |
| `exclude` | none | Boxes with the same keys as `zone`, added to the shape group as holes in which the loop stays silent. |
| `phase` | none | NAMS switches the loop's layer of the sound area on only while the story is in this phase group. |
| `subphases` | none | Narrows `phase` to these subphases. Needs `phase`. A name cannot be empty; upper and lower case do not matter. |
| `volume` | `0` | Quieter or louder, in decibels. Negative is quieter. `0` plays the file at its own level. From `-48` up to `6`. |
| `fade_ms` | `3000` | How long the loop takes to fade in when it starts and out when it stops, in milliseconds. `0` cuts hard. |
| `loop_fade_ms` | `0` | Softens the moment the file starts over by overlapping its end with its beginning, in milliseconds. At most `10000`. |

Every loop needs a `zone` or a `path`, never both. A loop always
repeats. `[[ambient]]` has no keys for fights, scripts or effects;
[`[[music]]`](27_custom_music.md) has them.

## Editing while you play

Save the `.toml` and the old audio of every `[[ambient]]` block in it is
released; NAMS builds the loop again the next time its room loads. A
changed `[[ambient_mute]]` takes hold the next time a room's sound areas
load or the phase or subphase changes.

Removing a block does not take it out: it stays until you delete the
file or restart.

## When the sound does not change

The log is described in
[How to debug when something goes wrong](17_debugging.md).

**You still hear a muted loop.** Check that `event` is exactly the
`InEvent` of the loop, apart from upper and lower case, and that
`phase` and `subphases` match the current story point. A mute saved
while you are in the room takes hold when the room's sound areas load
again or the phase changes.

**Your loop does not play, and after the room loaded there is no
`[Audio] loaded bank 'ambient_<name>' (<size> bytes)` line.** The file
at `data\sound\stream\<file name>.wem` is missing or is not a `.wem`
NAMS can read. NAMS skips that loop without a message of its own.

**The log has `[NAMS] Register failed for ambient from mod '<mod id>': <reason>`**
(or `for ambient_mute`). The block was refused, and the reason says what
is wrong, for example `ambient '<name>' needs a zone or a path`.

**The log has `[Native] <file> [ambient]: <message>`.** The TOML of one
block could not be read, and then every `[[ambient]]` block in that
file is dropped. The same goes for `[ambient_mute]`.

**Nothing changes at all.** Check that **Ambient sounds** is on. In the
launcher it is on the **NAMS** tab, **CONTENT FEATURES**, **Audio**; in
`nams.toml` it is `ambient_sounds`
([How to configure NAMS](13_nams_toml.md)). A key NAMS does not know is
skipped without a message, so check its spelling.
