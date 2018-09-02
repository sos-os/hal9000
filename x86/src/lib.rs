#![no_std]
#![feature(custom_attribute)]
#![feature(step_trait)]
#![feature(asm)]

extern crate hal9000;
#[macro_use]
extern crate hal9000_derive;

pub mod x64;
pub mod paging;
