# FETH Fixed Growths

Adds fixed growths to Fire Emblem: Three Houses.

[![build](https://github.com/jinghaihan/feth-fixed-growths/actions/workflows/build.yml/badge.svg)](https://github.com/jinghaihan/feth-fixed-growths/actions/workflows/build.yml)

> [!IMPORTANT]
> This plugin has not yet been verified on real Nintendo Switch hardware. Back
> up your saves before testing it. It stores fixed-growth state in unused,
> save-backed unit fields; earned stat changes remain after uninstalling it.

## Requirements

- Fire Emblem: Three Houses 1.2.0
- Atmosphere with Skyline

Aldebaran is not required. Do not run this together with My 3H Plugin's growth
features: both hook the same level-up function and use the same unit storage
range.

## Install

1. Download `feth-fixed-growths.nro` from a successful
   [build workflow](https://github.com/jinghaihan/feth-fixed-growths/actions/workflows/build.yml),
   or from a release after hardware testing is complete.
2. Copy it to:

   ```text
   sdmc:/atmosphere/contents/010055D009F78000/romfs/skyline/plugins/feth-fixed-growths.nro
   ```

3. Fully restart the game and load Fire Emblem: Three Houses 1.2.0.

The plugin checks both the title ID and display version before installing its
hook. Unsupported versions are left unchanged.

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

## Prior art

This is an independent implementation informed by publicly documented and
reverse-engineered behavior from these projects:

- [My 3H Plugin](https://gamebanana.com/mods/543352)
- [`triabolicals/fe3h`](https://github.com/triabolicals/fe3h)
- [`triabolicals/fe-growths`](https://github.com/triabolicals/fe-growths)
- [Aldebaran](https://github.com/three-houses-research-team/aldebaran-rs)
- [FETH Overlays](https://github.com/3096/feth-overlays)

Fire Emblem and related names are trademarks of Nintendo and Intelligent
Systems. This unofficial fan project is not affiliated with or endorsed by
them.
