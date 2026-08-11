#![deny(unsafe_op_in_unsafe_fn)]

pub mod game;
pub mod growth;

#[cfg(target_os = "horizon")]
#[skyline::main(name = "feth_fixed_growths")]
pub fn skyline_main() {
  println!("[feth-fixed-growths] initialized");
}
