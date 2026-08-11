use core::mem::size_of;

use crate::growth::STAT_COUNT;

pub const UNIT_CHARACTER_OFFSET: usize = 0x24;
pub const UNIT_LEVEL_OFFSET: usize = 0x4A;
pub const UNIT_CLASS_OFFSET: usize = 0x4B;
pub const UNIT_HP_OFFSET: usize = 0x4C;
pub const UNIT_STATS_OFFSET: usize = 0x4E;
pub const UNIT_CLASS_LEVEL_OFFSET: usize = 0x1E7;
pub const UNIT_SIZE: usize = 0x24C;

pub const PERSON_HP_CAP_OFFSET: usize = 0x22;
pub const PERSON_HP_GROWTH_OFFSET: usize = 0x29;
pub const PERSON_GROWTHS_OFFSET: usize = 0x3C;
pub const PERSON_CAPS_OFFSET: usize = 0x45;
pub const PERSON_SIZE: usize = 0x50;

pub const CLASS_HP_GROWTH_OFFSET: usize = 0x14;
pub const CLASS_GROWTHS_OFFSET: usize = 0x15;

#[repr(C)]
pub struct Unit {
  _before_character: [u8; UNIT_CHARACTER_OFFSET],
  pub character: i16,
  _before_level: [u8; UNIT_LEVEL_OFFSET - UNIT_CHARACTER_OFFSET - size_of::<i16>()],
  pub level: u8,
  pub class: u8,
  pub hp: u8,
  pub hp_modifiers: u8,
  pub stats: [u8; STAT_COUNT - 1],
  _before_class_level: [u8; UNIT_CLASS_LEVEL_OFFSET - UNIT_STATS_OFFSET - (STAT_COUNT - 1)],
  pub class_level: [u8; 100],
}

impl Unit {
  pub fn current_stats(&self) -> [u8; STAT_COUNT] {
    let mut result = [0; STAT_COUNT];
    result[0] = self.hp;
    result[1..].copy_from_slice(&self.stats);
    result
  }
}

#[repr(C, align(4))]
pub struct PersonData {
  _before_hp_cap: [u8; PERSON_HP_CAP_OFFSET],
  pub hp_cap: u8,
  _before_hp_growth: [u8; PERSON_HP_GROWTH_OFFSET - PERSON_HP_CAP_OFFSET - 1],
  pub hp_growth: u8,
  _before_growths: [u8; PERSON_GROWTHS_OFFSET - PERSON_HP_GROWTH_OFFSET - 1],
  pub growths: [u8; STAT_COUNT - 1],
  pub caps: [u8; STAT_COUNT - 1],
  _tail: [u8; PERSON_SIZE - PERSON_CAPS_OFFSET - (STAT_COUNT - 1)],
}

impl PersonData {
  pub fn personal_growths(&self) -> [i16; STAT_COUNT] {
    let mut result = [0; STAT_COUNT];
    result[0] = i16::from(self.hp_growth);
    for (target, source) in result[1..].iter_mut().zip(self.growths) {
      *target = i16::from(source);
    }
    result
  }

  pub fn stat_caps(&self) -> [u8; STAT_COUNT] {
    let mut result = [0; STAT_COUNT];
    result[0] = self.hp_cap;
    result[1..].copy_from_slice(&self.caps);
    result
  }
}

#[repr(C, align(2))]
pub struct ClassData {
  _before_hp_growth: [u8; CLASS_HP_GROWTH_OFFSET],
  pub hp_growth: i8,
  pub growths: [i8; STAT_COUNT - 1],
}

impl ClassData {
  pub fn class_growths(&self) -> [i16; STAT_COUNT] {
    let mut result = [0; STAT_COUNT];
    result[0] = i16::from(self.hp_growth);
    for (target, source) in result[1..].iter_mut().zip(self.growths) {
      *target = i16::from(source);
    }
    result
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::mem::{align_of, offset_of, size_of};

  #[test]
  fn unit_layout_matches_the_1_2_0_profile() {
    assert_eq!(offset_of!(Unit, character), UNIT_CHARACTER_OFFSET);
    assert_eq!(offset_of!(Unit, level), UNIT_LEVEL_OFFSET);
    assert_eq!(offset_of!(Unit, class), UNIT_CLASS_OFFSET);
    assert_eq!(offset_of!(Unit, hp), UNIT_HP_OFFSET);
    assert_eq!(offset_of!(Unit, stats), UNIT_STATS_OFFSET);
    assert_eq!(offset_of!(Unit, class_level), UNIT_CLASS_LEVEL_OFFSET);
    assert_eq!(size_of::<Unit>(), UNIT_SIZE);
  }

  #[test]
  fn person_layout_matches_the_1_2_0_profile() {
    assert_eq!(offset_of!(PersonData, hp_cap), PERSON_HP_CAP_OFFSET);
    assert_eq!(offset_of!(PersonData, hp_growth), PERSON_HP_GROWTH_OFFSET);
    assert_eq!(offset_of!(PersonData, growths), PERSON_GROWTHS_OFFSET);
    assert_eq!(offset_of!(PersonData, caps), PERSON_CAPS_OFFSET);
    assert_eq!(size_of::<PersonData>(), PERSON_SIZE);
    assert_eq!(align_of::<PersonData>(), 4);
  }

  #[test]
  fn class_layout_matches_the_1_2_0_profile() {
    assert_eq!(offset_of!(ClassData, hp_growth), CLASS_HP_GROWTH_OFFSET);
    assert_eq!(offset_of!(ClassData, growths), CLASS_GROWTHS_OFFSET);
    assert_eq!(align_of::<ClassData>(), 2);
  }
}
