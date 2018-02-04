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
