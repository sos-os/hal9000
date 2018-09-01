//
// SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website), and the SOS contributors.
//
//  Copyright (c) 2018 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Architecture-independent representation of platform-specific memory maps.
use super::{Address, Page};

use core::ops;

/// A memory region.
///
/// This represents a region of memory with a base address and a length,
/// such as those given to us by multiboot2 or other bootloaders. It is
/// not necessarily page-aligned; a given `Region` will likely encompass
/// several frames.
pub trait Region {
    type Addr: Address;

    /// Returns the base address of the memory region.
    fn base_address(&self) -> Self::Addr;

    /// Returns the size in bytes of the memory region.
    fn size(&self) -> usize;

    /// Returns the end  address of the memory region.
    fn end_address(&self) -> Self::Addr
    where
        Self::Addr: ops::Add<usize, Output = Self::Addr>,
    {
        self.base_address() + self.size()
    }

    /// Returns the number of frames contained in a given region.
    fn num_frames<P: Page>(&self) -> usize {
        self.size() / P::SIZE
    }

    fn is_usable(&self) -> bool;

    fn is_used(&self) -> bool;

    fn is_kernel(&self) -> bool;

}


pub mod imp {
    use super::*;

    use core::cmp;

    #[derive(Copy, Clone, Debug, Eq)]
    pub struct GenericRegion<A> {

        pub base_address: A,

        /// The size in bytes of the memory region.
        pub size: usize,

        /// Whether this memory region is usable.
        pub kind: RegionKind,
    }

    /// Marker for the type of a memory region.
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[repr(C)]
    pub enum RegionKind {
        /// Usable RAM
        Usable,
        /// RAM in use by the kernel.
        InUse,
        /// RAM reserved (i.e. by ACPI).
        ///
        /// TODO(eliza): should we add specific variants for various types of
        ///              reserved regions?
        Unusable,
        Kernel,
        /// Additional region types may be added.
        __Nonexhaustive,
    }

    // ===== impl Region =====

    impl<A: Address + Copy> Region for GenericRegion<A> {
        type Addr = A;


        /// Returns the base address of the memory region.
        fn base_address(&self) -> Self::Addr {
            self.base_address
        }

        /// Returns the size in bytes of the memory region.
        fn size(&self) -> usize {
            self.size
        }

        /// Returns true if this region is usable.
        fn is_usable(&self) -> bool {
            match self.kind {
                RegionKind::Usable => true,
                _ => false,
            }
        }

        /// Returns true if this region is in use.
        fn is_used(&self) -> bool {
            match self.kind {
                RegionKind::InUse => true,
                _ => false,
            }
        }

        /// Returns true if this region is used by the kernel..
        fn is_kernel(&self) -> bool {
            match self.kind {
                RegionKind::Kernel => true,
                _ => false,
            }
        }
    }

    impl<A> cmp::PartialEq for GenericRegion<A>
    where
        A: cmp::PartialEq,
    {
        fn eq(&self, other: &Self) -> bool {
            self.base_address == other.base_address && self.size == other.size
                && self.kind == other.kind
        }
    }

    impl<A> cmp::PartialOrd for GenericRegion<A>
    where
        A: cmp::PartialOrd,
    {
        fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
            self.base_address.partial_cmp(&other.base_address)
        }
    }
}
