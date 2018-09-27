#![allow(dead_code)]

use crate::arch::i386::instructions::Port;
use crate::arch::i386::pit::PIT;

pub fn play_frequency(frequency: u32) {
    PIT.lock().play_sound(frequency);
}

pub fn enable() {
    unsafe {
        // FIXME: This is architecture dependent and should be in arch module
        let mut port = Port::<u8>::new(0x61);
        let tmp = port.read();
        port.write(tmp | 3);
    }
}

pub fn disable() {
    unsafe {
        // FIXME: This is architecture dependent and should be in arch module
        let mut port = Port::<u8>::new(0x61);
        let tmp = port.read();
        port.write(tmp & !3);
    }
}
