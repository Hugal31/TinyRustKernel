use core::convert::TryFrom;

use super::Error;

type Result<T> = ::core::result::Result<T, Error>;

const ELFCLASS32: u8 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElfClass {
    Class32,
}

impl TryFrom<u8> for ElfClass {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            ELFCLASS32 => Ok(ElfClass::Class32),
            _ => Err(Error::UnknownElf),
        }
    }
}

pub enum ElfEncoding {
    Lsb,
}

impl TryFrom<u8> for ElfEncoding {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            1 => Ok(ElfEncoding::Lsb),
            _ => Err(Error::UnknownElf),
        }
    }
}

pub enum ElfType {
    None,
    Rel,
    Exec,
    Dyn,
    Core,
    Num,
}

impl TryFrom<u16> for ElfType {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self> {
        match value {
            0 => Ok(ElfType::None),
            1 => Ok(ElfType::Rel),
            2 => Ok(ElfType::Exec),
            3 => Ok(ElfType::Dyn),
            4 => Ok(ElfType::Core),
            5 => Ok(ElfType::Num),
            _ => Err(Error::UnknownElf),
        }
    }
}
