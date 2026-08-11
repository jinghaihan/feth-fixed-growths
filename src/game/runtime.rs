use core::{ffi::c_void, mem, ptr::NonNull};

use super::{
  layout::{ClassData, PersonData, Unit},
  profile,
  table::table_data_pointer_offset,
};

type GetUnitFromSave = unsafe extern "C" fn(*mut c_void, u32) -> *mut Unit;
type GetUnitAbilityParameter = unsafe extern "C" fn(*mut Unit, i32) -> i32;

#[derive(Clone, Copy)]
pub struct Runtime {
  text_base: NonNull<u8>,
}

impl Runtime {
  pub fn detect() -> Option<Self> {
    if skyline::info::get_program_id() != profile::TITLE_ID {
      return None;
    }

    let text_base = NonNull::new(unsafe {
      skyline::hooks::getRegionAddress(skyline::hooks::Region::Text).cast::<u8>()
    })?;

    Some(Self { text_base })
  }

  pub fn person(self, character: i16) -> Option<NonNull<PersonData>> {
    let index = usize::try_from(character).ok()?;
    unsafe { self.fixed_table_entry(profile::PERSON_TABLE_OFFSET, index, profile::PERSON_COUNT) }
  }

  pub fn class(self, class: u8) -> Option<NonNull<ClassData>> {
    unsafe {
      self.fixed_table_entry(
        profile::CLASS_TABLE_OFFSET,
        usize::from(class),
        profile::CLASS_COUNT,
      )
    }
  }

  pub fn save_unit(self, character: i16) -> Option<NonNull<Unit>> {
    let character = u32::try_from(character).ok()?;
    let save_pointer_address = self
      .address(profile::SAVE_DATA_OFFSET)
      .cast::<*mut c_void>();
    let save = unsafe { save_pointer_address.read() };
    if save.is_null() {
      return None;
    }

    let function_address = self
      .address(profile::GET_UNIT_FROM_SAVE_OFFSET)
      .cast::<()>();
    let get_unit: GetUnitFromSave = unsafe { mem::transmute(function_address) };
    NonNull::new(unsafe { get_unit(save, character) })
  }

  pub fn growth_bonus(self, unit: NonNull<Unit>) -> i32 {
    let function_address = self
      .address(profile::UNIT_ABILITY_PARAMETER_OFFSET)
      .cast::<()>();
    let get_parameter: GetUnitAbilityParameter = unsafe { mem::transmute(function_address) };
    unsafe { get_parameter(unit.as_ptr(), profile::GROWTH_BONUS_ABILITY_PARAMETER) }
  }

  fn address(self, offset: usize) -> *mut u8 {
    unsafe { self.text_base.as_ptr().add(offset) }
  }

  unsafe fn fixed_table_entry<T>(
    self,
    table_offset: usize,
    index: usize,
    entry_count: usize,
  ) -> Option<NonNull<T>> {
    let data_pointer_offset = table_data_pointer_offset(index, entry_count)?;
    let table_pointer_address = self.address(table_offset).cast::<*mut u8>();
    let table = unsafe { table_pointer_address.read() };
    if table.is_null() {
      return None;
    }

    let data_pointer_address = unsafe { table.add(data_pointer_offset) }.cast::<*mut T>();
    NonNull::new(unsafe { data_pointer_address.read() })
  }
}
