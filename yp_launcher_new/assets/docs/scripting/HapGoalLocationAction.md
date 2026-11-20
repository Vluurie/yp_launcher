# HapGoalLocationAction

Shows the player where to go next: the marker on the map, and the objective text on screen.

## The cycle

```
   WAIT
     │
     ▼
   ACTIVE     the marker is up
     │
     ▼
   DONE
```

The marker is renewed every frame while the action is active, so it keeps up with a target that
moves. Resetting the action takes it away.

## Marking a place

```
   ┌─ HapGoalLocationAction ─────────────────────┐
   │   location    -450.8  13.2  467.2           │
   │   purposeIdx  60                            │
   │   icon        1                             │
   └─────────────────────────────────────────────┘
```

`location` is the spot. `radius` makes it a region instead of a point; leave it at 0 for a
point.

## Marking somebody

Set `parentID` to a target and the marker follows them around instead. It only appears once the
player is close enough.

The target is a **layout id**: the `id` of one `<value>` inside an
[EntityLayoutAction](EntityLayoutAction.md), the same number a
[SendCommand](SendCommand.md) uses when its `puid` is an `app::EntityLayout`.

```xml
   the marker                        a command aimed at the same entity

   <parentID>0x6aae9bcb</parentID>   <puid>
                                         <code str="app::EntityLayout">0xd8efbb45</code>
                                         <id>0x6aae9bcb</id>
                                     </puid>
```

`buddyParent` marks your companion, without naming anyone.

Leave `parentID` at `-1` for no target. A target marked this way keeps the objective icon even
if it already has a shop or quest marker of its own; if it dies, the icon drops back to a plain
one.

## purposeIdx

The objective text to show, given as a number. Setting it turns the objective display on, and
the game remembers it in the save.

The number picks the line out of the game's main story texts. It is filled into two keys,
four digits wide, so `420` becomes:

```
   CORE_MAIN_SCENARIO_TITLE_0420      the heading
   CORE_MAIN_SCENARIO_TEXT_0420       the line below it
```

Those live in `txt_core_add.tmd`, with one exception the game has hardcoded: 40, 50, 60, 70,
80 and 90 are looked up in `txt_core.tmd` instead. That is why the numbers in
`txt_core_add.tmd` jump from 0030 to 0100.

**Leave it out for a quest of your own.** Both files hold main story lines only, and there is
no way to add to them, so the number cannot say anything about your quest. Without it the
marker appears on its own, which is what the one vanilla side quest using this action does.

The objective line of a side quest comes from `[run.N]` in its `quest_<id>.toml` instead, and
moves along with the `RUN_N` scene states.

This could change one day. Custom phases of the game's own kind, `p500` and the like, would
put main story content within reach and with it these texts, but that only becomes worth
building once a lot of people are making quests.

## icon

Whether to draw the marker. Switch it off for an objective that has no single place to point
at, and only the text is shown.

## bunkerFlag

Which of the two worlds the marker belongs to.

```
   bunkerFlag 0    ──►  shown out in the world, hidden in the bunker
   bunkerFlag 1    ──►  shown in the bunker, hidden out in the world
```

## InAnimDisp

Adds an animated target icon on the minimap, on top of the plain marker.

```
   without              with

      ◆                    ◆
   plain marker         plain marker
                       + animated target icon
```

At most ten of these can be up at once. They disappear along with the marker.

## A real one from the game

```xml
<action>
	<code str="HapGoalLocationAction">0x7c4724ad</code>
	<name eng="destination location">目的地ロケーション</name>
	<id>0x883b60bd</id>
	<attribute>0x10</attribute>
	<parentID>0x6aae9bcb</parentID>
	<location>
		<position>1.857849 -54.710205 -47.842087</position>
		<rotation>2.773025 -1.570796 -2.738465</rotation>
		<parent>
			<id>
				<id>
					<code str="app::EntityLayout">0xd8efbb45</code>
					<id>0x188344c3</id>
				</id>
			</id>
		</parent>
	</location>
	<radius>0</radius>
	<icon>1</icon>
	<purposeIdx>130</purposeIdx>
	<bunkerFlag>1</bunkerFlag>
</action>
```

Left out here: `InAnimDisp` and `buddyParent`. They are optional, and leaving them out keeps the defaults.
