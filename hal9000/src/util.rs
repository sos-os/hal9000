use core::{iter, ops};

/// Trait adding `align_up` and `align_down` methods to numbers.
pub trait Align:
    Sized
    + Copy
    + ops::Add<Output = Self>
    + ops::Sub<Output = Self>
    + ops::BitAnd<Output = Self>
    + ops::Not<Output = Self>
    + iter::Step
{
    #[inline]
    fn align_up(&self, to: Self) -> Self {
        let align = to.sub_one();
        (*self + align) & !align
    }

    #[inline]
    fn align_down(&self, to: Self) -> Self {
        *self & !to.sub_one()
    }
}

impl Align for u8 {}
impl Align for u16 {}
impl Align for u32 {}
impl Align for u64 {}
impl Align for usize {}
