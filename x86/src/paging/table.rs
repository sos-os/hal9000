use super::{PageSize, Virtual};
use core::{marker::PhantomData, ops};
use hal9000::mem::{page, Address, Page, VAddr};

pub const NUM_ENTRIES: usize = 512;

/// Marker indicating a page table level.
pub trait Level {
    /// How much to shift an address by to find its index in this table.
    const ADDR_SHIFT: usize;

    /// Mask for indices
    const INDEX_MASK: usize = 0o777;
}

pub trait Sublevel: Level {
    type Next: Level;
}

pub trait HoldsSize<S: PageSize>: Level {}

/// Trait implemented by `Level` for types which can index a page.
///
/// Consumers need not implement this --- it's used here to make `Table`'s
/// `Index`/`IndexMut` impls require less repeated code.
pub trait IndexedBy<I> {
    fn index_of(idx: &I) -> usize;
}

pub trait Entry {
    type PAddr: Address;
    type Frame: Page<Address = Self::PAddr>;
    type Error;
    type Flags: EntryOpts;

    /// Access the entry's bitflags.
    fn flags(&self) -> Self::Flags;

    /// Returns the physical address pointed to by this page table entry
    fn pointed_addr(&self) -> Self::PAddr;

    /// Returns the frame in memory pointed to by this page table entry.
    fn pointed_frame(&self) -> Result<Self::Frame, Self::Error>;

    fn validate_as_table(&self) -> Result<(), Self::Error>;

    fn set_addr(
        &mut self,
        addr: Self::PAddr,
        flags: Self::Flags,
    ) -> Result<(), Self::Error>;

    fn set_frame(
        &mut self,
        frame: Self::Frame,
        flags: Self::Flags,
    ) -> Result<(), Self::Error>;

    fn set_flags(&mut self, flags: Self::Flags);
}

pub trait EntryOpts {
    /// Returns true if this entry is mapped to a frame, or false if it is
    /// unused.
    fn is_present(&self) -> bool;

    /// Returns true if this entry is writable.
    fn is_writable(&self) -> bool;

    /// Returns true if the page is executable.
    fn can_execute(&self) -> bool;

    /// Sets whether or not the entry is present.
    fn set_present(self, unused: bool) -> Self;

    /// Sets whether the entry is writable.
    fn set_writable(self, writable: bool) -> Self;

    fn set_executable(self, writable: bool) -> Self;
}

/// A page table.
#[repr(C)]
pub struct Table<E, L: Level> {
    entries: [E; NUM_ENTRIES],
    _level: PhantomData<L>,
}

// ===== impl Table =====

impl<E, L> Table<E, L>
where
    E: Entry,
    L: Sublevel,
{
    /// Returns the address of the next table, or None if none exists.
    #[inline]
    fn next_table_addr(&self, i: usize) -> Option<VAddr> {
        if self[&i].flags().is_present() {
            let table_addr = self as *const _ as usize;
            Some(VAddr::from((table_addr << 9) | (i << 12)))
        } else {
            None
        }
    }

    /// Returns the next table, or `None` if none exists
    #[inline]
    pub fn next_table<I>(&self, i: &I) -> Option<&Table<E, L::Next>>
    where
        L: IndexedBy<I>,
    {
        self.next_table_addr(L::index_of(i))
            .map(|table_addr| unsafe { &*(table_addr.as_ptr()) })
    }

    /// Mutably borrows the next table.
    #[inline]
    pub fn next_table_mut<'b, I>(
        &self,
        i: &I,
    ) -> Option<&'b mut Table<E, L::Next>>
    where
        L: IndexedBy<I>,
    {
        self.next_table_addr(L::index_of(i))
            .map(|table_addr| unsafe { &mut *(table_addr.as_mut_ptr()) })
    }

    /// Returns the next table, creating it if it does not exist.
    pub fn create_next<A, I>(
        &mut self,
        i: &I,
        alloc: &mut A,
    ) -> Result<&mut Table<E, L::Next>, page::MapError<A, E::Error>>
    where
        L: IndexedBy<I>,
        A: page::FrameAllocator<Frame = <E as Entry>::Frame>,
    {
        // First, ensure that the entry at the given index is valid to point to
        // a page table (on 32-bit systems, this is a no-op, on x86_64, it
        // checks that the pointed frame isn't huge).
        self[i].validate_as_table()?;

        // If the entry is valid, and a page table already exists in that
        // frame, just return it.
        if let Some(next) = self.next_table_mut(i) {
            return Ok(next);
        }

        // Otherwise, the next table for that index doesn't exist yet.
        // Allocate a new frame to hold the page table.
        let frame = unsafe { alloc.alloc() }.map_err(page::MapError::Alloc)?;

        // Set the new page to be present and writable, and then update the
        // entry to point at that page with those flags set.
        let flags = self[i].flags().set_present(true).set_writable(true);
        self[i].set_frame(frame, flags)?;

        let table = self
            .next_table_mut(i)
            .expect("next page table was just created, so it should exist");
        // Zero out the new table before returning it.
        table.zero();

        Ok(table)
    }
}

impl<'a, Entry, L, I> ops::Index<&'a I> for Table<Entry, L>
where
    L: Level + IndexedBy<I>,
{
    type Output = Entry;
    #[inline]
    fn index(&self, i: &'a I) -> &Entry {
        &self.entries[L::index_of(i)]
    }
}

impl<'a, Entry, L, I> ops::IndexMut<&'a I> for Table<Entry, L>
where
    L: Level + IndexedBy<I>,
{
    #[inline]
    fn index_mut(&mut self, i: &'a I) -> &mut Entry {
        &mut self.entries[L::index_of(i)]
    }
}

impl<E, L> Table<E, L>
where
    E: Entry,
    L: Level,
{
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            let flags = entry.flags().set_present(false);
            entry.set_flags(flags);
        }
    }
}
// ===== impl Level =====

impl<T: Level> IndexedBy<VAddr> for T {
    fn index_of(addr: &VAddr) -> usize {
        (addr.as_usize() >> Self::ADDR_SHIFT) & Self::INDEX_MASK
    }
}

impl<T, S> IndexedBy<Virtual<S>> for T
where
    T: Level + HoldsSize<S>,
    S: PageSize,
{
    #[inline]
    fn index_of(page: &Virtual<S>) -> usize {
        Self::index_of(&page.base_address())
    }
}

impl<T: Level> IndexedBy<usize> for T {
    /// Indexing a table by a `usize` just gets positional indices.
    #[inline(always)]
    fn index_of(idx: &usize) -> usize {
        *idx
    }
}

pub mod level {
    use super::{HoldsSize, Level, Sublevel};
    use paging::{Size2Mb, Size4Kb};

    /// Marker for page directory (level 2) page tables.
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum Pd {}

    /// Marker for page table (level 1) page tables.
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum Pt {}

    impl Level for Pd {
        const ADDR_SHIFT: usize = 21;
    }

    impl Sublevel for Pd {
        type Next = Pt;
    }

    impl HoldsSize<Size4Kb> for Pd {}
    impl HoldsSize<Size2Mb> for Pd {}

    impl Level for Pt {
        const ADDR_SHIFT: usize = 12;
    }

    impl HoldsSize<Size4Kb> for Pt {}

}
