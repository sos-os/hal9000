pub mod table;

pub mod size {
    use paging::MEGABYTE;
    pub use paging::{PageSize, Size2Mb, Size4Kb};

    /// 1 gigabyte "huge" pages.
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Size1Gb;

    impl PageSize for Size1Gb {
        const SIZE: usize = MEGABYTE * 1024;
    }
}
