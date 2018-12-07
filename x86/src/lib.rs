#![no_std]
#![feature(custom_attribute)]
#![feature(step_trait)]
#![feature(asm)]
#![feature(associated_type_defaults)]

#[macro_use]
extern crate bitflags;
extern crate hal9000;
#[macro_use]
extern crate hal9000_derive;

#[cfg(test)]
#[macro_use]
extern crate proptest;
#[cfg(test)]
extern crate bit_field;
#[cfg(test)]
#[macro_use]
extern crate std;

pub mod paging;
pub mod x64;

#[cfg(target_arch = "x86_64")]
pub use self::x64::{PAddr, VAddr};
