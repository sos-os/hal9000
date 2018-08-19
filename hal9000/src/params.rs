use core::ops::Range;
use mem::{self, Address, Page};

pub trait Architecture {
    /// This architecture's physical address type.
    type PAddr: Address;

    /// This architecture's physical page type.
    type Frame: Page;

    type MemRegion = mem::Region<Self::PAddr>;
}

pub trait BootParams {
    /// This architecture's physical address type.
    type Arch: Architecture;
    /// Abstracts over platform-specific details to represent
    /// the platform's memory map.
    type MemMap: Iterator<Item = <Self::Arch as Architecture>::MemRegion>;

    /// Returns the base address of the kernel memory region.
    fn kernel_base(&self) -> <Self::Arch as Architecture>::PAddr;

    /// Returns the end address of the kernel memory region.
    fn kernel_end(&self) -> <Self::Arch as Architecture>::PAddr;

    /// Returns the memory map.
    fn mem_map(&self) -> Self::MemMap;

    /// Returns the range of frames containing the kernel binary.
    ///
    /// The kernel _should_ start on the first address in the frame range,
    fn kernel_frames(&self) -> Range<<Self::Arch as Architecture>::Frame>;
}
