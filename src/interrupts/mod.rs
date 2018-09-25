mod handlers;

use core::ops::Deref;
use core::mem::size_of_val;

use lazy_static::lazy_static;

use super::memory::KERNEL_CODE_SEGMENT;
use crate::arch::i386::instructions::lgdt::DPL;
use crate::arch::i386::instructions::idt::{IDTEntry, IDTR, lidt};

// TODO Use #[naked] and asm! when stabilized
extern "C" {
    fn isr_0() -> !;
    fn isr_65() -> !;
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct InterruptContext {
    edi: u32,
    esi: u32,
    ebp: u32,
    esp: u32,
    ebx: u32,
    edx: u32,
    ecx: u32,
    eax: u32,
    interrupt_number: u32,
    error_code: u32,
}

#[no_mangle]
pub extern "C" fn isr_generic_handler(context: &mut InterruptContext) {
    use crate::*;
    write_serial!("Context: {:#?}", context);
    if context.interrupt_number == 65 {
        handlers::keyboard_handler(context);
    }
}

lazy_static! {
    static ref IDT: [IDTEntry; 255] = {
        let mut idt = [IDTEntry(0); 255];

        idt[0] = IDTEntry::new_interrupt_gate(isr_0,
                                              KERNEL_CODE_SEGMENT,
                                              DPL::Ring0);
        idt[65] = IDTEntry::new_interrupt_gate(isr_65,
                                               KERNEL_CODE_SEGMENT,
                                               DPL::Ring0);

        idt
    };
}

pub fn init_interrupts() {
    load_idt();

    init_pic();

    enable();
}

fn load_idt() {
    let idtr = IDTR {
        limit: size_of_val(IDT.deref()) as u16 - 1,
        base: IDT.deref() as *const _ as u32,
    };

    lidt(&idtr);
}

// TODO Move
use crate::arch::i386::instructions::Port;

fn init_pic() {
    let mut master_a = Port::new(0x20);
    let mut master_b = Port::new(0x21);
    let mut slave_a = Port::new(0xA0);
    let mut slave_b = Port::new(0xA1);

    unsafe {
        // ICW1
        master_a.write(0x11);
        slave_a.write(0x11);

        // ICW2
        master_b.write(0x40);
        slave_b.write(0x50);

        // ICW3
        master_b.write(0b10);
        slave_b.write(0x2);

        // ICW4
        master_b.write(1);
        slave_b.write(1);

        // Mask all interrupts except keyboard
        master_b.write(0xFD);
    }
}

pub fn send_eoi_to_master() {
    unsafe { Port::new(0x20).write(0x20) }
}

fn enable() {
    unsafe { asm!("sti" :::: "volatile") }
}
