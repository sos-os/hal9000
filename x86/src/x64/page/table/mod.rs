use core::fmt;
use hal9000::{mem::page, prelude::*};
use {
    paging::{
        table::{self, Entry, EntryOpts},
        FlushTlb,
    },
    x64::{page::*, PAddr, X86_64},
};

pub mod entry;
pub mod level;

pub use self::entry::Entry64;

/// Struct representing the currently active PML4 instance.
pub struct CurrentPageTable<'a> {
    pml4: &'a mut Table<level::Pml4>,
}

pub type Table<L> = table::Table<Entry64, L>;

const ADDR_MASK: u64 = 0x000f_ffff_ffff_f000;

const PML4_PTR: *mut Table<level::Pml4> = 0xffff_ffff_ffff_f000 as *mut _;


impl CurrentPageTable<'static> {
    pub unsafe fn new() -> Self {
        Self {
            pml4: &mut *PML4_PTR,
        }
    }
}

impl<'a> page::Map<Virtual<size::Size4Kb>, Physical<size::Size4Kb>>
    for CurrentPageTable<'a>
{
    type Arch = X86_64;
    type PAddr = PAddr;
    /// Architecture-dependent flags that configure a virtual page.
    type Flags = entry::Flags;
    /// The type returned by a page table update.
    ///
    /// This must be committed for the update to have an effect.
    type Update = FlushTlb<size::Size4Kb>;
    /// Any errors that can occur when mapping a page.
    type Error = entry::Error;

    /// Translates a virtual address to the corresponding physical address.
    ///
    /// # Returns
    /// + `Some(PAddr)` containing the physical address corresponding to
    ///                 `vaddr`, if it is mapped.
    /// + `None`: if the address is not mapped.
    fn translate(&self, vaddr: VAddr) -> Option<Self::PAddr> {
        unimplemented!()
    }

    /// Translates a virtual page to a physical frame.
    fn translate_page(
        &self,
        page: Virtual<size::Size4Kb>,
    ) -> Option<Physical<size::Size4Kb>> {
        // Get the level 1 page table for the virtual page.
        let page_table = self.pml4.page_table_for(&page)?;
        // Return the frame pointed to by the entry in the level 1 table.
        page_table[&page].pointed_frame().ok()
        // TODO(eliza): figure out how to handle huge pages...
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
        page: Virtual<size::Size4Kb>,
        frame: Physical<size::Size4Kb>,
        flags: Self::Flags,
        alloc: &mut A,
    ) -> Result<Self::Update, page::MapError<A, Self::Error>>
    where
        A: page::FrameAllocator<Frame = Physical<size::Size4Kb>>,
    {
        let page_table = self
            .pml4
            .create_next(&page, alloc)
            .and_then(|pdpt| pdpt.create_next(&page, alloc))
            .and_then(|pd| pd.create_next(&page, alloc))?;

        let entry = &mut page_table[&page];
        let flags = entry.flags();

        if flags.is_present() {
            return Err(page::MapError::AlreadyMapped);
        }

        entry
            .set_frame(frame, flags.set_present(true))
            .map_err(page::MapError::Other)?;

        Ok(FlushTlb::new(page))
    }

    /// Identity maps a given `frame`.
    ///
    /// # Arguments
    /// + `frame`: the physical `Frame` to identity map
    /// + `flags`: the page table entry flags.
    /// + `alloc`: a memory allocator
    fn identity_map<A>(
        &mut self,
        frame: Physical<size::Size4Kb>,
        flags: Self::Flags,
        alloc: &mut A,
    ) -> Result<Self::Update, page::MapError<A, Self::Error>>
    where
        A: page::FrameAllocator<Frame = Physical<size::Size4Kb>>,
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
        page: Virtual<size::Size4Kb>,
        flags: Self::Flags,
        alloc: &mut A,
    ) -> Result<Self::Update, page::MapError<A, Self::Error>>
    where
        A: page::FrameAllocator<Frame = Physical<size::Size4Kb>>,
    {
        unimplemented!()
    }

    /// Unmaps the given `VirtualPage`.
    ///
    /// All freed frames are returned to the given `page::FrameAllocator`.
    fn unmap<A>(
        &mut self,
        page: Virtual<size::Size4Kb>,
        alloc: &mut A,
    ) -> Result<Self::Update, Self::Error>
    where
        A: page::FrameAllocator<Frame = Physical<size::Size4Kb>>,
    {
        unimplemented!()
    }

    /// Updates the flags on the given `page`.
    fn set_flags(
        &mut self,
        page: Virtual<size::Size4Kb>,
        flags: Self::Flags,
    ) -> Result<Self::Update, Self::Error> {
        unimplemented!()
    }
}

impl Table<level::Pml4> {
    /// Returns a reference to the page table for the given index.
    #[inline]
    pub fn page_table_for<I>(&self, idx: &I) -> Option<&Table<level::Pt>>
    where
        level::Pml4: table::IndexedBy<I>,
        level::Pdpt: table::IndexedBy<I>,
        level::Pd: table::IndexedBy<I>,
    {
        self.next_table(idx)
            .and_then(|pdpt| pdpt.next_table(idx))
            .and_then(|pd| pd.next_table(idx))
    }

    /// Returns a mutable reference to the page table for the given index.
    #[inline]
    pub fn page_table_mut_for<I>(
        &mut self,
        idx: &I,
    ) -> Option<&mut Table<level::Pt>>
    where
        level::Pml4: table::IndexedBy<I>,
        level::Pdpt: table::IndexedBy<I>,
        level::Pd: table::IndexedBy<I>,
    {
        self.next_table_mut(idx)
            .and_then(|pdpt| pdpt.next_table_mut(idx))
            .and_then(|pd| pd.next_table_mut(idx))
    }
}
