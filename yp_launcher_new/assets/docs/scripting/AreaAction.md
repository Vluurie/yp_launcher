# AreaAction

Reports whether the player is inside a region, and nothing else.

It places nothing and sends nothing. Other actions watch its state, which makes it the
standard way to ask "is the player here yet".

```
   ┌─ AreaAction ────────────────────────────────┐
   │                                             │
   │   area   a box around the bridge            │
   │                                             │
   └─────────────────────────────────────────────┘
```

The region itself is described under [areas](_areas.md), the same way it is for every action
that carries one.

## The cycle

```
   HIT  ─────────►  INSIDE          the player is in the region
    ▲                  │
    │                  │
    │                  ▼
   HITOUT ◄─────────  OUTSIDE       the player is not
```

Four states, in two pairs. `HIT` and `HITOUT` are the moments of crossing; `INSIDE` and
`OUTSIDE` are the situations that follow.

Crossing in reports `HIT` for one frame and `INSIDE` from then on. Crossing out reports
`HITOUT` once and `OUTSIDE` after that.

Which one you watch for depends on what you want:

| You want | Watch for |
|---|---|
| something to happen the moment the player arrives | `HIT` |
| something to be true the whole time they are there | `INSIDE` |
| something to happen when they leave | `HITOUT` |
| something to be true while they are away | `OUTSIDE` |

A [ConditionBlock](ConditionBlock.md) or a
[ConditionCommands](ConditionCommands.md) does the watching. On its own an AreaAction changes
nothing in the game.

## Watching it from a script

The original game watches areas this way most of the time: a [script](04_scripts.md)
registers a handler on one of the four states, exactly as it would for any other signal:

```ruby
self.event_trigger("area_1_HIT/a0", "area_1", "HIT")
```

The name `area_1` is the variable pointing at the AreaAction, and `HIT` is the state to react
to. Add a scene state as a fourth argument and the trigger only counts during that part of
the story:

```ruby
self.event_trigger("area0_INSIDE/h0", "area0", "INSIDE", "q802/REQ")
```

A script can also ask directly, without waiting for the state:

```ruby
if @area_1.hit?
```

`hit?` answers whether the player is in the region at this moment, and it works on any action
that carries an area, not just this one. The original game does not use it anywhere; the
trigger above is the usual way.

## Blocking

Marked as blocking, the action holds the list until the player walks in. `HIT` releases
it. This is a common way to make a quest wait for the player to reach a place before
continuing.

## Several regions

`area` is a list, so one action can cover several regions at once and report `INSIDE` when
the player is in any of them. See [areas](_areas.md).

## Attribute 0x400

If the group is off, the action still checks the region and reports `INSIDE` or `OUTSIDE`:
once when the group goes off, and again after every reset.

## A real one from the game

```xml
<action>
	<code str="AreaAction">0x8cf2e32</code>
	<name eng="area">エリア</name>
	<id>0xcdc8e234</id>
	<attribute>0x1b</attribute>
	<area>
		<size>5</size>
		<value>
			<code str="app::area::BoxArea">0x18cffd98</code>
			<position>830.677002 -25.546516 -221.779846</position>
			<rotation>0 0.523599 0</rotation>
			<scale>6.093229 24.436195 14.072188</scale>
			<points>-1 -1 1 -1 1 1 -1 1</points>
			<height>1</height>
		</value>
		<value>
			<code str="app::area::BoxArea">0x18cffd98</code>
			<position>794.349976 -42.68623 -257.606506</position>
			<rotation>0 0.523599 0</rotation>
			<scale>2.032006 24.436195 7.27856</scale>
			<points>-1 -1 1 -1 1 1 -1 1</points>
			<height>1</height>
		</value>
		<value>
			<code str="app::area::BoxArea">0x18cffd98</code>
			<position>877.194641 -35.157867 -305.657227</position>
			<rotation>0 0.523599 0</rotation>
			<scale>1.179235 26.448841 8.97479</scale>
			<points>-1 -1 1 -1 1 1 -1 1</points>
			<height>1</height>
		</value>
		<value>
			<code str="app::area::BoxArea">0x18cffd98</code>
			<position>797.069885 -42.68623 -267.372009</position>
			<rotation>0 0.523599 0</rotation>
			<scale>9.460578 24.436195 1.67835</scale>
			<points>-1 -1 1 -1 1 1 -1 1</points>
			<height>1</height>
		</value>
		<value>
			<code str="app::area::BoxArea">0x18cffd98</code>
			<position>804.274597 -42.68623 -254.893188</position>
			<rotation>0 0.523599 0</rotation>
			<scale>9.460578 24.436195 1.67835</scale>
			<points>-1 -1 1 -1 1 1 -1 1</points>
			<height>1</height>
		</value>
	</area>
</action>
```

That is every field this action has.
