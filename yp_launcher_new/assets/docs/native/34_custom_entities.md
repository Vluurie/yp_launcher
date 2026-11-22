# How to add a new game object (entity)

Every object is built from its entry in the **object table**. The entry
names the class whose code runs the object, and the object's update
**lane**: the group of objects it is updated with every frame. A new
object id, like the one of your own model file, has no entry there.

NAMS adds entries at runtime. **For accessories and weapons this is
automatic**: the object id from `filename` (accessory) or `obj_id`
(weapon) gets a copy of the vanilla entry of its **category**, the object
with the same name but the last three hex digits set to `0`. So an
`ete696` gets a copy of the entry of `ete000`. For any other new object,
or when that copy is not what you want, add the entry yourself.

## How do I add the entry?

Write `entities\entity_<name>.toml`, where `<name>` is the object name,
the same as the model file name without extension:

```toml
# entities\entity_ete696.toml
template = "ete000"
```

| Field | Default | What it does |
|---|---|---|
| `<name>` (from the file name) | – | The object name: two letters and four hex digits, as in the object's `.dat`. `ete696` is object id `0x4e696`. A vanilla object id is refused. |
| `template` | the object's category, `ete000` for `ete696` | The vanilla object whose entry is copied: its class, lane and the rest of the entry. It has to exist in the object table. |
| `lane` | the template's lane | The update lane, 0 to 17. |
| `class` | the template's class | The name a plugin publishes a behavior class under. The object then runs that class; the rest of the entry still comes from the template. |
| `[intel]` | – | A Unit Data entry for the object, see [How to show your enemy in Unit Data](42_intel.md). |

Pick a template with the same two letters as your object: part of the
setup an object gets when it spawns depends on its own id, not on the
template.

## How do I see that it works?

Every entry NAMS adds is written to the NAMS log, see
[How to debug when something goes wrong](17_debugging.md):

```
[CustomEntity] 0x4e696 -> <class> (from 0x4e000, lane <lane>, class template)
```

The line shows the class and the lane the object got; `from` is the id of
the template. With `class` set, the last part names that class instead
of `template`.

Saving the entity file during play registers the entry again. Objects
created after that use the new entry.

## What if the file is refused?

The log has a line starting with
`[NAMS] Register failed for entity_bundle from mod '<mod id>':`,
followed by one of these:

| Message | Fix |
|---|---|
| `'<name>' is not an object name the game understands (expected a prefix such as et, em or wp followed by three hex digits, e.g. ete696)` | Name the file after the object: two letters and four hex digits. |
| `template '<template>' is not an object name` | Fix `template`. |
| `entity '<name>': 0x<id> is a vanilla object id` | The object already exists; pick a new object name. |
| `entity '<name>': lane <n> is outside 0..=17` | Use a lane from 0 to 17. |
| `entity '<name>': no vanilla object 0x<id> to copy for 0x<id>` | Set `template` to an object that exists. |
| `entity '<name>': unknown behavior class '<class>'` | No loaded plugin publishes a class under that name. |

**The log says `Two mods try to use the same entity_bundle number '<name>'`.**
Another mod adds the same object; the mod that loads first keeps it.

**An accessory or weapon object gets no entry.** The log says
`[CustomEntity] 0x<id>: no vanilla object 0x<id> to copy for 0x<id>`:
its category has no vanilla entry. Write an entity file for the object
with a `template`; the file replaces the automatic copy.
