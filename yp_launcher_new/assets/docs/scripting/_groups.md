# Groups

A group is the power switch for the lists inside it. For what a list does when its switch
goes on and off, see [The HAP](01_hap.md).

## What a group carries

```
   ┌─ group ─────────────────────────────────────┐
   │  id          0xfcb689f6                     │
   │  name        "disabled in bunker"           │
   │  parent      0x94163e31                     │
   │  tokens      the bunker room                │
   │  attribute   0x10                           │
   └─────────────────────────────────────────────┘
```

The name is for you, the id is handed out by the game, and the parent makes groups form a
tree. The two that decide anything are the tokens and the attribute.

**Tokens** are what the group watches: a scene state, a room, a part of the story. Each kind
of token is described under [token categories](_token_categories.md).

**The attribute** decides how the group reacts to what it watches.

## The four directions

The group turns its content off. The question is when, and how hard.

| Attribute | The content is |
|---|---|
| `0x0` | **off while the watched thing is absent**, on while it is there |
| `0x10` | **off while the watched thing is there**, on while it is absent |
| `0x20` | **paused while it is there** |
| `0x30` | **paused while it is absent** |

`0x0` is the plain group from [The HAP](01_hap.md): the scene state gets set, the group
comes on. `0x10` is the same switch read the other way round, and the pair of them is what
swaps a place from one version to another with a single scene state.

The names in the original game say it directly. This group is off whenever the player is in
the bunker room:

```xml
<group>
	<id>0xfcb689f6</id>
	<name eng="disabled in bunker">バンカーで無効</name>
	<parent>0x94163e31</parent>
	<tokens>
		<size>1</size>
		<value>
			<code str="GlobalRoom">0x5cda8ece</code>
			<id>0x550</id>
		</value>
	</tokens>
	<attribute>0x10</attribute>
</group>
```

And this one pauses its content in six particular rooms:

```xml
<group>
	<id>0x3b7fcae0</id>
	<name eng="Stop during special stage">特殊ステージ中停止</name>
	<parent>0x864df678</parent>
	<tokens>
		<size>6</size>
		<value>
			<code str="GlobalRoom">0x5cda8ece</code>
			<id>0x502</id>
		</value>
		<value>
			<code str="GlobalRoom">0x5cda8ece</code>
			<id>0x503</id>
		</value>
		<value>
			<code str="GlobalRoom">0x5cda8ece</code>
			<id>0x530</id>
		</value>
		<value>
			<code str="GlobalRoom">0x5cda8ece</code>
			<id>0x5a8</id>
		</value>
		<value>
			<code str="GlobalRoom">0x5cda8ece</code>
			<id>0x5a9</id>
		</value>
		<value>
			<code str="GlobalRoom">0x5cda8ece</code>
			<id>0x5aa</id>
		</value>
	</tokens>
	<attribute>0x20</attribute>
</group>
```

Several tokens mean *any of them*: one loaded room out of the six is enough.

`0x30` is valid but the original game never uses it.

## Off and paused are not the same

**Off** ends the content. Every action is frozen and told that it ended: each one sends its
`ON_DISABLE` signal, so anything watching those actions sees them stop. When the group comes
back, the list continues where it stood, or starts over if the list is marked to reset (see
[what being switched off actually does](01_hap.md)).

**Paused** only freezes. Nothing is told anything; when the group comes back, everything
carries on mid-step as if no time had passed.

That is why the hacking groups in the original game pause (停止) rather than disable: after
hacking a fight resumes exactly where it was, instead of every enemy announcing its end and
coming back reset.

## Parents

A group that is off takes everything below it along: every child group and every list under
them is off too, whatever their own tokens say.

```
   Resistance Camp        off
    ├─ before the attack       off  ◄── no matter what it watches
    └─ after the attack        off  ◄──
```

A group that is on forces nothing. Its children still decide for themselves.

## Starting switched off, 0x2

Add `0x2` and the group starts off, whatever its tokens say, until a
[script](04_scripts.md) turns it on:

```ruby
the_group.enable     # or .on
the_group.disable    # or .off
```

A reset puts it back to its starting side: off with `0x2`, on without.

## Loading its own file, 0x40

A group marked `0x40` is a loading point. Its content lives in a file of its own, and the
group fetches that file when the place it belongs to (a phase, a quest or a room) is
there, and throws it away again when it is not. Everything in between happens by itself.

This is how the game keeps a whole area's quests out of memory until the player gets there,
and it is the mark on the group folders you see as `..._grp` files among the extracted data.

## 0x4

You will see `0x4` in original files, next to `0x2`. The game writes it as its own bookkeeping
and never reads it as a switch. Leave it out.
