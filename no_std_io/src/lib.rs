#![cfg_attr(feature = "no_std", no_std)]

pub type Error = ();

pub type Result<T> = ::core::result::Result<T, Error>;

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
}

pub trait Seek {
    fn seek(&mut self, from: SeekFrom) -> Result<usize>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeekFrom {
    Start(usize),
    Current(isize),
    End(isize)
}
