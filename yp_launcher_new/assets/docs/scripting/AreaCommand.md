# AreaCommand

Sends a command when the player walks into a region, and optionally another when they leave.

It is an [AreaAction](AreaAction.md) that acts on its own instead of only reporting. Saves you
an AreaAction plus a [ConditionBlock](ConditionBlock.md) plus a
[SendCommand](SendCommand.md) for the common case.

```
   ┌─ AreaCommand ───────────────────────────────┐
   │   area     a box in front of the door       │
   │   puid     the door                         │
   │   hit      ENABLE                           │
   │   hitout   DISABLE                          │
   └─────────────────────────────────────────────┘
```

## puid

Who receives the commands, picked the same way as on a [SendCommand](SendCommand.md).

## hit and hitout

`hit` goes out the moment the player enters, `hitout` the moment they leave. Command names work
exactly as they do on a [SendCommand](SendCommand.md): a plain name like `ENABLE` is sent as a
state; a name with a dot calls a method.

Both have a value even when you leave them out:

| Left out | What is sent |
|---|---|
| `hit` | `ENABLE` |
| `hitout` | `DISABLE` |

So an AreaCommand with nothing but an area and a target switches that target on when the player
walks in and off when they leave.

## The region

Described under [areas](_areas.md). The four states of the underlying
[AreaAction](AreaAction.md) still apply, so other actions can watch this one as well.

## A real one from the game

```xml
<action>
	<code str="AreaCommand">0x58534a9e</code>
	<name eng="Area command - turn on the drawing of large trees">エリアコマンド-大木の描画をONにする</name>
	<id>0x5c3d72c4</id>
	<attribute>0x0</attribute>
	<area>
		<size>5</size>
		<value>
			<code str="app::area::BoxArea">0x18cffd98</code>
			<position>1046.209961 -19.403515 -406.160645</position>
			<rotation>0 0.523599 0</rotation>
			<scale>2.265525 10.059905 0.5</scale>
			<points>-1 -1 1 -1 1 1 -1 1</points>
			<height>1</height>
		</value>
		<value>
			<code str="app::area::BoxArea">0x18cffd98</code>
			<position>1075.756348 -6.392559 -424.72995</position>
			<rotation>0 0.523599 0</rotation>
			<scale>2.265524 9.192224 0.5</scale>
			<points>-1 -1 1 -1 1 1 -1 1</points>
			<height>1</height>
		</value>
		<value>
			<code str="app::area::BoxArea">0x18cffd98</code>
			<position>1079.315674 -25.5951 -423.539734</position>
			<rotation>0 0.523599 0</rotation>
			<scale>0.747884 7.036579 1.672865</scale>
			<points>-1 -1 1 -1 1 1 -1 1</points>
			<height>1</height>
		</value>
		<value>
			<code str="app::area::BoxArea">0x18cffd98</code>
			<position>1079.315674 -4.217354 -423.539734</position>
			<rotation>0 0.523599 0</rotation>
			<scale>0.747884 29.043217 1.672865</scale>
			<points>-1 -1 1 -1 1 1 -1 1</points>
			<height>1</height>
		</value>
		<value>
			<code str="app::area::BoxArea">0x18cffd98</code>
			<position>1044.756714 -4.258922 -402.696777</position>
			<rotation>0 0.523599 0</rotation>
			<scale>0.747884 31.386681 2.158055</scale>
			<points>-1 -1 1 -1 1 1 -1 1</points>
			<height>1</height>
		</value>
	</area>
	<puid>
		<code str="app::EntityLayout">0xd8efbb45</code>
		<id>0xdcbcdaf7</id>
	</puid>
	<hit>
		<command>
			<label>Behavior.setSuspend</label>
		</command>
	</hit>
</action>
```

Left out here: `hitout`. It is optional, and leaving it out keeps the default.
