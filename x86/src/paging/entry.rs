use ::{
    PAddr,
    paging::{
        PageSize,
        Physical,
    }
};
use hal9000::mem::{Address, Page};
use core::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct Handle<'a, S: PageSize, R: Repr + 'a> {
    repr: &'a R,
    _size: PhantomData<S>,
}

#[derive(Debug)]
pub struct HandleMut<'a, S: PageSize, R: Repr + 'a> {
    repr: &'a mut R,
    _size: PhantomData<S>,
}

pub trait Repr {
    type Flags: Opts;

    /// Access the entry's bitflags.
    fn flags(&self) -> Self::Flags;

    /// Returns the physical address pointed to by this page table entry
    fn pointed_addr(&self) -> PAddr;

    // /// Returns the frame in memory pointed to by this page table entry.
    // fn pointed_frame(&self) -> Result<Physical<Size>, Self::Error>;

    fn validate_as_table(&self) -> Result<(), Error>;

    fn set_addr(
        &mut self,
        addr: PAddr,
        flags: Self::Flags,
    ) -> Result<(), Error>;

    // fn set_frame(
    //     &mut self,
    //     frame: Physical<Size>,
    //     flags: Self::Flags,
    // ) -> Result<(), Self::Error>;

    fn set_flags(&mut self, flags: Self::Flags);
}

#[derive(Debug, PartialEq)]
pub enum Error {
    /// The entry was not present.
    NotPresent,
    /// The entry's address was not aligned on a page boundary.
    NotAligned,
    Huge,
}

pub trait Opts {
    /// Returns true if this entry is mapped to a frame, or false if it is
    /// unused.
    fn is_present(&self) -> bool;

    /// Returns true if this entry is writable.
    fn is_writable(&self) -> bool;

    /// Returns true if the page is executable.
    fn can_execute(&self) -> bool;

    /// Sets whether or not the entry is present.
    fn set_present(self, unused: bool) -> Self;

    /// Sets whether the entry is writable.
    fn set_writable(self, writable: bool) -> Self;

    fn set_executable(self, writable: bool) -> Self;
}

impl<'a, S, R> Handle<'a, S, R>
where
    S: PageSize,
    R: Repr + 'a,
{
    #[inline]
    pub(in ::paging) fn new(repr: &'a R) -> Self {
        Self {
            repr,
            _size: PhantomData,
        }
    }

    /// Access the entry's bitflags.
    #[inline]
    pub fn flags(&self) -> R::Flags {
        self.repr.flags()
    }

    /// Returns the physical address pointed to by this page table entry
    #[inline]
    pub fn pointed_addr(&self) -> PAddr {
        self.repr.pointed_addr()
    }

    /// Returns the frame in memory pointed to by this page table entry.
    pub fn pointed_frame(&self) -> Result<Physical<S>, Error> {
        // TODO: validate huge correctly?
        if self.flags().is_present() {
            return Ok(Physical::from_addr_down(self.pointed_addr()));
        }
        Err(Error::NotPresent)
    }
}

impl<'a, S, R> HandleMut<'a, S, R>
where
    S: PageSize,
    R: Repr + 'a,
{
    #[inline]
    pub(in ::paging) fn new(repr: &'a mut R) -> Self {
        Self {
            repr,
            _size: PhantomData,
        }
    }

    /// Access the entry's bitflags.
    #[inline]
    pub fn flags(&self) -> R::Flags {
        self.repr.flags()
    }

    /// Returns the physical address pointed to by this page table entry
    #[inline]
    pub fn pointed_addr(&self) -> PAddr {
        self.repr.pointed_addr()
    }

    /// Returns the frame in memory pointed to by this page table entry.
    pub fn pointed_frame(&self) -> Result<Physical<S>, Error> {
        // TODO: validate huge correctly?
        if self.flags().is_present() {
            return Ok(Physical::from_addr_down(self.pointed_addr()));
        }
        Err(Error::NotPresent)
    }

    #[inline]
    pub fn set_addr(
        &mut self,
        addr: PAddr,
        flags: R::Flags,
    ) -> Result<(), Error> {
        self.repr.set_addr(addr, flags)
    }

    #[inline]
    pub fn set_frame(
        &mut self,
        frame: Physical<S>,
        flags: R::Flags,
    ) -> Result<(), Error> {
        self.set_addr(frame.base_address(), flags)
    }
}
