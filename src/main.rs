#![no_std]
#![no_main]

mod vga_buffer;

use core::arch::global_asm;
use core::panic::PanicInfo;

// Set executable up for loading by multi-boot
global_asm!(include_str!("boot.s"), options(att_syntax));

#[no_mangle]
/// Kernel entry point, called from `boot.s` above
pub extern "C" fn kernel_main() -> ! {
    println!("Hello World{}", "!");
    println!("Hello World{}", "! again!");
    loop {}
}

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {} // FIXME - Don't hang
}
