use hal9000::mem::{self, Address};

use ::{
    paging::{Page, Small, PageSize},
    x64::{PAddr, VAddr},
};

pub mod table;

pub type Physical<S = Small> = Page<PAddr, S>;
pub use ::paging::Virtual;

pub mod size {
    use super::PageSize;
    pub use paging::Small;

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Large;
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Huge;

    impl PageSize for Large {
        const SIZE: usize = Small::SIZE * 512;
    }

    impl PageSize for Huge {
        const SIZE: usize = Large::SIZE * 512;
    }
}
