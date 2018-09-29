#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(core_intrinsics)]
#![feature(global_asm)]
#![feature(min_const_fn)]
#![feature(option_replace)]
#![feature(naked_functions)]
#![feature(const_transmute)]
#![feature(lang_items)]
#![no_std]

extern crate bitfield;
extern crate lazy_static;
extern crate spin;
extern crate volatile;
extern crate vga; // TODO Move

mod arch;
mod interrupts;
mod kfs;
mod memory;
mod multiboot;
mod peripherals;

use core::fmt::Write;
use core::intrinsics::transmute;
use core::panic::PanicInfo;

use self::interrupts::init_interrupts;
use self::memory::init_memory;
use self::peripherals::serial::SERIAL_PORT;
use self::peripherals::vga::{ScreenChar, TEXT_WRITER};

// NOTE: Must use to expose
pub use self::interrupts::isr_generic_handler;

static SPLASH_SCREEN: &[ScreenChar; 2000] = unsafe {
    transmute(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/logo-ascii.vga"
    )))
};

use crate::peripherals::speaker::*;

const STARTUP_MELODY: &[Tone] = &[
    Tone::new(659, 400), // Mi 4
    Tone::new(494, 400), // Si 3
    Tone::new(440, 500), // La 3
    Tone::new(659, 330), // Mi 4
    Tone::new(494, 500), // Mi 3
];

#[no_mangle]
pub extern "C" fn k_main(magic: u32, infos: &multiboot::MultibootInfo) -> ! {
    if magic != multiboot::MULTIBOOT_BOOT_MAGIC {
        write_serial!("Wrong multiboot magic\n");
        abort();
    }

    do_system_init_steps();
    say_welcome();

    loop {
        unsafe { asm!("hlt\n\t" :::: "volatile") }
    }
}

fn say_welcome() {
    write_serial!("RedK booting!\n");

    let mut writer = TEXT_WRITER.lock();
    writer.disable_cursor();

    // Display splash screen
    writer.write_raw(SPLASH_SCREEN);

    start_melody(STARTUP_MELODY, false);

    let duration: u32 = STARTUP_MELODY.iter().map(|t| t.duration).sum();
    //peripherals::timer::sleep(duration as usize);
}

fn do_system_init_steps() {
    write_serial!("Init memory...");
    init_memory();
    write_serial!("DONE!\n");

    write_serial!("Init interrupts...");
    init_interrupts();
    write_serial!("DONE!\n");
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
            "panic occured {}, {}:{}",
            location.file(),
            location.line(),
            location.column()
        );
        write_vga!(
            "panic occured {}, {}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    } else {
        write_serial!("panic occured");
        write_vga!("panic occured");
    }
    loop {}
}
