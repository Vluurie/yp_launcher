# SendCommand

Tells something in the game to do something.

Where [ConditionBlock](ConditionBlock.md) waits for news, this one delivers it. It is how a
list changes something outside itself.

```
   ┌─ SendCommand ───────────────────────────────┐
   │                                             │
   │   puid      a placed door (Ba0210)          │
   │   command   Ba0210.lock                     │
   │                                             │
   └─────────────────────────────────────────────┘
```

## The cycle

```
   trying     offering the command, once per frame
     │
     ├─────────────────────────┐
     │  the receiver took it   │  there is no receiver
     ▼                         ▼
   DONE                     FAILURE
```

**trying** is where it starts. The command is offered again on every frame, so a receiver
that is not ready yet still gets it a moment later.

**DONE** means the command arrived.

**FAILURE** means there was nothing at the other end, because the target is missing or
already gone. The action stops and does not try again.

The list does not wait for any of this. The lines below run right away, unless the action is
marked as blocking, and then they wait for `DONE`.

## puid

Who receives the command, picked the same way as any other [pointer between
actions](02_action.md).

The usual targets are a placed entity (`app::EntityLayout`), another action
(`hap::Action`), a group (`hap::GroupImpl`), the scene state system (`@SceneState`), or a
named part of the scene such as `player` (`hap::SceneEntities`).

## command

What to do, given as a name.

The game first looks for a function of that name. If there is one it is called, and that is
the ordinary case: `Ba0210.lock` locks a door, `Pl0000.onTowerMode` switches the player into
tower mode.

If nothing has that name, the command does not fail. The name is set as a **state** on the
receiver instead. That is what `DISABLE` and `ENABLE` do.

```
   command "Ba0210.lock"
        │
        ├─ a function by that name exists  ─►  call it
        │
        └─ nothing by that name            ─►  set it as a state on the target
```

The second path is used on purpose. A state is what a condition elsewhere might be watching
for, so a command can release a wait somewhere else in the quest.

Which names do something depends on the receiver, since each one brings its own.

**A misspelled command does not complain.** Because an unrecognised name is treated as a
state rather than an error, it reports `DONE` exactly like a correct one and sets a state
nobody is watching. If a command appears to arrive but nothing happens, check the spelling
first.

## Attribute 0x400

A command that answers no leaves the action without any state at all.

```
   no attribute                      with 0x400

   send the command                  send the command
     │                                 │
     ├─ nobody there ──► FAILURE       ├─ nobody there ──► FAILURE
     │                                 │
     ├─ answer yes  ──► DONE           └─ any answer   ──► DONE
     │
     └─ answer no   ──► no state
                        send again next frame
```

One from the game:

```
   ┌─ SendCommand ───────────────────────────────┐
   │   attribute  0x418                          │
   │   command    SceneStateSystem.reset         │
   │   args       PV_NPCLayout_1_2               │
   └─────────────────────────────────────────────┘
```

`SceneStateSystem.reset` clears a scene state, and answers no when that state was not set in
the first place. `0x400` is what lets the action finish in that case.

## args

Extra text passed along with the command, for functions that need one. `Behavior.setSuspend`
takes a 1 here to hide an entity.

The most common use of the whole action needs it too: sending `SceneStateSystem.set` with a
scene state name sets that scene state, and that is what switches on the next group.

```
   ┌─ SendCommand ───────────────────────────────┐
   │   puid      the scene state system          │
   │   command   SceneStateSystem.set            │
   │   args      q900/CLR                        │
   └─────────────────────────────────────────────┘
             │
             ▼
      scene state q900/CLR is set
             │
             ▼
      the group waiting for it switches on
```

This is the line that moves a quest from one stage to the next.

## A real one from the game

```xml
<action>
	<code str="SendCommand">0xa10e3dbc</code>
	<name eng="Send command (3)">コマンド送信(3)</name>
	<id>0xe99ad4ff</id>
	<attribute>0x2</attribute>
	<puid>
		<code str="app::EntityLayout">0xd8efbb45</code>
		<id>0x59bf4998</id>
	</puid>
	<command>
		<label>HackingEnemy.setMoveSpeedRate</label>
	</command>
	<args>0.1</args>
</action>
```

`args` is optional and other blocks leave it out, but this one fills it in.
