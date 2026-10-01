# FETH Fixed Growths

Adds fixed growths to Fire Emblem: Three Houses.

[![build](https://github.com/jinghaihan/feth-fixed-growths/actions/workflows/build.yml/badge.svg)](https://github.com/jinghaihan/feth-fixed-growths/actions/workflows/build.yml)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Requirements

- Fire Emblem: Three Houses 1.2.0
- The Fire Emblem: Three Houses Skyline loader. The
  [Aldebaran release](https://github.com/three-houses-research-team/aldebaran-rs/releases)
  provides the game-specific `exefs` files, including `main.npdm` and `subsdk9`.

## Install on Switch

Back up your saves before installing the plugin.

1. Install the FE3H 1.2.0 Skyline loader using the Aldebaran instructions.
2. Download `feth-fixed-growths.nro` from a successful
   [build workflow](https://github.com/jinghaihan/feth-fixed-growths/actions/workflows/build.yml)
   or [release](https://github.com/jinghaihan/feth-fixed-growths/releases).
3. Copy it to:

   ```text
   sdmc:/atmosphere/contents/010055D009F78000/romfs/skyline/plugins/feth-fixed-growths.nro
   ```

4. Fully restart the game and load Fire Emblem: Three Houses 1.2.0.

## Install on Eden or Ryubing

In both emulators, use the FE3H 1.2.0 Aldebaran loader first. Its `exefs`
directory must end up at
`atmosphere/contents/010055D009F78000/exefs/`, with `main.npdm` and
`subsdk9` inside. Then put our `feth-fixed-growths.nro` at
`atmosphere/contents/010055D009F78000/romfs/skyline/plugins/`. Merge
directories; do not replace another plugin's files. The release ZIP contains
our plugin, **not** the loader.

- **Eden:** Open Eden's emulated SD card directory (`%AppData%\eden\sdmc` on
  Windows). Extract the Aldebaran `sd` contents there, then extract our release
  ZIP there or copy the `.nro` to the plugin path above. For Skyline plugins,
  use the emulated SD card rather than **Open Mod Data Location**.
- **Ryubing:** Right-click Fire Emblem: Three Houses and select **Open
  Atmosphere Mods Directory**. This opens the game's
  `sdcard/atmosphere/contents/010055D009F78000` directory. Copy Aldebaran's
  `exefs` there and our `.nro` into its `romfs/skyline/plugins` directory.
  Ryubing's interface may still label this menu as Ryujinx.

Restart the emulator completely before starting the game. If it fails to
launch, remove our `.nro` first to isolate the loader setup from the plugin.

The plugin checks the title ID, display version, and original 1.2.0 level-up
instructions before installing its hook. Unsupported or conflicting executable
patches are left unchanged.

## Behavior

Each stat starts with growth points equal to that character's personal growth.
On every level, the plugin adds personal growth, class growth, and applicable
growth-skill bonuses. Every 100 points grants one stat point and leaves the
remainder for later levels. Movement does not receive the growth-skill bonus.
Every unit then gains at least two distinct stats per level if at least two
uncapped stats are available. Missing gains go to the uncapped stats with the
highest effective growth rates; accumulated points break growth-rate ties.
Fallback gains do not consume accumulated points.

Fixed-growth counters and the most recent level-up result are stored in unused,
save-backed unit fields:
`Unit.class_level[60..81]`, the same 21-byte range used by the reference
plugin. No extra version marker is written. Invalid or stale state is ignored,
and the game's new-game and unit-initialization paths clear the owned range.
The plugin does not clear saved state merely because an internal call targets
level 1. A level-1 unit always recalculates level 2 from a fresh seed, even if
stale bytes survive a save transition.

Existing high-level units begin tracking on their next level; past random
levels are not recalculated. Removing the plugin returns future levels to the
vanilla random system, but it does not undo stats already earned.

## Early-game test reference

For a new game with this plugin active from level 1, these are Byleth's
expected *per-level increases*. Keep Byleth as a Commoner through level 5;
do not equip growth-modifying abilities or reach a stat cap. Gain each level
separately. Compare the stat changes, not Byleth's total stats. A class change
or a save first played without the plugin will produce different results.

| Level | Expected gains |
| --- | --- |
| 1 → 2 | Str, Dex |
| 2 → 3 | HP, Str, Mag, Dex, Spd, Lck, Def, Cha |
| 3 → 4 | Str, Res |
| 4 → 5 | HP, Str, Dex, Spd, Lck, Cha |

Each listed stat gains **one point**; unlisted stats do not change. Byleth's
1 → 2 gains (Str, Dex) and 3 → 4 gains (Str, Res) include the two-stat
minimum: those levels would otherwise have zero or only one natural gain.
No unit should have an empty or single-stat level-up while at least two stats
can still increase.

## Documentation

- [Development guide](docs/development.md) — local builds, checks, artifacts,
  and release preparation
- [Architecture](docs/architecture.md) — hook flow, algorithm, version profile,
  and persistence layout
- [Hardware test plan](docs/hardware-testing.md) — Switch verification checklist

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
