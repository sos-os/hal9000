use {mem::Address, Architecture};

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

/// On some architectures, such as x86, page table changes must be flushed
/// from one or more caches or buffers before they are valid. This type
/// represents that operation.Address
///
/// On architectures where a commit operation is not necessary, this trait
/// is implemented by `()`; commiting a `()`-typed update will do nothing.
pub trait TableUpdate {
    /// Optionally, information about the committed change may be returned to
    /// the kernel.
    type Item;

    /// Commit the page table update, returning an `Item`.
    unsafe fn commit(self) -> Self::Item;
}

#[derive(Debug)]
pub enum MapError<A: FrameAllocator, B> {
    AlreadyMapped,
    Alloc(A::Error),
    Other(B),
}

pub trait Map<P, F>
where
    P: Page<Address = Self::VAddr>,
    F: Page<Address = Self::PAddr>,
    Self::PAddr: Address,
{
    type Arch: Architecture;
    type PAddr: Address = <Self::Arch as Architecture>::PAddr;
    type VAddr: Address = <Self::Arch as Architecture>::VAddr;

    /// Architecture-dependent flags that configure a virtual page.
    type Flags;
    /// The type returned by a page table update.
    ///
    /// This must be committed for the update to have an effect.
    type Update: TableUpdate;
    /// Any errors that can occur when mapping a page.
    type Error;

    /// Translates a virtual address to the corresponding physical address.
    ///
    /// # Returns
    /// + `Some(PAddr)` containing the physical address corresponding to
    ///                 `vaddr`, if it is mapped.
    /// + `None`: if the address is not mapped.
    fn translate(&self, vaddr: Self::VAddr) -> Option<Self::PAddr>;

    /// Translates a virtual page to a physical frame.
    fn translate_page(&self, page: P) -> Option<F>;

    /// Modifies the page tables so that `page` maps to `frame`.
    ///
    /// # Arguments
    /// + `page`: the virtual `Page` to map
    /// + `frame`: the physical `Frame` that `Page` should map to.
    /// + `flags`: the page table entry flags.
    /// + `alloc`: a memory allocator
    fn map<A>(
        &mut self,
        page: P,
        frame: F,
        flags: Self::Flags,
        alloc: &mut A,
    ) -> Result<Self::Update, MapError<A, Self::Error>>
    where
        A: FrameAllocator<Frame = F>;

    /// Identity maps a given `frame`.
    ///
    /// # Arguments
    /// + `frame`: the physical `Frame` to identity map
    /// + `flags`: the page table entry flags.
    /// + `alloc`: a memory allocator
    fn identity_map<A>(
        &mut self,
        frame: F,
        flags: Self::Flags,
        alloc: &mut A,
    ) -> Result<Self::Update, MapError<A, Self::Error>>
    where
        A: FrameAllocator<Frame = F>;

    /// Maps the given `VirtualPage` to any free frame.
    ///
    /// This is like the fire and forget version of `map_to`: we just pick the
    /// first available free frame and map the page to it.
    ///
    /// # Arguments
    /// + `page`: the`VirtualPage` to map
    /// + `flags`: the page table entry flags.
    /// + `alloc`: a memory allocator
    fn map_to_any<A>(
        &mut self,
        page: P,
        flags: Self::Flags,
        alloc: &mut A,
    ) -> Result<Self::Update, MapError<A, Self::Error>>
    where
        A: FrameAllocator<Frame = F>;

    /// Unmaps the given `VirtualPage`.
    ///
    /// All freed frames are returned to the given `FrameAllocator`.
    fn unmap<A>(
        &mut self,
        page: P,
        alloc: &mut A,
    ) -> Result<Self::Update, Self::Error>
    where
        A: FrameAllocator<Frame = F>;

    /// Updates the flags on the given `page`.
    fn set_flags(
        &mut self,
        page: P,
        flags: Self::Flags,
    ) -> Result<Self::Update, Self::Error>;
}

/// Represents a contiguous range of pages.
#[derive(Copy, Clone, Debug)]
pub struct Range<P> {
    start: P,
    end: P,
}

impl TableUpdate for () {
    type Item = ();
    unsafe fn commit(self) -> Self::Item {
        // do nothing.
    }
}

impl<A: FrameAllocator, B> From<B> for MapError<A, B> {
    fn from(error: B) -> Self {
        MapError::Other(error)
    }
}
