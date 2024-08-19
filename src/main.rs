#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

global_asm!(include_str!("boot.s"), options(att_syntax));

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    let mut writer = TerminalWriter::new();

    writer.write(b"Hello rust kernel!");

    loop {}
}

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {} // FIXME - Don't hang
}

enum VgaColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGrey = 7,
    DarkGrey = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    LightBrown = 14,
    White = 15,
}

impl VgaColor {
    fn entry_color(fg: Self, bg: Self) -> u8 {
        fg as u8 | (bg as u8) << 4
    }

    fn entry(char: u8, color: u8) -> u16 {
        char as u16 | (color as u16) << 8
    }
}

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

struct TerminalWriter {
    row: usize,
    column: usize,
    color: u8,
    buffer: *mut u16,
}

impl TerminalWriter {
    fn new() -> Self {
        let buffer = 0xB8000 as *mut u16;
        let color = VgaColor::entry_color(VgaColor::LightGrey, VgaColor::Black);

        let mut writer = Self {
            row: 0,
            column: 0,
            buffer,
            color,
        };

        // Blank the screen
        for y in 0..VGA_HEIGHT {
            for x in 0..VGA_WIDTH {
                writer.put_entry_at(b' ', color, x, y);
            }
        }

        writer
    }

    fn write(&mut self, data: &[u8]) {
        for c in data {
            self.put_char(*c);
        }
    }

    fn put_char(&mut self, c: u8) {
        self.put_entry_at(c, self.color, self.column, self.row);
        self.column += 1;
        if self.column == VGA_WIDTH {
            self.column = 0;
            self.row += 1;
            if self.row == VGA_HEIGHT {
                self.row = 0;
            }
        }
    }

    fn put_entry_at(&mut self, c: u8, color: u8, x: usize, y: usize) {
        let index = y * VGA_WIDTH + x;
        unsafe {
            *self.buffer.add(index) = VgaColor::entry(c, color);
        }
    }
}
