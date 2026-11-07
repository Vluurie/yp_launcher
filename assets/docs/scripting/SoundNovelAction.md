# SoundNovelAction

Plays a novel: a text file shown page by page in a text window, with choices.

The text is a file of its own. Its tags (pages, choices, labels, scene states, text style,
pictures, music) are described on the novel page of the NAMS content docs.

```
   ┌─ SoundNovelAction ──────────────────────────┐
   │   filename   M6010_S0050_N.txt              │
   │   type       0                              │
   │   attribute  0x1000                         │
   └─────────────────────────────────────────────┘
```

## Start here

1. Set `filename` to the name of the text file, `.txt` included.
2. Set `type` to `0`.
3. Set the [attribute](02_action.md) to `0x808`: the novel starts as soon as the list reaches
   the action, and the list waits until it is over.

## The cycle

```
   played
     │
     ▼
   LOADING ──────────────────────────► FAILURE   the file is missing or unreadable
     │  the file arrived
     ▼
   PLAYING ──────────────────────────► FAILURE   the text stopped on an error,
     │                                           or it was closed before the end
     │  <EXIT> or the end of the file
     ▼
   DONE
```

Nothing happens until the action is played, either by attribute `0x800` or by `play`.

**FAILURE** comes from a missing file, from an unknown tag in the text, from a `sele`, `go` or
`then` whose label does not exist, and from `close` while the text is still running. Its
number is always 0.

**DONE** is reached at `<EXIT>` or when the text runs out. That is also the moment the flag
below is switched on.

## filename

The text file. The text language picks the file that is actually read:

| Text language | File read for `M6010_S0050_N.txt` |
|---|---|
| English (US) | `M6010_S0050_N_ENG.txt` |
| French | `M6010_S0050_N_FRA.txt` |
| Italian | `M6010_S0050_N_ITA.txt` |
| German | `M6010_S0050_N_GER.txt` |
| Spanish | `M6010_S0050_N_ESP.txt` |
| every other language | `M6010_S0050_N.txt` |

Keep the `.txt` in the field. A name without an extension still plays (`.txt` is added for
reading), but the flag below is then never set.

## The flag at the end

On `DONE` the action switches on the scenario flag named like `filename` without its
extension, `M6010_S0050_N` for the one above. A scenario flag is a named on/off value kept in
the save, separate from scene states.

With NAMS, a custom novel shows up under Intel → Novels once the flag with its name is on. A
`SoundNovelAction` whose `filename` is that name plus `.txt` therefore unlocks it as soon as
the player has read it to the end.

## type

Where the text appears.

| type | Window |
|---|---|
| 0 | the novel window, the same one as under Intel → Novels |
| 1 | the loading screen interface instead of the novel window |

Pictures from `BGLOAD` and `BGON` only show with `0`.

## Signals from the text

A `<SIG…>` tag in the text sends a signal from this action the moment the text reaches it.
Everything between `SIG` and `>` is the signal's name, so `<SIG_A>` sends `_A`. A script or a
[ConditionBlock](ConditionBlock.md) waiting for a signal from this action reacts to it.

No vanilla novel uses it.

## Playing it from a script

Without `0x800` the action waits until something plays it. Vanilla plays three novels about
Anemone from a script when the player checks an object named `novel`. The script holds the
actions in its [variables](ScriptAction.md#variables):

```ruby
def novel_ON_CHECK_ev1()
	self.call(:fadestart)
	self.reset(self.novel_1())
	self.novel_1().play()
	self.wait(30)
	self.inout().fadeIn()
	self.wait_state("DONE", self.novel_1())
	self.call(:fadeend)
	...
```

`reset` first, so a novel that already reached `DONE` can play again. These three carry
`0x1000`, so their window stays open after the last page, and a later step of the same script
closes it with `self.novel_1().close()`.

| Method | What it does |
|---|---|
| `play` | loads the file and starts from the top; while it is already running, starts over |
| `close` | closes the window and drops the text; before `DONE` this ends in `FAILURE` |
| `jumpTo(label)` | while `PLAYING`, continues at `<label_NAME>` with `NAME` = `label`; does nothing if there is no such label |

## Commands

The same three can be sent by a [SendCommand](SendCommand.md) pointing at the action:

| Command | What it does |
|---|---|
| `SoundNovelAction.play` | same as `play` |
| `SoundNovelAction.close` | same as `close` |
| `SoundNovelAction.jumpTo` | same as `jumpTo`, with the label name in `args` |

Vanilla uses `jumpTo` to cut a running novel short once a scene state is set. The novel
carries `0x1c08`, so the list carries on while it plays, a [ConditionBlock](ConditionBlock.md)
waits for the scene state, and then this jumps the text to `<label_END>`:

```xml
<action>
	<code str="SendCommand">0xa10e3dbc</code>
	<name eng="Command send label jump">コマンド送信ラベルジャンプ</name>
	<id>0x457fb5cb</id>
	<attribute>0x0</attribute>
	<puid>
		<code str="hap::Action">0xa6aaf7a4</code>
		<id>0x25a7c588</id>
	</puid>
	<command>
		<label>SoundNovelAction.jumpTo</label>
	</command>
	<args>END</args>
</action>
```

## Attributes

Beyond the usual ones, this action reads three of its own.

`0x800` plays the novel as soon as the list reaches the action. Without it nothing happens
until `play`.

`0x400` makes a blocking action release at `PLAYING` instead of `DONE`, so the list carries on
while the novel is still running.

`0x1000` keeps the window open after the last page, until `close`. Without it the window
closes by itself at the end.

## A vanilla one

```xml
<action>
	<code str="SoundNovelAction">0xa8c59a0e</code>
	<name eng="Sound Novel_Anemone 1 _">サウンドノベル_アネモネ1</name>
	<id>0xc2be04a</id>
	<attribute>0x1000</attribute>
	<filename>M6010_S0050_N.txt</filename>
	<type>0</type>
</action>
```

Every vanilla one carries the same set of fields, so nothing is missing from this one.
