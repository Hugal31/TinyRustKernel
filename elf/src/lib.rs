#![cfg_attr(feature = "no_std", no_std)]
#![feature(try_from)]

extern crate no_std_io;

mod enums;
mod iterators;

use core::convert::TryFrom;
use core::fmt;
use core::intrinsics::transmute;
use core::mem::{size_of, uninitialized};
use core::slice;

use no_std_io::{Error as IoError, Read, Seek, SeekFrom};

pub use self::enums::*;

const ELFMAG: &[u8] = b"\x7FELF";
const EI_NIDENT: usize = 16;

#[derive(Debug)]
pub enum Error {
    Io(IoError),
    /// The file is not an elf (e.g. wrong magic number / too small for the header)
    NotAnELF,
    /// The file is probably too small, or an offset is out of the file.
    OutOfBounds,
    /// A value is unknown
    UnknownElf,
}

type Result<T> = ::core::result::Result<T, Error>;

pub struct Elf<R>
where
    R: Read + Seek,
{
    reader: R,
    header: Elf32Header,
}

impl<R> Elf<R>
where
    R: Read + Seek,
{
    pub fn new(mut reader: R) -> Result<Self> {
        let mut header = read_struct(&mut reader)
            .map_err(|_| Error::NotAnELF)?;

        reader.seek(SeekFrom::Start(0)).ok();

        Elf {
            reader,
            header,
        }
        .validate()
    }

    pub fn program_headers<'a>(&'a mut self) -> impl 'a + Iterator<Item = impl ElfProgramHeader> {
        iterators::ElfProgramHeaderIterator::<'a, R, Elf32ProgramHeader>::new(self)
    }

    fn validate(mut self) -> Result<Self> {
        self.len()
            .and_then(|size| self.header.validate(size))?;

        Ok(self)
    }

    /// Calculate the remaining size of the file from the current position
    fn len(&mut self) -> Result<usize> {
        // Save position
        let current_pos = self
            .reader
            .seek(SeekFrom::Current(0))
            .map_err(|e| Error::Io(e))?;

        // Read size of file
        let size = self
            .reader
            .seek(SeekFrom::End(0))
            .map_err(|e| Error::Io(e))?;

        // Restore position
        self.reader
            .seek(SeekFrom::Current(current_pos as isize))
            .map_err(|e| Error::Io(e))?;

        Ok(size)
    }
}

impl<R> fmt::Debug for Elf<R>
    where R: Read + Seek {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.header.fmt(f)
    }
}

#[repr(C)]
#[derive(Debug)]
struct Elf32Header {
    ident: [u8; EI_NIDENT],
    typ: u16,
    machine: u16,
    version: u32,
    entry: u32,
    phoff: u32,
    shoff: u32,
    flags: u32,
    ehsize: u16,
    phentsize: u16,
    phnum: u16,
    shentsize: u16,
    shnum: u16,
    shstrndx: u16,
}

impl Elf32Header {
    /// len: Size of elf file
    fn validate(&self, len: usize) -> Result<()> {
        if &self.ident[0..4] != ELFMAG {
            return Err(Error::NotAnELF);
        }

        // Must be a valid class
        ElfClass::try_from(self.ident[4])
            // Must be Class32
            .and_then(|class| {
                if class == ElfClass::Class32 {
                    Ok(())
                } else {
                    Err(Error::UnknownElf)
                }
            })
            // Must be a valid encoding
            .and_then(|_| ElfEncoding::try_from(self.ident[5]))
            // Must be version 1
            .and_then(|_| {
                if self.ident[6] == 1 {
                    Ok(())
                } else {
                    Err(Error::UnknownElf)
                }
            })
            // Must be a valid file type
            .and_then(|_| ElfType::try_from(self.typ))?;

        // TODO Validate other fields

        if self.phentsize as usize != size_of::<Elf32ProgramHeader>() {
            return Err(Error::UnknownElf);
        }

        // Program header table must be valid
        if (self.phoff + (self.phnum * self.phentsize) as u32) as usize > len {
            return Err(Error::OutOfBounds);
        }

        // Symbol header table must be valid
        if (self.shoff + (self.shnum * self.shentsize) as u32) as usize > len {
            return Err(Error::OutOfBounds);
        }

        Ok(())
    }
}

pub trait ElfProgramHeader: ::core::fmt::Debug {

}

#[repr(C)]
#[derive(Debug)]
pub struct Elf32ProgramHeader {
    typ: u32,
    offset: u32,
    vaddr: u32,
    paddr: u32,
    filesz: u32,
    memsz: u32,
    flags: u32,
    align: u32,
}

impl ElfProgramHeader for Elf32ProgramHeader {

}

fn read_struct<'a, R, S>(reader: &mut R) -> Result<S>
    where R: Read
{
    let mut strct: S = unsafe { uninitialized() };

    reader
        .read(unsafe {
            slice::from_raw_parts_mut(transmute(&mut strct as *mut S), size_of::<S>())
        })
        .map_err(|io| Error::Io(io))
        .and_then(|size| {
            if size == size_of::<S>() {
                Ok(strct)
            } else {
                Err(Error::OutOfBounds)
            }
        })
}
