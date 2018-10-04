use core::marker::PhantomData;

use no_std_io::{Read, Seek, SeekFrom};

use super::{Elf, read_struct};

pub(super) struct ElfProgramHeaderIterator<'a, R, P>
    where R: Read + Seek {
    elf: &'a mut Elf<R>,
    /// Current entry number
    entry: u16,
    _marker: PhantomData<P>,
}

impl<'a, R, P> ElfProgramHeaderIterator<'a, R, P>
    where R: Read + Seek {

    pub fn new(elf: &'a mut Elf<R>) -> ElfProgramHeaderIterator<'a, R, P> {
        // TODO Do not unwrap ? Maybe use allocation later.
        elf.reader.seek(SeekFrom::Start(elf.header.phoff as usize)).unwrap();

        ElfProgramHeaderIterator {
            elf,
            entry: 0,
            _marker: PhantomData::default(),
        }
    }
}

impl<'a, R, P> Iterator for ElfProgramHeaderIterator<'a, R, P>
    where R: Read + Seek {

    type Item = P;

    fn next(&mut self) -> Option<P> {
        if self.entry == self.elf.header.phnum {
            return None;
        }

        self.entry += 1;
        Some(read_struct(&mut self.elf.reader).unwrap())
    }
}
