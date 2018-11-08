use crate::arch::i386::instructions::Port;
use crate::arch::i386::pic::PIC;
use crate::peripherals::keyboard;
use crate::peripherals::speaker;
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
const SYSCALL_GETKEY: u32 = 3;
const SYSCALL_GETTICK: u32 = 4;
const SYSCALL_PLAYSOUND: u32 = 11;

// TODO Check pointers come from userland, and copy them ?
#[allow(safe_packed_borrows)]
pub fn syscall_handler(context: &mut InterruptContext) {
    trace!("Received syscall {} ({:X?})", context.eax, context);
    let ret = match context.eax {
        SYSCALL_WRITE => syscall_write(context.ebx as *const u8, context.ecx as usize),
        SYSCALL_GETKEY => syscall_getkey(),
        SYSCALL_GETTICK => syscall_gettick(),
        SYSCALL_PLAYSOUND => {
            syscall_playsound(context.ebx as *const speaker::Tone, context.ecx != 0)
        }
        _ => ::core::u32::MAX,
    };

    trace!("Sycall returned {}", ret);

    context.eax = ret;
}

fn syscall_write(buffer: *const u8, size: usize) -> u32 {
    use crate::peripherals::serial::SERIAL_PORT;
    use crate::peripherals::vga::TEXT_WRITER;

    trace!("write(0x{:X?}, {})", buffer, size);

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

fn syscall_getkey() -> u32 {
    use crate::peripherals::keyboard::BUFFER;

    let mut buffer = BUFFER.lock();
    buffer.read()
        .map(|scan| scan as u32)
        .unwrap_or(::core::u32::MAX)
}

fn syscall_gettick() -> u32 {
    use crate::peripherals::timer::uptime;
    uptime() as u32
}

fn syscall_playsound(melody: *const speaker::Tone, repeat: bool) -> u32 {
    speaker::start_melody_from(melody, repeat);

    0
}
