#![macro_use]

use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

use crate::arch::i386::instructions::Port;

// Mostly from https://os.phil-opp.com/vga-text-mode/

lazy_static! {
    pub static ref TEXT_WRITER: Mutex<TextWriter> = Mutex::new(TextWriter {
        column_position: 0,
        row_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut TextBuffer) },
    });
}

#[macro_export]
macro_rules! write_vga {
    ($($arg:tt)*) => { write!(TEXT_WRITER.lock(), $($arg)*) }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorCode(u8);

impl ColorCode {
    pub const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const TEXT_BUFFER_HEIGHT: usize = 25;
const TEXT_BUFFER_WIDTH: usize = 80;

struct TextBuffer {
    chars: [[Volatile<ScreenChar>; TEXT_BUFFER_WIDTH]; TEXT_BUFFER_HEIGHT],
}

pub struct TextWriter {
    column_position: usize,
    row_position: usize,
    color_code: ColorCode,
    buffer: &'static mut TextBuffer,
}

#[allow(dead_code)]
impl TextWriter {
    const CLEAR_CHAR: ScreenChar = ScreenChar {
        ascii_character: b'\x00',
        color_code: ColorCode::new(Color::White, Color::Black),
    };

    pub fn set_cursor_position(&mut self, row: usize, column: usize) {
        assert!(row < TEXT_BUFFER_HEIGHT);
        assert!(column < TEXT_BUFFER_WIDTH);

        self.column_position = column;
        self.row_position = row;
    }

    pub fn set_color_code(&mut self, color: ColorCode) {
        self.color_code = color;
    }

    pub fn disable_cursor(&mut self) {
        unsafe {
            Port::<u8>::new(0x3D4).write(0x0A);
            Port::<u8>::new(0x3D5).write(0x20);
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= TEXT_BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;
                let color_code = self.color_code;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    /// Clear the buffer and move the cursor to the top
    pub fn clear(&mut self) {
        for row in 0..TEXT_BUFFER_HEIGHT {
            for col in 0..TEXT_BUFFER_WIDTH {
                self.buffer.chars[row][col].write(Self::CLEAR_CHAR);
            }
        }

        self.column_position = 0;
        self.row_position = 0;
    }

    /// Write raw VGA data, starting from the top-left corner.
    pub fn write_raw(&mut self, array: &[ScreenChar; 2000]) {
        let mut column = 0;
        let mut row = 0;
        for c in array.iter() {
            self.buffer.chars[row][column].write(*c);
            column += 1;
            if column == TEXT_BUFFER_WIDTH {
                column = 0;
                row += 1;
            }
        }
    }

    fn new_line(&mut self) {
        self.column_position = 0;
        if self.row_position + 1 < TEXT_BUFFER_HEIGHT {
            self.row_position += 1;
        } else {
            // Move every character up
            for row in 0..TEXT_BUFFER_HEIGHT - 1 {
                for col in 0..TEXT_BUFFER_WIDTH {
                    self.buffer.chars[row][col].write(self.buffer.chars[row + 1][col].read());
                }
            }

            // Clear last line
            for column in 0..TEXT_BUFFER_WIDTH {
                self.buffer.chars[TEXT_BUFFER_HEIGHT - 1][column].write(Self::CLEAR_CHAR);
            }
        }
    }
}

impl fmt::Write for TextWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.as_bytes() {
            self.write_byte(*byte);
        }

        Ok(())
    }
}
