use crate::growth::STAT_COUNT;

pub const LAST_TARGET_LEVEL_SLOT: usize = 60;
pub const ACCUMULATOR_START_SLOT: usize = 61;
pub const CACHED_STATS_START_SLOT: usize = 71;
pub const STORAGE_END_SLOT: usize = 81;

const MAX_ACCUMULATED_POINTS: u16 = 99;
const MIN_PERSISTED_LEVEL: u8 = 2;
const MAX_PERSISTED_LEVEL: u8 = 99;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PersistedGrowthState {
  pub last_target_level: u8,
  pub accumulated_points: [u16; STAT_COUNT],
  pub cached_stats: [u8; STAT_COUNT],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageError {
  AccumulatorOutOfRange { stat: usize, points: u16 },
}

impl PersistedGrowthState {
  pub fn load(class_level: &[u8; 100]) -> Option<Self> {
    let last_target_level = class_level[LAST_TARGET_LEVEL_SLOT];
    if !(MIN_PERSISTED_LEVEL..=MAX_PERSISTED_LEVEL).contains(&last_target_level) {
      return None;
    }

    let mut accumulated_points = [0; STAT_COUNT];
    for (stat, points) in accumulated_points.iter_mut().enumerate() {
      *points = u16::from(class_level[ACCUMULATOR_START_SLOT + stat]);
      if *points > MAX_ACCUMULATED_POINTS {
        return None;
      }
    }

    let mut cached_stats = [0; STAT_COUNT];
    cached_stats
      .copy_from_slice(&class_level[CACHED_STATS_START_SLOT..CACHED_STATS_START_SLOT + STAT_COUNT]);

    Some(Self {
      last_target_level,
      accumulated_points,
      cached_stats,
    })
  }

  pub fn store(self, class_level: &mut [u8; 100]) -> Result<(), StorageError> {
    for (stat, points) in self.accumulated_points.iter().copied().enumerate() {
      if points > MAX_ACCUMULATED_POINTS {
        return Err(StorageError::AccumulatorOutOfRange { stat, points });
      }
    }

    class_level[LAST_TARGET_LEVEL_SLOT] = self.last_target_level;
    for (stat, points) in self.accumulated_points.iter().copied().enumerate() {
      class_level[ACCUMULATOR_START_SLOT + stat] = points as u8;
    }
    class_level[CACHED_STATS_START_SLOT..CACHED_STATS_START_SLOT + STAT_COUNT]
      .copy_from_slice(&self.cached_stats);

    Ok(())
  }

  pub fn cached_stats_for(self, target_level: u8) -> Option<[u8; STAT_COUNT]> {
    (self.last_target_level == target_level).then_some(self.cached_stats)
  }

  pub fn is_plausible_for(self, stat_caps: [u8; STAT_COUNT]) -> bool {
    self.cached_stats[0] > 0
      && self
        .cached_stats
        .iter()
        .zip(stat_caps)
        .all(|(stat, cap)| *stat <= cap)
  }

  pub fn clear(class_level: &mut [u8; 100]) {
    class_level[LAST_TARGET_LEVEL_SLOT..STORAGE_END_SLOT].fill(0);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn state() -> PersistedGrowthState {
    PersistedGrowthState {
      last_target_level: 12,
      accumulated_points: [0, 10, 20, 30, 40, 50, 60, 70, 80, 90],
      cached_stats: [30, 20, 15, 18, 21, 11, 17, 13, 5, 16],
    }
  }

  #[test]
  fn uses_the_reference_plugins_contiguous_slot_range() {
    assert_eq!(LAST_TARGET_LEVEL_SLOT, 60);
    assert_eq!(ACCUMULATOR_START_SLOT, 61);
    assert_eq!(CACHED_STATS_START_SLOT, 71);
    assert_eq!(STORAGE_END_SLOT, 81);
  }

  #[test]
  fn round_trips_growth_state_without_touching_adjacent_slots() {
    let mut class_level = [0; 100];
    class_level[59] = 0x59;
    class_level[81] = 0x81;

    state().store(&mut class_level).unwrap();

    assert_eq!(PersistedGrowthState::load(&class_level), Some(state()));
    assert_eq!(class_level[59], 0x59);
    assert_eq!(class_level[81], 0x81);
  }

  #[test]
  fn treats_zero_last_target_level_as_uninitialized() {
    assert_eq!(PersistedGrowthState::load(&[0; 100]), None);
  }

  #[test]
  fn rejects_values_that_cannot_be_plugin_state() {
    let mut class_level = [0; 100];
    state().store(&mut class_level).unwrap();

    class_level[LAST_TARGET_LEVEL_SLOT] = 1;
    assert_eq!(PersistedGrowthState::load(&class_level), None);

    class_level[LAST_TARGET_LEVEL_SLOT] = 12;
    class_level[ACCUMULATOR_START_SLOT + 4] = 100;
    assert_eq!(PersistedGrowthState::load(&class_level), None);
  }

  #[test]
  fn returns_cached_stats_only_for_the_same_target_level() {
    assert_eq!(state().cached_stats_for(12), Some(state().cached_stats));
    assert_eq!(state().cached_stats_for(13), None);
  }

  #[test]
  fn validates_cached_stats_against_character_caps() {
    let valid = state();
    assert!(valid.is_plausible_for([99; STAT_COUNT]));

    let mut zero_hp = valid;
    zero_hp.cached_stats[0] = 0;
    assert!(!zero_hp.is_plausible_for([99; STAT_COUNT]));

    let mut over_cap = valid;
    over_cap.cached_stats[3] = 31;
    assert!(!over_cap.is_plausible_for([30; STAT_COUNT]));
  }

  #[test]
  fn rejects_non_normalized_accumulators_before_writing() {
    let mut invalid = state();
    invalid.accumulated_points[3] = 100;
    let mut class_level = [0xAA; 100];

    assert_eq!(
      invalid.store(&mut class_level),
      Err(StorageError::AccumulatorOutOfRange {
        stat: 3,
        points: 100,
      }),
    );
    assert_eq!(class_level, [0xAA; 100]);
  }

  #[test]
  fn clears_only_the_owned_slot_range() {
    let mut class_level = [0xFF; 100];

    PersistedGrowthState::clear(&mut class_level);

    assert!(class_level[LAST_TARGET_LEVEL_SLOT..STORAGE_END_SLOT]
      .iter()
      .all(|value| *value == 0));
    assert_eq!(class_level[59], 0xFF);
    assert_eq!(class_level[81], 0xFF);
  }
}
