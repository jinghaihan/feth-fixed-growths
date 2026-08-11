# FETH Fixed Growths

Adds fixed growths to Fire Emblem: Three Houses.

> [!IMPORTANT]
> This project is in early development. It does not yet install gameplay hooks
> or modify save-backed game data.

## Target

- Fire Emblem: Three Houses 1.2.0
- Nintendo Switch
- Skyline plugin runtime

## Development

Install Rust and
[`cargo-skyline`](https://github.com/jam1garner/cargo-skyline), then run:

```sh
cargo fmt --check
cargo skyline check
cargo skyline build
```

The generated NRO is intended for the following plugin directory:

```text
sdmc:/atmosphere/contents/010055D009F78000/romfs/skyline/plugins/
```

## Prior art

This is an independent implementation informed by publicly documented and
reverse-engineered behavior from these projects:

- [My 3H Plugin](https://gamebanana.com/mods/543352)
- [`triabolicals/fe3h`](https://github.com/triabolicals/fe3h)
- [`triabolicals/fe-growths`](https://github.com/triabolicals/fe-growths)
- [Aldebaran](https://github.com/three-houses-research-team/aldebaran-rs)
- [FETH Overlays](https://github.com/3096/feth-overlays)
