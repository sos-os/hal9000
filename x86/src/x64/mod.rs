use hal9000::{
    mem::{Address, Page},
    Architecture,
};

pub mod page;
pub use self::page::Physical as PhysicalPage;
pub use hal9000::mem::VAddr;

pub struct X86_64;

/// An `x86_64` physical memory address.
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Address)]
#[address_repr(u64)]
pub struct PAddr(pub u64);

impl Architecture for X86_64 {
    /// This architecture's physical address type.
    type PAddr = PAddr;

    /// This architecture's physical page type.
    type Frame = PhysicalPage;

    /// The name of the architecture (for logging, etc).
    const NAME: &'static str = "x86_64";

    const BITS: &'static str = "64";
}
