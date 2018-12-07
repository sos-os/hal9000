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
#[address_repr(u64)]
#[repr(transparent)]
pub struct VAddr(u64);

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
    #[inline]
    fn from(addr: u64) -> Self {
        Self::new(addr)
    }
}

impl From<u64> for VAddr {
    #[inline]
    fn from(addr: u64) -> Self {
        Self::new(addr)
    }
}


impl VAddr {
    /// Constructs a new canonical virtual address.
    pub fn new(value: u64) -> Self {
        // Bits 64-48 of a canonical virtual address must match the value of
        // bit 47.
        const SIGN_EXTEND_BITS: u64 = 0xFFFF_0000_0000_0000;
        let canonical = if (value >> 47) & 1 == 1 {
            // If bit 47 is 1, set all of SIGN_EXTEND_BITS to 1.
            value | SIGN_EXTEND_BITS
        } else {
            // If bit 47 is 0, set all of SIGN_EXTEND_BITS to 0.
            value & !SIGN_EXTEND_BITS
        };
        VAddr(canonical)
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl PAddr {
    /// Constructs a new canonical physical address.
    #[inline]
    pub fn new(value: u64) -> Self {
        // The twelve highest bits must be set to 0.
        const MASK: u64 = 0x000F_FFFF_FFFF_FFFFu64;
        PAddr(value & MASK)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bit_field::BitField;

    proptest! {
        #[test]
        fn vaddrs_are_canonical(value: u64) {
            let vaddr = VAddr::new(value);
            if value.get_bit(47) {
                prop_assert_eq!(vaddr.as_u64().get_bits(48..64), 0xffff)
            } else {
                prop_assert_eq!(vaddr.as_u64().get_bits(48..64), 0x0000)
            }
        }

        #[test]
        fn paddrs_are_canonical(value: u64) {
            let paddr = PAddr::new(value);
            prop_assert_eq!(paddr.as_u64().get_bits(52..64), 0)
        }
    }
}
