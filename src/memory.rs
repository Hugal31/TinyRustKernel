use core::mem::{size_of, size_of_val};

use crate::arch::i386::instructions::lgdt;

pub use crate::arch::i386::instructions::lgdt::DPL;

static mut GDT: [lgdt::GDTEntry; 6] = [
    lgdt::GDTEntry(0),
    lgdt::GDTEntry::new_code_segment(0, 0xFFFFF, DPL::Ring0, true),
    lgdt::GDTEntry::new_data_segment(0, 0xFFFFF, DPL::Ring0, true),
    lgdt::GDTEntry::new_code_segment(0, 0xFFFFF, DPL::Ring3, true),
    lgdt::GDTEntry::new_data_segment(0, 0xFFFFF, DPL::Ring3, true),
    lgdt::GDTEntry(0), // Place for TSS
];

static mut TSS: lgdt::TSSEntry = lgdt::TSSEntry::new();

pub const KERNEL_CODE_SEGMENT: usize = size_of::<lgdt::GDTEntry>() * 1;
pub const KERNEL_DATA_SEGMENT: usize = size_of::<lgdt::GDTEntry>() * 2;
pub const USER_CODE_SEGMENT: usize = size_of::<lgdt::GDTEntry>() * 3;
pub const USER_DATA_SEGMENT: usize = size_of::<lgdt::GDTEntry>() * 4;
const TSS_SEGMENT: usize = size_of::<lgdt::GDTEntry>() * 5;

pub fn segment() {
    // TODO Make this clean
    unsafe {
        // Init TSS SS0 and ESP0
        let stack: usize;

        asm!("movl %esp, $0" : "=r" (stack));

        TSS.ss0 = KERNEL_DATA_SEGMENT as u32;
        TSS.esp0 = stack as u32;

        // Init TSS GDT entry
        GDT[5] = lgdt::GDTEntry::new_tss_segment(&TSS as *const lgdt::TSSEntry as u32);
    };

    load_gdt();
    set_protected_mode();
    load_segments(KERNEL_CODE_SEGMENT, KERNEL_DATA_SEGMENT);
    init_tss_register();
}

fn load_segments(code_segment: usize, data_segment: usize) {
    trace!("Load code segment {} and data segment {}", code_segment, data_segment);

    unsafe {
        // TODO Refactor, move architecture-dependent code
        // Update code segment
        asm!("pushl $0
        pushl $$1f
        lret
        1:\n\t"
             :
             : "{eax}" (code_segment as u32)
             :
             : "volatile");

        // Update data segments
        asm!("movw $0, %ds
        movw $0, %es
        movw $0, %fs
        movw $0, %gs
        movw $0, %ss"
             :
             : "{ax}" (data_segment as u16)
             :
             : "volatile");
    }
}

fn load_gdt() {
    trace!("Load GDT");

    let gdtr = lgdt::GDTR {
        limit: size_of_val(unsafe { &GDT }) as u16 - 1,
        base: unsafe { &GDT } as *const _ as u32,
    };

    lgdt::set_lgdt(&gdtr);
}

fn set_protected_mode() {
    trace!("Switch to protected mode");

    lgdt::set_protected_mode()
}

fn init_tss_register() {
    unsafe {
        asm!("ltr $0\n\t"
             :
             : "{ax}" (TSS_SEGMENT as u16)
             :
             : "volatile");
    }
}
