# How to translate your mod

Every text block takes language codes:

| Code | Language |
|---|---|
| `""` | Japanese |
| `_us` | English |
| `_de` | German |
| `_fr` | French |
| `_it` | Italian |
| `_es` | Spanish |
| `_kr` | Korean |
| `_cn` | Chinese (Simplified) |
| `_tw` | Chinese (Traditional) |

Provide whichever ones you have. A text shows in the language the player
plays in. Korean and both Chinese languages show the `_us` text where
their own code is missing.

```toml
[text.name]
""  = "Japanese title"
_us = "English title"
_de = "German title"
```

## How do I change one single text?

A UI's texts come from its message file (`.mcd`), where every text has
an event name. A `[[localized_string]]` row sets the text of one event
name in one language: it replaces the text with that event name, or
adds it, in every message file that is loaded. The UI editor writes
these rows for the texts you set on a part's message layers (the layers
that show text), see [How to change a menu or HUD layout](45_ui_override.md).
To translate such a text, add a row with the same `event_name` and
another `lang`.

```toml
[[localized_string]]
event_name = "<event name>"
lang = "_us"
text = "<English text>"

[[localized_string]]
event_name = "<event name>"
lang = "_de"
text = "<German text>"
```

| Field | Required | What it does |
|---|---|---|
| `event_name` | yes | The event name of the text. Upper and lower case letters count as the same. |
| `lang` | yes | A language code from the table above. |
| `text` | yes | The text. Each line break starts a new line; line breaks at the start and at the end are dropped. |
| `font_id` | no | Draws the text in this font as one block, unless another font of the message file has more of its characters. Without it, the text keeps the layout and fonts of the text it replaces; a new event name takes them from a text whose event name is the same up to the last `_`. |

Each event name can have one row per language across all mods.

## How do I give a new character a name?

`entities\char_name_<key>.toml` adds the character key `<key>`. Its
`[display]` block holds the name per language code from the table
above; `_ct` also works for Chinese (Traditional) here, and other codes
are skipped. To use the key in a quest, see
[how to name a new character](07_quest.md#how-do-i-name-a-new-character).

## Why is my text missing?

**It is missing in one language.** That language has no text in your
mod. Add it under its code; Korean and Chinese show the `_us` text
instead of a missing one.

**No row of a file loads.** The log shows

```
[Native] <file> [localized_string]: [localized_string][<n>] parse localized_string: <reason>
```

Row `<n>`, counted from 0, has a missing field or a wrong value. One
bad row stops every `[[localized_string]]` row of that file.

**Letters are missing.** The font has no letter for them, so they are
left out. When one line of a text keeps no letter at all, the whole
text is left out.

**A change does not show until you restart.** Rows and character names
that are already loaded are not replaced while you play. The log shows

```
[NAMS] hot register failed for localized_string '<event name>:<lang>': duplicate localized_string for (<event name>, <lang>)
[NAMS] hot register failed for char_name_bundle '<key>': duplicate char_name for key '<key>'
```

Restart to load the change.

**Two mods set the same text.**

```
[NAMS] Two mods try to use the same localized_string number '<event name>:<lang>'.
```

The lines below it name the mod that keeps it.

**A character has no name.**

```
[NAMS] Register failed for char_name_bundle from mod '<mod id>': char_name '<key>' has no [display] names
[NAMS] char_name '<key>' has no recognized language suffixes; skipping
```

The `[display]` block is missing or empty, or none of its codes is in
the table above.
