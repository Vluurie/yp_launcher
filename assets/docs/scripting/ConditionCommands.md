# ConditionCommands

Checks something, and sends one set of commands if it holds and another if it does not.

Where [ConditionBlock](ConditionBlock.md) waits for a condition and
[SendCommand](SendCommand.md) sends one command, this action does both at once and adds a
second branch. It is the if/else of the quest system, and after `SendCommand` it is the most
used action in the game.

```
   ┌─ ConditionCommands ─────────────────────────┐
   │                                             │
   │   condition   both shopkeepers are CREATED  │
   │   commands    ENABLE the map marker         │
   │   else        DISABLE the map marker        │
   │                                             │
   └─────────────────────────────────────────────┘
```

That example is the robot shop from the original game: the marker on your map appears only
while the traders are actually standing there.

## Start here

For a plain "when this happens, do that":

1. Add one entry to `conditions`: who to watch and which state to wait for.
2. Put what should happen into `commands`.
3. Leave `else` empty.
4. Set `type_` to `2`, so it fires whenever the condition becomes true.
5. Leave everything else alone.

## The cycle

```
   WAIT      testing the condition
     │
     ├── holds ────────────► THEN ──────► SENDING
     │                       delay        commands go out
     │
     └── does not hold ────► ELSE ──────► ELSE_SENDING
                             delay        else goes out
                                               │
                                               ▼
                                    DONE, or back to WAIT
```

Both branches take the same shape: a wait for the delay, then the sending. With no delay set
they pass through in the same frame, so what you normally see is `WAIT` and then whatever
comes at the end.

What comes at the end is decided by `type_` below: either the action is finished, or it
returns to `WAIT` and can fire again.

If a command cannot be delivered the action ends in `FAILURE` instead.

## Attribute 0x400

```
   no attribute                        with 0x400

   the commands go out                 the commands go out
     │                                   │
     ├─ all taken ──► DONE               └─ whatever happens ──► DONE
     │                                                           or back to WAIT
     └─ one refused ──► FAILURE
```

Both branches are treated the same way, `commands` and `else` alike.

## conditions

A **list**, not a single test, and each entry is one [condition](_conditions.md): a `puid`
saying who to watch, a `type` saying what kind of question to ask, a `state` with a name and
number, a `pred` saying how to compare, and optional `args`.

The robot shop watches two entities for `CREATED`, which is why it needs a list.

## type

How the entries are combined.

| Value | Name | Passes when |
|---|---|---|
| 0 | and | every entry holds |
| 1 | or | at least one holds |
| 2 | nand | every entry holds, checked fresh each time |

**And is not what you probably expect.** Each entry remembers that it once held, and keeps
counting from then on, so they do not all have to hold at the same moment. Once the first
shopkeeper has been seen as `CREATED`, that entry stays satisfied even if he later goes away.

Nand is the version without that memory: every entry is re-tested from scratch, so it only
passes while all of them hold right now.

## commands, else and resets

Three lists of commands, each entry the same shape as a [SendCommand](SendCommand.md): a
`puid` for the receiver, a `command` name, and optional `args`.

`commands` goes out when the condition holds, `else` when it does not. Both are optional, so
leaving `else` empty gives you a plain "when this happens, do that".

`resets` is a third list, used to put things back.

## type_

Confusingly, this is a second field also called a type, and it has nothing to do with
combining conditions. It decides **what happens after the commands have gone out**.

| Value | After sending | Else branch | Memory |
|---|---|---|---|
| 1 | ends in `DONE`, but keeps watching and flips between the branches | yes | cleared after sending |
| 2 | back to `WAIT`, ready to fire again | none; the condition failing does nothing | kept |
| 3 | back to `WAIT`, ready to fire again | yes | cleared before every check |

Value 1 is what the robot shop uses. It is the only one that keeps re-checking once it is
`DONE`, which is why the marker comes back when the traders return: the action flips between
`ENABLE` and `DISABLE` for as long as it exists.

Value 2 is a one-way repeating trigger. The else list is skipped entirely, so this is the
setting for "whenever this becomes true, do that".

Value 3 is like 2 but clears the memory described under `type` before each check, so the
condition is always judged on the present moment.

## delay

Seconds to wait between the condition being decided and the commands going out. It applies
to both branches, and zero means immediately.

## A real one from the game

```xml
<action>
	<code str="ConditionCommands">0x7d30118c</code>
	<name eng="conditional command">条件コマンド</name>
	<id>0xefe76dd1</id>
	<attribute>0x1</attribute>
	<condition>
		<conditions>
			<size>1</size>
			<value>
				<puid>
					<code str="hap::Action">0xa6aaf7a4</code>
					<id>0x2c14ff3c</id>
				</puid>
				<condition>
					<state>
						<label>INSIDE</label>
					</state>
					<pred>0</pred>
				</condition>
			</value>
		</conditions>
		<type>0</type>
	</condition>
	<commands>
		<items>
			<size>1</size>
			<value>
				<puid>
					<code str="hap::Action">0xa6aaf7a4</code>
					<id>0x5dfb67e7</id>
				</puid>
				<command>
					<command>
						<label>ENABLE</label>
					</command>
				</command>
			</value>
		</items>
	</commands>
	<else>
		<items>
			<size>1</size>
			<value>
				<puid>
					<code str="hap::Action">0xa6aaf7a4</code>
					<id>0x5dfb67e7</id>
				</puid>
				<command>
					<command>
						<label>RESET</label>
					</command>
				</command>
			</value>
		</items>
	</else>
	<type_>1</type_>
	<delay>0</delay>
</action>
```

Left out here: `resets`. It is optional, and leaving it out keeps the default.
