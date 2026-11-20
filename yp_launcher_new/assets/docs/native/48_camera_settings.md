# How to change the player camera

A `[player_camera_settings]` table in a mod shifts the player camera by
fixed offsets and sets how far the player can turn it up and down. One
set of camera settings is active at a time.

## How do I make one?

Put the table into a TOML file in your mod's `entities` folder (see
[how to lay out your mod folder](02_mod_folder_layout.md)). Name the
file `camera.toml`: the launcher's mod summary then lists it as
`player_camera_settings`.

```toml
[player_camera_settings]
name             = "camera"
side_offset      = 0.0
height_offset    = 0.0
distance_offset  = 0.0
fov_offset       = 0.0
pitch_up_limit   = 72.0
pitch_down_limit = 72.0
```

These are the values the launcher's sliders show for a field the table
leaves out. Change one and save.

## When does the camera change?

At once. Saving the file while you play reads it again and the camera
uses the new values. The log says:

```
[NAMS] hot register <file>: 1 registered, 0 failed, <N> purged
```

In the launcher, the details of a mod with this table show a **Camera**
section with a slider for each field. A change is written into the
file, so a running session picks it up at once. The sliders find the
table only when it is written as `[player_camera_settings]`, not as
`[[player_camera_settings]]`.

Deleting the file switches the settings off. Taking the table out of a
file that stays does not; that takes effect at the next start.

## Why does the camera not change?

- **`... is not applied: '<name>' from mod '<mod>' is already active`**
  Another set of camera settings is already active, and only one can
  be. At start the mod folders are read in name order, and the first
  set wins. At start the line begins with
  `[NAMS] Register failed for player_camera_settings from mod '<mod>':`,
  after a save while playing with
  `[NAMS] hot register failed for player_camera_settings '<mod>/<name>':`.
- **`pitch limits must be above 0 and below 90 degrees`** A pitch limit
  is 0 or less, or 90 or more. The settings are refused.
- **`[Native] <file> [player_camera_settings]: parse player_camera_settings: <reason>`**
  `name` is missing, a value is not a number, or an offset is `nan` or
  `inf`.
- **No log line, one value does nothing.** A misspelled field name is
  ignored without a message.
- **No camera settings from any mod.** The **Camera settings** switch is
  off (`camera_settings` under `[content]` in `nams.toml`, see
  [nams.toml](13_nams_toml.md)). It takes effect after a restart.
- **The field of view stays as it is.** While the field of view plus
  `fov_offset` would be 0° or less, or 180° or more, the offset is left
  out.
- **During a multiplayer round** the camera settings are not applied.

## What does each field do?

| Field | Left out | What it does |
|---|---|---|
| `name` | needed | A name for the settings. The log names them by it. |
| `side_offset` | `0` | Meters. Positive moves the camera right, negative left. |
| `height_offset` | `0` | Meters. Positive moves the camera up, negative down. |
| `distance_offset` | `0` | Meters. Positive moves the camera away from the player, negative closer. |
| `fov_offset` | `0` | Degrees added to the field of view. Positive widens the view, negative narrows it. |
| `pitch_up_limit` | 72° | Degrees above level the player can turn the camera up to. Above 0 and below 90. |
| `pitch_down_limit` | 72° | Degrees below level the player can turn the camera down to. Above 0 and below 90. |

A `pitch_limit` under `[internal]` in `naiom.toml` can change the 72° a
left-out limit falls back to. A boss camera, or a camera mode that sets
a range of its own for a while, can only narrow the limits further.
