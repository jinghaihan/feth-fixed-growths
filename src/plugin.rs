use core::ptr::NonNull;

use crate::{
  game::{layout::Unit, profile, runtime::Runtime},
  level_up::{decide_level_up, GrowthSources, LevelUpDecision},
  persistence::PersistedGrowthState,
};

pub fn install() {
  if Runtime::detect().is_none() {
    println!("[feth-fixed-growths] unsupported title; hook not installed");
    return;
  }

  skyline::install_hooks!(unit_level_up_hook);
  println!("[feth-fixed-growths] fixed growths enabled");
}

#[skyline::hook(offset = profile::UNIT_LEVEL_UP_OFFSET)]
fn unit_level_up_hook(unit: *mut Unit, target_level: i32) {
  let Some(unit) = NonNull::new(unit) else {
    call_original!(unit, target_level);
    return;
  };

  let Ok(target_level_byte) = u8::try_from(target_level) else {
    call_original!(unit.as_ptr(), target_level);
    return;
  };
  if target_level_byte == 0 || target_level_byte > profile::MAX_LEVEL {
    call_original!(unit.as_ptr(), target_level);
    return;
  }

  if target_level_byte == 1 {
    call_original!(unit.as_ptr(), target_level);
    clear_growth_state(unit);
    return;
  }

  if unsafe { try_apply_fixed_growths(unit, target_level_byte) } {
    return;
  }

  call_original!(unit.as_ptr(), target_level);
}

fn clear_growth_state(unit: NonNull<Unit>) {
  let Some(runtime) = Runtime::detect() else {
    return;
  };
  let character = unsafe { unit.as_ref().character };
  let Some(mut save_unit) = runtime.save_unit(character) else {
    return;
  };

  unsafe {
    PersistedGrowthState::clear(&mut save_unit.as_mut().class_level);
  }
}

unsafe fn try_apply_fixed_growths(mut unit: NonNull<Unit>, target_level: u8) -> bool {
  let Some(runtime) = Runtime::detect() else {
    return false;
  };

  let (character, class, current_level, current_stats) = {
    let unit = unsafe { unit.as_ref() };
    (unit.character, unit.class, unit.level, unit.current_stats())
  };
  let Some(person) = runtime.person(character) else {
    return false;
  };
  let Some(class_data) = runtime.class(class) else {
    return false;
  };
  let Some(mut save_unit) = runtime.save_unit(character) else {
    return false;
  };

  let personal_growths = unsafe { person.as_ref().personal_growths() };
  let class_growths = unsafe { class_data.as_ref().class_growths() };
  let caps = unsafe { person.as_ref().stat_caps() };
  let persisted = unsafe { PersistedGrowthState::load(&save_unit.as_ref().class_level) }
    .filter(|state| state.is_plausible_for(caps));
  let ability_bonus = clamp_i32_to_i16(runtime.growth_bonus(unit));
  let sources = GrowthSources {
    personal: personal_growths,
    class: class_growths,
    ability_bonus,
    caps,
  };

  match decide_level_up(
    current_level,
    target_level,
    current_stats,
    sources,
    persisted,
  ) {
    LevelUpDecision::NoChange => false,
    LevelUpDecision::RestoreCached { stats } => {
      unsafe {
        unit.as_mut().apply_level_up(target_level, stats);
      }
      true
    }
    LevelUpDecision::Apply { result, state } => {
      if unsafe { state.store(&mut save_unit.as_mut().class_level) }.is_err() {
        return false;
      }
      unsafe {
        unit.as_mut().apply_level_up(target_level, result.stats);
      }
      true
    }
  }
}

fn clamp_i32_to_i16(value: i32) -> i16 {
  value.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn clamps_game_parameters_before_growth_arithmetic() {
    assert_eq!(clamp_i32_to_i16(i32::MIN), i16::MIN);
    assert_eq!(clamp_i32_to_i16(-20), -20);
    assert_eq!(clamp_i32_to_i16(20), 20);
    assert_eq!(clamp_i32_to_i16(i32::MAX), i16::MAX);
  }
}
