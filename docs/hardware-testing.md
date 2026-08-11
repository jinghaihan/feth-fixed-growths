# Hardware test plan

No release should be tagged until this plan passes on Fire Emblem: Three
Houses 1.2.0 running on a real Nintendo Switch.

## Preparation

1. Back up every save slot outside the console.
2. Remove My 3H Plugin and any other plugin that hooks unit level-up.
3. Install the CI artifact as:

   ```text
   sdmc:/atmosphere/contents/010055D009F78000/romfs/skyline/plugins/feth-fixed-growths.nro
   ```

4. Fully restart the console or game process.
5. Keep an unmodified copy of the pre-test save for byte and stat comparison.

## Required checks

### Startup and version gate

- Confirm FE3H 1.2.0 reaches the title screen and loads a save normally.
- Confirm the game starts with only this plugin installed.
- If another FE3H update is available for testing, confirm the plugin logs an
  unsupported-version message and leaves vanilla level-ups unchanged.

### First fixed level

- Record a level-1 unit's personal growths, class growths, current stats, and
  any growth ability bonus.
- Seed each expected counter with personal growth.
- Add the current total growth once and record every field reaching 100.
- Gain exactly one level and compare all ten stored fields. Movement must not
  receive the ability growth bonus; Charm must receive it.

### Carry and class changes

- Gain several levels and verify fractional points carry between levels.
- Test a 100% total growth and an over-100% total growth if a controlled setup
  is available.
- Change class, gain another level, and verify existing counters continue with
  the new class growths.
- Confirm a capped stat does not increase and its counter resumes correctly if
  a legal cap increase becomes available.
- Specifically test a Saint Statue maximum-stat bonus near the raw personal
  cap; the first release is blocked until the effective in-game cap behavior
  is confirmed or implemented.

### Save lifecycle

- Save, close the game completely, reload, and verify the next level continues
  the same counters.
- Copy the save to another slot and verify the copied slot keeps its own state.
- Delete a disposable save slot and confirm no external plugin file or state
  remains for it.
- Start a new game and an NG+ file, then verify their first tracked level does
  not reuse counters from the previous run.

### Repeated calculations and Divine Pulse

- Trigger a level-up, use Divine Pulse to rewind it, then repeat the same level
  and verify the exact same stats return.
- If possible, rewind across more than one level-up and verify each repeated
  target. Record any mismatch before continuing the release process.
- Confirm the level-up presentation reports the same gains that remain on the
  unit afterward.

### Removal and conflicts

- Remove the plugin, restart the game, and confirm future levels use vanilla
  random growths without a crash.
- Confirm previously earned stats remain; uninstalling is not a stat rollback.
- Do not enable My 3H Plugin's growth feature at the same time. A conflict test
  is not required because both plugins replace the same function.

## Evidence to retain

For each test, retain the CI run URL, artifact commit, FE3H version, Skyline
version, before/after stat screenshots, and whether the save was new, existing,
or NG+. Hardware verification applies only to the exact commit and artifact
that passed.
