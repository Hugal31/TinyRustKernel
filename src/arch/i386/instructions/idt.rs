use bitfield::*;

use super::lgdt::DPL;

const IDT_INTERRUPT_GATE_32: u8 = 0b01110;

pub type HandlerFunc = unsafe extern "C" fn() -> !;

bitfield! {
    #[derive(Clone, Copy)]
    pub struct IDTEntry(u64);
    impl Debug;

    pub begin_offset, set_begin_offset: 15, 0;
    segment_selector, set_segment_selector: 31, 16;
    interrupt_type, set_interrupt_type: 44, 40;
    dpl, set_dpl: 46, 45;
    present, set_present: 47;
    pub end_offset, set_end_offset: 63, 48;
}

impl IDTEntry {

    fn new(typ: u8,
           handler: HandlerFunc,
           segment: u16,
           dpl: DPL) -> IDTEntry {
        IDTEntry(
            handler as u64 & 0xFFFF
                | (segment as u64) << 16
                | (typ as u64) << 40
                | (dpl as u64) << 45
                | 1 << 47
                | ((handler as u64) >> 16) << 48
        )
    }

    pub fn new_interrupt_gate(handler: HandlerFunc,
                              segment: u16,
                              dpl: DPL) -> IDTEntry {
        IDTEntry::new(IDT_INTERRUPT_GATE_32,
                      handler,
                      segment,
                      dpl)
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct IDTR {
    pub limit: u16,
    pub base: u32,
}

pub fn lidt(idtr: &IDTR) {
    unsafe {
        asm!("lidt ($0)\n\t"
             :
             : "r" (idtr)
             : "memory");
    }
}
