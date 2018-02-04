use util::Align;

use core::ops;

/// Trait representing an address, whether physical or virtual.
pub trait Address
    : ops::Add<Self>
    + ops::Sub<Self>
    + ops::Mul<Self>
    + ops::Div<Self>
    + ops::Shl<Self>
    + ops::Shr<Self>
    + ops::BitOr<Self>
    + ops::BitAnd<Self>
    + From<*mut u8>
    + From<*const u8>
    + Sized {

    /// The primitive numeric type used to represent this address.
    type Repr: Align;

    /// Align this address down to the provided alignment.
    fn align_down(&self, align: Self::Repr) -> Self;

    /// Align this address up to the provided alignment.
    fn align_up(&self, align: Self::Repr) -> Self;

    /// Returns true if this address is aligned on a page boundary.
    fn is_page_aligned(&self) -> bool;
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