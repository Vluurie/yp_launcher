# DelayAction

Waits for a number of seconds, and does nothing else.

```
   1.  open the door
   2.  DelayAction  2.0        ⏸  two seconds pass
   3.  say the line               runs afterwards
```

It has to be marked as blocking ([attribute](02_action.md) `0x8`), or the list runs straight
past it.

## The cycle

```
   DELAY  ─────►  DONE
```

It starts in `DELAY` at once, with the number counting the seconds still to go, and reaches
`DONE` at zero.

## delay

The seconds to wait, and the only setting there is. It follows the game clock, so it slows
down with everything else in slow motion.

To wait for something to happen rather than for a fixed time, use a
[ConditionBlock](ConditionBlock.md), which can hold on for a moment afterwards through its
own `delay`.

## A real one from the game

```xml
<action>
	<code str="DelayAction">0x149c7bdf</code>
	<name eng="delay block">遅延ブロック</name>
	<id>0xc4216f70</id>
	<attribute>0x8</attribute>
	<delay>2</delay>
</action>
```

That is every field this action has.
