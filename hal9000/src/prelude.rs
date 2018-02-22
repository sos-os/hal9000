//! Re-exports of all HAL 9000 traits.
//!
//! Traits are renamed so that a *-import of `hal900::prelude::*`
//! will not collide with other types.
pub use ::mem::{
    Address as __hal9000_mem_Address,
    Page as __hal9000_mem_Page,
    PhysicalAddress as __hal9000_mem_PhysicalAddress,
    Region as __hal9000_mem_Region,
};
pub use ::params::{
    Architecture as __hal9000_params_Architecture,
    BootParams as __hal9000_params_BootParams,
};
pub use ::util::Align as __hal9000_util_Align;
