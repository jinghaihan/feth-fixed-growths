# Repository instructions

## Project scope

- The deliverable is a Nintendo Switch Skyline plugin that adds fixed growths
  to Fire Emblem: Three Houses.
- Target Fire Emblem: Three Houses 1.2.0 unless another version is explicitly
  requested and separately profiled.
- Keep the plugin focused on fixed growths. Do not add perfect growths, zero
  growths, seeded growths, random recruitment, or unrelated cheats.
- Do not describe behavior as hardware-verified until it has been tested on a
  real Switch.

## Independent implementation

- Treat My 3H Plugin, `triabolicals/fe3h`, `triabolicals/fe-growths`,
  Aldebaran, and FETH Overlays as prior art and factual references, not source
  templates.
- Do not copy source code, documentation, binaries, or assets from reference
  projects unless their license and attribution requirements have been
  reviewed and intentionally adopted.
- Reverse-engineered facts such as function offsets, structure layouts, and
  observed algorithms may be independently reimplemented and must be
  documented with their target game version.
- Inspect local reference repositories by their external paths. Do not add
  them as remotes or import their tags and branch history.

## Architecture

- Keep the fixed-growth calculation independent from Skyline hooks and raw
  game memory so it can be covered by host tests.
- Keep FE3H structures, offsets, build identifiers, and unsafe pointer access
  in a dedicated game-facing module.
- Keep hook functions small: validate the target, translate game data into
  typed inputs, call the core calculation, then apply validated results.
- Isolate all `unsafe` code and document the invariant that makes each unsafe
  operation valid.
- Preserve vanilla behavior unrelated to growth rolls, including level
  changes, stat caps, promotion bonuses, records, and level-up presentation.

## Save-data safety

- Do not write growth accumulators, markers, or other metadata into vanilla
  save-backed memory until the chosen persistence design is explicitly
  documented and approved.
- Treat writes to unused Unit fields as save modifications even when vanilla
  code ignores those fields.
- Before any save-backed write, verify the complete field range for FE3H
  1.2.0, including DLC data, and provide a cleanup path for bytes owned by the
  plugin.
- If external sidecar storage is used, account for manual saves, autosaves,
  New Game+, Divine Pulse, copied saves, restored saves, and deleted slots.
- Never claim that save modification is reversible when earned stat changes
  have already been committed.

## Target validation

- Fail closed when the running title, update version, or Build ID does not
  match a supported profile.
- Keep offsets and expected instruction signatures together in the versioned
  profile that owns them.
- Validate pointers, indexes, level deltas, growth totals, and stat caps before
  writing game memory.
- Do not enable a partially matched profile or guess offsets at runtime.

## Verification

- Run `cargo fmt --check` after changing Rust code.
- Run host tests for the fixed-growth core after changing algorithms or
  persistence rules.
- Run `cargo skyline check` and `cargo skyline build` in a configured Skyline
  environment before considering the NRO build successful.
- Test zero, fractional, 100%, and over-100% growth rates; multi-level gains;
  class changes; stat caps; recruitment; New Game+; save and reload; and
  Divine Pulse behavior.
- A successful build or emulator test does not prove real-hardware safety.

## Formatting

- Use two spaces for indentation in Rust, TOML, Markdown, YAML, JSON, and other
  text files. Do not use tabs except where a file format requires them.
- Follow `.editorconfig` and format Rust files with the repository's
  `rustfmt.toml` rules.
- Keep public names and comments in English. Prefer comments that explain
  invariants or reverse-engineering evidence over comments that restate code.

## Git and releases

- Use Conventional Commits, for example:
  - `feat(growth): add fixed accumulation`
  - `fix(level-up): handle multi-level gains`
  - `docs: document save-data behavior`
  - `chore: update Skyline dependency`
- Avoid generic scopes such as `switch` while Nintendo Switch is the only
  target. Prefer a responsibility such as `growth`, `level-up`, `save`, or
  `profile`.
- Split distinct concerns into separate commits.
- Preserve unrelated user changes and never rewrite shared history unless the
  user explicitly requests it.
- Do not push commits, create tags, or publish releases unless explicitly
  requested.

## Documentation

- Keep `README.md` concise and user-oriented.
- Put contributor setup and build details in `docs/development.md` when they
  outgrow the README.
- Put module boundaries, hook flows, and persistence behavior in
  `docs/architecture.md` once implementation begins.
- Clearly distinguish confirmed reverse-engineering facts from hypotheses that
  still require binary analysis or runtime testing.
