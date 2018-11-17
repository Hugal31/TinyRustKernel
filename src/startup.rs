use core::cmp::{max, min};
use core::intrinsics::transmute;
use core::ptr::NonNull;

use elf::ElfSectionHeader;

use crate::interrupts;
use crate::memory;
use crate::multiboot;
use crate::peripherals::speaker::{start_melody, Tone};
use crate::peripherals::vga::{ScreenChar, TEXT_WRITER};
use crate::ALLOCATOR;

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

pub fn startup(infos: &multiboot::MultibootInfo) {
    debug!("Memory segmentation...");
    memory::segment();
    info!("Memory segmentation DONE!");

    debug!("Initialize interrupts...");
    interrupts::init();
    info!("Initialize interrupts DONE!");

    let mut min_memory_addr = infos
        .mmap()
        .filter(|m| m.is_available())
        .map(|m| m.base_addr as usize)
        .min()
        .expect("Should have at least one usable memory section");
    let max_memory_addr = infos
        .mmap()
        .filter(|m| m.is_available())
        .map(|m| (m.base_addr + m.length) as usize)
        .max()
        .expect("Should have at least one usable memory section");

    let kernel_end = infos
        .elf_sections()
        .expect("Elf sections should be readable")
        .map(|s| s.addr() + s.size())
        .max()
        .expect("There should be at least one elf section");

    min_memory_addr = max(min_memory_addr, kernel_end);

    let mut min_memory_addr = min_memory_addr as *mut u8;
    let max_memory_addr = max_memory_addr as *mut u8;

    min_memory_addr = unsafe { min_memory_addr.add(min_memory_addr.align_offset(8)) };

    debug!(
        "Usable memory: [0x{:X?} - 0x{:X?}]",
        min_memory_addr, max_memory_addr
    );
    ALLOCATOR.set_memory_bounds(
        NonNull::new(min_memory_addr).unwrap(),
        NonNull::new(max_memory_addr).unwrap(),
    );

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
