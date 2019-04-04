use hal9000::mem::page;
use crate::{
    paging::FlushTlb,
    x64::{page::*, X86_64},
};

/// Struct representing the currently active PML4 instance.
///
/// The `ActivePML4` is a `Unique` reference to a PML4-level page table. It's
/// unique because, well, there can only be one active PML4 at a given time.
pub struct ActivePml4 {
    // TODO: implement
//plm4: Unique<Table<Pml4Level>>
}

pub struct PageFlags {
    // TODO: implement
}

pub enum Error {
    Alloc,
    // TODO: Implement
}

impl page::Mapper for ActivePml4 {
    type Arch = X86_64;

    type Virtual = Virtual;

    /// Architecture-dependent flags that configure a virtual page.
    type Flags = PageFlags;
    /// The type returned by a page table update.
    ///
    /// This must be committed for the update to have an effect.
    type Update = FlushTlb<size::Small>;
    /// Any errors that can occur when mapping a page.
    type Error = Error;

    /// Translates a virtual address to the corresponding physical address.
    ///
    /// # Returns
    /// + `Some(PAddr)` containing the physical address corresponding to
    ///                 `vaddr`, if it is mapped.
    /// + `None`: if the address is not mapped.
    fn translate(&self, _vaddr: VAddr) -> Option<Self::PAddr> {
        unimplemented!()
    }

    /// Translates a virtual page to a physical frame.
    fn translate_page(&self, _page: Self::Virtual) -> Option<Self::Physical> {
        unimplemented!()
    }

    /// Modifies the page tables so that `page` maps to `frame`.
    ///
    /// # Arguments
    /// + `page`: the virtual `Page` to map
    /// + `frame`: the physical `Frame` that `Page` should map to.
    /// + `flags`: the page table entry flags.
    /// + `alloc`: a memory allocator
    fn map<A>(
        &mut self,
        _page: Self::Virtual,
        _frame: Self::Physical,
        _flags: Self::Flags,
        _alloc: &mut A,
    ) -> Result<Self::Update, Self::Error>
    where
        A: page::FrameAllocator<Frame = Self::Physical>,
    {
        unimplemented!()
    }

    /// Identity maps a given `frame`.
    ///
    /// # Arguments
    /// + `frame`: the physical `Frame` to identity map
    /// + `flags`: the page table entry flags.
    /// + `alloc`: a memory allocator
    fn identity_map<A>(
        &mut self,
        _frame: Self::Physical,
        _flags: Self::Flags,
        _alloc: &mut A,
    ) -> Result<Self::Update, Self::Error>
    where
        A: page::FrameAllocator<Frame = Self::Physical>,
    {
        unimplemented!()
    }

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
        _page: Self::Virtual,
        _flags: Self::Flags,
        _alloc: &mut A,
    ) -> Result<Self::Update, Self::Error>
    where
        A: page::FrameAllocator<Frame = Self::Physical>,
    {
        unimplemented!()
    }

    /// Unmaps the given `VirtualPage`.
    ///
    /// All freed frames are returned to the given `page::FrameAllocator`.
    fn unmap<A>(
        &mut self,
        _page: Self::Virtual,
        _alloc: &mut A,
    ) -> Result<Self::Update, Self::Error>
    where
        A: page::FrameAllocator<Frame = Self::Physical>,
    {
        unimplemented!()
    }

    /// Updates the flags on the given `page`.
    fn set_flags(
        &mut self,
        _page: Self::Virtual,
        _flags: Self::Flags,
    ) -> Result<Self::Update, Self::Error> {
        unimplemented!()
    }
}
