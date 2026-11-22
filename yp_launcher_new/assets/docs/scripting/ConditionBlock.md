# ConditionBlock

Holds up everything below it until something has happened.

Where most actions do something, this one waits. It is what turns a list of steps into a
sequence with timing: without it every line runs in the same instant.

```
   1.  place a robot
   2.  ConditionBlock          ⏸  waiting until the robot is dead
   3.  open the door               blocked
   4.  give the reward             blocked
```

## The cycle

```
   WAIT      checking the condition, once per frame
     │
     │  the condition passes
     ▼

   DELAY     counting down, the number is the time left
     │
     │  the countdown reaches zero
     ▼
   DONE      the lines below start running
```

**WAIT** is where it starts. The condition is tested every frame, and nothing below the
action runs.

**DELAY** is the pause after the condition passes. Without a delay set this state passes
straight through in the same frame, so most of the time the action goes from `WAIT` to
`DONE` and you never see it.

**DONE** releases the list.

For the action to hold anything up it has to be marked as blocking, which is
[attribute](02_action.md) `0x8`. Nearly every one in the original game is.

## Attribute 0x400

While the block is waiting, the player cannot leave the hacking minigame: the screen stays
black and control comes back only once the block is done. Fast travel is held the same way.

## The condition

What it waits for is one [condition](_conditions.md), written the same way as on every other
action that carries one and described in full on that page.

## puid

Who to watch, picked the same way as any other [pointer between actions](02_action.md). It
can be another action, a placed entity, or the scene state system.

## type

What kind of question to ask: a state the target reports, a signal, a function call, or a
placed entity's loading state. Left out, it means the ordinary one: read the target's current
state. See [conditions](_conditions.md).

## state

What to watch for: one of the state names the target reports, such as `DONE` for finished or
`FAILURE` for failed. Which names exist depends on what the target is, and they are listed on
its own page.

A state also carries a number, and the condition can test that too.

## pred

How to compare the number, described in full under [conditions](_conditions.md).

**The number you wrote comes first**, which is the opposite of what most people expect:

```
   you wrote:   Value >= Result, 2
   reads as:    2  >=  live
   so it passes while the live number is 2 or fewer
```

The two to reach for when only the name matters are `Value exists` and `Value !exists`, which
ignore the number entirely. Everything else compares the number **and** requires the state name
to match as well.

An [EnemyGenerator](EnemyGenerator.md) in `BATTLE` puts the count of living enemies in its
state, so this holds a door shut until two or fewer are left:

```
   ┌─ ConditionBlock ────────────────────────────┐
   │   puid        the generator                 │
   │   state       BATTLE                        │
   │   pred        Value >= Result               │
   │   value       2                             │
   └─────────────────────────────────────────────┘
             │
             ▼
      2 >= living   passes at 2, 1, 0
```

Comparing lets a condition fire partway through something instead of only at its end. Waiting for a fight to finish needs the state name alone; waiting for "only
two left" needs the number.

## args

Extra text for conditions that need one. Watching the scene state system takes the name of
the scene state here, which is how a list waits for a stage of the story rather than for
another action.

## delay

How long to hold on after the condition passes, in seconds. Zero means release immediately.

This is for letting something breathe, such as a short beat between the last enemy dying and
the door opening.

## bDisable

When set, the target is told to switch off at the moment the condition passes. It saves
following the ConditionBlock with a [SendCommand](SendCommand.md) that does nothing but
disable what was just being watched.

## Switching it off

A ConditionBlock can itself be switched off through its [attribute](02_action.md), and then
it holds nothing up: the lines below run as though it were not there. That is mostly useful
while building something, to skip a step for a moment.

## A real one from the game

```xml
<action>
	<code str="ConditionBlock">0x606fe5c4</code>
	<name eng="condition block">条件ブロック</name>
	<id>0xa50f3443</id>
	<attribute>0x8</attribute>
	<condition>
		<type>2</type>
		<puid>
			<code str="hap::StateObject">0xcecad9ce</code>
			<id str="@MISC">0xf6a78746</id>
		</puid>
		<condition>
			<state>
				<label>MiscState.getSaveData</label>
				<value>1000</value>
			</state>
			<pred>3</pred>
		</condition>
		<args>TotalEnemyKillCount</args>
	</condition>
	<delay>0</delay>
	<bDisable>0</bDisable>
</action>
```

That is every field this action has.
