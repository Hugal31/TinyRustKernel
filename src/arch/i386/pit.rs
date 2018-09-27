use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref PIT: Mutex<Pit> = Mutex::new(Pit::new_8254());
}

use super::instructions::Port;

pub struct Pit {
    control: Port<u8>,
    counter_0: Port<u8>,
}

impl Pit {
    const BINARY_COUNTER: u8 = 0;

    const RATE_GENERATOR_MODE: u8 = 2 << 1;

    const POLICY_LSB_MSB: u8 = 3 << 4;

    const SETUP_COUNTER_0: u8 = 0 << 6;

    fn new_8254() -> Self {
        Pit {
            control: Port::new(0x43),
            counter_0: Port::new(0x40),
        }
    }

    pub unsafe fn init_rate_generator(&mut self) {
        self.control.write(
            Pit::BINARY_COUNTER
                | Pit::RATE_GENERATOR_MODE
                | Pit::POLICY_LSB_MSB
                | Pit::SETUP_COUNTER_0
        );
        self.counter_0.write((11931 & 0xFF) as u8);
        self.counter_0.write(((11931 >> 8) & 0xFF) as u8);
    }
}
