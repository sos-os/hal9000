//
// SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website), and the SOS contributors.
//
//  Copyright (c) 2018 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! # HAL-9000: the base SOS Hardware Abstraction Layer
#![feature(step_trait)]
#![feature(associated_type_defaults)]
#![no_std]

#[macro_use]
extern crate hal9000_derive;

pub mod cpu;
pub mod mem;
pub mod params;
pub mod util;

pub trait Architecture {
    /// This architecture's physical address type.
    type PAddr: mem::Address;

    /// This architecture's physical page type.
    type Frame: mem::Page;

    /// The name of the architecture (for logging, etc).
    const NAME: &'static str;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    // #[test]
    // fn vaddr_name() {
    //     use super::mem::VAddr;
    //     panic!("{:?}", VAddr(1))
    // }
}
