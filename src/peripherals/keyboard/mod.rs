#![allow(dead_code)]

mod qwerty;

use spin::Mutex;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Key {
    Esc,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Backspace,
    Alt,
    Super,
    Home,
    PgUp,
    PgDown,
    End,
    Tab,
    LShift,
    RShift,
    Enter,
    Space,
    Ctrl,
    Suppr,
    Up,
    Down,
    Left,
    Right,
    Char(u8),
}

impl Key {
    fn from_scan(scan: u8, shift: bool) -> Option<Key> {
        if shift {
            qwerty::SHIFT_LOOKUP_TABLE[(scan & 0b01111111) as usize]
        } else {
            qwerty::LOOKUP_TABLE[(scan & 0b01111111) as usize]
        }
    }

    fn into_char(self) -> Option<char> {
        match self {
            Key::Enter => Some('\n'),
            Key::Space => Some(' '),
            Key::Char(c) => Some(c as char),
            _ => None,
        }
    }
}

#[derive(Default)]
struct Modifiers {
    l_shift: bool,
    r_shift: bool,
}

impl Modifiers {
    const fn new() -> Self {
        Modifiers {
            l_shift: false,
            r_shift: false,
        }
    }

    fn shift(&self) -> bool {
        self.l_shift || self.r_shift
    }

    fn receive_scan(&mut self, scan: u8) {
        let is_pressed = (scan & 0b10000000) == 0;

        match Key::from_scan(scan, false) {
            Some(Key::LShift) => self.l_shift = is_pressed,
            Some(Key::RShift) => self.r_shift = is_pressed,
            _ => (),
        }
    }
}

static MODIFIERS: Mutex<Modifiers> = Mutex::new(Modifiers::new());

/// Circular buffer for scan codes
pub struct ScanBuffer {
    read: u8,
    write: u8,
    overlap: bool,
    buffer: [u8; ScanBuffer::SIZE],
}

impl ScanBuffer {
    const SIZE: usize = 16;

    pub const fn new() -> Self {
        ScanBuffer {
            read: 0,
            write: 0,
            overlap: false,
            buffer: [0; ScanBuffer::SIZE],
        }
    }

    pub fn is_full(&self) -> bool {
        self.overlap
    }

    pub fn is_empty(&self) -> bool {
        !self.overlap && self.read == self.write
    }

    // TODO Only write when pressed ?
    /// Write the scan into the buffer, or return false if the buffer is full
    pub fn write(&mut self, scan: u8) -> bool {
        if self.is_full() {
            false
        } else {
            self.buffer[self.write as usize] = scan;
            self.write = (self.write + 1) % Self::SIZE as u8;
            self.overlap = self.write == self.read;
            true
        }
    }

    pub fn read(&mut self) -> Option<u8> {
        if self.is_empty() {
            None
        } else {
            let scan = self.buffer[self.read as usize];
            self.read = (self.read + 1) % Self::SIZE as u8;
            Some(scan)
        }
    }
}

pub static BUFFER: Mutex<ScanBuffer> = Mutex::new(ScanBuffer::new());

pub fn receive_scan(scan: u8) {
    let mut buffer = BUFFER.lock();
    buffer.write(scan);
}
