# ScriptAction

Runs a Ruby script. Writing the Ruby itself is on the [Scripts](04_scripts.md) page.

```
   ┌─ ScriptAction ──────────────────────────────┐
   │   script      which script to run           │
   │   variables   the names it may use          │
   └─────────────────────────────────────────────┘
```

## The cycle

```
   STARTUP     loading
     │
     ▼
   RUN         one step of the script every frame
     │
     ▼
   DONE        the script has nothing left to do
```

A script stops at every `mess_ex`, `sleep` or choice and picks up there next frame. That is why
a conversation can take a minute while the game keeps running.

A script that fails to load, or fails while running, also ends in `DONE`. There is no failure
state.

## variables

Ruby has no idea what `shopkeeper` or `area1` means. This is where a name gets tied to
something real.

```
   ┌─ variables ─────────────────────────────────┐
   │     shopkeeper  ──►  a placed entity        │
   │     area1       ──►  an AreaAction          │
   │     this        ──►  this action itself     │
   └─────────────────────────────────────────────┘
```

The name is yours to choose, and the script has to spell it the same way. Nothing checks that.

Two kinds of target get extra treatment, and the vanilla names give them away.

A variable named like `area1` usually points at an [AreaAction](AreaAction.md), which is a
condition: it answers whether the player is inside. The script action re-checks it every
frame, so the script always reads a current answer rather than one from an earlier frame.

A variable named like `command1` usually points at a [SendCommands](SendCommands.md). That
one gets reset together with the script action, so its commands go out again on the next run.

## Reporting back

A script signals with one of its variables as the sender, and other actions wait for it.

```ruby
self.signal(:pascal_talk1_end, self.this())
```

## Switching off mid-script

The script is thrown away and starts from the top next time.
[Attribute](02_action.md) `0x400` keeps its place instead.

## Surviving a save

[Attribute](02_action.md) `0x10` writes the action's state into the save file. A conversation
that has to stay finished after loading needs it; one that may run again does not. The
original game uses both.

## A real one from the game

```xml
<action>
	<code str="ScriptAction">0xbeaf915f</code>
	<name>..M1040_S0900_GM</name>
	<id>0x54ff9faf</id>
	<attribute>0x10</attribute>
	<script>
		<id>0xb99094bd</id>
	</script>
	<variables>
		<size>1</size>
		<value>
			<id>0x63089d5f</id>
			<name>this</name>
			<value>
				<code str="hap::Action">0xa6aaf7a4</code>
				<value>0x54ff9faf</value>
			</value>
		</value>
	</variables>
</action>
```

That is every field this action has.
