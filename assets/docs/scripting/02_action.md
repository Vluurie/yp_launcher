# Actions

An action is one line in a list. There are many kinds, from placing an entity to opening
a door to running a script. Around 110 of them appear across the original game.

They all differ in what they do, but they share the same handful of values.

## What every action carries

```
   ┌─ action ────────────────────────────────────┐
   │  what kind    EntityLayoutAction            │
   │  name         "spawn the bridge robot"      │
   │  id           0x51c116d9                    │
   │  attribute    0                             │
   │                                             │
   │  ... and then whatever this kind needs      │
   └─────────────────────────────────────────────┘
```

The kind decides everything else. An action that places an entity needs to know which
entity and where; an action that waits needs to know what it waits for. Those extra
values differ from kind to kind and are described on the page for each one.

The name is for you, not for the game. Nothing reads it, so write whatever helps you find
this line again later.

The id is how other actions point at this one. You never invent it: the game hands it out
when the file is loaded.

## Every action reports a state

While an action exists it reports a **state**, a short name saying what it is currently
doing. The rest of the quest finds out what is going on by reading it.

```
   1.  place a robot          CREATED
   2.  wait for the talk      WAIT
   3.  open the door          not started yet
```

Which names an action uses is decided by its kind, and they are listed on that kind's page.
There is no shared vocabulary: a name that means something for one kind means nothing for
another.

Alongside the name there is a number. Its meaning also depends on the kind: an action that
placed three entities puts the number still alive there, and one that is counting down
puts the time remaining there.

```
   CREATED 3      three of them placed and alive
   CREATED 1      two have been killed
   DONE 0         none left
```

The number matters because other actions can look at it. Waiting for `DONE` is one thing;
waiting for `CREATED` **with the number 1** is another.

A state name is just text, and the set of them is not fixed. An action can report a name
nobody has used before, and another action can wait for that name. Around 84 names appear
across the original game, but that is what the original content happens to use, not a limit.

## Attribute: how the line behaves in the list

By default the game runs through a list without stopping anywhere, so every line happens in
the same instant. The attribute is where you change that.

It is one number, and it is a **sum of separate switches**. Each switch owns one value, and
you add together the ones you want. This is why the numbers double instead of counting up,
and why they are normally written in hex:

| Value | Switch | What it does |
|---|---|---|
| `0x1` | always on | runs even when its group is off |
| `0x2` | off on reset | starts switched off, and switches itself off again on every reset |
| `0x8` | blocking | the list stops here until this action is done |
| `0x10` | saved | the action's state goes into the save file and comes back on loading |

`0x0` means none of them: the list runs the line and carries straight on.

To combine, add the values: blocking plus off-on-reset is `0x8 + 0x2 = 0xa`. The combinations
you will see most are `0xa`, `0x18` (blocking and saved) and `0x12` (saved and
off-on-reset).

```
   0x0    ────────  plain
   0x8    ───────▄  blocking
   0x2    ▄──────   off on reset
   0xa    ▄──────▄  both
```

**Blocking** is the one you will reach for most, because it is what turns a list into a
sequence. "Done" here means the action has reached the state that counts as finished for its
kind.

```
   without blocking                with blocking
   ┌────────────────────────┐      ┌────────────────────────┐
   │ 1. place a robot       │      │ 1. place a robot       │
   │ 2. wait for the talk   │      │ 2. wait for the talk ⏸ │
   │ 3. open the door       │      │ 3. open the door       │
   │                        │      │                        │
   │ all three at once      │      │ 3 waits for 2          │
   └────────────────────────┘      └────────────────────────┘
```

Any kind of action can be marked this way, not only the ones built for waiting. A
[ConditionBlock](ConditionBlock.md) nearly always is, since holding the list up is its whole
purpose, and a command that has to arrive before the next step makes sense gets the same
mark.

**Off on reset** is for a line that should happen on the first run through the list and not
on any later one. It starts switched off, so something has to switch it on, and when the list
[starts over](01_hap.md) it switches itself off again while every other line is put back
ready to run.

There is one more value, `0x4`, which is the switched-off state itself. You do not write it:
the game sets it for you when it loads an action marked `0x2`, and again on every reset.

Higher values than these exist (`0x400`, `0x800`, `0x1000`, `0x2000`, `0x10000`), and they
work differently: each is read by only a few kinds of action and ignored by the rest, and the
same value means different things depending on the kind it sits on. Setting one on an action
that does not read it does nothing. They are described on the page of each action that uses
one.

## Keeping a saved state forever, `ff` in front

`0x10` alone is not enough for something that has to stay done for the rest of the
playthrough. Every time the game writes a save it throws most saved states away again, and
an entry only survives if it is marked to.

That mark is the `ff` in front of values like `0xff000010`. It means the entry is kept
through every save, even when the player uses chapter select to jump back.

That is what the original game puts on its treasure chests. A chest opened in the desert is
still open after saving, after reloading, and after jumping back to an earlier chapter:
without the `ff` it would be closed again at the next save.

`Action.reset` is the one thing that clears it: the chest goes back to unopened, and opening
it again marks it as taken from then on.

## Reaching an action while the game runs

An action is not only something that runs on its own. Other parts of the quest can reach it
and ask it things, either from a [SendCommand](SendCommand.md) or from a script.

In a script that looks like `the_action.enable`, and the name in front is a variable holding
the action.

| Name | What it does |
|---|---|
| `enable` / `on` | switch it on. Two spellings of the same thing |
| `disable` / `off` | switch it off |
| `enabled?` / `disabled?` | ask whether it is on or off |
| `done?` | ask whether it has finished |
| `reset` | put it back to the way it started |
| `state` | read its state |
| `set_state` / `set_state_value` | give it a different state, or just a different number |
| `state?` | ask whether it currently has a particular state |
| `signal?` | ask whether it sent a particular signal |
| `signal_once?` | the same, but only answers yes once |
| `signal_and_state?` | ask for a signal and a state together |
| `name` / `puid` | ask what it is called, and its id |
| `exists?` | ask whether it is still there |

`enable` is the counterpart to an action marked `0x2`, which starts switched off and needs
something to turn it on. `reset` is how a finished action is made to run a second time.

The two lookalikes differ: `state?` asks about a state, which stays until it changes, while
`signal?` asks about a signal, which does not.

A **signal** is a one-off announcement, and it lasts a single frame: whatever is sent during
one frame can be picked up on the next, and is then thrown away.

Every action sends a few of these on its own, without being asked: one when it first runs,
one when it stops, and one when it is reset.

## Pointing at another action

Some actions need to name another one. An action that waits has to say what it waits for,
and an action that sends a command has to say who receives it.

```
   ┌─ list ──────────────────────────────────────┐
   │  1.  place a robot            ◄───────┐     │
   │  2.  wait until  ─────────────────────┘     │
   │      that robot is gone                     │
   └─────────────────────────────────────────────┘
```

That pointer is the id from the top of this page, so it is a number rather than a name.
Nothing requires you to know the number: an editor shows the name of the target and fills the
number in for you.

---

Next: [EntityLayoutAction](EntityLayoutAction.md), the action that places entities.
