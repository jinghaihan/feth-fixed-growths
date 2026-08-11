pub const STAT_COUNT: usize = 10;
pub const POINTS_PER_GAIN: u32 = 100;
pub const MOVEMENT_STAT_INDEX: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GrowthStep {
  pub gain: u8,
  pub remaining_points: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LevelUpResult {
  pub levels_gained: u8,
  pub gains: [u8; STAT_COUNT],
  pub stats: [u8; STAT_COUNT],
}

pub fn initial_points_from_personal_growth(personal_growth: i16) -> u16 {
  personal_growth.max(0) as u16
}

pub fn advance_stat(
  accumulated_points: u16,
  growth_rate: i16,
  current_stat: u8,
  stat_cap: u8,
) -> GrowthStep {
  if current_stat >= stat_cap {
    return GrowthStep {
      gain: 0,
      remaining_points: accumulated_points,
    };
  }

  let effective_growth = growth_rate.max(0) as u32;
  let total_points = u32::from(accumulated_points) + effective_growth;
  let available_gains = stat_cap.saturating_sub(current_stat);
  let earned_gains = total_points / POINTS_PER_GAIN;
  let gain = earned_gains.min(u32::from(available_gains)) as u8;

  GrowthStep {
    gain,
    remaining_points: (total_points % POINTS_PER_GAIN) as u16,
  }
}

pub fn advance_level(
  accumulated_points: &mut [u16; STAT_COUNT],
  growth_rates: [i16; STAT_COUNT],
  current_stats: [u8; STAT_COUNT],
  stat_caps: [u8; STAT_COUNT],
) -> [u8; STAT_COUNT] {
  let mut gains = [0; STAT_COUNT];

  for stat in 0..STAT_COUNT {
    let result = advance_stat(
      accumulated_points[stat],
      growth_rates[stat],
      current_stats[stat],
      stat_caps[stat],
    );
    accumulated_points[stat] = result.remaining_points;
    gains[stat] = result.gain;
  }

  gains
}

pub fn advance_to_level(
  current_level: u8,
  target_level: u8,
  accumulated_points: &mut [u16; STAT_COUNT],
  growth_rates: [i16; STAT_COUNT],
  current_stats: [u8; STAT_COUNT],
  stat_caps: [u8; STAT_COUNT],
) -> LevelUpResult {
  let levels_gained = target_level.saturating_sub(current_level);
  let mut stats = current_stats;
  let mut total_gains = [0_u8; STAT_COUNT];

  for _ in 0..levels_gained {
    let gains = advance_level(accumulated_points, growth_rates, stats, stat_caps);
    for stat in 0..STAT_COUNT {
      stats[stat] = stats[stat].saturating_add(gains[stat]);
      total_gains[stat] = total_gains[stat].saturating_add(gains[stat]);
    }
  }

  LevelUpResult {
    levels_gained,
    gains: total_gains,
    stats,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn initializes_from_non_negative_personal_growth() {
    assert_eq!(initial_points_from_personal_growth(45), 45);
    assert_eq!(initial_points_from_personal_growth(0), 0);
    assert_eq!(initial_points_from_personal_growth(-10), 0);
  }

  #[test]
  fn carries_fractional_growth_between_levels() {
    let first = advance_stat(45, 45, 10, 99);
    let second = advance_stat(first.remaining_points, 45, 10, 99);

    assert_eq!(
      first,
      GrowthStep {
        gain: 0,
        remaining_points: 90
      }
    );
    assert_eq!(
      second,
      GrowthStep {
        gain: 1,
        remaining_points: 35
      }
    );
  }

  #[test]
  fn grants_guaranteed_and_multiple_gains() {
    assert_eq!(
      advance_stat(0, 100, 10, 99),
      GrowthStep {
        gain: 1,
        remaining_points: 0
      },
    );
    assert_eq!(
      advance_stat(90, 135, 10, 99),
      GrowthStep {
        gain: 2,
        remaining_points: 25
      },
    );
  }

  #[test]
  fn clamps_negative_growth_to_zero() {
    assert_eq!(
      advance_stat(40, -20, 10, 99),
      GrowthStep {
        gain: 0,
        remaining_points: 40
      },
    );
  }

  #[test]
  fn respects_the_stat_cap() {
    assert_eq!(
      advance_stat(90, 250, 19, 20),
      GrowthStep {
        gain: 1,
        remaining_points: 40
      },
    );
    assert_eq!(
      advance_stat(90, 50, 20, 20),
      GrowthStep {
        gain: 0,
        remaining_points: 90
      },
    );
  }

  #[test]
  fn resumes_frozen_points_if_a_later_class_raises_the_cap() {
    let capped = advance_stat(90, 50, 20, 20);
    let raised_cap = advance_stat(capped.remaining_points, 50, 20, 21);

    assert_eq!(
      raised_cap,
      GrowthStep {
        gain: 1,
        remaining_points: 40,
      },
    );
  }

  #[test]
  fn advances_all_stats_in_one_level() {
    let mut points = [0; STAT_COUNT];
    let growth_rates = [100, 0, 200, 99, -10, 50, 100, 25, 100, 1];
    let current_stats = [10; STAT_COUNT];
    let stat_caps = [99; STAT_COUNT];

    let gains = advance_level(&mut points, growth_rates, current_stats, stat_caps);

    assert_eq!(gains, [1, 0, 2, 0, 0, 0, 1, 0, 1, 0]);
    assert_eq!(points, [0, 0, 0, 99, 0, 50, 0, 25, 0, 1]);
  }

  #[test]
  fn advances_once_for_each_new_level() {
    let mut points = [0; STAT_COUNT];
    points[0] = initial_points_from_personal_growth(35);
    let mut growth_rates = [0; STAT_COUNT];
    growth_rates[0] = 45;

    let result = advance_to_level(
      1,
      3,
      &mut points,
      growth_rates,
      [10; STAT_COUNT],
      [99; STAT_COUNT],
    );

    assert_eq!(result.levels_gained, 2);
    assert_eq!(result.gains[0], 1);
    assert_eq!(result.stats[0], 11);
    assert_eq!(points[0], 25);
  }

  #[test]
  fn does_not_reapply_an_already_reached_level() {
    let mut points = [50; STAT_COUNT];
    let result = advance_to_level(
      10,
      10,
      &mut points,
      [100; STAT_COUNT],
      [20; STAT_COUNT],
      [99; STAT_COUNT],
    );

    assert_eq!(result.levels_gained, 0);
    assert_eq!(result.gains, [0; STAT_COUNT]);
    assert_eq!(result.stats, [20; STAT_COUNT]);
    assert_eq!(points, [50; STAT_COUNT]);
  }

  #[test]
  fn checks_caps_after_each_level() {
    let mut points = [90; STAT_COUNT];
    let result = advance_to_level(
      1,
      4,
      &mut points,
      [250; STAT_COUNT],
      [19; STAT_COUNT],
      [20; STAT_COUNT],
    );

    assert_eq!(result.gains, [1; STAT_COUNT]);
    assert_eq!(result.stats, [20; STAT_COUNT]);
    assert_eq!(points, [40; STAT_COUNT]);
  }
}
