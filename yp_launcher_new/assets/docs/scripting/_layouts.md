# Layouts

A layout is one entity standing in one place: which entity it is, where it stands,
and how it behaves.

Layouts do not belong to a single action. Several kinds of action carry them, and they all
work the same way.

These carry layouts:

| Action | |
|---|---|
| [`EntityLayoutAction`](EntityLayoutAction.md) | places entities at fixed spots |
| [`EntityLayoutArea`](EntityLayoutArea.md) | the same, tied to a region |
| `EnemySetAction` | places enemies at fixed spots |
| `EnemySetArea` | the same, tied to a region |
| `SQ090_Layout` | a special case used by one quest |

Two more place entities without using layouts.
[`EnemyGenerator`](EnemyGenerator.md) picks from a weighted pool and gets its positions from
spawn points instead. `EntityLayoutLottery` places nothing itself: it points at one of the
actions above and picks some of its layouts at random.

## What one layout carries

```
   ┌─ layout ────────────────────────────────────┐
   │  which one   em0060                         │
   │  position    12.4  0.0  -88.1               │
   │  rotation    0  90  0                       │
   │  scale       1  1  1                        │
   │  delay       0.0                            │
   └─────────────────────────────────────────────┘
```

**Which one.** Every entity in the game has an identifier such as `em0060`. An editor
shows a readable name next to it.

**Position and rotation.** Where it stands and which way it faces, three numbers each.

For trying something out, `256.99 9.44 657.14` is worth keeping around: the middle of the
resistance camp, on solid ground and easy to walk to.

**Scale.** Defaults to normal size.

**Delay.** How long after the list starts this entity appears. Zero means straight away.

## Placement relative to something else

A layout can be attached to another object rather than to the world. The position is then
measured from that object instead of from the map origin, and there is a slot for naming
which part of it to attach to.

This is how something can be placed on a moving platform, or on a specific limb of a large
enemy.

## Extra settings

Beyond position and identity, a layout can carry named settings. They are optional, and an
entity without any behaves normally.

These are the ones the game reads:

| Setting | What it does |
|---|---|
| `Lv` | The entity's level |
| `Lv_B`, `Lv_C`, `Lv_D` | A different level, used only on route B, C or D |
| `MaxLv` | An upper limit on the level |
| `HP_MAX` | Sets the health directly |
| `HP_MIN` | A floor on health, given as a fraction. At 0.5 the entity cannot drop below half |
| `HP_BAR_OFF` | Hides the health bar |
| `NO_HACK` | 9S cannot hack this entity |
| `LOOK` | Also feeds into whether hacking is allowed |
| `NO_SCARE` | The entity does not flinch when hit |
| `SUPER_ARMOR` | Behaves as if it had just been hit by a heavy attack |
| `EVENT_AUTO_KILL` | The entity is removed automatically when an event ends |
| `CAPTION_NO_STOP` | The entity keeps moving in situations where others are frozen |
| `ANIM_FRAME` | Where in its animation the entity starts, as a fraction between 0 and 1 |
| `path`, `wait` | A route to walk, and how long to pause along it |
| `ParamSave` | Remembers this entity's condition across a save. See [save state](_save_state.md) |
| `UniqueID` | A group marker. Sub-units spawned by this entity inherit it |
| `Share` | Marks the entity as a target |
| `EMP`, `AI_pure` | Read by the game, but what they change could not be established |

## What one layout looks like in the file

This is the shape you see in the original content and write in hand-made XML. One `<value>`
is one layout:

```xml
<value>
	<id>0x5ee6f7dc</id>
	<location>
		<position>255 9.48 650</position>
		<rotation>0 5.235988 0</rotation>
	</location>
	<objId>em0007</objId>
	<alias>em1</alias>
	<param>
		<count>0x2</count>
		<value>
			<name>Lv</name>
			<code str="int">0x1451dab1</code>
			<body>27</body>
		</value>
		<value>
			<name>Lv_B</name>
			<code str="int">0x1451dab1</code>
			<body>45</body>
		</value>
	</param>
</value>
```

`objId` is which entity, `location` where it stands, and rotation is in radians rather than
degrees: the `5.235988` above is 300°.

The named settings from the table sit in `param`, one `<value>` each, with `count` saying how
many follow, and `body` carrying the value.

`alias` is a name you give this one entity, and the game can find it again by that name from
anywhere. It does not matter which action the layout sits in, and if the entity is not
standing in the world yet, looking it up puts it there.

Cutscenes work like this: a cutscene asks for its actors by name, and whoever carries that
alias plays the part.

Give each one its own name. If two layouts share an alias, only the first one is ever found.

`setType` and `setFlag` are written as their own tags next to `objId` when used:

```xml
	<objId>em0017</objId>
	<setType>1</setType>
```

Three tags of that kind exist. `setType` picks a variant of the entity; which one depends on
the entity. `setRtn` is the animation the entity starts in when it is placed. `setFlag` is
the subject of the next section.

## Turning an enemy into someone you can talk to

An entity placed through a layout is an enemy by default. Talking to it does nothing.

To make it a conversation partner, add `0x80000` to its `setFlag`. That also stops 9S from
hacking it.

```
   without 0x80000               with 0x80000
   ┌────────────────────┐        ┌────────────────────┐
   │  attacks you       │        │  can be talked to  │
   │  can be hacked     │        │  cannot be hacked  │
   └────────────────────┘        └────────────────────┘
```

`setFlag` is a sum of switches, like the [attribute](02_action.md) on an action, so `0x80000`
is added to whatever is already there rather than replacing it.

The other switches are not a shared list. Each kind of entity reads them for itself, so the
same bit can mean one thing on one entity and something else on another.
