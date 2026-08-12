# FETH Fixed Growths

Adds fixed growths to Fire Emblem: Three Houses.

[![build](https://github.com/jinghaihan/feth-fixed-growths/actions/workflows/build.yml/badge.svg)](https://github.com/jinghaihan/feth-fixed-growths/actions/workflows/build.yml)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> [!IMPORTANT]
> This plugin has not yet been verified on real Nintendo Switch hardware. Back
> up your saves before testing it. It stores fixed-growth state in unused,
> save-backed unit fields; earned stat changes remain after uninstalling it.

## Requirements

- Fire Emblem: Three Houses 1.2.0
- Atmosphere with Skyline

## Install

1. Download `feth-fixed-growths.nro` from a successful
   [build workflow](https://github.com/jinghaihan/feth-fixed-growths/actions/workflows/build.yml),
   or from a release after hardware testing is complete.
2. Copy it to:

   ```text
   sdmc:/atmosphere/contents/010055D009F78000/romfs/skyline/plugins/feth-fixed-growths.nro
   ```

3. Fully restart the game and load Fire Emblem: Three Houses 1.2.0.

The plugin checks the title ID, display version, and original 1.2.0 level-up
instructions before installing its hook. Unsupported or conflicting executable
patches are left unchanged.

## Behavior

Each stat starts with growth points equal to that character's personal growth.
On every level, the plugin adds personal growth, class growth, and applicable
growth-skill bonuses. Every 100 points grants one stat point and leaves the
remainder for later levels. Movement does not receive the growth-skill bonus.

Counters and the most recent level-up result are stored in
`Unit.class_level[60..81]`, the same 21-byte range used by the reference
plugin. No extra version marker is written. Invalid or stale state is ignored,
and the owned range is cleared when a unit is initialized at level 1. A
level-1 unit also always recalculates level 2 from a fresh seed, even if stale
bytes survive a save transition.

Existing high-level units begin tracking on their next level; past random
levels are not recalculated. Removing the plugin returns future levels to the
vanilla random system, but it does not undo stats already earned.

## Early-game test reference

The following results provide a quick hardware test for a new game. They
assume that fixed-growth tracking starts at level 1, Byleth remains a
Commoner, each house leader remains a Noble, no growth-modifying ability is
active, no stat is capped, and every level is gained individually.

Every listed stat gains one point. Stats not listed gain nothing.

| Unit | Level 1 → 2 | Level 2 → 3 | Level 3 → 4 | Level 4 → 5 |
| --- | --- | --- | --- | --- |
| Byleth | None | HP, Str, Mag, Dex, Spd, Lck, Def, Cha | Res | HP, Str, Dex, Spd, Lck, Cha |
| Edelgard | Str, Cha | HP, Mag, Dex, Spd, Def, Res | Str, Lck, Cha | HP, Mag, Dex, Spd, Cha |
| Dimitri | HP, Str, Dex, Spd, Cha | Def | HP, Str, Dex, Spd, Lck, Cha | Str, Mag, Def, Res |
| Claude | Dex, Spd, Cha | HP, Str, Lck | Mag, Dex, Spd, Def, Res, Cha | Str, Dex, Lck |

Byleth's empty level 2 and large level 3 are expected. Commoner adds no class
growths, so several equal personal growth rates cross 100 points together.
Later class changes alter future growth totals and gradually separate many of
these synchronized stats.

## Documentation

- [Development guide](docs/development.md) — local builds, checks, artifacts,
  and release preparation
- [Architecture](docs/architecture.md) — hook flow, algorithm, version profile,
  and persistence layout
- [Hardware test plan](docs/hardware-testing.md) — the required Switch checks
  before the first release

## Development

Install Rust and `cargo-skyline`, then run:

```sh
cargo fmt --check
cargo test
cargo skyline check
cargo skyline build
```

## Credits

This is an independent implementation. The following projects inspired this
plugin or provided public technical references; their source, documentation,
binaries, and assets are not copied into this repository.

- [My 3H Plugin](https://gamebanana.com/mods/543352) — the original inspiration
  for a focused, standalone fixed-growths plugin.
- [`triabolicals/fe-growths`](https://github.com/triabolicals/fe-growths) —
  reverse-engineering reference for Fire Emblem Engage's fixed-growth
  accumulator update.
- [`triabolicals/fe3h`](https://github.com/triabolicals/fe3h) — public Fire
  Emblem: Three Houses data structures, tables, and runtime function research.
- [FEUniverse Fixed Growths Mode](https://feuniverse.us/t/fe6-fe7-fe8-fixed-growths-mode/4482)
  — earlier accumulator-based fixed-growth implementations and discussion.
- [Aldebaran](https://github.com/three-houses-research-team/aldebaran-rs) — Fire
  Emblem: Three Houses runtime modding and loader research.
- [FETH Overlays](https://github.com/3096/feth-overlays) — Fire Emblem: Three
  Houses 1.2.0 process metadata and Build ID validation reference.
- [`skyline-rs`](https://github.com/ultimate-research/skyline-rs) — Skyline
  plugin runtime dependency included through Cargo under its own license.

Fire Emblem and related names are trademarks of Nintendo and Intelligent
Systems. This unofficial fan project is not affiliated with or endorsed by
them.

## License

[MIT](./LICENSE) License © [jinghaihan](https://github.com/jinghaihan)
