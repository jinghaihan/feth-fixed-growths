# Development

## Toolchain

Install stable Rust and
[`cargo-skyline`](https://github.com/jam1garner/cargo-skyline). Let
`cargo-skyline` install and link its custom `skyline-v3` toolchain when first
prompted.

## Checks

Run the complete local verification set after changing Rust code:

```sh
cargo fmt --check
cargo test
cargo skyline check
cargo skyline build --release
python3 tools/verify_nro.py \
  target/aarch64-skyline-switch/release/libfeth_fixed_growths.nro
```

The release NRO is generated at:

```text
target/aarch64-skyline-switch/release/libfeth_fixed_growths.nro
```

CI renames the downloadable copy to `feth-fixed-growths.nro`.

## Profiles and unsafe code

All FE3H offsets, counts, and version identifiers belong in
`src/game/profile.rs`. Memory layouts belong in `src/game/layout.rs`, and raw
runtime access belongs in `src/game/runtime.rs` or the small hook adapter.

Never add an offset for another update to the 1.2.0 profile. Create a separate
complete profile and validate the title version before installing any hook.

## Releases

`VERSION` is the release tag source, and CI verifies that it matches the Cargo
package version. Tags must use `v<VERSION>`.

The normal release command is:

```sh
uv run tools/release.py
```

The script runs local checks, updates version files, creates a Conventional
Commit and annotated tag, and explicitly pushes `main` and that one tag. A
`v*` tag triggers `.github/workflows/release.yml`, which builds and verifies
the NRO, creates an installable SD-card ZIP, generates checksums and release
notes, and uploads every file to the GitHub Release page.

Do not run the release command or push a release tag until the CI artifact has
passed the real-hardware test plan.
