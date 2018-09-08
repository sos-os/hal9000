use hal9000::mem::{self, page::TableUpdate, Address, VAddr};

use core::{marker::PhantomData, ops};

pub mod table;

pub trait PageSize: Copy + Eq + PartialOrd + Ord {
    const SIZE: usize;
}

pub type Virtual<S = Size4Kb> = Page<VAddr, S>;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size4Kb {}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size2Mb {}

pub const KILOBYTE: usize = 1024;
pub const MEGABYTE: usize = KILOBYTE * 1024;

#[derive(Debug)]
pub struct Page<A, S: PageSize> {
    start_addr: A,
    _sz: PhantomData<S>,
}

#[must_use = "the TLB must be flushed to commit page table updates"]
pub struct FlushTlb<S: PageSize> {
    page: Virtual<S>,
}

impl<S: PageSize> FlushTlb<S> {
    pub fn new(page: Virtual<S>) -> Self {
        Self { page }
    }
}

impl<S: PageSize> TableUpdate for FlushTlb<S> {
    type Item = ();
    unsafe fn commit(self) -> Self::Item {
        asm!( "invlpg [$0]"
             :
             : "r" (self.page)
             : "memory"
             : "intel", "volatile" );
        // TODO: consider returning the page?
    }
}

// ===== impl Page =====

impl<A, S> mem::Page for Page<A, S>
where
    A: Address + From<usize> + ops::Add<Output = A> + Copy,
    S: PageSize,
{
    /// Page alignment.
    const SHIFT: usize = 12;

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

impl PageSize for Size4Kb {
    const SIZE: usize = KILOBYTE * 4;
}

impl PageSize for Size2Mb {
    const SIZE: usize = MEGABYTE * 2;
}
