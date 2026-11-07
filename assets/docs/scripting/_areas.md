# Areas

An area is a region of the world, used to ask whether the player is inside it.

Many actions carry one: some place entities only while you are near, some start a fight when
you walk in, some move the camera. They all describe the region the same way.

## The three shapes

| Shape    | What it is                            | Fields it uses                                      |
| -------- | ------------------------------------- | --------------------------------------------------- |
| box      | a rectangular block, the usual choice | `position`, `rotation`, `scale`, `points`, `height` |
| cylinder | a circular column                     | `position`, `radius`, `height`                      |
| sphere   | a ball                                | `position`, `radius`                                |

The original game uses boxes for nearly everything, cylinders where a round region reads
better, and spheres almost never.

```
   box                 cylinder            sphere
   ┌─────────┐            ╭───╮              ╭───╮
   │         │            │   │             │     │
   │         │            │   │             │     │
   └─────────┘            ╰───╯              ╰───╯
```

## Where it sits

`position` is the centre. `rotation` turns it, which only means something for a box, and
`scale` sets its size.

A `parent` can be named instead, and then the position is measured from that object rather
than from the map. That is how a region can sit on a moving platform or follow an entity.

## points and height

A box is not only its scale. `points` gives four corners as pairs of numbers, so the base
does not have to be a rectangle, and `height` says how far it extends upward.

For a cylinder and a sphere, `radius` replaces the corners.

## Several regions in one action

The `area` field is a **list**. An action can carry more than one region, and the player
counts as inside when any of them contains them.

That is how one action can cover a room that is not a single simple shape: describe it with
two or three boxes rather than trying to find one that fits.

## link

Instead of describing regions itself, an action can point at another action and borrow its
regions. Changing the shape then only has to happen in one place.
