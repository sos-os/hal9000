use core::{marker::PhantomData, ops};
use hal9000::mem::{self, Address};

use ::x64::{PAddr, VAddr};

pub mod table;

pub trait PageSize: Copy + Eq + PartialOrd + Ord {
    const SIZE: usize;
}

pub type Physical<S = size::Small> = Page<PAddr, S>;

pub type Virtual<S = size::Small> = Page<VAddr, S>;

#[derive(Debug)]
pub struct Page<A, S> {
    start_addr: A,
    _sz: PhantomData<S>,
}

const PAGE_SHIFT: usize = 12;

impl<A, S> mem::Page for Page<A, S>
where
    A: Address + From<usize> + ops::Add<Output = A> + Copy,
    S: PageSize,
{
    /// Page alignment.
    const SHIFT: usize = PAGE_SHIFT;

    /// The size of a page in bytes.
    const SIZE: usize = S::SIZE;

    /// The type of address used to address this `Page`.
    ///
    /// If this is a physical page frame, then its `Address` should be the
    /// architecture's corresponding physical address type, and if this is a
    /// virtual page, then its `Address` should be the virtual address type.
    type Address = A;

    /// Round `addr` up to the closest `Page`.
    fn from_addr_up(addr: Self::Address) -> Self {
        Self {
            start_addr: addr.align_up(Self::SIZE),
            _sz: PhantomData,
        }
    }

    /// Round `addr` up to the closest `Page`.
    fn from_addr_down(addr: Self::Address) -> Self {
        Self {
            start_addr: addr.align_down(Self::SIZE),
            _sz: PhantomData,
        }
    }

    /// Returns the base `Address` where this page starts.
    fn base_address(&self) -> Self::Address {
        self.start_addr
    }

    /// Returns the end `Address` of this `Page`.
    fn end_address(&self) -> Self::Address {
        self.start_addr + Self::Address::from(Self::SIZE)
    }

    /// Return the page's number.
    fn number(&self) -> usize {
        unimplemented!()
    }
}

pub mod size {
    use super::PageSize;

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Small;
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Large;
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Huge;

    impl PageSize for Small {
        const SIZE: usize = 4096;
    }

    impl PageSize for Large {
        const SIZE: usize = Small::SIZE * 512;
    }

    impl PageSize for Huge {
        const SIZE: usize = Large::SIZE * 512;
    }
}
