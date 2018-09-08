use core::fmt;
use hal9000::{mem::page, prelude::*};
use {
    paging::{
        table::{self, Entry, EntryOpts},
        FlushTlb,
    },
    x64::{page::*, PAddr, X86_64},
};

/// Struct representing the currently active PML4 instance.
pub struct CurrentPageTable<'a> {
    pml4: &'a mut Table<level::Pml4>,
}

/// A 64-bit page table entry.
#[derive(Clone)]
#[repr(transparent)]
pub struct Entry64(u64);

pub type Table<L> = table::Table<Entry64, L>;

const ADDR_MASK: u64 = 0x000f_ffff_ffff_f000;

const PML4_PTR: *mut Table<level::Pml4> = 0xffff_ffff_ffff_f000 as *mut _;

bitflags! {
    #[derive(Default)]
    pub struct EntryFlags: u64 {
        /// Must be set to map a 2 MB page or reference a page table.
        const PRESENT =         1 << 0;
        /// If unset, writes may not be allowed to the 2 MB region controlled
        /// by this entry
        const WRITABLE =        1 << 1;
        /// If set, ring 3 code can access this page.
        const USER_ACCESSIBLE = 1 << 2;
        /// If set, a “write-through” policy is used for the cache, else a
        /// “write-back” policy is used.
        const WRITE_THROUGH =   1 << 3;
        /// Disables caching for the pointed entry if set.
        const NO_CACHE =        1 << 4;
        /// Set by the CPU when the pointed frame or table is read from.
        const ACCESSED =        1 << 5;
        /// Set by the CPU when the pointed frame is written to.
        const DIRTY =           1 << 6;
        /// If set, the entry maps a huge frame instead of a page table.
        ///
        /// Only allowed in the PDPT or PD tables.
        const HUGE_PAGE =       1 << 7;
        /// If set, the mapping is present in all address spaces.
        ///
        /// Global pages aren't flushed from the TLB on an address space
        /// switch.
        const GLOBAL =          1 << 8;
        /// If set, code in the mapped frames may not be executed.
        const NO_EXECUTE =      1 << 63;
    }
}

#[derive(Debug, PartialEq)]
pub enum EntryError {
    NotPresent,
    NotAligned,
    Huge,
}

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
    type Flags = EntryFlags;
    /// The type returned by a page table update.
    ///
    /// This must be committed for the update to have an effect.
    type Update = FlushTlb<size::Size4Kb>;
    /// Any errors that can occur when mapping a page.
    type Error = EntryError;

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

// ===== impl Entry64 =====

impl Entry64 {
    fn is_huge(&self) -> bool {
        self.flags().contains(EntryFlags::HUGE_PAGE)
    }
}

impl Entry for Entry64 {
    type PAddr = PAddr;
    type Frame = Physical;
    type Error = EntryError;
    type Flags = EntryFlags;

    /// Access the entry's bitflags.
    fn flags(&self) -> Self::Flags {
        EntryFlags::from_bits_truncate(self.0)
    }

    fn validate_as_table(&self) -> Result<(), Self::Error> {
        if self.is_huge() {
            Err(EntryError::Huge)
        } else {
            Ok(())
        }
    }

    /// Returns the physical address pointed to by this page table entry
    fn pointed_addr(&self) -> PAddr {
        PAddr::from(self.0 & ADDR_MASK)
    }

    /// Returns the frame in memory pointed to by this page table entry.
    fn pointed_frame(&self) -> Result<Physical, Self::Error> {
        match self.flags() {
            flags if !flags.contains(EntryFlags::PRESENT) => {
                Err(EntryError::NotPresent)
            },
            flags if flags.contains(EntryFlags::HUGE_PAGE) => {
                Err(EntryError::Huge)
            },
            _ => Ok(Physical::from_addr_down(self.pointed_addr())),
        }
    }

    fn set_addr(
        &mut self,
        addr: PAddr,
        flags: Self::Flags,
    ) -> Result<(), Self::Error> {
        if !addr.is_page_aligned::<Physical>() {
            return Err(EntryError::NotAligned);
        };
        let addr: u64 = addr.into();
        self.0 = addr | flags.bits();
        Ok(())
    }

    fn set_frame(
        &mut self,
        frame: Physical,
        flags: Self::Flags,
    ) -> Result<(), Self::Error> {
        if self.is_huge() {
            return Err(EntryError::Huge);
        }
        self.set_addr(frame.base_address(), flags)
    }

    fn set_flags(&mut self, flags: Self::Flags) {
        let addr: u64 = self.pointed_addr().into();
        self.0 = addr | flags.bits();
    }
}

impl fmt::Debug for Entry64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Entry")
            .field("bits", &"64")
            .field("addr", &self.pointed_addr())
            .field("flags", &self.flags())
            .finish()
    }
}

impl EntryOpts for EntryFlags {
    fn is_present(&self) -> bool {
        !self.contains(EntryFlags::PRESENT)
    }

    fn is_writable(&self) -> bool {
        self.contains(EntryFlags::WRITABLE)
    }

    fn can_execute(&self) -> bool {
        !self.contains(EntryFlags::NO_EXECUTE)
    }

    fn set_present(mut self, unused: bool) -> Self {
        if unused {
            self.remove(EntryFlags::PRESENT)
        } else {
            self.insert(EntryFlags::PRESENT)
        };
        self
    }

    fn set_writable(mut self, writable: bool) -> Self {
        if writable {
            self.insert(EntryFlags::WRITABLE)
        } else {
            self.remove(EntryFlags::WRITABLE)
        };
        self
    }

    fn set_executable(mut self, executable: bool) -> Self {
        if executable {
            self.remove(EntryFlags::NO_EXECUTE)
        } else {
            self.insert(EntryFlags::NO_EXECUTE)
        };
        self
    }
}

pub mod level {
    use super::super::size;
    pub use paging::table::{level::*, HoldsSize, Level, Sublevel};

    /// Marker for page directory meta-level 4 (level 4) page tables.
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum Pml4 {}

    /// Marker for page directory pointer table (level 3) page tables.
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum Pdpt {}

    impl Level for Pml4 {
        const ADDR_SHIFT: usize = 39;
    }

    impl Sublevel for Pml4 {
        type Next = Pdpt;
    }

    impl HoldsSize<size::Size4Kb> for Pml4 {}
    impl HoldsSize<size::Size2Mb> for Pml4 {}
    impl HoldsSize<size::Size1Gb> for Pml4 {}

    impl Level for Pdpt {
        const ADDR_SHIFT: usize = 30;
    }

    impl Sublevel for Pdpt {
        type Next = Pd;
    }

    impl HoldsSize<size::Size4Kb> for Pdpt {}
    impl HoldsSize<size::Size2Mb> for Pdpt {}
    impl HoldsSize<size::Size1Gb> for Pdpt {}

}
