#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(core_intrinsics)]
#![feature(global_asm)]
#![feature(min_const_fn)]
#![feature(naked_functions)]
#![feature(const_transmute)]
#![feature(lang_items)]
#![no_std]

extern crate bitfield;
extern crate lazy_static;
extern crate spin;
extern crate volatile;

mod arch;
mod interrupts;
mod memory;
mod peripherals;

use core::fmt::Write;
use core::intrinsics::transmute;
use core::panic::PanicInfo;

use self::interrupts::init_interrupts;
use self::memory::init_memory;
use self::peripherals::serial::SERIAL_PORT;
use self::peripherals::vga::{ScreenChar, TEXT_WRITER};

// Must use to expose
pub use self::interrupts::isr_generic_handler;

static SPLASH_SCREEN: &[ScreenChar; 2000] = unsafe {
    transmute(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/logo-ascii.vga"
    )))
};

#[no_mangle]
pub extern "C" fn k_main() {
    say_welcome();

    write_serial!("Init memory...");
    init_memory();
    write_serial!("DONE!\n");

    write_serial!("Init interrupts...");
    init_interrupts();
    write_serial!("DONE!\n");

    unsafe { asm!("movl $$42, %eax\n\tint $$0" ::: "eax" : "volatile"); }

    write_serial!("Tada!");

    // End
    unsafe { asm!("hlt\n\t" :::: "volatile") };
    loop {}
}

fn say_welcome() {
    let mut writer = TEXT_WRITER.lock();
    writer.disable_cursor();

    // Display splash screen
    writer.write_raw(SPLASH_SCREEN);

    write_serial!("RedK booting!\n");
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        write_serial!("panic occured {}, {}:{}", location.file(), location.line(), location.column());
        write_vga!("panic occured {}, {}:{}", location.file(), location.line(), location.column());
    } else {
        write_serial!("panic occured");
        write_vga!("panic occured");
    }
    loop {}
}
