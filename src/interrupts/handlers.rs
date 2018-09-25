use crate::arch::i386::instructions::Port;

use super::{InterruptContext, send_eoi_to_master};

//pub const HANDLERS: [Option<INTERRUPT_HANDLER>; 255] = [None; 255];

// TODO Pass a pointer
//pub type INTERRUPT_HANDLER = fn (&mut InterruptContext) -> ()

pub fn keyboard_handler(_context: &mut InterruptContext) {
    // TODO Move
    use crate::*;
    let is_full = (unsafe { Port::new(0x64).read() } & 0x1) == 1;
    if is_full {
        let scan = unsafe { Port::new(0x60).read() };
        let is_pressed = (scan & 0b10000000) == 0;
        let key = scan & !0b10000000;
        write_serial!("Key: 0x{:X} is pressed: {}\n", key, is_pressed);
    }
    send_eoi_to_master();
}
