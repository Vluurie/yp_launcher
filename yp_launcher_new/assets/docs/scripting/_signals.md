# Signals the game sends by itself

Besides the signals your own actions send, the game sends a handful on its own. You can hang a
handler on any of them, from a script or from a [ConditionBlock](ConditionBlock.md).

Each one is sent to a particular target, and that target is the second argument of
`event_trigger`. Get it wrong and the handler never runs. There is no error either way.

## ON_TALK

Sent to the entity the player is talking to. Every conversation in the game is built on it.

It does not appear on its own the moment the player walks up to somebody. A conversation is
started by `startTalk`, and starting one is what looks for a handler waiting on `ON_TALK` for
that entity and runs it.

```ruby
def buddy_9S_ON_TALK_ev0()
    self.mess_ex(:GREETING)
end

def update()
    self.event_trigger("buddy_9S_ON_TALK/ev0", "buddy_9S", "ON_TALK")
end
```

A fourth argument narrows it to a scene state, so the same entity says different things as the
story moves on. The original game uses that for Pascal, who has one handler tied to a state and
a second with no state at all as the fallback:

```ruby
self.event_trigger("buddy_pascal_ON_TALK/ev0", "buddy_pascal", "ON_TALK", "last_pascal")
self.event_trigger("buddy_pascal_ON_TALK_2/ev0", "buddy_pascal", "ON_TALK")
```

## ON_CHECK

Sent to the entity the player uses. Terminals, chests and switches all announce themselves this
way, so it is the signal to reach for whenever a quest turns on operating an object.

The flower quest shows what to do when the same interaction has to mean different things as the
quest moves along. Every flower can be interacted with, and they all belong to one quest, so the
handler starts by asking how far the quest has got and hands over from there:

```ruby
def flower_ON_CHECK_s0()
    case
    when "q330/RUN_1".on?()
        self.call("run_1")
    when "q330/RUN_2".on?()
        self.call("run_2")
    when "Route/C".on?()
        self.call("req_c")
    else
        self.call("req")
    end
end

def update()
    self.event_trigger("flower_ON_CHECK/s0", "flower", "ON_CHECK")
end
```

## ON_ROUTE_END

Sent by an entity to itself when it reaches the end of a route it was sent along with
[move](scripting.md#move):

```ruby
def guide_robo_ON_ROUTE_END_a0()
    turn(loc(), guide_robo())
end

def update()
    event_trigger("guide_robo_ON_ROUTE_END/a0", "guide_robo", "ON_ROUTE_END")
end
```

The guide robot walks its route, and at the end this handler turns it to its spot.

## ON_HACKING_CLEAR and ON_HACKING_FAILED

Sent to the hacked object when hacking it succeeds or fails. Name the object and give the two
outcomes one handler each:

```ruby
def up_baka_ON_HACKING_CLEAR_ev1()
    self.call(:talk24)
    self.getReWard("59")
    self.set_scene_state("q250/DONE")
end

def up_baka_ON_HACKING_FAILED_ev2()
    self.call(:talk23)
end

def update()
    self.event_trigger("up_baka_ON_HACKING_CLEAR/ev1", "up_baka", "ON_HACKING_CLEAR", "q250/RUN_1")
    self.event_trigger("up_baka_ON_HACKING_FAILED/ev2", "up_baka", "ON_HACKING_FAILED", "q250/RUN_1")
end
```

`ON_HACKING_CLEAR` is also sent to the global group when the hacking minigame is left after
winning it. A list waits for that with a [ConditionBlock](ConditionBlock.md) pointing at the
global group, which is a `hap::GroupImpl` with id `0x0`:

```xml
<condition>
	<type>1</type>
	<puid>
		<code str="hap::GroupImpl">0x6a69c84d</code>
		<id>0x0</id>
	</puid>
	<condition>
		<state>
			<label>ON_HACKING_CLEAR</label>
		</state>
		<pred>0</pred>
	</condition>
</condition>
```

Losing the minigame sends `ON_HACKING_FAILD` there instead. Watch the spelling: the one that
goes to the group has no `E` before the `D`, the one that goes to the object does.

## NEAR_EMILE

Sent to the global group by Emil, once the player is close enough to him across the ground and
within a range of heights. He sends it once and then goes quiet, so it does not repeat while
the player stands next to him. It is also held back entirely while a script event is running.

The original game uses it to start his greetings, with the fourth argument giving each
character their own:

```ruby
def global_NEAR_EMILE_a0()
    case
    when "auto/blocked".on?()
        self.call("nop")
    else
        self.call_fiber("auto_2_see2B")
    end
    self.set_trigger_state("done")
end

def update()
    self.event_trigger("global_NEAR_EMILE/a0", "global", "NEAR_EMILE", "PL/2B")
    self.event_trigger("global_NEAR_EMILE_2/a0", "global", "NEAR_EMILE", "PL/9S")
    self.event_trigger("global_NEAR_EMILE_3/a0", "global", "NEAR_EMILE", "PL/A2")
end
```

## ON_GET_ITEM

Sent to the global group when an item is added to the inventory. Every kind counts: consumables,
weapons, pod programs, plugin chips, archives and gallery entries all send the same signal.

It only fires when the item really arrived. Adding one that is already at its maximum, or that
does not fit, stops before the signal, so a handler on it never sees a refused pickup.

```ruby
self.wait_signal("ON_GET_ITEM", "global")
```

`wait_signal` stops the script until the signal arrives, so it belongs inside a handler that is
already running, not in `update`.

## EM_LIST_UNLOCK

Sent to the global group when an enemy is added to the enemy database, and only the first time
for that entry. Meeting the same enemy again changes nothing and sends nothing, so a handler on
it runs once per kind of enemy rather than once per fight.

Nothing in the original game listens for it.

## ON_UNIT_GETON

Sent to the global group at the end of getting into a flight unit. It is not limited to the
character being played: the buddy getting into one sends it too.

If the unit cannot be created nothing is sent, so a handler on it only ever runs when somebody
really got in.

Nothing in the original game listens for it.

## SUBPHASE_START

Sent to the global group when the game enters a new subphase.

Every group watching that subphase is switched over first, and the signal goes out
afterwards. So by the time a handler on it runs, the groups belonging to the new
subphase are already on.

Nothing in the original game listens for it.

## ON_OBJ_HACKING_CLEAR

Sent to the global group when a hackable object has been hacked open. The original game uses it
a lot, and always the same way: one trigger, and `set_trigger_state` moves on to the next
handler, so the same signal drives the first hacking point of a scene and then the second.

```ruby
def global_ON_OBJ_HACKING_CLEAR_s0()
    self.disable(self.cam())
    self.enable(self.cam2())
    self.wait(60)
    self.enable(self.attack1())
    self.enable(self.hac2())
    self.set_trigger_state("s1")
end

def global_ON_OBJ_HACKING_CLEAR_s1()
    self.reset(self.cam2())
    self.enable(self.cam2())
    self.wait(60)
    self.enable(self.attack2())
end

def update()
    self.event_trigger("global_ON_OBJ_HACKING_CLEAR/s0", "global", "ON_OBJ_HACKING_CLEAR")
end
```

---

Back to [Scripts](04_scripts.md).
