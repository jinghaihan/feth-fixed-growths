pub const TITLE_ID: u64 = 0x0100_55D0_09F7_8000;
pub const DISPLAY_VERSION_1_2_0: &[u8] = b"1.2.0";

pub const BUILD_ID_1_2_0: [u8; 0x20] = [
  0x89, 0x04, 0x84, 0x49, 0xBA, 0x23, 0x8C, 0x8C, 0xF5, 0x65, 0x51, 0x8B, 0x83, 0xBF, 0x02, 0xD3,
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

pub const UNIT_LEVEL_UP_OFFSET: usize = 0x003D_3020;
pub const UNIT_ABILITY_PARAMETER_OFFSET: usize = 0x000A_7E80;
pub const GET_UNIT_FROM_SAVE_OFFSET: usize = 0x003C_AF30;
pub const SAVE_DATA_OFFSET: usize = 0x01B1_2190;
pub const PERSON_TABLE_OFFSET: usize = 0x01B3_87E8;
pub const CLASS_TABLE_OFFSET: usize = 0x01B3_8798;

pub const PERSON_COUNT: usize = 1201;
pub const CLASS_COUNT: usize = 100;
pub const GROWTH_BONUS_ABILITY_PARAMETER: i32 = 0x3A;
pub const MAX_LEVEL: u8 = 99;

// Original instructions in both player and enemy growth paths for FE3H 1.2.0
// Build ID 89048449BA238C8CF565518B83BF02D3. Besides identifying the profile,
// these reject the common "all stats +1" patches that modify the same code.
pub const TEXT_SIGNATURES_1_2_0: [(usize, u32); 6] = [
  (0x003D_3774, 0xF100_271F),
  (0x003D_3830, 0x1A93_A668),
  (0x003D_3A30, 0x1A93_A668),
  (0x003D_43BC, 0xF100_251F),
  (0x003D_44B8, 0x1A89_D539),
  (0x003D_4644, 0x1A89_D539),
];

pub fn is_supported_display_version(display_version: &[u8; 16]) -> bool {
  let length = display_version
    .iter()
    .position(|byte| *byte == 0)
    .unwrap_or(display_version.len());
  &display_version[..length] == DISPLAY_VERSION_1_2_0
}

pub fn matches_text_signatures(instructions: impl Fn(usize) -> u32) -> bool {
  TEXT_SIGNATURES_1_2_0
    .iter()
    .all(|(offset, expected)| instructions(*offset) == *expected)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn accepts_only_the_1_2_0_display_version() {
    let mut supported = [0; 16];
    supported[..DISPLAY_VERSION_1_2_0.len()].copy_from_slice(DISPLAY_VERSION_1_2_0);

    assert!(is_supported_display_version(&supported));
    assert!(!is_supported_display_version(
      b"1.1.1\0\0\0\0\0\0\0\0\0\0\0"
    ));
    assert!(!is_supported_display_version(b"1.2.0-extra\0\0\0\0\0"));
  }

  #[test]
  fn accepts_only_the_1_2_0_text_signatures() {
    assert!(matches_text_signatures(|offset| {
      TEXT_SIGNATURES_1_2_0
        .iter()
        .find_map(|(candidate, instruction)| (*candidate == offset).then_some(*instruction))
        .unwrap()
    }));
    assert!(!matches_text_signatures(|offset| {
      if offset == TEXT_SIGNATURES_1_2_0[3].0 {
        0xF100_1D1F
      } else {
        TEXT_SIGNATURES_1_2_0
          .iter()
          .find_map(|(candidate, instruction)| (*candidate == offset).then_some(*instruction))
          .unwrap()
      }
    }));
  }
}
