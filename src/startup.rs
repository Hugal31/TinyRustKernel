use core::intrinsics::transmute;

use crate::interrupts;
use crate::memory;
use crate::peripherals::speaker::{start_melody, Tone};
use crate::peripherals::vga::{ScreenChar, TEXT_WRITER};

static SPLASH_SCREEN: &[ScreenChar; 2000] = unsafe {
    transmute(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/logo-ascii.vga"
    )))
};

const STARTUP_MELODY: &[Tone] = &[
    Tone::new(659, 400), // Mi 4
    Tone::new(494, 400), // Si 3
    Tone::new(440, 500), // La 3
    Tone::new(659, 330), // Mi 4
    Tone::new(494, 500), // Mi 3
];

pub fn startup() {
    debug!("Memory segmentation...");
    memory::segment();
    info!("Memory segmentation DONE!");

    debug!("Initialize interrupts...");
    interrupts::init();
    info!("Initialize interrupts DONE!");

    say_welcome();
}

fn say_welcome() {
    info!("RedK booting!");

    let mut writer = TEXT_WRITER.lock();
    writer.disable_cursor();

    // Display splash screen
    writer.write_raw(SPLASH_SCREEN);

    start_melody(STARTUP_MELODY, false);

    //let duration: u32 = STARTUP_MELODY.iter().map(|t| t.duration).sum();
    //peripherals::timer::sleep(duration as usize);
}
