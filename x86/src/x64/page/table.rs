use hal9000::{
    prelude::*,
    mem::page,
    Architecture,
};
use ::{
    x64::{
        X86_64,
        PAddr,
        page::*,
    },
    paging::{table::Entry, FlushTlb},
};
use core::fmt;

/// Struct representing the currently active PML4 instance.
///
/// The `ActivePML4` is a `Unique` reference to a PML4-level page table. It's
/// unique because, well, there can only be one active PML4 at a given time.
pub struct ActivePml4 {
    // TODO: implement
//plm4: Unique<Table<Pml4Level>>
}

/// A 64-bit page table entry.
#[derive(Clone)]
#[repr(transparent)]
pub struct Entry64(u64);

pub enum Error {
    Alloc,
    // TODO: Implement
}

const ADDR_MASK: u64 = 0x000fffff_fffff000;

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

impl page::Mapper for ActivePml4 {
    type Arch = X86_64;

    type Virtual = Virtual;

    /// Architecture-dependent flags that configure a virtual page.
    type Flags = EntryFlags;
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
    fn translate(&self, vaddr: VAddr) -> Option<Self::PAddr> {
        unimplemented!()
    }

    /// Translates a virtual page to a physical frame.
    fn translate_page(&self, page: Self::Virtual) -> Option<Self::Physical> {
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
        page: Self::Virtual,
        frame: Self::Physical,
        flags: Self::Flags,
        alloc: &mut A,
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
        frame: Self::Physical,
        flags: Self::Flags,
        alloc: &mut A,
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
        page: Self::Virtual,
        flags: Self::Flags,
        alloc: &mut A,
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
        page: Self::Virtual,
        alloc: &mut A,
    ) -> Result<Self::Update, Self::Error>
    where
        A: page::FrameAllocator<Frame = Self::Physical>,
    {
        unimplemented!()
    }

    /// Updates the flags on the given `page`.
    fn set_flags(
        &mut self,
        page: Self::Virtual,
        flags: Self::Flags,
    ) -> Result<Self::Update, Self::Error> {
        unimplemented!()
    }
}

impl Entry64 {
    fn is_huge(&self) -> bool {
        self.flags().contains(EntryFlags::HUGE_PAGE)
    }

    fn is_present(&self) -> bool {
        self.flags().contains(EntryFlags::PRESENT)
    }
}

impl Entry for Entry64
// where
//     u64: From<PAddr>,
{
    type PAddr = PAddr;
    type Frame = Physical;
    type Error = EntryError;
    type Flags = EntryFlags;

    /// Returns true if this is an unused entry
    fn is_unused(&self) -> bool {
        unimplemented!()
    }

    /// Sets this entry to be unused
    fn set_unused(&mut self) -> &mut Self {
        unimplemented!()
    }

    /// Access the entry's bitflags.
    fn flags(&self) -> Self::Flags {
        EntryFlags::from_bits_truncate(self.0)
    }

    /// Returns the physical address pointed to by this page table entry
    fn pointed_addr(&self) -> PAddr {
        PAddr::from(self.0 & ADDR_MASK)
    }

    /// Returns the frame in memory pointed to by this page table entry.
    fn pointed_frame(&self) -> Result<Physical, Self::Error> {
        match self.flags() {
            flags if !flags.contains(EntryFlags::PRESENT) => Err(EntryError::NotPresent),
            flags if flags.contains(EntryFlags::HUGE_PAGE) => Err(EntryError::Huge),
            _ => Ok(Physical::from_addr_down(self.pointed_addr())),
        }
    }

    fn set_addr(&mut self, addr: PAddr, flags: Self::Flags) -> Result<(), Self::Error> {
        if !addr.is_page_aligned::<Physical>() {
            return Err(EntryError::NotAligned);
        };
        let addr: u64 = addr.into();
        self.0 = addr | flags.bits();
        Ok(())
    }

    fn set_frame(&mut self, frame: Physical, flags: Self::Flags) -> Result<(), Self::Error> {
        if self.is_huge() {
            return Err(EntryError::Huge);
        }
        self.set_addr(frame.base_address(),  flags)
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
