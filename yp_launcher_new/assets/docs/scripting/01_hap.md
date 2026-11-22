# The HAP

## What sits around the instructions

Alongside its instructions, a list carries a handful of values.

```
   ┌─ HAP ───────────────────────────────────────┐
   │  name       "bridge robot"                  │
   │  comment    "for the desert quest"          │
   │  owner      "vluurie"                       │
   │  priority   0                               │
   │  group      q900/REQ                        │
   │                                             │
   │  1.  Place a robot at this position         │
   │  2.  Wait until the player talks to it      │
   │  3.  Open the door                          │
   └─────────────────────────────────────────────┘
```

Name, comment and owner are never read by the game. They exist so a person can tell one
list from another six months later.

The group is the switch from the previous page.

Priority decides who goes first when two lists come up in the same moment, with the higher
number acting earlier. It is a whole number and may be negative. The original game mostly
stays between 0 and 50, uses 500 for a handful of lists that must run before everything else,
and goes negative for ones that should run last.

Two lists with the same priority have no guaranteed order between them, so if the order
matters, give them different numbers. Lists that never come up together do not need it at
all, and most leave it out.

## Ids

Every list carries a number the game uses to refer to it, and so does every instruction
inside it. You never invent these: they are handed out when the file is loaded, and if two
ever came out the same, the game hands one of them a fresh number.

You do need them for the other direction. An instruction that reaches out to something else,
such as switching a list off or sending it a command, has to name that list by its number.
There is no way to point at something by its name, because names are only there for people.

```
   ┌─ list "bridge robot"  id 0x51c116d9 ────────┐
   │  ...                                        │
   └─────────────────────────────────────────────┘

   ┌─ another list ──────────────────────────────┐
   │  1.  send DISABLE to 0x51c116d9   ◄── has   │
   │                                       to be │
   │                                       named │
   └─────────────────────────────────────────────┘
```

An editor takes that job over: it shows the readable name and fills in the number behind it.

## Groups can be read either way round

A group is either on while its scene state is set, or on while its scene state is **absent**
and off once it gets set. Which of the two a group uses is part of the group.

That second direction is what lets one scene state swap a place from one version to another.
The resistance camp gets attacked partway through the story, and there is a scene state for
it, `RC_NPCLayout_dieNPC`:

```
   ┌─ group "before the attack" ───────────────────────┐
   │  on while RC_NPCLayout_dieNPC is NOT set          │
   │     the camp as it normally is                    │
   └───────────────────────────────────────────────────┘

   ┌─ group "after the attack" ────────────────────────┐
   │  on while RC_NPCLayout_dieNPC IS set              │
   │     the camp with its casualties                  │
   └───────────────────────────────────────────────────┘
```

Setting that one scene state switches the first group off and the second one on, so the camp
changes over in a single moment.

Without the second direction, every such scene state would need an opposite twin, and the two
would have to be kept in step by hand.

## Nested groups

Groups sit inside other groups, so they form a tree. The resistance camp in the original
game looks like this, with the scene states each group waits for written next to it:

```
   Resistance Camp
    ├─ Dying NPCs           on while RC_NPCLayout_dieNPC or RC_NPCLayout_noNPC is set
    ├─ Surviving NPCs       on while RC_NPCLayout_noNPC is set
    ├─ room lock command    on while RC_onRoom is set
    ├─ resistance leader    on while q910/RUN is set
    ├─ jukebox              on while StopJukeBox or Serious_RC is set
    ├─ novel terminal       on while neither AneMemory_Lock nor AneMemory_Unlock is set
    ├─ q720 flowers         on while none of q720/FIN, q720/CLR, q720/DONE is set
    ├─ Warehouse
    └─ ladder
```

Each group watches its own scene states, in its own direction. Setting `q910/RUN` switches on
the resistance leader and nothing else, and the jukebox is unaffected by anything the flowers
do.

Warehouse and ladder carry no scene states at all. Those are pure folders, there to keep
related lists together.

Nesting only adds one thing: a group also counts as off when a group above it is off. The
game first looks at the group itself, and only if it finds no reason to be off there does it
ask the group above.

That makes the outer group a master **off** switch, and it is a blunt one: a parent that is
off takes every child down with it, whatever their own scene states say.

```
   Resistance Camp                     off
    ├─ before the attack               off  ◄── even though its scene state says on
    └─ after the attack                off  ◄──
```

The other direction does not exist. A parent that is on forces nothing: each child still
decides for itself. So the pure folders from the tree above, which have nothing to watch and
are never off, change nothing for the groups inside them.

## Lists that ignore their group

Not every list waits for a switch. A list can be marked as always running, and then the
group is never consulted at all.

```
   normal list                        always-running list
   ┌────────────────────────┐         ┌────────────────────────┐
   │  group q900/REQ    off │         │  group q900/REQ    off │
   │                        │         │                        │
   │  nothing happens       │         │  runs anyway           │
   └────────────────────────┘         └────────────────────────┘
```

The original game uses this for things that must work regardless of the state the story is
in.

## The list's attribute

The list itself carries an attribute, just as its actions do, and two values of it are
written by hand.

**`0x2`** makes the list start switched off, whatever its group says. Something else has to
turn it on: a [SendCommand](SendCommand.md) aimed at the list, with `ENABLE` as the command.
`DISABLE` switches it off again, and `RESET` sends it back to its first line.

The background airship from the [ConditionBlock](ConditionBlock.md) pages ends exactly this
way. The last line of its own list sends `DISABLE` to the list it sits in:

```xml
<action>
	<code str="SendCommand">0xa10e3dbc</code>
	<name eng="Send command_airship disabled">コマンド送信_飛行船無効</name>
	<id>0xdf2240e</id>
	<attribute>0x0</attribute>
	<puid>
		<code str="hap::Hap">0x66a261b7</code>
		<id>0x9b2cf837</id>
	</puid>
	<command>
		<label>DISABLE</label>
	</command>
</action>
```

**`0x8`** keeps the list moving through a loading screen. While the game is bringing the
world in (loading, fast travel, leaving the hacking space), lists do not start any new
lines. A list marked `0x8` keeps starting lines even then.

## What being switched off actually does

A list whose group goes off is not deleted and does not forget anything. Both it and every
instruction inside it are put to sleep, and the mark stays exactly where it had got to. When
the group comes back, all of them wake up and carry on from there.

```
   group goes off                     group comes back
   ┌────────────────────────┐         ┌────────────────────────┐
   │  1.  place robot    ✓  │         │  1.  place robot    ✓  │
   │  2.  wait for talk  ⏸ │   ─►    │  2.  wait for talk  ⏸ │
   │  3.  open the door     │         │  3.  open the door     │
   │        asleep          │         │     continues here     │
   └────────────────────────┘         └────────────────────────┘
```

Continuing is not always what is wanted. A robot that is already standing there should not
be placed a second time, so continuing is right. A countdown that was halfway through should
start over rather than resume near its end.

For that case a list can be marked to reset instead: `0x4` in its attribute. Coming back
then sends the mark to the top and clears every instruction (see
[starting over](00_hap_system.md)), and the list plays out again from its first line.

Individual instructions have a say in this too: one can be marked to switch itself off
whenever the list resets, so a line that should only ever happen on the first run stays out
of the replay. That mark is covered on the [next page](02_action.md).

---

Next: [Actions](02_action.md).
