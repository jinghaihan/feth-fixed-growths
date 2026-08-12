use core::{
  ffi::c_void,
  mem,
  ptr::NonNull,
  sync::atomic::{AtomicUsize, Ordering},
};

use super::{
  layout::{ClassData, PersonData, Unit},
  profile,
  table::table_data_pointer_offset,
};

type GetUnitFromSave = unsafe extern "C" fn(*mut c_void, u32) -> *mut Unit;
type GetUnitAbilityParameter = unsafe extern "C" fn(*mut Unit, i32) -> i32;

static INITIALIZED_TEXT_BASE: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy)]
pub struct Runtime {
  text_base: NonNull<u8>,
}

impl Runtime {
  pub fn initialize() -> bool {
    let Some(runtime) = Self::detect() else {
      return false;
    };
    INITIALIZED_TEXT_BASE.store(runtime.text_base.as_ptr() as usize, Ordering::Release);
    true
  }

  pub fn get() -> Option<Self> {
    let text_base = INITIALIZED_TEXT_BASE.load(Ordering::Acquire) as *mut u8;
    NonNull::new(text_base).map(|text_base| Self { text_base })
  }

  fn detect() -> Option<Self> {
    if skyline::info::get_program_id() != profile::TITLE_ID {
      return None;
    }
    let mut display_version = skyline::nn::oe::DisplayVersion { name: [0; 16] };
    // SAFETY: Nintendo's API writes exactly one initialized DisplayVersion to
    // the valid out-pointer supplied here.
    unsafe {
      skyline::nn::oe::GetDisplayVersion(&mut display_version);
    }
    if !profile::is_supported_display_version(&display_version.name) {
      return None;
    }

    // SAFETY: Skyline exposes the mapped main executable text-region base for
    // the lifetime of the process. Null is rejected before constructing Self.
    let text_base = NonNull::new(unsafe {
      skyline::hooks::getRegionAddress(skyline::hooks::Region::Text).cast::<u8>()
    })?;
    let runtime = Self { text_base };
    if !runtime.matches_supported_profile() {
      return None;
    }

    Some(runtime)
  }

  pub fn person(self, character: i16) -> Option<NonNull<PersonData>> {
    let index = usize::try_from(character).ok()?;
    // SAFETY: the version gate fixes the table layout, and fixed_table_entry
    // checks the index and both pointer loads before returning an entry.
    unsafe { self.fixed_table_entry(profile::PERSON_TABLE_OFFSET, index, profile::PERSON_COUNT) }
  }

  pub fn class(self, class: u8) -> Option<NonNull<ClassData>> {
    // SAFETY: the version gate fixes the table layout, and fixed_table_entry
    // checks the index and both pointer loads before returning an entry.
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
    // SAFETY: the supported profile identifies a readable global pointer slot.
    let save = unsafe { save_pointer_address.read() };
    if save.is_null() {
      return None;
    }

    let function_address = self
      .address(profile::GET_UNIT_FROM_SAVE_OFFSET)
      .cast::<()>();
    // SAFETY: the supported profile fixes this address and ABI; the save
    // pointer is non-null and the character index was converted to u32.
    let get_unit: GetUnitFromSave = unsafe { mem::transmute(function_address) };
    NonNull::new(unsafe { get_unit(save, character) })
  }

  pub fn growth_bonus(self, unit: NonNull<Unit>) -> i32 {
    let function_address = self
      .address(profile::UNIT_ABILITY_PARAMETER_OFFSET)
      .cast::<()>();
    // SAFETY: the supported profile fixes this address and ABI, and callers
    // provide a live Unit pointer received from the hooked game function.
    let get_parameter: GetUnitAbilityParameter = unsafe { mem::transmute(function_address) };
    unsafe { get_parameter(unit.as_ptr(), profile::GROWTH_BONUS_ABILITY_PARAMETER) }
  }

  fn address(self, offset: usize) -> *mut u8 {
    // SAFETY: every caller supplies a fixed offset from the validated 1.2.0
    // profile. Runtime never exposes this pointer without further validation.
    unsafe { self.text_base.as_ptr().add(offset) }
  }

  fn matches_supported_profile(self) -> bool {
    profile::matches_text_signatures(|offset| {
      // SAFETY: every signature offset is aligned and belongs to the mapped
      // text region of the expected FE3H profile. The title and display
      // version gates run before this read.
      unsafe { self.address(offset).cast::<u32>().read_volatile() }
    })
  }

  unsafe fn fixed_table_entry<T>(
    self,
    table_offset: usize,
    index: usize,
    entry_count: usize,
  ) -> Option<NonNull<T>> {
    let data_pointer_offset = table_data_pointer_offset(index, entry_count)?;
    let table_pointer_address = self.address(table_offset).cast::<*mut u8>();
    // SAFETY: the supported profile identifies a readable table pointer slot.
    let table = unsafe { table_pointer_address.read() };
    if table.is_null() {
      return None;
    }

    // SAFETY: table_data_pointer_offset bounded index to the fixed table count,
    // and the supported profile fixes the table's pointer-array layout.
    let data_pointer_address = unsafe { table.add(data_pointer_offset) }.cast::<*mut T>();
    NonNull::new(unsafe { data_pointer_address.read() })
  }
}
