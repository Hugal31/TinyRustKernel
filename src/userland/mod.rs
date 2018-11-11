mod process;

pub use self::process::*;

use core::alloc::{GlobalAlloc, Layout};

use elf::{Elf, ElfProgramHeader};
use no_std_io::{Read, Seek, SeekFrom};

use crate::ALLOCATOR;
use crate::memory::{USER_DATA_SEGMENT, DPL};

pub fn execute_file<R>(reader: R)
where
    R: Read + Seek + Clone {

    let (entry_point, base_address) = {
        let mut elf = Elf::new(reader.clone()).unwrap();
        let base_address = load_into_memory(&mut elf, reader);

        (elf.entry_point(), base_address)
    };

    // Allocate stack
    let stack_addr = unsafe { ALLOCATOR.alloc(Layout::from_size_align_unchecked(0x20000, 8)) };

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
               "{edx}" (base_address.add(entry_point)),
               "{ebx}" (stack_addr.sub(0x1FFF8))
             : "a"
             : "volatile")
    };
}

fn load_into_memory<R>(elf: &mut Elf<R>, mut reader: R) -> *mut u8
where
    R: Read + Seek {

    let max_align = elf.program_headers().map(|h| h.align())
        .max()
        .expect("There should be at least one section");
    let end_addr = elf.program_headers().map(|h| h.vaddr() + h.mem_size())
        .max()
        .expect("There should be at least one section");

    let base_address = unsafe {
        ALLOCATOR.alloc(Layout::from_size_align(end_addr, max_align)
                        .expect("Invalid userland layout"))
    };

    for segment in elf.program_headers() {
        reader.seek(SeekFrom::Start(segment.offset())).unwrap();
        let memory = unsafe {
            ::core::slice::from_raw_parts_mut(
                base_address.add(segment.vaddr()),
                segment.file_size())
        };
        reader.read(memory).unwrap();
    }

    base_address
}
