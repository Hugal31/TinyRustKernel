use crate::arch::i386::instructions::Port;
use crate::arch::i386::pic::PIC;
use crate::peripherals::keyboard;
use crate::peripherals::timer;

use super::InterruptContext;

pub fn pit_handler(_context: &mut InterruptContext) {
    timer::tick();
    PIC.lock().send_eoi_to_master();
}

pub fn keyboard_handler(_context: &mut InterruptContext) {
    let is_full = (unsafe { Port::<u8>::new(0x64).read() } & 0x1) == 1;
    if is_full {
        let scan = unsafe { Port::new(0x60).read() };
        keyboard::receive_scan(scan);
    }
    PIC.lock().send_eoi_to_master();
}

// TODO Use bingen ?
const SYSCALL_GETTICK: u32 = 4;

#[allow(safe_packed_borrows)]
pub fn syscall_handler(context: &mut InterruptContext) {
    use crate::peripherals::timer::uptime;
    let ret = match context.eax {
        SYSCALL_GETTICK => uptime() as u32,
        _ => ::core::u32::MAX,
    };

    context.eax = ret;
}
