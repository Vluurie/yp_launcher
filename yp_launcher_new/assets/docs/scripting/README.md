# HAP documentation

## Read in this order

1. [The HAP system](00_hap_system.md) - what the whole thing is and why it exists
2. [The HAP](01_hap.md) - the list of steps, and what switches it on
3. [Actions](02_action.md) - what a single step looks like
4. [Making a change](03_getting_started.md) - where the files are and how to edit one
5. [Scripts](04_scripts.md) - the Ruby half, for conversations and anything that reacts
6. [Script methods](scripting.md) - every method a script can call, one by one

## Individual actions

**Putting things in the world**

| Action | What it does |
|---|---|
| [EntityLayoutAction](EntityLayoutAction.md) | Puts entities into the world |
| [EntityLayoutArea](EntityLayoutArea.md) | The same, but only while the player is nearby |
| [EnemySetAction](EnemySetAction.md) | Places a fixed group of enemies and tracks the fight |
| [EnemySetArea](EnemySetArea.md) | The same, but only while the player is nearby |
| [EnemyGenerator](EnemyGenerator.md) | Produces enemies over and over while a fight lasts |
| [ShootingEnemyCurveAction](ShootingEnemyCurveAction.md) | A flight path for the shooting sections |
| [BezierCurveAction](BezierCurveAction.md) | A path through the world for something to travel along |

**Waiting and reacting**

| Action | What it does |
|---|---|
| [ConditionBlock](ConditionBlock.md) | Holds up the steps below it until something happens |
| [MultiConditionBlock](MultiConditionBlock.md) | The same, but waits on several things at once |
| [DelayAction](DelayAction.md) | Waits for a number of seconds |
| [AreaAction](AreaAction.md) | Reports whether the player is inside a region |

**Making things happen**

| Action | What it does |
|---|---|
| [SendCommand](SendCommand.md) | Tells something in the game to do something |
| [SendCommands](SendCommands.md) | A list of commands, all sent at once |
| [AreaCommand](AreaCommand.md) | Sends a command when the player enters a region |
| [ScriptAction](ScriptAction.md) | Runs a Ruby script |
| [PhaseEventAction](PhaseEventAction.md) | Plays a cutscene |
| [SoundNovelAction](SoundNovelAction.md) | Plays a novel |
| [HapMultiMarkLocationAction](HapMultiMarkLocationAction.md) | Puts a marker on the map |
| [HapGoalLocationAction](HapGoalLocationAction.md) | Shows the player where to go next |
| [HapLocationAction](HapLocationAction.md) | A spot in the world for something else to look up |
| [PhaseJumpAction](PhaseJumpAction.md) | Moves the game to a different part of the story |
| [HackingRequestEndAction](HackingRequestEndAction.md) | Ends the hacking minigame |
| [MemorySaveAction](MemorySaveAction.md) | Sets where the player comes back after dying |
| [PlayerCombatLimitAction](PlayerCombatLimitAction.md) | Takes abilities away from the player |
| [ConditionCommands](ConditionCommands.md) | Sends one set of commands if a condition holds, another if not |
| [CameraAction](CameraAction.md) | Takes over the camera |
| [CameraTargetAction](CameraTargetAction.md) | Turns the camera to look at something |

Settings that many actions share have their own pages: [what gets placed](_layouts.md),
[areas](_areas.md) and [conditions](_conditions.md). The switches around the lists are on
[groups](_groups.md) and the [token categories](_token_categories.md) they watch.

The game also [sends signals by itself](_signals.md), and a script or a ConditionBlock can
react to them. [Save state](_save_state.md) explains what an action writes into the save,
which is where the `ch`, `user0` and `layout` entries in a save file come from.

For reading the original content or writing the XML by hand, see
[the files a quest is made of](_files.md) and [packing them by hand](_packing.md) with
F-SERVO.

There are 110 kinds of action in total. The rest do not have a page yet.

## What comes up in almost every action

**States.** Every action has a short name saying what it is currently doing, such as `WAIT`
or `DONE`, plus a number whose meaning depends on the action. See
[Actions](02_action.md).

**Scene states.** Short pieces of text recording what has happened in the story, such as
`q900/REQ`. They switch groups on and off. See
[The HAP system](00_hap_system.md).

**Ids.** Every list, action and entity carries a number the game uses internally. None
of them have to be filled in by hand.
