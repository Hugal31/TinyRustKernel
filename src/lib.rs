#![feature(asm)]
#![feature(min_const_fn)]
#![feature(const_transmute)]
#![feature(lang_items)]
#![no_std]

extern crate bitfield;
extern crate lazy_static;
extern crate spin;
extern crate volatile;

mod arch;
mod memory;
mod peripherals;

use core::fmt::Write;
use core::intrinsics::transmute;
use core::panic::PanicInfo;

use self::memory::init_memory;
use self::peripherals::serial::SERIAL_PORT;
use self::peripherals::vga::{ScreenChar, TEXT_WRITER};

static SPLASH_SCREEN: &[ScreenChar; 2000] = unsafe {
    transmute(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/logo-ascii.vga"
    )))
};

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn k_main() {
    say_welcome();

    write_serial!("Init memory...");
    init_memory();
    write_serial!("DONE!\n");

    // End
    unsafe { asm!("hlt\n\t" :::: "volatile") };
    loop {}
}

fn say_welcome() {
    let mut writer = TEXT_WRITER.lock();
    writer.disable_cursor();

    // Display splash screen
    writer.write_raw(SPLASH_SCREEN);

    write!(SERIAL_PORT.lock(), "RedK booting!\n");
}
