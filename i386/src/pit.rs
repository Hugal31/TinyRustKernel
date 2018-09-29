use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref PIT: Mutex<Pit> = Mutex::new(Pit::new_8254());
}

use super::instructions::Port;

pub struct Pit {
    control: Port<u8>,
    counter_0: Port<u8>,
    counter_2: Port<u8>,
}

impl Pit {
    const FREQUENCY: u32 = 1193182;

    const BINARY_COUNTER: u8 = 0;

    const RATE_GENERATOR_MODE: u8 = 2 << 1;
    const SQUARE_MODE: u8 = 3 << 1;

    const POLICY_LSB_MSB: u8 = 3 << 4;

    const SETUP_COUNTER_0: u8 = 0 << 6;

    const SETUP_COUNTER_2: u8 = 2 << 6;

    fn new_8254() -> Self {
        Pit {
            control: Port::new(0x43),
            counter_0: Port::new(0x40),
            counter_2: Port::new(0x42),
        }
    }

    pub fn set_rate_generator(&mut self, rate: u32) {
        let div: u16 = (Self::FREQUENCY / rate) as u16;
        unsafe {
            self.control.write(
                Pit::BINARY_COUNTER
                    | Pit::RATE_GENERATOR_MODE
                    | Pit::POLICY_LSB_MSB
                    | Pit::SETUP_COUNTER_0,
            );

            self.counter_0.write((div & 0xFF) as u8);
            self.counter_0.write(((div >> 8) & 0xFF) as u8);
        }
    }

    pub fn play_sound(&mut self, frequency: u32) {
        let div: u16 = (Self::FREQUENCY / frequency) as u16;

        unsafe {
            self.control.write(
                Pit::BINARY_COUNTER | Pit::SQUARE_MODE | Pit::POLICY_LSB_MSB | Pit::SETUP_COUNTER_2,
            );
            self.counter_2.write(div as u8);
            self.counter_2.write((div >> 8) as u8);
        }
    }
}
