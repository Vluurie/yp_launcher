# Conditions

A condition asks a question about something else in the game and answers yes or no.

Several actions carry one: [ConditionBlock](ConditionBlock.md) waits on it,
[ConditionCommands](ConditionCommands.md) branches on it,
[MultiConditionBlock](MultiConditionBlock.md) combines a list of them, and
[EnemySetAction](EnemySetAction.md) holds its enemies back until one passes. They all describe
the question the same way.

```
   ┌─ one condition ─────────────────────────────┐
   │                                             │
   │   puid    who to ask                        │
   │   type    what kind of question             │
   │   label   the name to look for              │
   │   value   the number to compare against     │
   │   pred    how to compare                    │
   │   args    a parameter, for function calls   │
   │                                             │
   └─────────────────────────────────────────────┘
```

## When it cannot ask

A condition that cannot get an answer says **no**. It does not stop the quest, report an error,
or pass by default.

That covers every way it can go wrong: the target is missing or already gone, the id points at
nothing, the target has no state to read, or the function name does not exist. All of them come
out the same way: the condition never fires and nothing in the game complains.

## puid

Who the question is about, picked the same way as any other
[pointer between actions](02_action.md). Usually another action, a placed entity, or the scene
state system.

## type

What kind of question to ask. Five kinds exist.

| Value | Name | What it checks |
|---|---|---|
| 0 | Hap State | the state the target action is reporting right now |
| 1 | Event State | whether the target's event has happened |
| 2 | Function Call | calls `label` on the target and compares what comes back |
| 3 | Post Signal | see below, unused |
| 4 | Entity State | how far a placed entity has got through being loaded |

**Hap State** is the ordinary one and what you get when the field is left out. Every action
reports states (`DONE`, `BATTLE`, `INSIDE`), and this asks for the current one.

**Event State** looks for a signal rather than a state. Scripts raise these, and the names are
whatever the quest author chose: `R_OPENED`, `q150_EM_start`.

**Function Call** does not read a state at all. It calls a function named by `label` on the
target, hands it `args` if there is one, and compares the returned number. That is how a
condition can ask something no state exposes:

```
   label   BehaviorAppBase.getHpRate100      how much health is left, as a percentage
   pred    Value > Result
   value   30                                passes while the target is under 30%
```

**Post Signal** checks for a signal and, if it is missing, adds it. Nothing in the original
game uses it.

### Entity State

This one is not the entity's own behaviour reporting something. It answers **how far a placed
entity has got through appearing in the world**, and the answer is one of a fixed list:

```
   EMPTY  ─►  LOADING  ─►  LOADED  ─►  CREATED        the normal run
                                          │
                          STAY, ON_HOLD   │  waiting
                                          ▼
                                DEAD, TERMINATED      gone
                                     ERR              it went wrong
```

So `CREATED` means the thing is standing in the world, and `DEAD` that it was killed. Ask for
`ERR` if you want to notice something failing to appear at all.

It does **not** go through the action that placed the entity: it looks the entity up by its
own id and reads the state off it directly. The state also never carries a number (it is
always zero), so only the name is worth testing here. Use `exists` or
`!exists`, described below, rather than a numeric comparison.

## label and value

`label` is the name being looked for: a state name for the state kinds, a function name for a
function call.

`value` is the number to compare against. A state carries a number as well as a name (an
[EnemyGenerator](EnemyGenerator.md) in `BATTLE` puts the count of living enemies there), and
this is what that number is measured against.

## pred

How to compare. **The number you wrote comes first in every one of these.**

| Value | Reads as | Passes when |
|---|---|---|
| 0 | `value == result` | the number matches |
| 1 | `value != result` | the name **or** the number differs |
| 2 | `value < result` | the live number is larger |
| 3 | `value <= result` | the live number is larger or equal |
| 4 | `value > result` | the live number is smaller |
| 5 | `value >= result` | the live number is smaller or equal |
| 6 | `value exists` | the name matches, whatever the number |
| 7 | `value !exists` | the name does not match |

So writing "greater than 2" passes while the live number is **below** 2, not above it.

```
   you wrote:   Value > Result, 2
   reads as:    2  >  live
   so it passes at 1 and 0
```

**The name has to match as well.** For 0 to 5 the comparison only happens while the target is
reporting that exact state name: a `BATTLE >= 2` never passes while the target says `DONE`,
whatever the numbers are. Only 6 and 7 look at the name alone.

`value != result` is the odd one out: it passes when **either** the name or the number differs,
so it is not simply the opposite of `value == result`.

6 and 7 are the ones to reach for when only the name matters, which is most of the time.

## args

A parameter for the function being called, and it only means anything for a Function Call.

```
   label   SceneStateSystem.has
   args    q120/DONE
                                  asks whether that scene state is set
```

That pairing, `SceneStateSystem.has` with a scene state name, is the usual way a quest asks
how far the story has got.

## Real ones from the game

One of each kind, taken from the shipped quests. These are single entries out of a condition
list; the action around them is on its own page.

**Hap State** - waiting for another action to finish. No `type` field, because this is the
default kind:

```xml
<value>
	<puid>
		<code str="hap::Action">0xa6aaf7a4</code>
		<id>0xa12f9e40</id>
	</puid>
	<condition>
		<state>
			<label>DONE</label>
		</state>
		<pred>0</pred>
	</condition>
</value>
```

**Event State** - waiting for a signal a script raises:

```xml
<value>
	<type>1</type>
	<puid>
		<code str="hap::GroupImpl">0x6a69c84d</code>
		<id>0x0</id>
	</puid>
	<condition>
		<state>
			<label>ACCESS_BAN_BA2016</label>
		</state>
		<pred>0</pred>
	</condition>
</value>
```

**Function Call** - asking the scene state system how far the story has got. The `args` is the
scene state being asked about:

```xml
<value>
	<type>2</type>
	<puid>
		<code str="hap::StateObject">0xcecad9ce</code>
		<id str="@SceneState">0x35c92545</id>
	</puid>
	<condition>
		<state>
			<label>SceneStateSystem.has</label>
		</state>
		<pred>0</pred>
	</condition>
	<args>Beforename_ane</args>
</value>
```

**Entity State** - asking whether a placed entity is standing in the world:

```xml
<value>
	<type>4</type>
	<puid>
		<code str="app::EntityLayout">0xd8efbb45</code>
		<id>0xa5fb9a39</id>
	</puid>
	<condition>
		<state>
			<label>CREATED</label>
		</state>
		<pred>0</pred>
	</condition>
</value>
```

**Post Signal** has no example. No quest in the game uses it.

### A number comparison

Health checks are what the numeric predicates are mostly for. This one passes once the target
is at 70% health or less. The written number comes first, so `value >= result` reads
`70 >= live`:

```xml
<value>
	<type>2</type>
	<puid>
		<code str="app::EntityLayout">0xd8efbb45</code>
		<id>0x9d5dd39c</id>
	</puid>
	<condition>
		<state>
			<label>BehaviorAppBase.getHpRate100</label>
			<value>70</value>
		</state>
		<pred>5</pred>
	</condition>
</value>
```

### Name only

`pred 6` and `pred 7` ignore the number. This one passes whenever the target is in `BATTLE`,
however many enemies are left:

```xml
<value>
	<puid>
		<code str="hap::Action">0xa6aaf7a4</code>
		<id>0x58639932</id>
	</puid>
	<condition>
		<state>
			<label>BATTLE</label>
		</state>
		<pred>6</pred>
	</condition>
</value>
```

`pred 7` is the opposite. This one passes for as long as the target has **not** reached
`DONE`:

```xml
<value>
	<puid>
		<code str="hap::Action">0xa6aaf7a4</code>
		<id>0x1730ac2d</id>
	</puid>
	<condition>
		<state>
			<label>DONE</label>
		</state>
		<pred>7</pred>
	</condition>
</value>
```
