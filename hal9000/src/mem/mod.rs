//
// SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website), and the SOS contributors.
//
//  Copyright (c) 2018 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! # Architecture-independent memory abstractions.
pub use self::page::Page;

use core::ops;
use {params::BootParams, util::Align, Architecture};

pub mod map;
pub mod page;

/// Trait representing an address, whether physical or virtual.
pub trait Address: ops::Add + Sized {
    /// The primitive numeric type used to represent this address.
    type Repr: Align + Into<Self>;

    /// Align this address down to the provided alignment.
    fn align_down(&self, align: usize) -> Self;

    /// Align this address up to the provided alignment.
    fn align_up(&self, align: usize) -> Self;

    /// Returns true if this address is aligned on a page boundary.
    fn is_page_aligned<P: Page>(&self) -> bool;

    /// Convert this address into a const pointer to a value of type `T`.
    fn as_ptr<T>(&self) -> *const T;

    /// Convert this address into a raw pointer to a value of type `T`.
    fn as_mut_ptr<T>(&self) -> *mut T;
}

/// A physical address.
pub trait PhysicalAddress: Address {}

pub trait MemCtrl {
    type Arch: Architecture;
    type Frame = <Self::Arch as Architecture>::Frame;
    type Error;

    /// Initialize the MMU's paging.
    ///
    /// This function should not be called more than once. The kernel should
    /// not normally do so, but implementors of this trait may choose to add
    /// additional checks against repeated calls.
    unsafe fn init_paging<P, A>(
        params: &P,
        frame_allocator: &mut A,
    ) -> Result<(), Self::Error>
    where
        P: BootParams<Arch = Self::Arch>,
        A: page::FrameAllocator<Frame = Self::Frame>,
        Self::Frame: Page;

    /// Initializes the OS heap.
    ///
    /// This function should not be called more than once. The kernel should
    /// not normally do so, but implementors of this trait may choose to add
    /// additional checks against repeated calls.
    ///
    /// # Safety
    ///
    /// - Until this function has returned `Ok`, using heap-allocated types will
    ///   result in undefined behaviour.
    /// - Calling this function with incorrect parameters may result in
    ///   undefined behaviour.
    unsafe fn init_heap<P, A>(
        params: &P,
        frame_allocator: &mut A,
    ) -> Result<(), Self::Error>
    where
        P: BootParams<Arch = Self::Arch>,
        A: page::FrameAllocator<Frame = Self::Frame>,
        Self::Frame: Page;
}
