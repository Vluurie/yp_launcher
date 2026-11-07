# The HAP system

## The problem the game had to solve

A game world is full of things that must happen at the right moment. A robot should be
standing on a bridge, but only during a certain quest. A door should open, but only after
the player has talked to someone. A cutscene should play, but only once.

None of that can be built into the landscape. The bridge is always there; the robot is
not. So the game needs a second layer that says *when* things happen.

That layer is HAP. Saying *when* means writing down what should happen and in which order.

## Lists of instructions

A HAP is a list of instructions, read from top to bottom.

```
   ┌─ HAP ────────────────────────────────────┐
   │                                          │
   │   1.  Place a robot at this position     │
   │   2.  Wait until the player talks to it  │
   │   3.  Open the door                      │
   │                                          │
   └──────────────────────────────────────────┘
```

Each line is called an **action**. A quest is not one huge list but many small ones.

## How far the list has got

The game does not run a list once and forget it. It comes back every frame, and it
remembers how far down it has got.

Everything above that mark has been started. Everything below has not been reached yet.

```
   ┌──────────────────────────────────┐
   │  1.  Place a robot          ✓    │  started
   │  2.  Wait for the player    ✓    │  started
   │  ─────────────────────────────   │  ◄── how far it has got
   │  3.  Open the door               │  not yet
   └──────────────────────────────────┘
```

Each frame the game does two things. It goes over everything that was already started and
gives it a moment to carry on, and then it moves the mark down to start the next line.

Nothing is ever taken out of the list. A line that has done its job stays where it is, and
the mark stands below it. That is what lets a list run a second time.

## Waiting

If the mark moved down every frame no matter what, all three lines would be started almost
at once and the door would open before anyone had spoken.

So a line can hold the mark where it is. While it does, nothing below it starts.

```
   1.  Place a robot at this position        started
   2.  Wait until the player talks to it  ⏸  holding the mark here
   3.  Open the door                         not reached
   4.  Give the reward                       not reached
```

Step 3 cannot run early, because step 2 is holding the mark above it. Holding the mark is
what turns a list into a sequence.

## Starting over

Because nothing was deleted, a list can be sent back to the beginning: the mark returns to
the top and every line forgets what it was doing.

```
   before                                    after
   ┌──────────────────────────────────┐      ┌──────────────────────────────────┐
   │  1.  Place a robot          ✓    │      │  ───────────────────────────     │ ◄── mark
   │  2.  Wait for the player    ✓    │  ─►  │  1.  Place a robot               │
   │  ───────────────────────────     │      │  2.  Wait for the player         │
   │  3.  Open the door               │      │  3.  Open the door               │
   └──────────────────────────────────┘      └──────────────────────────────────┘
```

The whole list then plays out again from the first line.

The list is in order now, but nothing says when it should begin. As written it would start
the moment the game boots, and that robot would stand on the bridge in every save file.

## The switch

A list does not run all the time. It sits inside a **group**, and the group is a switch.

```
   group OFF                                 group ON
   ┌──────────────────────────────────┐      ┌──────────────────────────────────┐
   │  ┌────────────────────────────┐  │      │  ┌────────────────────────────┐  │
   │  │  1.  Place a robot         │  │      │  │  1.  Place a robot         │──┼─► happens
   │  │  2.  Wait for the player   │  │      │  │  2.  Wait for the player   │──┼─► happens
   │  │  3.  Open the door         │  │      │  │  3.  Open the door         │──┼─► happens
   │  └────────────────────────────┘  │      │  └────────────────────────────┘  │
   │         nothing happens          │      │                                  │
   └──────────────────────────────────┘      └──────────────────────────────────┘
```

## What flips the switch

The game keeps short **scene states** about the story, such as `q900/REQ` for "quest 900
accepted". Each group names the scene states it cares about, and can name several.

```
   scene states currently set       groups and what they wait for
   ┌──────────────────┐             ┌──────────────────────────────┐
   │  q900/REQ        │────────────►│  waits for q900/REQ    →  ON │
   │  q120/DONE       │             │  waits for q900/CLR    →  off│
   └──────────────────┘             │  waits for q120/DONE   →  ON │
                                    └──────────────────────────────┘
```

Scene state there, group on. Scene state gone, group off.

Since an instruction can set a scene state, one list can switch on the next:

```
   write q900/REQ
        │
        ▼
   ┌─ group q900/REQ ───────────────────────┐
   │  1.  Place a robot                     │
   │  2.  Wait until the player talks       │
   │  3.  Write q900/CLR  ──────────────────┼──┐
   └────────────────────────────────────────┘  │
                                               ▼
                           ┌─ group q900/CLR ───────────────┐
                           │  1.  Give the reward           │
                           └────────────────────────────────┘
```

That is a quest: accepted, in progress, finished.

Scene states work between one list and the next. Two lists running at the same time need
something else.

## Across lists

Holding the mark only works downwards inside one list. When two lists need to coordinate,
one sends a named message and the other waits for that name. The message is called a
**signal**.

```
   list A                                   list B
   ┌───────────────────────────┐            ┌───────────────────────────┐
   │  Send "doorOpen"  ────────┼───────────►│  Wait for "doorOpen"      │
   └───────────────────────────┘            └───────────────────────────┘
```

The difference from a scene state: a scene state stays until it is removed; a signal lasts
one frame.

Everything sent during a frame is collected, and the next frame is the one where it can be
picked up. After that it is thrown away, whether anyone read it or not.

```
   frame 1                    frame 2                    frame 3
   list A sends "doorOpen"    list B can see it          gone
```

So a signal is not a message addressed to somebody. It is put out, and whoever happens to
look on the following frame finds it. If nothing looks, nothing happens.

A signal suits a moment and a scene state suits a situation. "The door just
opened" is a signal. "The door is open" has to be a scene state, because a signal for it
would be gone the frame after.

---

Next: [The HAP](01_hap.md).
