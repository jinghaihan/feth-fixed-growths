pub const TABLE_DATA_POINTER_OFFSET: usize = 0x10;
pub const TABLE_ENTRY_SIZE: usize = 0x18;

pub fn table_data_pointer_offset(index: usize, entry_count: usize) -> Option<usize> {
  if index >= entry_count {
    return None;
  }

  index
    .checked_mul(TABLE_ENTRY_SIZE)?
    .checked_add(TABLE_DATA_POINTER_OFFSET)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn calculates_fixed_table_entry_pointer_offsets() {
    assert_eq!(table_data_pointer_offset(0, 100), Some(0x10));
    assert_eq!(table_data_pointer_offset(1, 100), Some(0x28));
    assert_eq!(table_data_pointer_offset(99, 100), Some(0x958));
  }

  #[test]
  fn rejects_out_of_range_table_indexes() {
    assert_eq!(table_data_pointer_offset(100, 100), None);
    assert_eq!(table_data_pointer_offset(usize::MAX, usize::MAX), None);
  }
}
