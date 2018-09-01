use super::Address;

/// A physical or virtual page.
pub trait Page {
    /// Page alignment.
    const SHIFT: usize;

    /// The size of a page in bytes.
    const SIZE: usize;

    /// The type of address used to address this `Page`.
    ///
    /// If this is a physical page frame, then its `Address` should be the
    /// architecture's corresponding physical address type, and if this is a
    /// virtual page, then its `Address` should be the virtual address type.
    type Address: Address;

    /// Round `addr` up to the closest `Page`.
    fn from_addr_up(addr: Self::Address) -> Self;

    /// Round `addr` up to the closest `Page`.
    fn from_addr_down(addr: Self::Address) -> Self;

    /// Returns the base `Address` where this page starts.
    fn base_address(&self) -> Self::Address;

    /// Returns the end `Address` of this `Page`.
    fn end_address(&self) -> Self::Address;

    /// Return the page's number.
    fn number(&self) -> usize;
}

/// An allocator that provides page frames.
pub unsafe trait FrameAllocator {
    /// Architecture-dependent size of a physical page.
    const FRAME_SIZE: usize = <Self::Frame as Page>::SIZE;

    /// Type representing frames provided by this allocator.
    ///
    /// A `Frame` must either be a pointer to a contiguous block of `FRAME_SIZE`
    /// bytes, or be a handle that may be converted into such a pointer.
    type Frame: Page;

    type Error;

    /// Returns a new `Frame`.
    unsafe fn alloc(&mut self) -> Result<Self::Frame, Self::Error>;

    /// Deallocate a `Frame`.
    ///
    /// # Unsafety
    /// This function is unsafe because undefined behaviour may result if the
    /// given `frame` was not originally allocated by this `Allocator`.
    unsafe fn dealloc(&mut self, frame: Self::Frame)
        -> Result<(), Self::Error>;

    // TODO: alloc_range/dealloc_range; requires an architecture-independent
    //       way of representing frame ranges.
}

/// Represents a contiguous range of pages.
#[derive(Copy, Clone, Debug)]
pub struct Range<P> {
    start: P,
    end: P,
}
