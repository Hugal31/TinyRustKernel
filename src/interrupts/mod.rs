mod handlers;

use core::ops::Deref;
use core::mem::size_of_val;

use lazy_static::lazy_static;

use super::memory::KERNEL_CODE_SEGMENT;
use crate::arch::i386::instructions::lgdt::DPL;
use crate::arch::i386::instructions::idt::{IDTEntry, IDTR, lidt};
use crate::arch::i386::pic::PIC;
use crate::arch::i386::pit::PIT;

// TODO Use #[naked] and asm!
extern "C" {
    fn isr_0() -> !;
    fn isr_64() -> !;
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
    match context.interrupt_number {
        64 => handlers::pit_handler(context),
        65 => handlers::keyboard_handler(context),
        _ => (),
    }
}

lazy_static! {
    static ref IDT: [IDTEntry; 255] = {
        let mut idt = [IDTEntry(0); 255];

        idt[0] = IDTEntry::new_interrupt_gate(isr_0,
                                              KERNEL_CODE_SEGMENT,
                                              DPL::Ring0);
        idt[64] = IDTEntry::new_interrupt_gate(isr_64,
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

    PIC.lock().init();
    unsafe { PIT.lock().init_rate_generator() };

    enable();
}

fn load_idt() {
    let idtr = IDTR {
        limit: size_of_val(IDT.deref()) as u16 - 1,
        base: IDT.deref() as *const _ as u32,
    };

    lidt(&idtr);
}

fn enable() {
    unsafe { asm!("sti" :::: "volatile") }
}
