use core::slice;

use no_std_io::SeekFrom;

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
const SYSCALL_OPEN: u32 = 5;
const SYSCALL_READ: u32 = 6;
const SYSCALL_SEEK: u32 = 7;
const SYSCALL_CLOSE: u32 = 8;
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
        SYSCALL_OPEN => syscall_open(unsafe { crate::strings::cstr_to_str_unchecked(context.ebx as *const u8) }, context.ecx),
        SYSCALL_READ => syscall_read(context.ebx, unsafe {
            slice::from_raw_parts_mut(context.ecx as *mut u8, context.edx as usize)
        }),
        SYSCALL_SEEK => syscall_seek(context.ebx, context.ecx as isize, context.edx),
        SYSCALL_CLOSE => syscall_close(context.ebx),
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
    buffer
        .read()
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

fn syscall_open(filename: &str, _flags: u32) -> u32 {
    use alloc::boxed::Box;

    let fs = crate::kfs::get_fs();

    if let Some(inode) = fs.inodes().find(|i| i.filename() == filename) {
        let reader = fs.reader(inode);
        crate::userland::USER_PROCESS.lock().store_file(Box::new(reader))
            .unwrap_or(::core::u32::MAX)
    } else {
        ::core::u32::MAX
    }
}

fn syscall_read(fd: u32, buffer: &mut [u8]) -> u32 {
    let mut process = crate::userland::USER_PROCESS.lock();

    process.get_file(fd)
        .and_then(|file| file.read(buffer).ok())
        .map(|r| r as u32)
        .unwrap_or(::core::u32::MAX)
}

fn syscall_seek(fd: u32, offset: isize, whence: u32) -> u32 {
    if let Ok(seek_from) = parse_seek_from(offset, whence) {
        let mut process = crate::userland::USER_PROCESS.lock();

        if let Some(file) = process.get_file(fd) {
            file.seek(seek_from)
                .map(|u| u as u32)
                .unwrap_or(::core::u32::MAX)
        } else {
            ::core::u32::MAX
        }
    } else {
        ::core::u32::MAX
    }
}

fn parse_seek_from(offset: isize, whence: u32) -> Result<SeekFrom, ()> {
    match whence {
        0 => Ok(SeekFrom::Start(offset as usize)),
        1 => Ok(SeekFrom::Current(offset)),
        2 => Ok(SeekFrom::End(offset)),
        _ => Err(()),
    }
}

fn syscall_close(fd: u32) -> u32 {
    let mut process = crate::userland::USER_PROCESS.lock();

    process.close_file(fd)
        .map(|_| 0)
        .unwrap_or(::core::u32::MAX)
}
