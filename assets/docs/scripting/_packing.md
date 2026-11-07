# Packing a quest by hand

To turn the documents from [the files a quest is made of](_files.md) into the `.dat` the game
reads, use [F-SERVO](https://github.com/ArthurHeitmann/F-SERVO).

If you work in the editor you can skip this: it builds the archive when you save.

## The folder F-SERVO expects

Open an existing `.dat` in F-SERVO and it unpacks into this shape. To write a quest from
nothing, build the same shape and let F-SERVO pack it:

```
   nier2blender_extracted/
    └─ <quest>.dat/                     the archive, as a folder
        ├─ dat_info.json                what goes into the archive
        ├─ <quest>_<id>_scp.bin         the script the game reads
        ├─ <quest>_<id>_scp.bin.rb      the Ruby you write
        └─ pakExtracted/
            └─ <quest>_hap.pak/         a pak, as a folder
                ├─ pakInfo.json         what goes into the pak
                ├─ 0.xml                readable form, what you edit
                ├─ 0.yax                packed form, what goes in
                ├─ 1.xml
                ├─ 1.yax
                └─ ...
```

Both halves of a file sit side by side, and you only ever touch the readable one: F-SERVO
turns `0.xml` into `0.yax` and `<quest>_<id>_scp.bin.rb` into `<quest>_<id>_scp.bin`. The
`.yax` and the `.bin` are never written by hand.

## The order

Each step packs the **output of the step before it**, not the readable file: the pak is built
from the `.yax`, the dat from the `.bin` and the `.pak`. For a quest written from scratch none
of those outputs exist yet, so the steps have to run in this order:

1. **`.xml` → `.yax`** — right-click each XML in F-SERVO → *Export YAX*.
2. **`.bin.rb` → `.bin`** — right-click the Ruby file → *Compile to .bin*.
3. **Repack the `.pak`** — right-click the pak → *Repack PAK*. Reads the `.yax` files; run
   before they exist and it fails on `0.yax`.
4. **Repack the `.dat`** — right-click the dat → *Repack DAT*. Reads the `.bin` and the
   `.pak`; a file listed in `dat_info.json` that is missing on disk makes F-SERVO treat the
   folder as a stale extract instead.

After the first full pack, editing a file in F-SERVO and saving runs the conversion for the
changed file on its own, so the full chain by hand is only needed the first time.

## dat_info.json

Names the files the archive holds, in the order they are stored:

```json
{
	"version": 3,
	"files": [
		"<quest>_<id>_scp.bin",
		"<quest>_hap.pak"
	],
	"original_order": [
		"<quest>_<id>_scp.bin",
		"<quest>_hap.pak"
	],
	"basename": "<quest>",
	"ext": "dat"
}
```

`files` and `original_order` hold the same names, and the scripts are listed as `.bin`.
`basename` and `ext` give the archive its name.

A quest with loading points lists their `<quest>_<groupid>_grp.pak` here as well.

## pakInfo.json

Names the documents inside one pak, in order, by their `.yax` name:

```json
{
	"files": [
		{ "name": "0.yax", "type": 4 },
		{ "name": "1.yax", "type": 6 },
		{ "name": "2.yax", "type": 6 }
	]
}
```

The order is the numbering from [the files page](_files.md): `0.yax` is the group tree, the
rest are lists.

`type` is worked out again while packing and whatever stands in the file is ignored, so it
does not have to be right when you add a document by hand. What it ends up as:

```
   4    the first document, over 1 KB          (compressed)
   1    the first document, under 1 KB
   6    a later document, over 1 KB            (compressed)
   3    a later document, under 1 KB
```

---

Back to [the list of actions](README.md).
