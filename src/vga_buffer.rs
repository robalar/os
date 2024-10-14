use core::fmt;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;

    // TODO: is this correct? see:
    //  - https://github.com/matklad/once_cell/issues/61
    //  - https://matklad.github.io//2020/01/02/spinlocks-considered-harmful.html
    static WRITER: spin::Mutex<Writer> = spin::Mutex::new(Writer::new());
    WRITER.lock().write_fmt(args).unwrap();
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Color {
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

/// VGA color code that specifies foreground and background color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    const fn new(foreground: Color, background: Color) -> Self {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct AsciiChar(u8);

impl From<u8> for AsciiChar {
    fn from(value: u8) -> Self {
        match value {
            // Printable ASCII or newline
            0x20..=0x7e | b'\n' => Self(value),
            // not printable, return ■
            _ => Self(0xfe),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: AsciiChar,
    color_code: ColorCode,
}

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

#[repr(transparent)]
struct Buffer {
    chars: [[ScreenChar; VGA_WIDTH]; VGA_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer_addr: usize,
}

impl Writer {
    const fn new() -> Self {
        Self {
            column_position: 0,
            color_code: ColorCode::new(Color::Yellow, Color::Black),
            buffer_addr: 0xB8000,
        }
    }

    fn buffer(&self) -> &'static mut Buffer {
        unsafe { &mut *(self.buffer_addr as *mut Buffer) }
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= VGA_WIDTH {
                    self.new_line();
                }

                let row = VGA_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;
                let char = ScreenChar {
                    ascii_character: byte.into(),
                    color_code,
                };

                unsafe {
                    core::ptr::write_volatile(&mut self.buffer().chars[row][col], char);
                }
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..VGA_HEIGHT {
            for column in 0..VGA_WIDTH {
                unsafe {
                    let char = core::ptr::read_volatile(&self.buffer().chars[row][column]);
                    core::ptr::write_volatile(&mut self.buffer().chars[row - 1][column], char);
                }
            }
        }

        self.clear_row(VGA_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' '.into(),
            color_code: self.color_code,
        };

        for col in 0..VGA_WIDTH {
            unsafe { core::ptr::write_volatile(&mut self.buffer().chars[row][col], blank) }
        }
    }

    fn clear_screen(&mut self) {
        for row in 0..VGA_HEIGHT {
            self.clear_row(row)
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte)
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
