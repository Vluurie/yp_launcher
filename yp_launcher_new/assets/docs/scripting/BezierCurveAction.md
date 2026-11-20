# BezierCurveAction

A path drawn through the world, for something else to travel along.

## It does nothing on its own

The action has no update at all. It holds the path and waits to be read.

```
   ┌─ the action ────────┐            ┌─ whoever uses it ───┐
   │  the curve          │  reads ◄───│  moves along it     │
   └─────────────────────┘            └─────────────────────┘
```

Emil is the clearest example: he keeps the id of one of these, and every frame he asks it for
his next position.

## The shape

A curve is a list of **nodes**, each with a **control point** that bends the line into the
next one.

```
              handle
                 ╲
        ●─────────╳         ● node
       ╱           ╲        ╳ handle
      ╱             ●───────────╲
     ●                           ╲
   start                          ●  end
```

Node and control point come in equal numbers, and each pair describes one segment.

## Open or closed

One setting decides whether the last node connects back to the first.

```
   open                          closed

   ●───●───●───●                 ●───●───●
                                 │       │
   travel ends at the last       ●───●───●
   node                          travel wraps round for ever
```

Both are common.

## Moving along it

Whoever uses the curve keeps one number: how far along it they are. The whole part is the
segment they are in; the fraction is where inside that segment.

```
   progress 0.0     at the first node
   progress 1.5     halfway through the second segment
   progress 3.0     at the fourth node
```

Each frame the user asks for a distance to advance, and gets back a new position and a new
progress number.

```
   advance by 2 metres
        │
        ▼
   ●─────────►●              still inside this segment
   ●──────────────────►●     ran past its end, carried into the next one
```

The distance is a real distance, not a share of the curve. So a long segment takes longer to
cross than a short one at the same speed, which is what makes movement look even.

To make that work each segment stores its own length, worked out once when the curve is
loaded.

Whoever uses the curve also asks whether the end has been reached. On a closed curve the
wrap-around segment counts, so the end never comes.

## upVec

Each node carries an up direction, and it defaults to straight up.

The user of the curve reads the up direction of the node it is currently at and builds its
facing from that. That lets a path roll whatever travels it.

Most of the ones in the game point **straight down**, which turns the traveller upside down
as it goes.

If the progress runs past the last node, the default straight up is used rather than the last
node's value.

## param

Each node has room for one more reference. Nothing in the game fills it in.

## A real one from the game

```xml
<action>
	<code str="BezierCurveAction">0x5874fcd9</code>
	<name eng="Bézier Curve - Small Flying Appearance (3)">ベジエカーブ-小型飛行出現(3)</name>
	<id>0xef525c43</id>
	<attribute>0x0</attribute>
	<curve>
		<attribute>0x637be90</attribute>
		<controls>
			<size>2</size>
			<value>
				<cp>-652.386108 -2.210246 308.553284 -652.47406 1.188534 308.500244</cp>
			</value>
			<value>
				<cp>-648.177368 -0.549903 314.141754 -648.177368 -0.549903 314.141754</cp>
			</value>
		</controls>
		<nodes>
			<size>2</size>
			<value>
				<point>-651.523376 -11.154499 308.732605</point>
			</value>
			<value>
				<point>-648.177368 -0.549903 314.141754</point>
			</value>
		</nodes>
	</curve>
</action>
```

That is every field this action has.
