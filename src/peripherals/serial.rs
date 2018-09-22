use core::fmt;

use bitfield::*;
use lazy_static::lazy_static;
use spin::Mutex;

use crate::arch::i386::instructions::Port;

const COM1: u16 = 0x3F8;

lazy_static! {
    pub static ref SERIAL_PORT: Mutex<SerialPort> = Mutex::new(SerialPort::default());
}

#[macro_export]
macro_rules! write_serial {
    ($($arg:tt)*) => { write!(SERIAL_PORT.lock(), $($arg)*) }
}

pub struct SerialPort {
    rbr_thr_and_dll: Port,
    ier_and_dlm: Port,
    iir_and_fcr: Port,
    lcr: Port,
}

// TODO Implement read
impl SerialPort {
    fn new_uart_16550(base: u16) -> SerialPort {
        let mut serial_port = SerialPort {
            rbr_thr_and_dll: Port::new(base),
            ier_and_dlm: Port::new(base + 1),
            iir_and_fcr: Port::new(base + 2),
            lcr: Port::new(base + 3),
        };

        serial_port.init_uart_16550();

        serial_port
    }

    fn init_uart_16550(&mut self) {
        let mut lcr = LCR(0);
        lcr.set_data_word_length(3);
        lcr.set_dlab(true);

        let mut fcr = FCR(0);
        fcr.set_fifo(true);
        fcr.set_clear_receive(true);
        fcr.set_clear_transmit(true);
        fcr.set_interrupt_trigger_level(2);

        let mut ier = IER(0);
        ier.set_transmit_holding_register_empty(true);

        unsafe {
            self.lcr.write(lcr.0);
            // Set speed
            self.rbr_thr_and_dll.write(0x03);
            self.ier_and_dlm.write(0x00);

            lcr.set_dlab(false);
            self.lcr.write(lcr.0);

            self.iir_and_fcr.write(fcr.0);
            self.ier_and_dlm.write(ier.0);
        }
    }

    #[inline]
    pub fn write_byte(&mut self, byte: u8) {
        // TODO: Wait for ack ?
        unsafe { self.rbr_thr_and_dll.write(byte) };
    }
}

impl Default for SerialPort {
    fn default() -> SerialPort {
        SerialPort::new_uart_16550(COM1)
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.as_bytes().iter() {
            self.write_byte(*b)
        }

        Ok(())
    }
}

bitfield! {
    struct LCR(u8);
    impl Debug;

    data_word_length, set_data_word_length: 1, 0;
    stop, set_stop: 2;
    parity, set_parity: 5, 3;
    brk, set_brk: 6;
    dlab, set_dlab: 7;
}

bitfield! {
    struct FCR(u8);
    impl Debug;

    fifo, set_fifo: 0;
    clear_receive, set_clear_receive: 1;
    clear_transmit, set_clear_transmit: 2;
    dma_mode, set_dma_mode: 3;
    // _reserved: 4;
    enable_64_fifo, set_enable_64_fifo: 5;
    interrupt_trigger_level, set_interrupt_trigger_level: 7, 6;
}

bitfield! {
    struct IER(u8);
    impl Debug;

    receive_data_available, set_receive_data_available: 0;
    transmit_holding_register_empty, set_transmit_holding_register_empty: 1;
    receiver_line_status_register_change, set_receiver_line_status_register_change: 2;
    modem_status_register_change, set_modem_status_register_change: 3;
    sleep_mode, set_sleep_mode: 4;
    low_power_mode, set_low_power_mode: 5;
}
