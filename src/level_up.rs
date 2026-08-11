use crate::{
  growth::{
    advance_to_level, initial_points_from_personal_growth, LevelUpResult, MOVEMENT_STAT_INDEX,
    STAT_COUNT,
  },
  persistence::PersistedGrowthState,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GrowthSources {
  pub personal: [i16; STAT_COUNT],
  pub class: [i16; STAT_COUNT],
  pub ability_bonus: i16,
  pub caps: [u8; STAT_COUNT],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LevelUpDecision {
  NoChange,
  RestoreCached {
    stats: [u8; STAT_COUNT],
  },
  Apply {
    result: LevelUpResult,
    state: PersistedGrowthState,
  },
}

impl GrowthSources {
  pub fn total_growths(self) -> [i16; STAT_COUNT] {
    let mut total = [0; STAT_COUNT];
    for (stat, value) in total.iter_mut().enumerate() {
      let ability_bonus = if stat == MOVEMENT_STAT_INDEX {
        0
      } else {
        self.ability_bonus
      };
      *value = self.personal[stat]
        .saturating_add(self.class[stat])
        .saturating_add(ability_bonus);
    }
    total
  }
}

pub fn decide_level_up(
  current_level: u8,
  target_level: u8,
  current_stats: [u8; STAT_COUNT],
  sources: GrowthSources,
  persisted: Option<PersistedGrowthState>,
) -> LevelUpDecision {
  // A level-1 unit may still contain bytes copied from an older run. Recomputing
  // level 2 from the personal-growth seed is deterministic and replaces them.
  if current_level > 1 {
    if let Some(state) = persisted {
      if let Some(stats) = state.cached_stats_for(target_level) {
        return LevelUpDecision::RestoreCached { stats };
      }
    }
  }

  if target_level <= current_level {
    return LevelUpDecision::NoChange;
  }

  let mut accumulated_points = persisted
    .filter(|state| state.last_target_level == current_level)
    .map(|state| state.accumulated_points)
    .unwrap_or_else(|| sources.personal.map(initial_points_from_personal_growth));
  let result = advance_to_level(
    current_level,
    target_level,
    &mut accumulated_points,
    sources.total_growths(),
    current_stats,
    sources.caps,
  );
  let state = PersistedGrowthState {
    last_target_level: target_level,
    accumulated_points,
    cached_stats: result.stats,
  };

  LevelUpDecision::Apply { result, state }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn sources(personal: i16, class: i16, ability_bonus: i16) -> GrowthSources {
    GrowthSources {
      personal: [personal; STAT_COUNT],
      class: [class; STAT_COUNT],
      ability_bonus,
      caps: [99; STAT_COUNT],
    }
  }

  #[test]
  fn initializes_with_personal_growth_before_the_first_level() {
    let decision = decide_level_up(1, 2, [10; STAT_COUNT], sources(35, 10, 0), None);

    let LevelUpDecision::Apply { result, state } = decision else {
      panic!("expected an applied level up");
    };
    assert_eq!(result.gains, [0; STAT_COUNT]);
    assert_eq!(state.accumulated_points, [80; STAT_COUNT]);
  }

  #[test]
  fn includes_the_ability_growth_bonus_except_for_movement() {
    let decision = decide_level_up(1, 2, [10; STAT_COUNT], sources(35, 10, 20), None);

    let LevelUpDecision::Apply { result, state } = decision else {
      panic!("expected an applied level up");
    };
    let mut expected_gains = [1; STAT_COUNT];
    expected_gains[MOVEMENT_STAT_INDEX] = 0;
    let mut expected_points = [0; STAT_COUNT];
    expected_points[MOVEMENT_STAT_INDEX] = 80;

    assert_eq!(result.gains, expected_gains);
    assert_eq!(state.accumulated_points, expected_points);
  }

  #[test]
  fn continues_existing_accumulators_after_a_class_change() {
    let persisted = PersistedGrowthState {
      last_target_level: 5,
      accumulated_points: [90; STAT_COUNT],
      cached_stats: [20; STAT_COUNT],
    };

    let decision = decide_level_up(5, 6, [20; STAT_COUNT], sources(20, 40, 0), Some(persisted));

    let LevelUpDecision::Apply { result, state } = decision else {
      panic!("expected an applied level up");
    };
    assert_eq!(result.gains, [1; STAT_COUNT]);
    assert_eq!(state.accumulated_points, [50; STAT_COUNT]);
  }

  #[test]
  fn restores_cached_results_for_duplicate_target_calls() {
    let persisted = PersistedGrowthState {
      last_target_level: 8,
      accumulated_points: [25; STAT_COUNT],
      cached_stats: [30; STAT_COUNT],
    };

    assert_eq!(
      decide_level_up(7, 8, [20; STAT_COUNT], sources(50, 10, 0), Some(persisted),),
      LevelUpDecision::RestoreCached {
        stats: [30; STAT_COUNT],
      },
    );
  }

  #[test]
  fn ignores_same_or_lower_target_levels_without_a_cache_match() {
    assert_eq!(
      decide_level_up(10, 9, [20; STAT_COUNT], sources(50, 10, 0), None,),
      LevelUpDecision::NoChange,
    );
  }

  #[test]
  fn reinitializes_points_when_saved_state_is_from_another_run() {
    let stale = PersistedGrowthState {
      last_target_level: 40,
      accumulated_points: [99; STAT_COUNT],
      cached_stats: [50; STAT_COUNT],
    };

    let decision = decide_level_up(1, 2, [10; STAT_COUNT], sources(35, 10, 0), Some(stale));

    let LevelUpDecision::Apply { result, state } = decision else {
      panic!("expected an applied level up");
    };
    assert_eq!(result.gains, [0; STAT_COUNT]);
    assert_eq!(state.accumulated_points, [80; STAT_COUNT]);
  }

  #[test]
  fn does_not_restore_a_level_two_cache_for_a_fresh_unit() {
    let stale = PersistedGrowthState {
      last_target_level: 2,
      accumulated_points: [75; STAT_COUNT],
      cached_stats: [40; STAT_COUNT],
    };

    let decision = decide_level_up(1, 2, [10; STAT_COUNT], sources(35, 10, 0), Some(stale));

    let LevelUpDecision::Apply { result, state } = decision else {
      panic!("expected a freshly calculated level up");
    };
    assert_eq!(result.stats, [10; STAT_COUNT]);
    assert_eq!(state.accumulated_points, [80; STAT_COUNT]);
  }
}
