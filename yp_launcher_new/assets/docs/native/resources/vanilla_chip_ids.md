# Vanilla chip ids

Plug-in chip ids that already belong to the base game. Don't reuse
these for your own chips.


## Leveled chips (base id + levels 0-8)

Each base id occupies the full range `base..=base+8` because the
chip can be installed at 9 different levels (0-8). For example, base
`0xBB9` (Weapon Attack Up) covers `0xBB9` through `0xBC1`.

| Base hex | Base decimal | Range | Name |
|---|---|---|---|
| `0x00000BB9` | 3001 | `0xBB9 – 0xBC1` | Weapon Attack Up |
| `0x00000BC2` | 3010 | `0xBC2 – 0xBCA` | Down-Attack Up |
| `0x00000BCB` | 3019 | `0xBCB – 0xBD3` | Critical Up |
| `0x00000BD4` | 3028 | `0xBD4 – 0xBDC` | Ranged Attack Up |
| `0x00000BDD` | 3037 | `0xBDD – 0xBE5` | Fast Cooldown |
| `0x00000BE6` | 3046 | `0xBE6 – 0xBEE` | Melee Defence Up |
| `0x00000BEF` | 3055 | `0xBEF – 0xBF7` | Ranged Defence Up |
| `0x00000BF8` | 3064 | `0xBF8 – 0xC00` | Anti Chain Damage |
| `0x00000C01` | 3073 | `0xC01 – 0xC09` | Max HP Up |
| `0x00000C0A` | 3082 | `0xC0A – 0xC12` | Offensive Heal |
| `0x00000C13` | 3091 | `0xC13 – 0xC1B` | Deadly Heal |
| `0x00000C1C` | 3100 | `0xC1C – 0xC24` | Auto-Heal |
| `0x00000C25` | 3109 | `0xC25 – 0xC2D` | Evade Range Up |
| `0x00000C2E` | 3118 | `0xC2E – 0xC36` | Moving Speed Up |
| `0x00000C37` | 3127 | `0xC37 – 0xC3F` | Drop Rate Up |
| `0x00000C40` | 3136 | `0xC40 – 0xC48` | EXP Gain Up |
| `0x00000C49` | 3145 | `0xC49 – 0xC51` | Shock Wave |
| `0x00000C52` | 3154 | `0xC52 – 0xC5A` | Last Stand |
| `0x00000C5B` | 3163 | `0xC5B – 0xC63` | Damage Absorb |
| `0x00000C64` | 3172 | `0xC64 – 0xC6C` | Vengeance |
| `0x00000C6D` | 3181 | `0xC6D – 0xC75` | Reset |
| `0x00000C76` | 3190 | `0xC76 – 0xC7E` | Overclock |
| `0x00000C7F` | 3199 | `0xC7F – 0xC87` | Resilience |
| `0x00000C91` | 3217 | `0xC91 – 0xC99` | Counter |
| `0x00000C9A` | 3226 | `0xC9A – 0xCA2` | Taunt Up |
| `0x00000CA3` | 3235 | `0xCA3 – 0xCAB` | Charge Attack |
| `0x00000CAC` | 3244 | `0xCAC – 0xCB4` | Auto-use Item |
| `0x00000CBE` | 3262 | `0xCBE – 0xCC6` | Hijack Boost |
| `0x00000CD9` | 3289 | `0xCD9 – 0xCE1` | Stun |
| `0x00000CE2` | 3298 | `0xCE2 – 0xCEA` | Combust |
| `0x00000CFD` | 3325 | `0xCFD – 0xD05` | Heal Drops Up |

## Unique chips (single-slot system / HUD / auto)

These chips do not have levels - they occupy exactly one id each.

| Hex id | Decimal | Name |
|---|---|---|
| `0x00000C88` | 3208 | Item Scan |
| `0x00000D06` | 3334 | Death Rattle |
| `0x00000D07` | 3335 | HUD: HP Gauge |
| `0x00000D08` | 3336 | HUD: Sound Waves |
| `0x00000D09` | 3337 | HUD: Enemy Data |
| `0x00000D0A` | 3338 | OS Chip |
| `0x00000D0B` | 3339 | Evasive System |
| `0x00000D0C` | 3340 | Continuous Combo |
| `0x00000D0D` | 3341 | Bullet Detonation |
| `0x00000D0E` | 3342 | Auto-collect Items |
| `0x00000D0F` | 3343 | HUD: Skill Gauge |
| `0x00000D10` | 3344 | HUD: Text Log |
| `0x00000D11` | 3345 | HUD: Mini-map |
| `0x00000D12` | 3346 | HUD: EXP Gauge |
| `0x00000D13` | 3347 | HUD: Save Points |
| `0x00000D14` | 3348 | HUD: Damage Values |
| `0x00000D15` | 3349 | HUD: Objectives |
| `0x00000D16` | 3350 | HUD: Control |
| `0x00000D19` | 3353 | HUD: Fishing Spots |
| `0x00000D1A` | 3354 | Auto-Attack |
| `0x00000D1B` | 3355 | Auto-Fire |
| `0x00000D1C` | 3356 | Auto-Evade |
| `0x00000D1D` | 3357 | Auto-Program |
| `0x00000D1E` | 3358 | Auto-Weapon Switch |
