use hal9000::mem::{VAddr, Page};
use super::{Virtual, PageSize};
use core::{
    ops,
    marker::PhantomData
};

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

/// Trait implemented by `Level` for types which can index a page.
///
/// Consumers need not implement this --- it's used here to make `Table`'s
/// `Index`/`IndexMut` impls require less repeated code.
pub trait IndexedBy<I> {
    fn index_of(idx: I) -> usize;
}

/// A page table.
#[repr(C)]
pub struct Table<E, L: Level> {
    entries: [E; NUM_ENTRIES],
    _level: PhantomData<L>,
}


// ===== impl Table =====

impl<Entry, L, I> ops::Index<I> for Table<Entry, L>
where
    L: Level + IndexedBy<I>,
{
    type Output = Entry;

    #[inline]
    fn index(&self, i: I) -> &Entry {
        &self.entries[L::index_of(i)]
    }
}

impl<Entry, L, I> ops::IndexMut<I> for Table<Entry, L>
where
    L: Level + IndexedBy<I>,
{
    #[inline]
    fn index_mut(&mut self, i: I) -> &mut Entry {
        &mut self.entries[L::index_of(i)]
    }
}

// ===== impl Level =====

impl<T: Level> IndexedBy<VAddr> for T {
    fn index_of(addr: VAddr) -> usize {
        (addr.as_usize() >> Self::ADDR_SHIFT) & Self::INDEX_MASK
    }
}

impl<T: Level, S: PageSize> IndexedBy<Virtual<S>> for T {
    #[inline]
    fn index_of(page: Virtual<S>) -> usize {
        Self::index_of(page.base_address())
    }
}

impl<T: Level> IndexedBy<usize> for T {
    /// Indexing a table by a `usize` just gets positional indices.
    #[inline(always)]
    fn index_of(idx: usize) -> usize {
        idx
    }
}
