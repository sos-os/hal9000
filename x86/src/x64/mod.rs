use hal9000::{
    mem::{Address, Page},
    Architecture,
};

pub mod page;

pub struct X86_64;

/// An `x86_64` physical memory address.
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Address)]
#[address_repr(u64)]
#[repr(transparent)]
pub struct PAddr(u64);

/// An `x86_64` virtual memory address.
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Address)]
#[address_repr(usize)]
#[repr(transparent)]
pub struct VAddr(usize);

impl Architecture for X86_64 {
    /// This architecture's physical address type.
    type PAddr = PAddr;
    type VAddr = VAddr;

    /// This architecture's physical page type.
    type Frame = ::paging::Physical;

    /// The name of the architecture (for logging, etc).
    const NAME: &'static str = "x86_64";

    const BITS: &'static str = "64";
}

impl From<u64> for PAddr {
    fn from(addr: u64) -> Self {
        const MASK: u64 = 0x000F_FFFF_FFFF_FFFFu64;
        PAddr(addr & MASK)
    }
}

// TODO: canonicalize addresses!
impl From<usize> for VAddr {
    fn from(addr: usize) -> Self {
        VAddr(addr)
    }
}

impl VAddr {
    pub fn as_usize(&self) -> usize {
        self.0
    }
}
