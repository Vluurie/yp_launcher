# How to make a mail

`entities\mail_<id>.toml` adds a mail with the number `<id>`. A mail
belongs to a quest from a loaded mod, see
[How to make a quest](07_quest.md). NAMS adds the mail to the mail
list only when the part of its `state_after` before the first `/` is
that quest's `name`; when `state_after` is empty, `state_before` is
used. A mail without such a state is never added.

Replace every `<…>` with your own value:

```toml
sender = "<sender>"
send_2b = true
send_9s = true
send_a2 = true
quest_id = <quest id>
state_before = "q<quest id>/<state>"
state_after = "q<quest id>/<state>"

[text.subject]
_us = "<subject>"

[text.body]
_us = """<first line>
<second line>"""
```

## Which fields does a mail have?

| Field | Default | What it does |
|---|---|---|
| `id` | number in the file name | The mail's number. NAMS keeps the mail list sorted by number. |
| `sender` | required | Written into the mail as its sender. The first 15 characters are kept. |
| `send_2b`, `send_9s`, `send_a2` | `false` | Set the mail's 2B, 9S and A2 send-target flags. |
| `attachment_item_id` | none | Written into the mail as its attached item. Must be a [vanilla item](resources/vanilla_item_ids.md) or an item from a loaded mod, otherwise the mail is skipped. |
| `quest_id` | none | Id of a quest the mail depends on. When that quest is not loaded, because it is missing, its file is broken or it lost an id clash, the mail is skipped too. It does not decide which quest the mail belongs to. |
| `state_before` | `""` | Written into the mail. The first 63 characters are kept. |
| `state_after` | `""` | Written into the mail. The first 63 characters are kept. |
| `[text.subject]`, `[text.body]` | none | Subject and body per language code from [How to translate your mod](12_translate.md). A language without its own text gets the `_us` text, then the `""` text. |

## How do I write a text over several lines?

Use triple quotes. NAMS passes the text on with its line breaks,
empty lines included.

```toml
[text.body]
_us = """<first line>
<second line>

<line after an empty line>"""
```

## Why is my mail missing?

**The mail is never added.** Check:

- The part of `state_after` before the first `/`, or of `state_before`
  when `state_after` is empty, is the `name` of a quest from a loaded
  mod.
- `custom_mails` and `custom_quests` are not `false` in the `[content]`
  table of [nams.toml](13_nams_toml.md).

**The log names your mail.**

```
[Native] <file> [mail_bundle]: parse mail_bundle: <reason>
```

`sender` is missing or a field has the wrong type. Nothing of the file
loads.

```
[NAMS] Skipping mail_bundle '<id>' from mod '<mod id>' because the quest '<quest id>' it depends on (from mod '<unstaged>') is not loaded.
```

No loaded mod has a quest with the id in `quest_id`. With
`item '<item id>'` the line names an unknown `attachment_item_id`. When
the quest was loaded but then skipped, the line names
`quest_bundle '<quest id>'` and the quest's mod.

```
[NAMS] Two mods try to use the same mail_bundle number '<id>'.
```

Another mod has a mail with this number. The lines below it name the
mod that keeps it.
