# Patching vanilla scripts

A **ScriptAction** is the action that runs a ruby script. With a
`ruby_patch` you change the script of one ScriptAction.

You don't need the script files. NAMS reads the source back from the
loaded script, applies your edits, checks that the result compiles, and
gives the script the patched source.

## The file

Put a `rubypatches.toml` in your mod. The filename must not start with
one of the prefixes that name an entity kind, like `quest_` or `item_`:
such a file is read as that kind, see
[How to lay out your mod folder](02_mod_folder_layout.md).

```toml
[[ruby_patch]]
action = "0xce7bc976"

[[ruby_patch.edits]]
replace_method = { name = "s0_select", code = """
	def s0_select()
		self.sele_ex([ :M6020_S0010_G0090_300_unknown ], [
			:M6020_S0010_G0100_300_sele,
			:M6020_S0010_G0110_300_sele,
			:M6020_S0010_G0120_300_sele,
			:M6020_S0010_G0130_300_sele,
			:M6020_S0010_G0140_300_sele,
			:M6020_S0010_G0150_300_sele,
			:M6020_S0010_G0160_300_sele
		], [ "nop", "nop", "nop", "nop", "nop", "nop", "nop" ], [ :backclose ])
		if $ans == 6
			return
		end
		@my_actor.setBuddyAiType($ans)
	end
""" }
```

`0xce7bc976` is the ScriptAction `global conversation`, and `s0_select`
is the buddy's combat-style menu in its script. The **Buddy outfit menu
(experimental)** switch changes the same method, so turn it off while
you patch it.

`action` is the id of the ScriptAction you want to change, as a string
in quotes: hex with `0x`, or a decimal number. In the YP Devkit, select
the action in the HAP tree: the inspector shows its **Action ID** with a
copy button.

Everything under `[[ruby_patch.edits]]` is applied in order, so several
edits to the same script are fine. Put all edits for one script into
one `[[ruby_patch]]`: NAMS loads only one `[[ruby_patch]]` per `action`.

## The four edits

**`replace_method`** swaps a whole method. `name` is the method without
`def`/`()`, `code` is the replacement, including the `def` and `end`
lines.

```toml
replace_method = { name = "s0_select", code = "..." }
```

NAMS finds a method only when its `def` line is indented by one tab or
two spaces and reads `def name()`, or `def name` at the end of the
line. A method that takes parameters is not found; change it with
`insert_before` or `insert_after`.

**`insert_before`** and **`insert_after`** put code at a piece of text
you name:

```toml
insert_before = { anchor = "\nend\n\nFiber.new()", code = "..." }
```

If the anchor appears several times, the **last** one is used.
`insert_before` puts a line break between the source before the anchor
and your code, `insert_after` puts your code right after the anchor.

**`append_methods`** adds methods at the end of the class without you
having to name an anchor:

```toml
append_methods = { code = "..." }
```

Your code goes in exactly as written, tabs and blank lines included.

## When it doesn't work

**Nothing changed and the log has no `[ruby_patch]` warning.** The
**Ruby script edits** switch (`ruby_patches` in `nams.toml`; in the
launcher, tab **NAMS**, card **CONTENT FEATURES**, group **Scripts**)
is off, and turning it on needs a restart. Or no action with that id is
loaded.

**One edit did nothing.** NAMS skips an edit whose method or anchor it
cannot find, logs a warning, and still applies the other edits:

```
[ruby_patch] 0xce7bc976: method 's0_select' not found
[ruby_patch] 0xce7bc976: anchor "…" not found; source ends with "…"
[ruby_patch] 0xce7bc976: no place to append methods
```

The source NAMS edits is decompiled from the loaded script, not taken
from a `.rb` file. The `insert_before` warning shows the last 120
characters of that source; take your anchor from there.

**The whole patch did nothing.** The patched script does not compile,
and NAMS keeps the unpatched script:

```
[ruby_patch] 0xce7bc976: patched script does not compile, keeping the original: …
```

When NAMS cannot run the compiler, it applies the patch unchecked and
logs `no mrbc available, applying patch unchecked`.

`[ruby_patch] action 0x… is not a ScriptAction` means the id belongs to
another kind of action.

**The patch was refused when the mod loaded.**

| The log says | Fix |
|---|---|
| `parse ruby_patch: invalid type: integer` | write `action` in quotes |
| `is not a valid action id` | write the id as `0x…` hex or as a decimal number |
| `ruby_patch for '0xce7bc976' has no edits` | add a `[[ruby_patch.edits]]` block |
| `Two mods try to use the same ruby_patch number '0xce7bc976'` | only one mod can patch an action; disable one of the two |
| `lists the same ruby_patch number '0xce7bc976' twice` | merge both blocks into one |

**Your patch on `0xce7bc976` goes away while playing.** The **Buddy
outfit menu (experimental)** switch (`buddy_outfit_menu`, group
**Items & equipment**) changes the same script. Turn it off while you
patch that action.

A ruby patch file changed while playing is added again next to the old
patch instead of replacing it. Restart after you edit it.
