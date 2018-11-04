use core::mem::{size_of, size_of_val};

use crate::arch::i386::instructions::lgdt;

pub static GDT: [lgdt::GDTEntry; 6] = [
    lgdt::GDTEntry(0),
    lgdt::GDTEntry::new_code_segment(0, 0xFFFFF, lgdt::DPL::Ring0, true),
    lgdt::GDTEntry::new_data_segment(0, 0xFFFFF, lgdt::DPL::Ring0, true),
    lgdt::GDTEntry::new_code_segment(0, 0xFFFFF, lgdt::DPL::Ring3, true),
    lgdt::GDTEntry::new_data_segment(0, 0xFFFFF, lgdt::DPL::Ring3, true),
    lgdt::GDTEntry::new_tss_segment(),
];

pub const KERNEL_CODE_SEGMENT: usize = size_of::<lgdt::GDTEntry>() * 1;
pub const KERNEL_DATA_SEGMENT: usize = size_of::<lgdt::GDTEntry>() * 2;
pub const USER_CODE_SEGMENT: usize = size_of::<lgdt::GDTEntry>() * 3;
pub const USER_DATA_SEGMENT: usize = size_of::<lgdt::GDTEntry>() * 4;
const TSS_SEGMENT: usize = size_of::<lgdt::GDTEntry>() * 5;

pub fn segment() {
    load_gdt();
    set_protected_mode();
    switch_to_kernel_segments();
    init_tss_register();
}

pub fn switch_to_kernel_segments() {
    unsafe { load_segments(KERNEL_CODE_SEGMENT, KERNEL_DATA_SEGMENT) };
}

pub fn switch_to_userland_segments() {
    unsafe { load_segments(USER_CODE_SEGMENT, USER_DATA_SEGMENT) };
}

unsafe fn load_segments(code_segment: usize, data_segment: usize) {
    trace!("Load code segment {} and data segment {}", code_segment, data_segment);

    debug_assert!(code_segment % size_of::<lgdt::GDTEntry>() == 0, "Invalid code segment");
    debug_assert!(code_segment < size_of_val(&GDT), "Out of range code segment");
    debug_assert!(data_segment % size_of::<lgdt::GDTEntry>() == 0, "Invalid data segment");
    debug_assert!(data_segment < size_of_val(&GDT), "Out of range data segment");

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

fn load_gdt() {
    trace!("Load GDT");

    let gdtr = lgdt::GDTR {
        limit: size_of_val(&GDT) as u16 - 1,
        base: &GDT as *const _ as u32,
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
