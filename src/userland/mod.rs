use elf::{Elf, ElfProgramHeader};
use no_std_io::{Read, Seek, SeekFrom};

const USERLAND_ADDR: usize = 0x10000;

pub fn execute_file<R>(mut reader: R)
where
    R: Read + Seek + Clone {

    let mut elf = Elf::new(reader.clone()).unwrap();
    for segment in elf.program_headers() {
        reader.seek(SeekFrom::Start(segment.offset())).unwrap();
        let memory = unsafe {
            ::core::slice::from_raw_parts_mut(
                (USERLAND_ADDR + segment.paddr()) as *mut u8,
                segment.file_size())
        };
        reader.read(memory).unwrap();
    }

    unsafe {
        asm!("nop
        jmp $0"
             :
             : "r" (USERLAND_ADDR + elf.entry_point())
             :
             : "intel")
    };
}
