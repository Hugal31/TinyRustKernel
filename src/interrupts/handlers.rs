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
const SYSCALL_WRITE: u32 = 1;
const SYSCALL_GETTICK: u32 = 4;

#[allow(safe_packed_borrows)]
pub fn syscall_handler(context: &mut InterruptContext) {
    let ret = match context.eax {
        SYSCALL_WRITE => syscall_write(context.ebx as *const u8, context.ecx as usize),
        SYSCALL_GETTICK => syscall_gettick(),
        _ => ::core::u32::MAX,
    };

    context.eax = ret;
}

fn syscall_write(buffer: *const u8, size: usize) -> u32 {
    use crate::peripherals::serial::SERIAL_PORT;
    use crate::peripherals::vga::TEXT_WRITER;

    let mut serial = SERIAL_PORT.lock();
    let mut vga = TEXT_WRITER.lock();
    let mut c = 0;
    while c < size {
        let byte = unsafe { *buffer.add(c) };
        serial.write_byte(byte);
        vga.write_byte(byte);
        c += 1;
    }

    c as u32
}

fn syscall_gettick() -> u32 {
    use crate::peripherals::timer::uptime;
    uptime() as u32
}
