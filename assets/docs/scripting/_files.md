# The files a quest is made of

Where a quest sits on disk is covered in [Making a change](03_getting_started.md). What is
inside only matters for reading the original content or writing the XML by hand. Working in
the editor you never see any of it.

Everything lives in one archive named after the quest, `q899.dat`, and inside it:

```
   q899.dat
    ├─ q899_hap.pak            the group tree, and the lists that load with the quest
    ├─ q899_<groupid>_grp.pak  the lists of one loading point, if the quest has any
    └─ q899_<id>_scp.bin       one compiled script per script
```

## Inside the paks

Every document is one XML file, numbered from zero. `q899_hap.pak` always starts with the
group tree:

```
   q899_hap.pak
    ├─ 0.xml     the group tree      all groups of the quest, in one document
    ├─ 1.xml     a list              its actions
    └─ ...
```

A quest with sixteen groups and a dozen lists still has exactly one group document.

A group marked as a loading point (`0x40` on the [groups page](_groups.md)) keeps its lists
in a pak of its own, named after that group's id, and holding nothing but lists:

```
   q020_hap.pak                 0.xml, the group tree, and nothing else
   q020_92a5bc1b_grp.pak        the lists of group 0x92a5bc1b
   q020_7164bb19_grp.pak        the lists of group 0x7164bb19
```

That is how the game gets to load a part of a quest only when it is needed. A quest without
loading points keeps everything in `q899_hap.pak`.

## The group document

One `<group>` per group, all of them side by side under `<root>`, and `<count>` says how many
follow:

```xml
<?xml version="1.0" encoding="utf-8"?>
<root>
	<count>1</count>
	<group>
		<id>0xf58ede6f</id>
		<name eng="Pause while hacking (7)">ハッキング中停止(7)</name>
		<parent>0x4f590fa5</parent>
		<tokens>
			<size>1</size>
			<value>
				<code str="Hacking">0x743b077d</code>
				<id>0x1</id>
			</value>
		</tokens>
		<attribute>0x20</attribute>
	</group>
</root>
```

The tree is not written as nesting. Every group stands flat under `<root>` and names its
parent by id, so the shape from the [groups page](_groups.md) is built out of those `parent`
lines.

`tokens` is what the group watches, described on the
[token categories page](_token_categories.md). `attribute` is the group attribute from the
[groups page](_groups.md).

A group without tokens leaves `tokens` out.

The topmost group's parent is the quest name as a group id: `CRC32("q899") & 0xefffffff`.

## A list document

One document per list. The list's own values sit at the top, then its actions:

```xml
<?xml version="1.0" encoding="utf-8"?>
<root>
	<name eng="script (8)">スクリプト(8)</name>
	<id>0x570f8101</id>
	<group>0xf58ede6f</group>
	<priority>7</priority>
	<size>2</size>
	<action>
		<code str="ScriptAction">0xbeaf915f</code>
		...
	</action>
	<action>
		...
	</action>
</root>
```

`group` is the id of the group the list belongs to, the switch from [the HAP
page](01_hap.md). `size` is how many actions follow. The actions themselves are described on
their own action pages.

## Where a script's id points

A [ScriptAction](ScriptAction.md) names its script by id:

```xml
<script>
	<id>0xfffadf03</id>
</script>
```

That id is the file name. The script is `q005_fffadf03_scp.bin` next to the pak, the compiled
form of the Ruby from the [Scripts](04_scripts.md) page.

```
   q005.dat
    ├─ q005_hap.pak
    │   └─ 1.xml    <script><id>0xfffadf03</id></script>
    │                              │
    └─ q005_fffadf03_scp.bin   ◄───┘
```

## The readable attributes are not in the game's files

Two attributes turn up all over these documents:

```xml
<code str="ScriptAction">0xbeaf915f</code>
<name eng="script (8)">スクリプト(8)</name>
```

Neither is in the archive. The game stores the number, and the name; the tool that converts a
document to readable XML adds `str` and `eng` beside them so you can read what a number stands
for.

Changing one of those attributes changes nothing. The value next to it is what counts.

Names are yours to write, in any language, and the game keeps whatever it finds. Where the
original content had nothing to translate it writes the name plainly:

```xml
<name>q020/CLR</name>
```

## Where the numbers come from

Most numbers in these files are not counted up. They are the name, run through **CRC32** —
the ordinary one, the same as in zip files: table polynomial `0xEDB88320`, starting at
`0xFFFFFFFF`, result inverted.

```
   "ScriptAction"   ─►  0xbeaf915f       the code of an action
   "ON_TALK"        ─►  0x28cb0efb       the name of a signal
```

So a `<code>` can be worked out from the name it stands for, and the same name always gives
the same number.

Two details the game adds:

**Group ids** are that CRC with the top bits trimmed, `crc & 0x0fffffff`.

**Some names are hashed case-insensitively**, folded to lower case first, and the result cut
to 31 bits with `& 0x7fffffff`. Names of 128 characters or more hash to 0 there.

## Ids when writing by hand

Ids are handed out when the file is loaded, and two that come out the same get one of them
replaced. Writing XML by hand you pick them yourself, and only two rules matter: unique
inside the quest, and every reference spells the same number as the thing it points at.

The editor takes this over completely.

---

Back to [the list of actions](README.md).
