# HapMultiMarkLocationAction

Puts a marker on the map.

This is what draws the quest markers, the shop icons, and the exclamation mark over somebody
who has something to say.

## Two ways to place it

```
   ┌─ at a place ────────────┐      ┌─ on somebody ───────────┐
   │   location  x y z       │      │   parentID  <an entity> │
   │   radius    12.0        │      │                         │
   │      ●                  │      │       ☺                 │
   │    a spot on the map    │      │   follows them around   │
   └─────────────────────────┘      └─────────────────────────┘
```

`location` marks a fixed spot, with `radius` for how large the marked area is.

`parentID` instead names an entity, and the marker rides along with it. The entity is looked up
among the ones the game can lock onto, and only once the player is close enough. Until then
nothing is drawn.

Leave `parentID` at 0 to mark a place; set it to mark somebody.

## markIdx

Which marker to draw.

| markIdx | What it shows |
|---|---|
| 3 | a quest marker |
| 4 | a main quest marker |
| 5 | a finished quest marker |
| 1, 2 | shop markers |
| 8 | somebody has something to say |
| 9 | an unnamed one |

Only 3, 4 and 5 see much use in the game.

## Markers do not overwrite each other freely

When two of these aim at the same entity, the more important one wins.

```
   main quest marker    ──►  never replaced by anything
   finished quest       ──►  replaced only by the main quest one
   quest, shop, talk    ──►  step aside for either of the above
```

So a shopkeeper who is also a quest target keeps the quest marker, not the shop icon.

## radius

How large the marked region is, for a marker at a place. It only matters when `areaIdx` is set
to 1, which switches the marker to an area rather than a point.

## ftTag

Names a fast travel point, and the marker is then tied to it. Left empty in nearly every one of
them; the handful that use it name points like `ft_RC` or `ft_PV`.

## questId and uiMessID

`questId` says which quest the marker belongs to. `uiMessID` picks the line of text shown with
it.

## The states it reports

The action reports `ICON_VISIBLE` while its marker is on screen and `ICON_INVISIBLE` while it
is not. Nothing in the game waits for either, so they are there to be read rather than acted
on.

## A real one from the game

```xml
<action>
	<code str="HapMultiMarkLocationAction">0xb0757267</code>
	<name eng="Generic mark location">汎用マークロケーション</name>
	<id>0x7f5c50e9</id>
	<attribute>0x10</attribute>
	<location>
		<position>-33.6138 2.154199 -42.896538</position>
		<rotation>-1.593264 -1.570308 0.029378</rotation>
		<parent>
			<id>
				<id>
					<code str="app::EntityLayout">0xd8efbb45</code>
					<id>0x188344c3</id>
				</id>
			</id>
		</parent>
	</location>
	<parentID>0xa3e15a4e</parentID>
	<questId>0x410</questId>
	<uiMessID>44</uiMessID>
	<radius>0</radius>
	<ftTag>""</ftTag>
	<markIdx>5</markIdx>
	<areaIdx>1</areaIdx>
</action>
```

`areaIdx` and `markIdx` are optional and other blocks leave them out, but this one fills them in.
