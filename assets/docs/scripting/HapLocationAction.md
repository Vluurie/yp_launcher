# HapLocationAction

A spot in the world: a position, and a direction to face in.

```
   ┌─ HapLocationAction ─────────────────────────┐
   │   location   0  0  -2.5                     │
   └─────────────────────────────────────────────┘
```

`location` is the only setting. Name an entity instead of coordinates and the spot moves with
it.

## It does nothing on its own

The action has no update, no start, no end. It sits there and waits to be looked up.

```
   ┌─ the action ────────┐            ┌─ whoever needs it ──┐
   │  a position         │  reads ◄───│  looks it up        │
   │  a rotation         │            │                     │
   └─────────────────────┘            └─────────────────────┘
```

The hacking minigame is one thing that does this: it takes the position and rotation from one
of these to aim its camera.

## A real one from the game

```xml
<action>
	<code str="HapLocationAction">0x43173c6f</code>
	<name eng="Location - for positioning">ロケーション-位置確認用</name>
	<id>0x647554ba</id>
	<attribute>0x1</attribute>
	<location>
		<position>-647.98291 0.008633 443.070374</position>
		<rotation>0 3.089233 0</rotation>
	</location>
</action>
```

That is every field this action has.
