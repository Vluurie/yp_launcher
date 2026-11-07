# SendCommands

A list of commands, all sent at once.

A [SendCommand](SendCommand.md) sends one command to one target; this one sends every entry
in its list in the same frame.

```
   ┌─ SendCommands ──────────────────────────────────┐
   │  entry 1   the gate         ──►  ENABLE         │
   │  entry 2   the enemy group  ──►  DEACTIVATE     │
   │  entry 3   the marker       ──►  DISABLE        │
   └─────────────────────────────────────────────────┘
```

Each entry is a `puid` and a `command`, both as described on the
[SendCommand](SendCommand.md) page.

## The cycle

```
   SENDING     at least one entry is still waiting
     │
     ├─ every entry got through ──────────► DONE
     │
     └─ one was refused ─────────────────► FAILURE
```

**SENDING** retries the outstanding entries every frame. An entry that has already gone out is
never sent again.

**DONE** is the end. An action with no entries reports it immediately.

**FAILURE** does not stop anything, so the action keeps trying from there. Only `DONE` ends it.

Wait for `DONE`.

## resets

A second list of the same kind, used to undo the first. If the commands list sends `ENABLE` to
a gate, put `DISABLE` to the same gate here.

Leaving it empty is fine.

On reset:

```
   the resets list    ──►  sent, once, right away
   the commands list  ──►  marked unsent, so the next run sends it again
```

The resets list is not retried on later frames.

## A real one from the game

```xml
<action>
	<code str="SendCommands">0xd9783e91</code>
	<name eng="Send multiple commands">複数コマンド送信</name>
	<id>0xee0c3e26</id>
	<attribute>0x2</attribute>
	<commands>
		<items>
			<size>4</size>
			<value>
				<puid>
					<code str="hap::Action">0xa6aaf7a4</code>
					<id>0xabb4590</id>
				</puid>
				<command>
					<command>
						<label>ScrollMap.setCam_Behind</label>
					</command>
				</command>
			</value>
			<value>
				<puid>
					<code str="hap::Action">0xa6aaf7a4</code>
					<id>0xabb4590</id>
				</puid>
				<command>
					<command>
						<label>ScrollMap.setWorldSpd</label>
					</command>
					<args>"-4 0.3"</args>
				</command>
			</value>
			<value>
				<puid>
					<code str="hap::Action">0xa6aaf7a4</code>
					<id>0xe819e996</id>
				</puid>
				<command>
					<command>
						<label>DISABLE</label>
					</command>
				</command>
			</value>
			<value>
				<puid>
					<code str="hap::Action">0xa6aaf7a4</code>
					<id>0x1603d34</id>
				</puid>
				<command>
					<command>
						<label>ENABLE</label>
					</command>
				</command>
			</value>
		</items>
	</commands>
</action>
```

Left out here: `resets`. It is optional, and leaving it out keeps the default.
