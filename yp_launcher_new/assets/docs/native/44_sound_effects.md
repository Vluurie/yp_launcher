# How to add your own sound effect

Put an audio file in your mod and write a few lines of TOML. NAMS builds
a Wwise sound bank around the file and registers two events: `<name>`
plays the sound and `<name>_Stop` stops it. An event is the name a
sound is played by; Wwise is the audio system NieR:Automata plays its
sound with.

## The short version

```toml
[[sound]]
name = "<sound name>"
wem  = "<file name>"
```

Put this into any `.toml` file in your mod's `entities\` folder, and the
audio file at `data\sound\stream\<file name>.wem`
([How to lay out your mod folder](02_mod_folder_layout.md)). The parts
in angle brackets are your own names.

NAMS loads the sound when the first room loads. From then on a plugin
plays it with `play_sound("<sound name>")` and stops it with
`play_sound("<sound name>_Stop")`.

## Which audio file works

A Wwise `.wem` encoded as Vorbis, ADPCM or PCM. NAMS copies the whole
file into the sound's bank.

## `[[sound]]`

| Key | Default | What it does |
|---|---|---|
| `name` | required | The name of the sound. With `wem` it is also the event that plays it, and then only ASCII letters, digits and `_` are allowed. Every sound needs its own name, and two sounds cannot play the same event. |
| `wem` | | The audio file in `data\sound\stream`, without `.wem`. Use either `wem` or `bank`, never both. |
| `volume` | `0` | Quieter or louder, in decibels, from `-48` to `6`. `0` keeps the file's own level. |
| `loop` | `false` | `true` repeats the sound until `<name>_Stop` is played. |
| `fade_ms` | `0` | How long the sound fades in when it starts and out when it is stopped, in milliseconds. `0` starts and stops at once. |
| `bus` | `world` | The Wwise bus the sound plays on. A bus is a channel of the Wwise mix. One of `world`, `enemy`, `weapon`, `item`, `player` and `menu`. |
| `effects` | none | Up to four of the effects listed in [Adding effects](27_custom_music.md#adding-effects), each once. |
| `bank` | | A Wwise bank you built yourself, as a path inside your mod folder. |
| `events` | | With `bank`: the events inside the bank, which are then played by their own names. Required with `bank`, refused with `wem`. A name cannot be empty. |

## A bank you built in Wwise

If you already work with Wwise, give the bank and the events inside it
instead of `wem`:

```toml
[[sound]]
name   = "creeper"
bank   = "data/sound/creeper_sfx.bnk"
events = ["creeper_fuse", "creeper_death"]
```

`volume`, `loop`, `fade_ms`, `bus` and `effects` are refused together
with `bank`; a bank carries its own settings.

## Changing a sound

`[[sound]]` is not reloaded while you play. Saving the file again logs
`[NAMS] hot register failed for sound '<name>': duplicate sound '<name>'`;
restart to apply the change. A sound that failed to load is not tried
again before a restart.

## When it does not play

The log is described in
[How to debug when something goes wrong](17_debugging.md).

**`[Audio] sound '<name>': cannot read <path>: <error>`.** The audio
file is not where `wem` says it is. With `wem = "<file name>"` it has to
be at `data\sound\stream\<file name>.wem`.

**`[Audio] sound '<name>': <path> is not a wem`.** NAMS found no format
header in the file.

**`[Audio] sound '<name>': <path> uses format 0x<format>, which no sound plugin decodes`.**
Encode the file as Vorbis, ADPCM or PCM.

**`[Audio] failed to read "<path>": <error>`.** The file `bank` names
could not be read. `bank` is a path inside your mod folder.

**`[Audio] LoadBankMemoryView failed for 'nams_sound_<name>': result=<number>`.**
Wwise refused the bank.

**`[NAMS] Register failed for sound from mod '<mod id>': <reason>`.**
The block was refused, and the reason says what is wrong, for example
`sound '<name>' needs either bank or wem`.

**`[Native] <file> [<kind>]: unknown entity kind '<kind>'`.** The table
header is misspelled; it is `[[sound]]`.

**There is no `[Audio] loaded bank 'nams_sound_<name>' (<size> bytes)`
line.** No room has loaded yet, or **New sound effects** is off. In the
launcher it is on the **NAMS** tab, **CONTENT FEATURES**, **Audio**; in
`nams.toml` it is `custom_sounds`
([How to configure NAMS](13_nams_toml.md)). A key NAMS does not know is
skipped without a message, so check its spelling.
