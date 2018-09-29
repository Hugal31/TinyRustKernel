#![allow(dead_code)]

use lazy_static::lazy_static;
use spin::Mutex;

use super::instructions::Port;

lazy_static! {
    pub static ref PIC: Mutex<Pic> = Mutex::new(Pic::new_8254());
}

pub struct Pic {
    master_a: Port<u8>,
    master_b: Port<u8>,
    slave_a: Port<u8>,
    slave_b: Port<u8>,
}

impl Pic {
    fn new_8254() -> Self {
        Pic {
            master_a: Port::new(0x20),
            master_b: Port::new(0x21),
            slave_a: Port::new(0xA0),
            slave_b: Port::new(0xA1),
        }
    }

    pub fn init(&mut self) {
        unsafe {
            // ICW1
            self.master_a.write(0x11);
            self.slave_a.write(0x11);

            // ICW2
            self.master_b.write(0x40);
            self.slave_b.write(0x50);

            // ICW3
            self.master_b.write(0b10);
            self.slave_b.write(0x2); // FIXME Or 0x1 ?

            // ICW4
            self.master_b.write(1);
            self.slave_b.write(1);

            // Mask all interrupts except keyboard and PIT
            self.master_b.write(0b11111100);
        }
    }

    pub fn send_eoi_to_master(&mut self) {
        unsafe { self.master_a.write(0x20) }
    }

    pub fn send_eoi(&mut self) {
        self.send_eoi_to_master();
        unsafe { self.slave_a.write(0x20) }
    }
}
