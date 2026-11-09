# Scripts

A list can place things and wait. It cannot ask a question, so conversations are written in
**Ruby**. The game runs a small Ruby interpreter while it plays.

The list builds the world; the script does the talking. They meet at the scene states from
the [first page](00_hap_system.md): a script sets one exactly as a
[SendCommand](SendCommand.md) would.

The methods you call inside a script each have an entry on [Script methods](scripting.md).

## The handler

Everything a script does happens in a **handler**: a few lines that run when something happens
in the world.

```ruby
def shopkeeper_ON_TALK_ev0()
    mess_ex(:GREETING)
    set_scene_state("q900/REQ")
end
```

`mess_ex` shows a line and waits until the player has read it. `set_scene_state` sets a scene
state, which is how a conversation switches on the next part of a quest.

`shopkeeper` is a name you invent. It stands for an entity placed by a list, and
[the section below](#where-shopkeeper-comes-from) is where the two get tied together. Until
then the game has never heard the name.

## Waking it up

```ruby
event_trigger("shopkeeper_ON_TALK/ev0", "shopkeeper", "ON_TALK")
```

When `shopkeeper` sends `ON_TALK`, run the handler named first. That name is the method with
its **last** underscore replaced by a slash:

```
   shopkeeper_ON_TALK_ev0   ->   shopkeeper_ON_TALK/ev0
```

> **Common mistake:** get the name wrong and nothing happens. There is no error; the handler
> is never found.

A fourth argument names a scene state that must be set for the trigger to count, so the same
entity says different things at different points in the story:

```ruby
event_trigger("shopkeeper_ON_TALK/ev1", "shopkeeper", "ON_TALK", "q900/REQ")
```

## Signals the game sends by itself

`ON_TALK` is one of a set the game sends on its own, without any list or script asking for it.
Some go to the entity it happened to, others to the global group, and that target is the second
argument of `event_trigger`.

Each one is listed on [signals the game sends](_signals.md).

## The frame

Handlers sit inside a fixed frame. Copy it and change only the marked parts:

```ruby
proxy = ScriptProxy.new()
class << proxy

    GREETING = [ "こんにちは", "Hello there", ... ]     # -- dialogue

    def shopkeeper_ON_TALK_ev0()                       # -- handlers
        mess_ex(:GREETING)
    end

    def update()
        event_trigger("shopkeeper_ON_TALK/ev0", "shopkeeper", "ON_TALK")   # -- triggers
    end

    :update
end

Fiber.new() { proxy.update() }
# STOP
```

`class << proxy`, `Fiber.new`, `:update` and `# STOP` are how the game loads a script. They
never change and never need to be understood, but `# STOP` must be there despite looking like
a comment.

`update` is not the conversation itself. It is the registration area: it tells the game which
signals should wake which handlers.

## Your first conversation

The frame above filled in: a complete script that works as it stands.

```ruby
proxy = ScriptProxy.new()
class << proxy

    GREETING = [
        "こんにちは",
        "Hello!",
        "Hello!",
        "Hello!",
        "Hallo!",
        "Hello!",
        "Hello!",
        "Hello!"
    ]

    def shopkeeper_ON_TALK_ev0()
        mess_ex(:GREETING)
    end

    def update()
        event_trigger("shopkeeper_ON_TALK/ev0", "shopkeeper", "ON_TALK")
    end

    :update
end

Fiber.new() { proxy.update() }
# STOP
```

When the player talks to `shopkeeper`:

```
   ON_TALK arrives
        │
        ▼
   the trigger from update matches
        │
        ▼
   shopkeeper_ON_TALK_ev0 runs
        │
        ▼
   mess_ex shows the greeting
```

## Dialogue

A message holds eight languages in this order: **jp en fr it de es ko zh**. All eight slots
must be filled; an empty one shows an empty box. Repeat the English if that is all you have.

The constant's last underscore carries the **speaker**, and that is what the dialogue box
displays as the name:

```
   M6020_S0010_G0090_300_resiwoman
                         └── shown as the speaker
```

Common ones are `a2b`, `a9s`, `pod042`, `pod153`, `resiwoman`, `pascal`. Any identifier works.
Write `mess_ex(:GREETING)`. The colon means the message, not the text `"GREETING"`:
`:GREETING` is a Ruby symbol, and for scripting you can read it as the name of the message.

## Where shopkeeper comes from

Ruby does not know that name. A **ScriptAction** in a list names the script and the things it
may see:

```
   ┌─ EntityLayoutArea ──────────────────────────┐
   │   layout  <layout id>   em0060  ◄───┐       │
   └─────────────────────────────────────┼───────┘
   ┌─ ScriptAction ──────────────────────┼───────┐
   │   script     <script id>            │       │
   │   variables                         │       │
   │     shopkeeper  ────────────────────┘       │
   └─────────────────────────────────────────────┘
```

The layout is what actually stands in the world; the ScriptAction says that the script's
`shopkeeper` means that layout. The name exists only there, so pick anything. The script just
has to match.

An action usually also hands in itself as `this`, so a script can switch it off when done:
`disable(this())`.

## When a name is wrong

`mess_ex` and the rest are defined in no Ruby file: unknown names get passed to the game, and
a name nothing recognises is an error. With NAMS's `validate_scripts` setting on you get a
dialog naming the script and method; with it off it closes the game.

## What may I rename?

A script is full of names, and it is not obvious which are yours and which belong to the game:

| Name         | Can I change it?                                       |
| ------------ | ------------------------------------------------------ |
| `shopkeeper` | Yes, but the ScriptAction must use the same name      |
| `GREETING`   | Yes                                                    |
| `ev0`        | Yes, as long as handler and trigger still match       |
| `ON_TALK`    | No, this is a [signal the game sends](_signals.md)     |
| `q900/REQ`   | No, unless you defined it in the quest's `quest.toml` |
| `update`     | No                                                     |
| `# STOP`     | No                                                     |

## Nams

Things the original game has no equivalent for:

```ruby
Nams.add_exp(500)
Nams.add_gold(1000)
Nams.equip_outfit_mod("my_outfit")
Nams.unequip_outfit_mod()
Nams.equip_buddy_outfit_mod("my_outfit")
Nams.reset_buddy_outfit_mod()
Nams.tint_entity(scene_id, argb)
Nams.color_mesh("mesh_name", scene_id, argb)
```

---

Next: [Script methods](scripting.md), every method a script can call.
