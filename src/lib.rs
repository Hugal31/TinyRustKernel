#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(core_intrinsics)]
#![feature(global_asm)]
#![feature(naked_functions)]
#![feature(const_transmute)]
#![feature(lang_items)]
#![no_std]

extern crate bitfield;
extern crate elf;
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate no_std_io;
extern crate spin;
extern crate vga;
extern crate volatile; // TODO Move

mod arch;
mod interrupts;
mod kfs;
mod memory;
mod multiboot;
mod logger;
mod peripherals;
mod startup;

use core::fmt::Write;
use core::panic::PanicInfo;

use self::peripherals::serial::SERIAL_PORT;

// NOTE: Must use to expose
pub use self::interrupts::isr_generic_handler;

const USERLAND_ADDR: usize = 0x10000;
use elf::ElfProgramHeader;

#[no_mangle]
pub extern "C" fn k_main(magic: u32, infos: &multiboot::MultibootInfo) -> ! {
    logger::init();

    if magic != multiboot::MULTIBOOT_BOOT_MAGIC {
        error!("Wrong multiboot magic\n");
        abort();
    }

    startup::startup();

    let kmod = &infos.mods().unwrap()[0];
    let executable = infos.cmdline().and_then(|cmdline| {
        if cmdline.starts_with('/') {
            Some(&cmdline[1..])
        } else {
            None
        }
    }).unwrap();
    let fs = init_file_system(&kmod).unwrap();

    if let Some(inode) = fs.inodes().find(|i| i.filename() == executable) {
        use no_std_io::{Read, Seek, SeekFrom};

        let mut reader = fs.reader(inode);
        let mut elf = elf::Elf::new(reader.clone()).unwrap();

        for segment in elf.program_headers() {
            reader.seek(SeekFrom::Start(segment.offset())).unwrap();
            let memory = unsafe {
                ::core::slice::from_raw_parts_mut(
                    (USERLAND_ADDR + segment.paddr()) as *mut u8,
                    segment.file_size())
            };
            reader.read(memory).unwrap();
        }

        unsafe {
            asm!("nop
        jmp $0"
                 :
                 : "r" (USERLAND_ADDR + elf.entry_point())
                 :
                 : "intel")
        };
    }

    loop {
        unsafe { asm!("hlt\n\t" :::: "volatile") }
    }
}

fn init_file_system(m: &multiboot::MultibootMod) -> Result<kfs::Kfs, kfs::Error> {
    kfs::Kfs::new(m.mod_start, m.mod_end)
}

fn abort() -> ! {
    loop {
        unsafe { asm!("hlt\n\t" :::: "volatile") };
    }
}

// Languages elements

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        write_serial!(
            "[ERROR] panic occured {}, {}:{}",
            location.file(),
            location.line(),
            location.column()
        );
        write_vga!(
            "[ERROR] panic occured {}, {}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    } else {
        write_serial!("[ERROR] panic occured");
        write_vga!("[ERROR] panic occured");
    }
    if let Some(s) = info.payload().downcast_ref::<&str>() {
        write_serial!(", \"{}\"", s);
        write_vga!(", \"{}\"", s);
    }
    abort();
}
