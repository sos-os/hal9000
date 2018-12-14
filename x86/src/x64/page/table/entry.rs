use core::fmt;
use hal9000::mem::{Address, Page};
use {
    paging::{
        Physical,
    },
    x64::{page::*, PAddr},
};

pub use paging::entry::*;

/// A 64-bit page table entry.
///
/// Page table entries are laid out as follows:
///
/// ```notrust
/// 63    62 ...   52 ...                            12 ...   9 ...    0
/// +----+---------+---------------------------------+--------+-------+
/// | NX | unused | address of frame/next page table | unused | flags |
/// +----+--------+----------------------------------+--------+-------+
///      10 bits  40 bits                            2 bits   8 bits
/// ```
///
/// + The 63rd bit is the NO_EXECUTE flag. If this is set (and the NXE bit in
///   the EFER register is also set), data on this page may not be executed as
///   code.
/// + Bits 62 through 52 and 11 through 9 are unused by the CPU. The OS may use
///   these 12 bits for system-specific flags, or leave them unset.
/// + Bits 51 through 12 are the (4KiB) page-aligned 52-bit physical address of
///   either the frame or the next page table, depending on whether the entry
///   maps directly to a frame or to a lower-level page table. Note that this
///   section of the entry is only 40 bits; this is because these addresses are
///   _page-aligned. Since the 12 least significant bits of an address would
///   correspond to relative offsets within a 4KiB page, they are assumed to be
///   0 when translating a page table entry to a frame.
/// + Bits 9 through 0 are flags that configure the page table entry. See the
///   [Flags] type for details on these flags.
#[derive(Clone)]
#[repr(transparent)]
pub struct Entry64(u64);

// #[derive(Debug, PartialEq)]
// pub enum Error {
//     /// The entry was not present.
//     NotPresent,
//     /// The entry's address was not aligned on a page boundary.
//     NotAligned,
//     Huge,
// }

bitflags! {
    #[derive(Default)]
    pub struct Flags: u64 {
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

// ===== impl Entry64 =====

impl Entry64 {
    fn is_huge(&self) -> bool {
        self.flags().contains(Flags::HUGE_PAGE)
    }
}

impl Repr for Entry64 {
    type Flags = Flags;

    /// Access the entry's bitflags.
    fn flags(&self) -> Self::Flags {
        Flags::from_bits_truncate(self.0)
    }

    fn validate_as_table(&self) -> Result<(), Error> {
        if self.is_huge() {
            Err(Error::Huge)
        } else {
            Ok(())
        }
    }

    /// Returns the physical address pointed to by this page table entry
    fn pointed_addr(&self) -> PAddr {
        PAddr::from(self.0 & table::ADDR_MASK)
    }

    // /// Returns the frame in memory pointed to by this page table entry.
    // fn pointed_frame(&self) -> Result<Physical, Self::Error> {
    //     match self.flags() {
    //         flags if !flags.contains(Flags::PRESENT) => Err(Error::NotPresent),
    //         flags if flags.contains(Flags::HUGE_PAGE) => Err(Error::Huge),
    //         _ => Ok(Physical::from_addr_down(self.pointed_addr())),
    //     }
    // }

    fn set_addr(
        &mut self,
        addr: PAddr,
        flags: Self::Flags,
    ) -> Result<(), Error> {
        if !addr.is_page_aligned::<Physical>() {
            return Err(Error::NotAligned);
        };
        let addr: u64 = addr.into();
        self.0 = addr | flags.bits();
        Ok(())
    }

    // fn set_frame(
    //     &mut self,
    //     frame: Physical,
    //     flags: Self::Flags,
    // ) -> Result<(), Self::Error> {
    //     if self.is_huge() {
    //         return Err(Error::Huge);
    //     }
    //     self.set_addr(frame.base_address(), flags)
    // }

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

impl Opts for Flags {
    fn is_present(&self) -> bool {
        !self.contains(Flags::PRESENT)
    }

    fn is_writable(&self) -> bool {
        self.contains(Flags::WRITABLE)
    }

    fn can_execute(&self) -> bool {
        !self.contains(Flags::NO_EXECUTE)
    }

    fn set_present(mut self, unused: bool) -> Self {
        if unused {
            self.remove(Flags::PRESENT)
        } else {
            self.insert(Flags::PRESENT)
        };
        self
    }

    fn set_writable(mut self, writable: bool) -> Self {
        if writable {
            self.insert(Flags::WRITABLE)
        } else {
            self.remove(Flags::WRITABLE)
        };
        self
    }

    fn set_executable(mut self, executable: bool) -> Self {
        if executable {
            self.remove(Flags::NO_EXECUTE)
        } else {
            self.insert(Flags::NO_EXECUTE)
        };
        self
    }
}
