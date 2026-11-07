# MultiConditionBlock

Holds up everything below it until several things are true at once.

Where [ConditionBlock](ConditionBlock.md) waits on one question, this one waits on a list of
them and decides how they combine.

```
   1.  place three robots
   2.  MultiConditionBlock     ⏸  waiting until all three are dead
   3.  open the door               blocked
```

Each question in the list is written the same way as anywhere else and is described under
[conditions](_conditions.md).

## The cycle

```
   WAIT      testing the list, once per frame
     │
     │  the list passes
     ▼
   DELAY     counting down, the number is the time left
     │
     │  the countdown reaches zero
     ▼
   DONE      the lines below start running
```

**WAIT** is where it starts. Nothing below the action runs.

**DELAY** is the pause after the list passes. With no delay set it goes straight through in the
same frame, so most of the time you only ever see `WAIT` and then `DONE`.

**DONE** releases the list.

For the action to hold anything up it has to be marked as blocking,
[attribute](02_action.md) `0x8`. Nearly every one in the game is.

## conditions

The list of questions, each one a [condition](_conditions.md): who to ask, what kind of
question, what to compare and how.

Two is the usual number, but they run from one up to thirty-four in the original game.

## type

How the answers are combined.

| Value | Name | Passes when |
|---|---|---|
| 0 | and | every question has answered yes |
| 1 | or | at least one has |
| 2 | nand | every question is answering yes right now |

**And is not quite what it sounds like.** Each question remembers once it has answered yes, and
stays satisfied from then on. They do not all have to be true in the same frame; the action
waits until each of them has been true at some point.

```
   frame 1    robot A dead        A remembered
   frame 2    robot B dead        B remembered
   frame 3    robot A revived     A still counts as satisfied
              ─► the list passes
```

**Nand** is the version without that memory. Every question is asked afresh, so it only passes
while all of them are true at the same moment. Use it when you mean "all true right now".

**Or** passes as soon as any one question has answered yes, and it uses the same memory as and.

## delay

How long to hold on after the list passes, in seconds. Zero means release immediately.

Use it to leave a beat between the last enemy dying and the door opening.

## A real one from the game

```xml
<action>
	<code str="MultiConditionBlock">0xe58b4e99</code>
	<name eng="conditional block list">条件ブロックリスト</name>
	<id>0x98ed784f</id>
	<attribute>0x8</attribute>
	<condition>
		<conditions>
			<size>1</size>
			<value>
				<puid>
					<code str="hap::Action">0xa6aaf7a4</code>
					<id>0xc29aedd8</id>
				</puid>
				<condition>
					<state>
						<label>DONE</label>
					</state>
					<pred>0</pred>
				</condition>
			</value>
		</conditions>
		<type>0</type>
	</condition>
	<delay>0</delay>
</action>
```

That is every field this action has.
