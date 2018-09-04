//! Re-exports of all HAL 9000 traits.
//!
//! Traits are renamed so that a *-import of `hal900::prelude::*`
//! will not collide with other types.
pub use mem::{
    map::Region as __hal9000_mem_map_Region,
    page::{
        FrameAllocator as __hal9000_mem_page_FrameAllocator,
        Page as __hal9000_mem_page_Page,
    },
    Address as __hal9000_mem_Address,
    PhysicalAddress as __hal9000_mem_PhysicalAddress,
};
pub use params::BootParams as __hal9000_params_BootParams;
pub use util::Align as __hal9000_util_Align;
pub use Architecture as __hal9000_Architecture;
