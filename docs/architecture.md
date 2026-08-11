# Architecture

## Scope

The plugin replaces only Fire Emblem: Three Houses' unit level-up stat-roll
path. It does not provide selectable growth modes, perfect growths, zero growths,
random recruitment, overlays, or ROM file replacement.

The implementation has four boundaries:

- `growth.rs` contains deterministic, game-independent accumulation math.
- `level_up.rs` combines FE3H growth sources and decides whether to calculate,
  restore, or defer to vanilla behavior.
- `persistence.rs` owns the save-backed byte layout.
- `game/` and `plugin.rs` contain the FE3H 1.2.0 layouts, offsets, raw memory
  access, and Skyline hook.

## Runtime flow

The Skyline entry point verifies the title ID and the application display
version before installing the hook at FE3H 1.2.0 text offset `0x003D3020`.
Every hook call validates its unit pointer, target level, character ID, class
ID, data-table entries, and persistent save unit before using fixed growths.
If any required input is unavailable, the hook calls the original game
function.

For a valid level-up call, the hook:

1. Reads the unit's current level and ten stored stats.
2. Reads personal growths and personal stat caps from `PersonData`.
3. Reads player class growths from `ClassData`.
4. Calls the game's unit ability-parameter function for growth bonus `0x3A`.
5. Loads validated fixed-growth state from the persistent save unit.
6. Restores a cached result for a duplicate target level, or calculates every
   missing level through the requested target.
7. Stores normalized counters and the result cache before applying the level
   and stats to the unit passed by the game.

Level 1 follows the original game function and then clears the 21 bytes owned
by this plugin for that unit. Level 2 is always calculated from a fresh seed
when the current unit is level 1, so a matching stale cache cannot be restored.

## Stat order and growth formula

FE3H stores the ten fields in this order:

```text
HP, Str, Mag, Dex, Spd, Lck, Def, Res, Mov, Cha
```

The initial counter for each field is:

```text
personal_growth mod 100
```

For every gained level:

```text
effective_growth = max(personal_growth + class_growth + ability_bonus, 0)
points = points + effective_growth
stat_gain = floor(points / 100)
points = points mod 100
```

The ability bonus is omitted for Movement. Stat gains are bounded by the raw
character cap stored in `PersonData`. If a stat is already capped, its counter
is frozen so that it can resume if a later cap change permits another point.
Saint Statue maximum-stat bonuses still need hardware verification; do not
claim exact cap parity for those bonuses before that test passes.

This matches the observable fixed-growth model used by Fire Emblem Engage:
personal growth seeds the hidden counter, then total current growth is added
at each level. It intentionally differs from My 3H Plugin's fixed branch,
which reverse engineering shows seeds a new state by applying one extra total
growth step.

## Persistence

The plugin uses the same unused `Unit.class_level` range as My 3H Plugin:

| Slot | Bytes | Meaning |
| --- | ---: | --- |
| `60` | 1 | Most recent target level |
| `61..70` | 10 | Growth-point remainders, one per stat |
| `71..80` | 10 | Cached HP and nine non-HP stats |

The owned half-open range is `class_level[60..81]`. No separate version byte
or signature is written.

A stored state is accepted only when:

- its target level is between 2 and 99;
- every counter is between 0 and 99;
- cached HP is nonzero;
- cached stats do not exceed the character's caps; and
- it represents the current level or the target being repeated.

Otherwise the state is treated as uninitialized. This provides a natural
fallback for new runs and NG+ without adding another save marker. Save deletion
deletes the state with the save because no external sidecar file exists.

The cached result makes an immediate Divine Pulse replay or another duplicate
call for the same target deterministic. Multi-level rewind behavior still
requires real-hardware testing.

## Confirmed profile facts

The FE3H 1.2.0 profile uses Build ID
`89048449BA238C8CF565518B83BF02D3` and title ID `010055D009F78000`.
The following facts were cross-checked against public FE3H structures and the
reference plugin binary:

- level-up function offset `0x003D3020`;
- ability-parameter function offset `0x000A7E80`;
- persistent-unit lookup offset `0x003CAF30`;
- person and class table offsets and entry layout;
- `Unit` level, stat, and `class_level` offsets;
- storage slots `60..80` and duplicate-result cache behavior; and
- the omission of the ability growth bonus for Movement.

The NRO is compile-tested for the Skyline Switch target. None of these facts
should be described as hardware-verified until the hardware plan passes.
