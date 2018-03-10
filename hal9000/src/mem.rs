//
// SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website), and the SOS contributors.
//
//  Copyright (c) 2018 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! # Architecture-independent memory abstractions.
use util::Align;

use core::{cmp, ops};

/// Trait representing an address, whether physical or virtual.
pub trait Address {
    /// The primitive numeric type used to represent this address.
    type Repr: Align;

    /// Align this address down to the provided alignment.
    fn align_down(&self, align: Self::Repr) -> Self;

    /// Align this address up to the provided alignment.
    fn align_up(&self, align: Self::Repr) -> Self;

    /// Returns true if this address is aligned on a page boundary.
    fn is_page_aligned<P: Page>(&self) -> bool;
}

/// A physical address.
pub trait PhysicalAddress: Address {
    /// Convert this physical address into a raw pointer to a T.
    fn as_mut_ptr<T>(&self) -> *mut T;
}

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

/// A virtual memory address.
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Address)]
#[address_repr(usize)]
pub struct VAddr(pub usize);

/// A memory region.
///
/// This represents a region of memory with a base address and a length,
/// such as those given to us by multiboot2 or other bootloaders. It is
/// not necessarily page-aligned; a given `Region` will likely encompass
/// several frames.
#[derive(Copy, Clone, Debug, Eq)]
pub struct Region<A> {
    /// The base address of the memory region.
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
    /// Additional region types may be added.
    __Nonexhaustive,
}

// ===== impl Region =====

impl<A: Address + Copy> Region<A> {
    /// Returns the end address of the region.
    pub fn end_address(&self) -> A
    where
        A: ops::Add<usize, Output = A>,
    {
        self.base_address + self.size
    }
}

impl<A> Region<A> {
    /// Returns true if this region is usable.
    pub fn is_usable(&self) -> bool {
        match self.kind {
            RegionKind::Usable => true,
            _ => false,
        }
    }

    /// Returns the number of frames contained in a given region.
    pub fn num_frames<P>(&self) -> usize
    where
        P: Page,
    {
        self.size / P::SIZE
    }
}

impl<A> cmp::PartialEq for Region<A>
where
    A: cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.base_address == other.base_address && self.size == other.size
            && self.kind == other.kind
    }
}

impl<A> cmp::PartialOrd for Region<A>
where
    A: cmp::PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.base_address.partial_cmp(&other.base_address)
    }
}
