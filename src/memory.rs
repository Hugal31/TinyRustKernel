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

pub const KERNEL_CODE_SEGMENT: usize = (size_of::<lgdt::GDTEntry>() * 1);
pub const KERNEL_DATA_SEGMENT: usize = (size_of::<lgdt::GDTEntry>() * 2);
//pub const USER_CODE_SEGMENT: usize = (size_of::<lgdt::GDTEntry>() * 3);
//pub const USER_DATA_SEGMENT: usize = (size_of::<lgdt::GDTEntry>() * 4);
const TSS_SEGMENT: usize = (size_of::<lgdt::GDTEntry>() * 5);

pub fn init_memory() {
    load_gdt();
    set_protected_mode();
    init_segment_registers();
}

fn load_gdt() {
    let gdtr = lgdt::GDTR {
        limit: size_of_val(&GDT) as u16 - 1,
        base: &GDT as *const _ as u32,
    };

    lgdt::set_lgdt(&gdtr);
}

fn set_protected_mode() {
    lgdt::set_protected_mode()
}

fn init_segment_registers() {
    unsafe {
        // Update code segment
        asm!("ljmp $$0x8, $$1f
        1:\n\t"
             :
             :
             :
             : "volatile");

        // Update data segments
        asm!("movw $0, %ds
        movw $0, %es
        movw $0, %fs
        movw $0, %gs
        movw $0, %ss"
             :
             : "{ax}" (KERNEL_DATA_SEGMENT as u16)
             :
             : "volatile");

        // Update TSS
        asm!("ltr $0\n\t"
             :
             : "{ax}" (TSS_SEGMENT as u16)
             :
             : "volatile");
    }
}
