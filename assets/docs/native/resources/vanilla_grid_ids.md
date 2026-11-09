# Vanilla grid and room ids

Every grid id the vanilla game ships. Pick anything **not** on this
list for a custom map (see
[How to make a custom map](../32_custom_maps.md)). The first digit of
the grid id is the `wd` folder it lives in; the first three digits are
its room id (`st` folder).

## `wd5` — phase-loaded rooms (use this layer for custom maps)

Taken ids:

```
g50001  g50010  g50100  g50200  g50300
g52001  g53001
g5a000  g5a001  g5a002  g5a100  g5a200  g5a300  g5a400  g5a500
g5a800  g5a900  g5aa00  g5ab00  g5ac00
g5f000  g5f001
```

Everything else in `g50000`–`g5ffff` is free — for example `g5f100`,
`g5f200`, `g5b000`.

Known rooms: `g5ab00` is the Debug Room (a standalone copy of the
desert arena).

## `st5` — room files shipped by vanilla

```
r500  r501  r502  r503  r509  r520  r530  r550  r551
r5a0  r5a1  r5a2  r5a3  r5a4  r5a5  r5a8  r5a9  r5aa  r5ab  r5ac
```

(`r509`, `r550`, `r551` have no `.dtt`; all others come as
`.dat` + `.dtt` pairs.)

## Other layers (overworld streaming — not for custom maps)

- `wd1` (`st1` rooms `r100`–`r170`): the streamed overworld,
  `g102xx`–`g120xx`, one grid per world tile.
- `wd2` (`st2` room `r200`): `g202xx`–`g209xx`, plus `g20923` and
  `g29999`.
- `wd3`: `g311xx`–`g315xx`.
- `wd4`: `g406xx`–`g414xx`.
- `wda`: `ga0000`–`ga0002`.

These layers stream by world position; adding grids here does not give
you a loadable custom map. Use `wd5`.
