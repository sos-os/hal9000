use params::BootParams;
use Architecture;

pub trait Cpu: Sized {
    type Arch: Architecture;
    /// Identifier for a given CPU.
    type Id;
    /// Error returned by CPU initialization.
    type InitError;

    unsafe fn init<P: BootParams<Arch = Self::Arch>>(
        params: &P,
        id: Self::Id,
    ) -> Result<Self, Self::InitError>;
}

/// Controls CPU interrupts.
pub trait IrqCtrl {
    type Arch: Architecture;
    /// Error returned by interrupt initialization.
    type InitError;

    /// Initializes interrupts.
    unsafe fn init<P>(params: &P) -> Result<(), Self::InitError>
    where
        P: BootParams<Arch = Self::Arch>;

    /// Enables interrupts.
    unsafe fn enable(&mut self);

    /// Disables interrupts.
    unsafe fn disable_irq(&mut self);

    /// Returns `true` if interrupts are enabled.
    fn is_enabled(&self) -> bool;
}
