    use super::super::size;
    pub use paging::table::{level::*, HoldsSize, Level, Sublevel};

    /// Marker for page directory meta-level 4 (level 4) page tables.
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum Pml4 {}

    /// Marker for page directory pointer table (level 3) page tables.
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum Pdpt {}

    impl Level for Pml4 {
        const ADDR_SHIFT: usize = 39;
    }

    impl Sublevel for Pml4 {
        type Next = Pdpt;
    }

    impl HoldsSize<size::Size4Kb> for Pml4 {}
    impl HoldsSize<size::Size2Mb> for Pml4 {}
    impl HoldsSize<size::Size1Gb> for Pml4 {}

    impl Level for Pdpt {
        const ADDR_SHIFT: usize = 30;
    }

    impl Sublevel for Pdpt {
        type Next = Pd;
    }

    impl HoldsSize<size::Size4Kb> for Pdpt {}
    impl HoldsSize<size::Size2Mb> for Pdpt {}
    impl HoldsSize<size::Size1Gb> for Pdpt {}
