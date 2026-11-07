# PlayerCombatLimitAction

Takes abilities away from the player.

Used to hold the player still during a conversation, or to stop them fighting somewhere they
should not.

## The switches take away, they do not give

Each field is a thing the player can normally do. **Setting it to 1 forbids it.**

| Field | Set to 1 and the player cannot |
|---|---|
| `attack_` | attack |
| `escape_` | dodge |
| `run_` | run |
| `jump_` | jump |
| `gun_` | shoot |
| `skill_` | use skills |
| `suicide_` | self-destruct |
| `walk_` | walk |

So "walking only" means every field at 1 except `walk_`:

```
   attack_  1        run_     1        gun_     1
   escape_  1        jump_    1        skill_   1
   suicide_ 1        walk_    0   ◄── the one thing left
```

## on

The action does nothing at all while this is 0. Switch it to 1 to apply the restrictions.

## area_

Where the restrictions apply. With no area they apply everywhere, which is what most of them
do.

## forceWalkArea_

| forceWalkArea_ | What happens |
|---|---|
| 0 | the switches above apply |
| 1 | the player is made to walk to the area, and can do nothing else |

With 1 the individual switches are ignored while the player is inside the area.

## actIgnore

| actIgnore | What happens |
|---|---|
| 0 | the player can still interact with things nearby |
| 1 | every kind of interaction is blocked |

The block lasts a fifth of a second and is renewed every frame while the restrictions hold, so
it stays in force until they end.

## noCaption_

| noCaption_ | What happens |
|---|---|
| 0 | the subtitle line is hidden when the restrictions begin |
| 1 | the subtitle is left alone |

## A real one from the game

```xml
<action>
	<code str="PlayerCombatLimitAction">0x676cbe32</code>
	<name eng="Movement restrictions (2)">行動制限(2)</name>
	<id>0x1534f138</id>
	<attribute>0x10</attribute>
	<on>1</on>
	<attack_>1</attack_>
	<escape_>1</escape_>
	<run_>1</run_>
	<jump_>1</jump_>
	<gun_>1</gun_>
	<skill_>1</skill_>
	<suicide_>1</suicide_>
	<walk_>0</walk_>
	<area_>
		<size>2</size>
		<value>
			<code str="app::area::BoxArea">0x18cffd98</code>
			<position>2.258001 -46.575272 -4.794243</position>
			<rotation>1.846033 1.557229 1.872432</rotation>
			<scale>2.527436 4.708451 3.79578</scale>
			<parent>
				<id>
					<id>
						<code str="app::EntityLayout">0xd8efbb45</code>
						<id>0x188344c3</id>
					</id>
				</id>
			</parent>
			<points>-1 -1 1 -1 1 1 -1 1</points>
			<height>1</height>
		</value>
		<value>
			<code str="app::area::BoxArea">0x18cffd98</code>
			<position>13.647703 -45.63707 -4.761192</position>
			<rotation>1.749926 -1.551538 -1.493572</rotation>
			<scale>2.527436 6.169755 3.79578</scale>
			<parent>
				<id>
					<id>
						<code str="app::EntityLayout">0xd8efbb45</code>
						<id>0x188344c3</id>
					</id>
				</id>
			</parent>
			<points>-1 -1 1 -1 1 1 -1 1</points>
			<height>1</height>
		</value>
	</area_>
	<forceWalkArea_>1</forceWalkArea_>
	<noCaption_>0</noCaption_>
	<actIgnore>1</actIgnore>
</action>
```

That is every field this action has.
