use elf::{Elf, ElfProgramHeader};
use no_std_io::{Read, Seek, SeekFrom};

use crate::memory::{USER_DATA_SEGMENT, DPL};

const USERLAND_ADDR: usize = 0x1000000;

pub fn execute_file<R>(reader: R)
where
    R: Read + Seek + Clone {

    let entry_point = {
        let mut elf = Elf::new(reader.clone()).unwrap();
        load_into_memory(&mut elf, reader, USERLAND_ADDR);

        elf.entry_point()
    };

    unsafe {
        asm!(
            "mov $0, %ax // Init data segment registers
        mov %ax, %ds
        mov %ax, %es
        mov %ax, %fs
        mov %ax, %gs
        // Build iret stack
        push $0         // Push new SS
        push $3         // Push stack, to change
        pushf           // Push flags
        push $1         // Push new CS
        push $2         // Push new EIP
        iret"
             :
             : "i" (0x20 | 0x3u16), // TODO Use (USER_DATA_SEGMENT as u16 | DPL::Ring3 as u16)
               "i" (0x18 | 0x3u16),// TODO Use (USER_CODE_SEGMENT as u16 | DPL::Ring3 as u16)
               "{edx}" (USERLAND_ADDR + entry_point),
               "{ebx}" (USERLAND_ADDR - 0x1000)
             : "a"
             : "volatile")
    };
}

fn load_into_memory<R>(elf: &mut Elf<R>, mut reader: R, base_address: usize)
where
    R: Read + Seek {
    for segment in elf.program_headers() {
        reader.seek(SeekFrom::Start(segment.offset())).unwrap();
        let memory = unsafe {
            ::core::slice::from_raw_parts_mut(
                (base_address + segment.paddr()) as *mut u8,
                segment.file_size())
        };
        reader.read(memory).unwrap();
    }
}
