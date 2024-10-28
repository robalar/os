#![no_std]
#![no_main]
// Testing
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
mod vga_buffer;

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;

// Set executable up for loading by multi-boot
global_asm!(include_str!("boot.s"), options(att_syntax));

#[no_mangle]
/// Kernel entry point, called from `boot.s` above
pub extern "C" fn kernel_main() -> ! {
    println!("Hello World{}", "!");
    println!("Hello World{}", "! again!");

    #[cfg(test)]
    test_main();

    loop {}
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    println!("{}", info);
    exit_qemu(QemuExitCode::Failed);
}

// Testing harness
#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

/// Exit codes for QEMU
///
/// We cannot just exit using 0/1 as normal, as then there would be no way to distinguish between
/// qemu exiting and the test run failing. This way, we can distinguish between the two.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

/// Exit QEMU by writing exit_code to predefined port
fn exit_qemu(exit_code: QemuExitCode) -> ! {
    const EXIT_PORT: u16 = 0xf4; // defined in qemu_wrapper.sh

    unsafe {
        asm!(
            "out dx, eax",  // Output byte in `eax` to port `dx`
            in("dx") EXIT_PORT, // Loads `dx` with `exit_port`
            in("eax") exit_code as u32, // Loads `eax` with `exit_code`
            options(nomem, nostack) // Doesn't touch memory or stack
        )
    }

    // For the case that the QEMU exit attempt did not work, transition into an infinite loop.
    // Calling `panic!()` here is unfeasible, since there is a good chance this function here is
    // the last expression in the `panic!()` handler itself. This prevents a possible infinite
    // loop.
    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack));
        }
    }
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion... ");
    assert_eq!(1, 0);
    println!("[ok]");
}
