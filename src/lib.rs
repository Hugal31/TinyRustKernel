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
mod userland;

use core::fmt::Write;
use core::panic::PanicInfo;

use self::peripherals::serial::SERIAL_PORT;

// NOTE: Must use to expose
pub use self::interrupts::isr_generic_handler;

#[no_mangle]
pub extern "C" fn k_main(magic: u32, infos: &multiboot::MultibootInfo) -> ! {
    logger::init();

    if magic != multiboot::MULTIBOOT_BOOT_MAGIC {
        error!("Wrong multiboot magic\n");
        abort();
    }

    startup::startup();

    if let Some(module) = infos.mods().next() {
        load_and_execute_module(infos, &module);
    } else {
        warn!("No module detected");
    }

    info!("Shutdown");
    loop {
        unsafe { asm!("hlt\n\t" :::: "volatile") }
    }
}

fn init_file_system(m: &multiboot::MultibootMod) -> Result<kfs::Kfs, kfs::Error> {
    kfs::Kfs::new(m.mod_start, m.mod_end)
}

fn load_and_execute_module(infos: &multiboot::MultibootInfo, m: &multiboot::MultibootMod) {
    // TODO Refactor
    let executable = match infos.cmdline() {
        Some(cmdline) if cmdline.starts_with('/') => &cmdline[1..],
        Some(_) => {
            warn!("The command line argument doesn't start with a '/'");
            return;
        },
        None => {
            warn!("No command line argument");
            return;
        }
    };

    let fs = init_file_system(&m).unwrap();

    if let Some(inode) = fs.inodes().find(|i| i.filename() == executable) {
        let reader = fs.reader(inode);
        userland::execute_file(reader);
    };
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
