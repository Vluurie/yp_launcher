# EstId Catalog

An `EstId` is an index into `core.eff` (range `0..=665`) that the engine
plays as a particle / visual effect. These are the values you put in
`[effects].est_id_normal` and `[effects].est_id_berserk` in
`weapon_*.toml` (see [the weapon page](../04_weapon.md)). Try them live
with the **Effect Tester** panel in the egui overlay.

The notes below were written down by hand while sweeping the slider in a
running game. To know for sure what an id does, swap it in and swing.

---

## Categories at a glance

| Range | Typical content |
|---|---|
| `1` – `50` | Enemy / boss death effects and attack telegraphs |
| `80` – `130` | Mixed: energy bulbs, earthquakes, special clears |
| `150` – `230` | Pod programs (phy-def, range-def, mirage, bulb, grafity) |
| `230` – `330` | Item use, hacking, explosions, velocity blurs, fire |
| `410` – `520` | Motion blur, lightning, tornadoes, heals, hacking shockwaves |
| `520` – `570` | Cubic/energetic absorption effects, hacking-style cubes |
| `590` – `641` | **Attack trails for player weapons (all categories)** |
| `650` – `664` | Environmental / impact (metal, water) |

The 590-641 band is the "safe" weapon-trail zone — consistent per weapon
category. Ids outside that band can look good, but they may be scaled
or anchored oddly for a sword.

---

## Full catalog

### 1 – 50 — Enemy attacks & death

| EstId | Notes |
|---|---|
| `1`   | Enemy explosion on detonate |
| `19`  | Enemy explode effect |
| `42`  | Enemy attack — many circles together |
| `44`  | Medium Goliath starting attack (with SFX) |
| `45`  | Goliath in-attack-mode, eye lightning red |
| `46`  | Goliath start-attack (with SFX) |
| `48`  | Enemy start attack — red eye lightning with SFX (also used at go-to-attack mode) |

### 80 – 130 — Misc energy / special

| EstId | Notes |
|---|---|
| `84`  | On-identify: hidden item scanned |
| `87`  | Light telescope — brightens dark areas |
| `92`  | Weapon ring effect — start |
| `93`  | Weapon ring effect — loop |
| `94`  | Weapon ring effect — end |
| `95`  | Energetic charge whirl — small (non-explosive) |
| `96`  | Energetic charge whirl — large |
| `103` | Small earthquake |
| `106` | Special "on clear" effect |
| `109` | Energy electric attack — smooth |
| `123` | Grey energy pulse |

### 134 – 230 — Pod programs

| EstId | Notes |
|---|---|
| `134` | Pod program: Mirage — start |
| `135` | Pod program: Mirage — loop |
| `136` | Pod program: Mirage — end |
| `154` | Pod program: Range Defense — start |
| `155` | Pod program: Range Defense — loop |
| `156` | Pod program: Range Defense — end |
| `157` | Pod program: Physical Defense — start |
| `158` | Pod program: Physical Defense — loop |
| `159` | Pod program: Physical Defense — end |
| `166` | Small bulb energy effect (gravity-center style) |
| `168` | Pod program: Slow-mo end |
| `180` | Very bright — "my eyes hurt". Avoid unless you want a flashbang. |
| `181` | Dark-white energy pulse |
| `185` | Pod program: Gravity-center bulb |
| `201` | Small velocity / screen effect |
| `209` | Pod program: electric void effect |
| `214` | Dark smoke |
| `219` | Angelic big explosion particle with smoke |
| `221` | Hacking explosion on machine |
| `223` | 2D circle |
| `228` | Energetic hacking — yellow charge (bright) |
| `230` | Energetic hacking — yellow charge (softer) |

### 235 – 330 — Items, bombs, hacking

| EstId | Notes |
|---|---|
| `235` | Small light bulbs |
| `237` | Slow-motion bulb effect |
| `238` | Bomb detonation asset |
| `242` | Angelic tiny particles |
| `250` | Light blast with confetti |
| `285` | Very small earthquake + motion blur |
| `291` | Small earthquake + motion blur |
| `292` | Big short earthquake + motion blur |
| `295` | Big earthquake rendered as particles |
| `313` | Small blue whirl (on-hit distance) |
| `314` | Small red whirl (on-hit distance) |
| `324` | Laser-fire combination effect |

### 410 – 520 — Motion blur, lightning, elements

| EstId | Notes |
|---|---|
| `410` | Fire — big |
| `413` | Velocity screen effect with film filter (long duration) |
| `414` | Velocity screen effect |
| `421` | Item-use effect |
| `423` | Huge lightning explosion — "angel-like" |
| `424` | Hacking effect on dying enemies |
| `434` | Purple whirls with purple lightning |
| `435` | ⚠️ **Crashes the game.** Do not use. |
| `436` | ⚠️ **Crashes the game.** Do not use. |
| `437` | Special heal effect |
| `438` | Green heal effect |
| `442` | Blue tornado / whirl — looks great |
| `444` | Yellow/gold lightning (with star particles) |
| `446` | Purple / pink lightning |
| `448` | Hacking shockwaves |
| `485` | Electric-charge motion blur with thunder |
| `490` | Yellow particles |
| `503` | Motion blur with star-like charge-inward |
| `504` | Motion blur |
| `516` | Explosive red/black energetic cubes — candidate for a final/finisher |
| `519` | White circular shockwave shapes |

### 520 – 570 — Cubic / energetic absorption

| EstId | Notes |
|---|---|
| `529` | Like `532` but explosive |
| `532` | Red cube energetic absorption — dense |
| `533` | Red cube energetic absorption — base |
| `538` | Dark energy pulses |
| `560` | Blue hacking-style cubes |
| `561` | Red hacking-style cubes |
| `562` | White hacking-style cubes |

### 590 – 641 — Player weapon trails

These are the slots vanilla weapons use, paired `(normal, berserk)` per
category.

| EstId | Category | State |
|---|---|---|
| `590` / `595` / `600` | SmallSword | normal variants |
| `633`               | SmallSword | berserk |
| `591` / `596` / `610` | LargeSword | normal variants |
| `634`               | LargeSword | berserk |
| `592` / `597` / `620` | Spear | normal variants |
| `635`               | Spear | berserk |
| `593` / `594` / `630` / `640` | CombatBracer | normal variants |
| `636` / `641`       | CombatBracer | berserk |

Within a category, `600`, `610`, `620`, `630` are the generic "normal"
trail; `590–597` are per-weapon unique variants used by signature
weapons (Virtuous, Beastbane, etc.).

### 650 – 664 — Environmental / impact

| EstId | Notes |
|---|---|
| `650` | Attacking metal |
| `664` | Water splashing |

---

## Known crashes

These IDs will crash the game — **never ship a weapon with them**:

- `435`
- `436`

---

## How to extend this catalog

1. Open the NAMS egui menu (Insert key).
2. Expand **Effect Tester (EstId override)**.
3. Check the box, drag the slider, and attack with any weapon.
4. Every swing plays the chosen `EstId`; the slider updates it live.
5. Write down what you see and paste a new row into this file.
