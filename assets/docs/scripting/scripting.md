# Script methods

How a script is put together is on the [Scripts](04_scripts.md) page.

## If this is your first script

You only need a few methods to begin:

- [`mess_ex`](#mess_ex): show dialogue
- [`sele_ex`](#sele_ex): ask a question
- [`set_scene_state`](#set_scene_state): move a quest forward
- [`event_trigger`](#event_trigger): decide when a handler runs
- [`set_trigger_state`](#set_trigger_state): change what happens next time

## Names ending in ()

Names such as `shopkeeper()`, `door()`, `player()`, `this()` and `global()` return things the
game knows about. You do not write these methods yourself. `shopkeeper` and `door` are
variables from the [ScriptAction](ScriptAction.md); `player()` is the played character,
`this()` the ScriptAction itself, and `global()` the global group.

## sleep

Waits for a number of frames, then the script carries on.

```ruby
sleep()      # wait one frame
sleep(30)    # wait thirty frames
```

## wait

Waits for a number of frames, counted at sixty per second.

```ruby
wait(60)     # one second
wait(180)    # three seconds
```

Unlike `sleep`, `wait(60)` divides by sixty and waits that many real seconds, so it
takes just as long on any framerate. `sleep(60)` counts sixty actual frames, which is faster on
a fast PC. Use `wait` for pauses the player should feel, like a beat between two lines.

## wait_sec

The same, in seconds.

```ruby
wait_sec(1)      # one second
wait_sec(2.5)    # two and a half
```

## mess_ex

Shows dialogue and waits until the player has read it. The symbol names a message written at
the top of the script, as shown on the [Scripts](04_scripts.md) page:

```ruby
mess_ex(:GREETING)
```

A list plays several lines, one after the other, each waiting for the player:

```ruby
mess_ex([ :LINE_1, :LINE_2 ])
```

The speaker is the entity the handler is tied to. To let somebody else speak, pass them in as
the third argument. Here the player says the lines instead:

```ruby
mess_ex([ :LINE_1, :LINE_2 ], nil, player())
```

The fifth argument picks lines by condition. Each entry says whether its line is shown, and the
original game uses it to give each character their own version of the same moment:

```ruby
mess_ex([ :FOR_2B, :FOR_9S ], nil, nil, nil, [ player_2B?(), player_9S?() ])
```

Playing as 2B shows the first line, playing as 9S the second.

## simple_mess_ex

Like `mess_ex`, but nobody is talked to; the lines just show. The game uses it for pod lines
when the player walks into an area:

```ruby
simple_mess_ex(:LINE)
```

## player_2B? player_9S? player_A2?

Say whether that character is the one being played right now:

```ruby
if player_9S?()
    mess_ex(:FOR_9S)
else
    mess_ex(:FOR_EVERYONE_ELSE)
end
```

## player

The played character, for passing into other calls, most often as the speaker of a line:

```ruby
mess_ex([ :LINE ], nil, player())
```

## sele_ex

Asks the player a question. It takes three lists: the lines leading up to the question, the
answers to pick from, and where each answer leads.

```ruby
sele_ex([ :QUESTION ], [
    :ANSWER_YES,
    :ANSWER_NO
], [ "ev1", "ev2" ])
```

The lines play like `mess_ex`, except the last one stays on screen as the question. Answers and
destinations line up by position.

**A destination is only the ending**: the part after the last underscore, exactly as in
[`set_trigger_state`](#set_trigger_state). It is completed with the trigger the script is
currently in, so from a handler named `shopkeeper_ON_TALK_ev0` the two above reach
`shopkeeper_ON_TALK_ev1` and `shopkeeper_ON_TALK_ev2`. Writing out a full path such as
`"shopkeeper/ON_TALK/accepted"` does not resolve to anything.

```
   :QUESTION shows and stays open
        │  the player picks
        │
        ├─ :ANSWER_YES ──►  shopkeeper_ON_TALK_ev1 runs,  $ans = 0
        │
        └─ :ANSWER_NO ───►  shopkeeper_ON_TALK_ev2 runs,  $ans = 1
```

`$ans` holds which answer was picked, counting from zero, in case the handler wants to know.

## call

Runs another handler of the same trigger, named by the part after the last underscore. A first
handler decides:

```ruby
def flower_ON_CHECK_s0()
    call("run_1")
end
```

The name is completed with the trigger the script is currently in, so this runs
`flower_ON_CHECK_run_1`:

```ruby
def flower_ON_CHECK_run_1()
    mess_ex(:LINE)
end
```

The original game uses this to split one long conversation into pieces: a first handler
decides, the others carry the parts.

A symbol runs any method of the script directly, wherever it is:

```ruby
def up_baka_ON_HACKING_CLEAR_ev1()
    call(:talk24)
end

def talk24()
    mess_ex(:LINE)
end
```

## enable

Switches an action or entity on from inside a script, the same as a
[SendCommand](SendCommand.md) with `ENABLE`.

What it acts on is a variable from the [ScriptAction](ScriptAction.md), the same way
`shopkeeper` is one on the [Scripts](04_scripts.md) page. With a variable named `door` for the
action that opens the door:

```ruby
def shopkeeper_ON_TALK_ev0()
    enable(door())
end
```

## disable

Switches it off again.

```ruby
disable(door())
```

## reset

Puts it back to the way it started, as `Action.reset` does.

```ruby
reset(door())
```

## reset_enable

Both at once: back to the start and switched on, so it runs again from the beginning.

```ruby
reset_enable(door())
```

## set_scene_state

Sets a scene state, the pieces of text from [the first page](00_hap_system.md) that record how
far the story has got and switch groups on and off.

```ruby
def shopkeeper_ON_TALK_ev0()
    set_scene_state("q900/REQ")
end
```

This is how a conversation moves the quest forward: the state switches on the group that holds
the next part.

The names are not free to invent: the game only knows the ones registered in its data, and a
name it does not know does nothing, without an error.

For your own quest you do not list them one by one. Registering the quest in
`entities/quest_<id>.toml` is enough, and its `name` becomes the prefix of a fixed set:

```
   name = "q899"
        │
        ├─ q899/REQ      q899/CLR      q899/DONE      q899/BREAK      always
        ├─ q899/RUN_1    q899/RUN_2    ...            one per [run.N] you write
        ├─ q899/FAIL_1   ...                          one per [fail.N] you write
        └─ q899/END      q899/FIN      q899/READY     when that text exists
```

So `set_scene_state("q899/RUN_2")` works as soon as the quest has a `[run.2]` text. The full
field list of that file is in the [quest page](../native/07_quest.md)
of the NAMS content docs.

## reset_scene_state

Unsets a scene state:

```ruby
reset_scene_state("q900/REQ")
```

Everything that watches the state reacts as [the first page](00_hap_system.md) describes:
groups that are on while it is set go off, groups that are on while it is absent come on.

## clear_scene_state

Wipes **all** scene states at once. It takes no name: everything that is set goes, and every
group in the game reacts.

```ruby
clear_scene_state()
```

> **Important:** this is not a bigger `reset_scene_state`. That one clears a single state;
> this one clears the whole board.

## forward_scene_state

Like `set_scene_state`, for the states of a quest, with one safety built in: it only ever
moves the quest **forward**. If the quest is already at that state or past it, nothing happens.
The quest log moves with it.

```ruby
forward_scene_state("q330/RUN_5")
```

```
   the quest's states, in their defined order:

   REQ ── RUN_1 ── RUN_2 ── RUN_3 ── RUN_4 ── RUN_5 ── CLR
                     ▲
                     └── the quest stands here

   forward_scene_state("q330/RUN_5")   ─►  moves it to RUN_5
   forward_scene_state("q330/RUN_1")   ─►  nothing, that is behind it
```

That safety is the reason to use it. The flower quest calls the `RUN_5` line from four
different handlers. Whichever runs first moves the quest; the others change nothing.

**It is the wrong call for accepting a quest.** "Forward" is measured against the quest's own
order, and a quest that has not started yet is not behind `RUN_1` in the way you would expect,
so the jump lands somewhere else: `CLR` rather than `RUN_1`. Use
[`set_scene_state`](#set_scene_state) to move a quest to a named state and keep
`forward_scene_state` for the case it is built for: several handlers that may each set the
same later state, where only the first one should count.

## "...".on?

Asks whether a scene state is set. It hangs off the text itself:

```ruby
if "q900/REQ".on?()
    mess_ex(:GREETING_QUEST_RUNNING)
else
    mess_ex(:GREETING)
end
```

This is the other half of `set_scene_state`: one handler sets the state, another asks about it
and answers differently. The flower handler under [call](#call) is built entirely out of these
questions.

## scene_state?

The same question as `"...".on?`, as a plain call:

```ruby
when scene_state?("q004/DONE") && scene_state?("q005/DONE")
```

Pick whichever spelling reads better.

## event_trigger

Ties a handler to a signal, so it runs when the signal arrives. Written inside `update`, once
per handler:

```ruby
def update()
    event_trigger("shopkeeper_ON_TALK/ev0", "shopkeeper", "ON_TALK")
end
```

How the pieces fit (the name, the target, the signal, and the optional fourth argument) is
covered on the [Scripts](04_scripts.md) page. The signals the game sends on its own are on
[their own page](_signals.md).

## set_trigger_state

Changes which method a trigger runs from now on. The usual reason: the first conversation with
somebody should not repeat itself.

The trigger names its method with an ending, here `/ev0`:

```ruby
def update()
    event_trigger("shopkeeper_ON_TALK/ev0", "shopkeeper", "ON_TALK")
end
```

So talking to the shopkeeper runs `..._ev0`, and inside it sits the switch:

```ruby
def shopkeeper_ON_TALK_ev0()
    mess_ex(:BRING_ME_AN_APPLE)
    set_trigger_state("ev1")
end
```

`set_trigger_state("ev1")` changes the trigger's ending from `/ev0` to `/ev1`. From now on,
talking to him runs `..._ev1`:

```ruby
def shopkeeper_ON_TALK_ev1()
    mess_ex(:DO_YOU_HAVE_IT_YET)
end
```

He asks for the apple once, and every talk after that only asks whether you have it yet:

```
   the trigger stands on /ev0
        │  first talk
        ▼
   shopkeeper_ON_TALK_ev0 runs          "Bring me an apple."
        │  set_trigger_state("ev1")
        ▼
   the trigger stands on /ev1
        │  every talk after
        ▼
   shopkeeper_ON_TALK_ev1 runs          "Do you have it yet?"
```

**A handler that ends a conversation for good sets `"done"`**, which is a state with no
method behind it, so the trigger stops firing:

```ruby
def shopkeeper_ON_TALK_ev1()
    mess_ex(:THANK_YOU)
    set_scene_state("q899/DONE")
    set_trigger_state("done")
end
```

Leaving this out is what makes an NPC stop responding. The handler keeps being re-entered on
every talk, and a conversation that has already run its course has nothing left to show, so
the entity looks stuck. Every vanilla handler that finishes something ends on
`set_trigger_state("done")`.

## wait_signal

Stops the script here until a signal arrives, then carries on with the next line.

The original game mostly uses it to switch an action on and then wait until it reports
`DONE`:

```ruby
def shopkeeper_ON_TALK_ev0()
    enable(door())
    wait_signal("DONE", door())
end
```

The second argument is who sends the signal. With `"global"` it waits for one of the
[signals the game sends by itself](_signals.md):

```ruby
wait_signal("ON_GET_ITEM", "global")
```

Because it stands still, it belongs inside a handler like the one above, never in `update`.

## automess_ex

Like `mess_ex`, but nothing waits for the player: the lines are handed to the window in one go
and it plays them on its own.

The original game uses it for conversations that run alongside the play, like this exchange
between 9S and 2B:

```ruby
automess_ex([
    :M5020_S0070_G0000_001_a9s,
    :M5020_S0070_G0010_001_a2b
])
```

The speaker sits in the message name, as described under Dialogue on the
[Scripts](04_scripts.md) page.

## comumess_ex

A call over the pod. The calling sound plays, a hologram screen opens on the pod, and the
lines play there. This lets somebody who is not standing next to the player speak.

```ruby
M5330_S0020_G0020_001_eml = [
    "はい、何でしょう。",
    "Hey there, guys! What's up?",
    "Oh, salut ! Quoi de neuf ?",
    "Ciao, gente! Come va?",
    "Hallöchen, Leute! Was geht ab?",
    "¡Hola, chicos! ¿Cómo va?",
    "네, 무슨 일이세요?",
    "你好，有什麼事嗎？"
]
M5330_S0020_G0030_001_a9s = [
    "月の涙を見つけました。場所は……",
    "We found a Lunar Tear. It's located at—",
    "On a trouvé une larme lunaire.\nElle se trouve...",
    "Abbiamo trovato una lacrima lunare.\nSi trova presso...",
    "Wir haben eine Mondträne gefunden. Und zwar ...",
    "Encontramos una lágrima lunar. Está en...",
    "달의 눈물을 발견했어요, 장소는……",
    "我發現月之淚了。地點是……"
]

comumess_ex([
    :M5330_S0020_G0020_001_eml,
    :M5330_S0020_G0030_001_a9s
])
```

Emil answers the call, 9S reports, and the message names carry the speakers: `eml` and `a9s`.

## podtalk_ex

Opens the pod conversation screen and writes the lines into it. The player's input moves to
the next line, and the script waits until the last one has been confirmed.

The screen itself stays up afterwards. Close it with `kill_podtalk`:

```ruby
M3005_S1020_G0040_001_pod042 = [
    "ポッド０４２：了解。",
    "Pod 042: Affirmative.",
    "Pod 042 : Affirmatif.",
    "Pod 042: Affermativo.",
    "Pod 042: Bestätigt.",
    "Pod 042: Afirmativo.",
    "포드 042: 알겠다",
    "輔助機０４２：了解。"
]
M3005_S1020_G0050_001_pod042 = [
    "ポッド０４２：Ａ２の随行支援を開始する。",
    "Pod 042: Commencing support of unit A2.",
    "Pod 042 : Soutien de l'unité A2...",
    "Pod 042: Inizio supporto unità A2.",
    "Pod 042: Beginne Unterstützung von Einheit A2.",
    "Pod 042: Iniciando apoyo de la unidad A2.",
    "포드 042: A2의 수행 지원을 시작한다",
    "輔助機０４２：開始隨行支援Ａ２。"
]

podtalk_ex([ :M3005_S1020_G0040_001_pod042, :M3005_S1020_G0050_001_pod042 ])
kill_podtalk()
```

```
   podtalk_ex([ :LINE_1, :LINE_2 ])
        │
        ▼
   the screen opens, LINE_1 shows
        │  input
        ▼
   LINE_2 shows
        │  input
        ▼
   the script carries on, the screen is still up
        │
        ▼
   kill_podtalk()                the screen closes
```

## kill_podtalk

Closes the pod conversation screen.

```ruby
kill_podtalk()
```

## podsele_ex

A question on the pod conversation screen, built like `sele_ex`: the lines, the answers, and
where each answer leads. It opens the screen itself, plays the lines the way `podtalk_ex`
does, and then shows the answers to pick from.

```ruby
M3005_S1020_G0010_001_pod042 = [
    "ポッド０４２：随行支援する機体を選択。",
    "Pod 042: Select unit to support.",
    "Pod 042 : Choix de l'unité à soutenir.",
    "Pod 042: Selezione unità a cui fornire supporto.",
    "Pod 042: Einheit zum Unterstützen auswählen.",
    "Pod 042: Selecciona unidad a la que apoyar.",
    "포드 042: 수행 지원할 기체를 선택",
    "輔助機０４２：選擇隨行支援的機體。"
]
M3005_S1020_G0020_001_sele = [
    "Ａ２を随行支援する。",
    "Support A2.",
    "Soutenir A2.",
    "Supporto ad A2.",
    "A2 unterstützen",
    "Apoyar a A2.",
    "A2를 수행 지원한다",
    "隨行支援Ａ２"
]
M3005_S1020_G0030_001_sele = [
    "９Ｓを随行支援する。",
    "Support 9S.",
    "Soutenir 9S.",
    "Supporto a 9S.",
    "9S unterstützen",
    "Apoyar a 9S.",
    "9S를 수행 지원한다",
    "隨行支援９Ｓ"
]

podsele_ex([ :M3005_S1020_G0010_001_pod042 ], [
    :M3005_S1020_G0020_001_sele,
    :M3005_S1020_G0030_001_sele
], [ "nop", "nop" ], [ :CLEAR ], "x")
kill_podtalk()
```

The third list is where each answer leads: the short endings from [call](#call), one handler
per answer. `"nop"` leads nowhere, so here the picked answer only lands in `$ans`: 0 for the
first, 1 for the second. The original game never puts more than two answers on this screen.

The original game always sends `:CLEAR` with the question, and the last argument is the
speaker. Inside a handler the speaker can be left out; the entity the handler is tied to fills
in. Run outside of one, like here, there is nobody to fall back on and the call fails silently,
so pass a speaker.

## sysmess_ex

A message in the middle of the screen, without a speaker, and the script waits until the
player has read it. The original game uses it for system lines:

```ruby
M3050_S0038_G0075_001_system = [
    "NFCS修復完了。",
    "NFCS fully recovered.",
    "SCR à nouveau opérationnel.",
    "SCVP ripristinati.",
    "NKSS wieder voll einsatzbereit.",
    "NFCS recuperado.",
    "NFCS 복원 완료",
    "ＮＦＣＳ修復完畢。"
]

sysmess_ex(:M3050_S0038_G0075_001_system)
```

## wait_state

Like `wait_signal`, but for a state. A signal lasts a single frame; a state stays until it
changes, so `wait_state` also lets through when the thing already happened before the wait
began.

The original game uses it to hold a script until a fight is over. The
[EnemySetAction](EnemySetAction.md) reports `DONE` when its enemies are dead:

```ruby
def this_fight_a0()
    wait_state("DONE", emset_behind_1())
    mess_ex(:THAT_WAS_ALL_OF_THEM)
end
```

## stp_pl

Stops the player's input, so a conversation or camera moment cannot be walked away from.

```ruby
stp_pl()
```

> **Common mistake:** nothing turns the input back on by itself. Release it on every path out
> of the handler, including when the player backs out of a choice. A forgotten `stp_pl_off`
> leaves the game soft-locked.

## stp_pl_off

Gives the player their input back.

```ruby
stp_pl_off()
```

## stp_em

Freezes the enemies. They stop moving, and their model stops following them: the enemy itself
turns invisible and can still be pushed around, while the model stands frozen where it was.

```ruby
stp_em()
```

## stp_em_off

Lets the enemies run again.

```ruby
stp_em_off()
```

## stp_plcam

Stops the player's input and the camera control together.

```ruby
stp_plcam()
```

## stp_plcam_off

Gives both back.

```ruby
stp_plcam_off()
```

## talk_camera

Turns the camera onto an entity the way a conversation does.

```ruby
talk_camera(shopkeeper())
```

## talk_camera_off

Gives the camera back.

```ruby
talk_camera_off()
```

## fadeout

Fades the screen to black over a number of frames.

```ruby
fadeout(120)
wait(120)
```

The script does not wait for the fade; it carries straight on. The original game always puts
a `wait` of the same length after it, so the next step comes once the screen is dark.

## fadein

Fades back in, the same way.

```ruby
fadein(60)
wait(60)
```

## whiteout, whitein

The same pair in white instead of black.

```ruby
whiteout(60)
wait(60)
whitein(60)
```

## call_se

Plays a sound by its name.

```ruby
call_se("p100_missile_hack_success")
```

## call_bgm

Sends a command to the music by its name, starting and stopping tracks:

```ruby
call_bgm("BGM_Stop_Ending")
```

## quest_mount

Loads a part of a quest, so its lists and scripts exist from then on.

A quest is split into parts. The part holding the accepting conversation is loaded from the
start; the quest itself sits in another part, and accepting is what loads it:

```ruby
def famima_ON_TALK_a1()
    set_scene_state("q005/RUN_1")
    quest_mount(20737)
    set_trigger_state("done")
end
```

The number reads as quest and part once written in hex: `20737` is `0x5101`, quest 005,
part 101. Part `000` is the one with the accepting conversation.

## quest_unmount

Unloads a part again. When a quest is finished, its handlers unload both parts:

```ruby
quest_unmount(20737)    # 0x5101, quest 005, part 101
quest_unmount(20480)    # 0x5000, quest 005, part 000
```

## var_set

Remembers a value under a name. Reading it back is calling that name:

```ruby
var_set("hack_homeru", hack_homeru() + 1)
```

That line is a counter: read the current value, add one, remember it again. Numbers and
`true`/`false` both work, and other scripts can read the same name.

The names are not free to invent: the game only knows the ones registered in its data. New
scene states work when the quest's `quest.toml` defines them beforehand; for vars there is no
such patching yet, so only names the original game already uses work.

## var_add

Adds a number onto a var, instead of reading and writing it yourself:

```ruby
var_add("var_240_pool", @donate)
```

That line is from the donation box: whatever the player gave joins the pool. The same rule as
`var_set` applies: the name must already exist as a key in the game's data.

## getReWard

Hands the player a reward. The number picks which one:

```ruby
getReWard("450")
```

The rewards the game ships with are fixed in its data and cannot be changed. Your own come
from a `reward_<id>.toml` next to the quest; the file name carries the id:

```toml
# reward_450.toml
gold = 0
exp = 20000
[[items]]
item_id = 8501
quantity = 1
```

`getReWard("450")` then hands out exactly that: the money, the experience, and the items. The
item here is itself a custom one, defined the same way in an `item_8501.toml` next to it.

## send_mail

Delivers a mail into the player's inbox. The number picks which one:

```ruby
send_mail(500)
```

Like rewards, the game's own mails are fixed in its data. Your own come from a
`mail_<id>.toml` next to the quest, with the sender, who receives it, and the text:

```toml
# mail_500.toml
sender = "pascal"
send_2b = false
send_9s = false
send_a2 = true
quest_id = 450

[text.subject]
_us = "Someone I met, once"
```

## is_read_mail

Asks whether the player has read that mail.

```ruby
if is_read_mail(500)
    mess_ex(:ABOUT_THAT_MAIL)
end
```

## addItem

Gives or takes an item, by its name. The second argument is how many; a negative number takes:

```ruby
addItem("item_uq_q020_letter", -1)
```

Custom items work the same way. Define one in an `item_<id>.toml` next to the quest and hand
it out under its `name`, which **must start with `item_`**, or the call fails:

```toml
# item_8500.toml
name = "item_scratched_recording_device"
category = 6
max_num = 1
no_trash = true

[text.name]
_us = "Scratched Recording Device"
```

```ruby
addItem("item_scratched_recording_device", 1)
```

## event_mode_begin

Turns the moment into a scene. Until `event_mode_end`, the game keeps interruptions out:
hacking cannot start, and automatic talk, like Emil's
[NEAR_EMILE](_signals.md#near_emile) greeting, waits until the scene is over.

```ruby
def flower_ON_CHECK_req()
    event_mode_begin()
    mess_ex(:LINE)
    event_mode_end()
end
```

Only one script holds the scene at a time. While another script's scene is running, calling
`event_mode_begin` does nothing.

```
   event_mode_begin()        the scene starts
        │
        │   hacking cannot start
        │   automatic talk waits
        │
   mess_ex(:LINE)            the conversation plays undisturbed
        │
        ▼
   event_mode_end()          the scene ends, the game carries on
```

## event_mode_end

Ends the scene and lets the game carry on. Only the script that began it can end it; another
script's call changes nothing.

```ruby
event_mode_end()
```

## call_fiber

Starts a side conversation that knows when to give way. What it does is hidden in the name it
is given, which has three parts:

```
   call_fiber("auto_2_see2B")
               │    │  │
               │    │  └── the rest of the method's name
               │    └── its strength
               └── its family
```

The method `auto_2_see2B` runs as its own little script, filed under the family `auto` with
strength 2. The family and the strength are what make it different from a plain [call](#call):

- Only one conversation per family runs at a time.
- Starting one while an **equally strong or weaker** one of the same family is running throws
  the running one out. Its window closes on the spot, mid-sentence if it has to.
- Starting one while a **stronger** one is running does nothing: the new one is thrown away
  instead.

```
   call_fiber("auto_2_see2B")
        │
        ▼
   is another auto conversation running?
        │
        ├─ no ──────────────────►  auto_2_see2B starts
        │                                │
        ├─ yes, strength 3 ─────►  stronger, nothing happens
        │                                │
        └─ yes, strength 1 or 2 ─►  it is cut off, its window closes,
                                   auto_2_see2B starts in its place
                                         │
                                         ▼
                                   call_fiber waits until it is over
                                         │
                                         ▼
                                   the next line runs
```

So ambient chatter all over the game can share one family, hand out strengths, and never talk
over itself. Emil's greeting is exactly that:

```ruby
def global_NEAR_EMILE_a0()
    call_fiber("auto_2_see2B")
    set_trigger_state("done")
end
```

If a stronger `auto` conversation is running when the player walks up to Emil, his greeting
does not happen. If a weaker one is running, it is cut off and Emil speaks.

Cutting off always hits the other, older conversation. The waiting is for its own: once its
conversation has started, `call_fiber` stands still until it is over, then the line after it
runs. `create_fiber` is the same without the waiting: it starts the conversation and carries
straight on.

## signal

Sends a signal of your own. One line announces that something happened, and anything (in this
script, in another script, or in a list) can react to it.

The name is free to invent. Unlike scene states and vars it needs no registering anywhere:
sender and receiver only have to spell it the same.

```ruby
signal(:battle_start_em0040, global())
```

The second argument is who it is sent to, and that decides who can hear it:

- `global()`: the global group. Anything anywhere can wait for it.
- `this()`: the [ScriptAction](ScriptAction.md) itself. For handlers of the same script
  signalling each other without anyone else listening in.
- an entity variable: that entity, the way the game's own
  [ON_TALK and friends](_signals.md) arrive.

A signal lasts a single frame: whatever is sent during one frame can be picked up on the next,
and is then gone. Something that should stay is a scene state, not a signal.

Receiving works with `wait_signal`, `event_trigger`, or a condition in a list. This condition
is how a list waits for `battle_start_em0040`. The action it sits on holds its enemies back
until the script sends the signal:

```xml
<condition>
	<type>1</type>
	<puid>
		<code str="hap::GroupImpl">0x6a69c84d</code>
		<id>0x0</id>
	</puid>
	<condition>
		<state>
			<label>battle_start_em0040</label>
		</state>
		<pred>0</pred>
	</condition>
</condition>
```

The `hap::GroupImpl` with id `0x0` is the global group, the same place `global()` sent it.

## Sending a signal from an action into a script

The other direction is just as common: an action fires, and a conversation should start. A
[SendCommand](SendCommand.md) aimed at the ScriptAction carries the signal, the script's
trigger catches it, and its handler plays the conversation:

```
   the action                                the script
   ┌─ SendCommand ──────────────┐
   │  puid     the ScriptAction │
   │  command  Action.signal    │───────►   event_trigger("this_podTalk_first/a0",
   │  args     "podTalk_first 0"│                          "this", "podTalk_first")
   └────────────────────────────┘                   │
                                                    ▼
                                          def this_podTalk_first_a0()
                                              podtalk_ex(...)
                                              podsele_ex(...)
                                          end
```

The SendCommand, as the original game writes it:

```xml
<action>
	<code str="SendCommand">0xa10e3dbc</code>
	<name eng="play pod conversation">コマンド送信_UI画面のポッド会話再生</name>
	<id>0x3e5f3d40</id>
	<attribute>0x0</attribute>
	<puid>
		<code str="hap::Action">0xa6aaf7a4</code>
		<id>0xbd85c8a5</id>
	</puid>
	<command>
		<label>Action.signal</label>
	</command>
	<args>"podTalk_first 0"</args>
</action>
```

And the script's side, trigger and handler:

```ruby
def this_podTalk_first_a0()
    podtalk_ex(:M3005_S1020_G0000_001_pod153)
    podsele_ex([ :M3005_S1020_G0010_001_pod042 ], [
        :M3005_S1020_G0020_001_sele,
        :M3005_S1020_G0030_001_sele
    ], [ "a1", "a2" ])
end

def update()
    event_trigger("this_podTalk_first/a0", "this", "podTalk_first")
end
```

The SendCommand fires, the pod conversation opens, and the picked answer carries on in
`..._a1` or `..._a2`.

## pod_noise_appear

Starts the pod's noise, the flicker and sound leading into a transmission. The original game
plays it a moment before opening the pod conversation:

```ruby
pod_noise_appear()
wait(60)
podtalk_ex(:M1080_S0075_G0000_101_pod153)
```

## pod_noise_disappear

Plays the noise the other way round, as the way out of a transmission. It brings its own
noise picture and sound; it is not an undo of `pod_noise_appear`.

Neither of the two removes the noise picture from the screen. `kill_podtalk` does, along with
everything else of the pod conversation.

```ruby
pod_noise_disappear()
wait(60)
kill_podtalk()
```

## phase_jump

Moves the game to a different part of the story, the same jump a
[PhaseJumpAction](PhaseJumpAction.md) makes, from inside a script.

```ruby
phase_jump(0x300, "60_CD_POD")
```

The first argument is the phase group's number, the second the subphase's name. The jump does
not happen on this line: it is noted, and the game rebuilds into the new phase on its next
update.

No script in the original game calls it (the original always jumps through the action), but
it works the same way.

## sphase?

Asks whether the game is currently in that subphase.

```ruby
case
when sphase?("Ending_E")
    call("e")
when sphase?("Ending_F")
    call("f")
end
```

## language

The game's current language, as its position in the message order, **jp en fr it de es ko
zh**, counting from zero. English is 1, German 4.

```ruby
if language() == 4
    mess_ex(:GERMAN_ONLY_JOKE)
end
```

A message uses the same position to show the right line: the game takes the entry at this
position from the eight-language list.

## motion

Lets an entity act: it plays one of the motions defined for its model.

```ruby
motion(:surprise, eng())
motion(:talk, eng())
motion(:idle, eng())
```

If you leave the entity out, it is the one the handler is tied to.

The motion names are not free. They come from a document named `Motion` in `core_hap.dat`,
which defines for each model which names exist and which animation each one plays: a `start`
animation going in, a `loop` animation that holds, and a count, where `-1` keeps looping until
another motion replaces it:

```
   pl0000
    └─ talk        start e050, loop e050, forever
    └─ idle        loop 0031, forever
    └─ dash        start 0008, loop 0002, forever
    └─ shake1      start 108a, once
```

What each model has:

```
pl0000:  talk idle walk dash shake1
pl0100:  talk idle walk lookup
pl0200:  talk sad walk dash idle hacking shake1
pl0300-pl0600:  talk idle walk dash
em0000:  appeal talk idle dash jump fly landing delight
em0010:  talk idle dash jump fly fall landing bark walk frightened
         torch/spear/axe/sword/shield + Idle/Walk/Dash/Jump/Fly/Landing
em0030:  idle walk dash talk
em0040:  talk idle
em0050:  talk idle walk dash jump fly landing delight frightened
         switchPush protect1 protect2 indicate1 raiseHand waveFlag
         theater1-4 Opera1-4, torch/spear/axe/sword/shield sets
em0090:  idle dash jump fly fall landing
em0061, em0100, em2001, emb054:  idle talk
dbfff0:  talk talk2 delight anger sorrow pleasure jump
pl1000:  idle talk talk2 thanks surprise trouble agree hello direct direct2 walk dash fall landing
pl1001:  idle walk dash talk talk2
pl1010:  idle talk
pl1020:  idle walk talk
pl1030:  idle talk dash walk fear
pl1040:  idle talk chairtalk chairtalk2 corpse1 corpsetalk stand stand_dbl
         recovery_start recovery_loop recovery_end
pl1050:  idle chairtalk corpse2 stand talk
pl1100:  idle talk surprise
pl2000:  talk talk2 talk3 delight anger sorrow pleasure bow surprise nod denial
         satisfaction gutspose apologize hand_off salute shock jump idle corpse1
         corpsetalk corpse2 dash angry amazed1 amazed2 walk thanks decent
pl2010:  idle walk dash talk talk2 delight anger sorrow pleasure jump
pl2030:  idle walk chairTalk dash corpse1 corpsetalk
pl2040:  idle talk talk2 walk chairTalk chairTalk2 corpse1 corpsetalk
pl2050:  idle talk talk2 walk chairTalk
pl2060:  idle walk dash talk
wp3000, wp3001:  idle talk talk2 talk3
em5000, em6000:  wait talk agree look, each also as _a and _b
```

A name the model does not have does nothing. The document sits in `core_hap.dat`, so the lists
are fixed.

## anime

The same as `motion` without options. It takes the same names from the same table.

```ruby
anime(:talk)
```

## move

Sends an entity along a route: a [BezierCurveAction](BezierCurveAction.md) passed in as a
variable, like everything else from the [ScriptAction](ScriptAction.md).

```ruby
move(route1(), gatekeeper2())
```

With `[ :run ]` they run instead of walking:

```ruby
move(route1(), pascal(), [ :run ])
```

The script does not stop here: the entity starts walking and the lines after run right away.
The moment worth reacting to comes later: on reaching the end of the route, the entity sends
itself the signal `ON_ROUTE_END`:

```
   move(route(), guide_robo())        the robot starts walking
        │                             the script carries on
        ▼
   the robot follows the route
        │  reaches the end
        ▼
   it sends ON_ROUTE_END
        │
        ▼
   guide_robo_ON_ROUTE_END_a0 runs
```

The guide robot is a complete example: one handler starts the walk, a second one runs at
the end of the route and turns the robot to its spot:

```ruby
def this_guide_route1_s1()
    move(route(), guide_robo())
end

def guide_robo_ON_ROUTE_END_a0()
    turn(loc(), guide_robo())
end

def update()
    event_trigger("this_guide_route1/s1", "this", "guide_route1")
    event_trigger("guide_robo_ON_ROUTE_END/a0", "guide_robo", "ON_ROUTE_END")
end
```

`move` also takes an entity or a [HapLocationAction](HapLocationAction.md) spot instead of a
route. The entity then walks straight to it. The route is for when the way matters, not just
the destination.

## turn

Turns an entity to face something: another entity, or a
[HapLocationAction](HapLocationAction.md) spot:

```ruby
turn(loc(), guide_robo())
```

## wait_ready

Waits until an entity really stands in the world. Switching an action on only starts the
placing, so anything that works with the entity right after has to wait for it first:

```ruby
def flower_ON_CHECK_emile_appear()
    enable(emile())
    wait_ready(emile())
    wait(30)
    mess_ex(:M5330_S0020_G0060_001_eml, nil, emile())
end
```

This is Emil appearing at the flower: switched on, waited for, and only then spoken to.

## buy

Takes money from the player. If they have at least that much, the money is taken and `buy`
returns true. If not, nothing is taken and it returns nil.

```ruby
def sc_f1_ON_TALK_ev1()
    case
    when buy(50000)
        call("ev1_ok")
    else
        call("ev1_ng")
    end
end
```

If the player can pay 50000, the purchase goes through and `ev1_ok` runs. If not, `ev1_ng` runs
and the conversation says so.

> **Important:** `add_money` looks like the way to give money, but its amount is ignored: the
> game's own library adds the result of `price()` instead of the argument. Give money with
> `Nams.add_gold`.

## novel_ex

Writes a passage into the full-screen text reader. The game builds its text adventure out of
it:

```ruby
novel_ex("風呂へ行く", [ :CLEAR ])
novel_ex("風呂にはいって、ゆう飯を済ませて、…")
novel_ex("振返ると、それは東京の藤木博士であった。", [ :CLOSE ])
```

The options ride along per line the way they do in `podtalk_ex`. `:CLEAR` empties the page
before the line writes; without it, new text lands under what is already standing there. The
game marks the last line of a passage with `:CLOSE`.

There is one reader for the whole game, and it keeps what nobody closed: text an earlier
script left behind is still there when the next one writes.

## novsele_ex

A choice in the text reader, built like `sele_ex`: the question, the answers, and where each
answer leads:

```ruby
novsele_ex([ "どこへ行きますか？" ], [
    "風呂へ行く",
    "玄関へ行く",
    "庭へ行く"
], [ "sele_2_1", "sele_2_2", "sele_2_3" ])
```

Where do you go: to the bath, the entrance, or the garden. Each destination is a handler of
the same trigger, reached by its short ending like in [call](#call).

Languages work here as everywhere: a message symbol shows the right line per language; a bare
string shows the same text in every language. The game's text adventure passes bare Japanese
strings, which is why it is Japanese in every language.

> **Note:** calling `novsele_ex` on its own from the script editor's test tool has caused
> problems: the answers came up as empty placeholders. Inside a proper handler it should
> work: the vanilla usage is the text adventure in `qa1c.dat`
> (`qa1c_8c1336a3_scp.bin.rb`), triggered by a PressAreaAction and running entirely as a
> normal script.

## shop

Opens a shop by its name. Every merchant conversation uses this line: the talk ends, the
shop screen opens.

```ruby
def shopkeeper_ON_TALK_ev0()
    mess_ex(:GREETING)
    shop("shop_RC_Grocery")
end
```

The names the game ships with follow one pattern, `shop_<place>_<kind>`: `shop_RC_Grocery`,
`shop_FC_Smith`, `shop_DP_Grocery`, and special ones like `shop_Emile` and
`shop_Achievement`.

## shop_open?

Asks whether a shop screen is open right now.

```ruby
if shop_open?()
    sleep()
end
```

## set_talkrange

Sets the range of the running conversation. The game uses small values like 10 for ordinary
talks and 100 when the speaker is far away or moving around during the conversation.

```ruby
set_talkrange(100)
```

## reset_talkrange

Puts the range back to normal.

```ruby
reset_talkrange()
```

## ui_call

Knows exactly one word: `"DAMAGE"` triggers the player HUD's damage display, anything else
does nothing.

```ruby
ui_call("DAMAGE")
```

## playerHpRate100

The player's health as a number from 0 to 100.

```ruby
event_trigger("player_none/s0", "player", "none") { playerHpRate100() < 75 }
```

That line is from the game: a block after `event_trigger` is an extra condition, and this
trigger only fires while the player is below three quarters health.

## playerHpRate75

Asks directly whether the player is below 75 health percent.

```ruby
if playerHpRate75()
    mess_ex(:YOU_LOOK_HURT)
end
```

## get_em_cnt_rate

How much of the enemy database is filled, as a number from 0 to 100. It counts the same
entries whose unlocking sends [EM_LIST_UNLOCK](_signals.md#em_list_unlock).

```ruby
@rate = get_em_cnt_rate()
```

---

Back to [Scripts](04_scripts.md).
