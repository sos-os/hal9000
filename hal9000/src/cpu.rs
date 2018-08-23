use ::Architecture;
use params::BootParams;

pub trait Cpu {
    type Arch: Architecture;

    unsafe fn init<B: BootParams<Arch = Self::Arch>>(&P);

    unsafe fn enable_irq();
    unsafe fn disable_irq();
}
