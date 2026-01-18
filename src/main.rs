#![no_std]
#![no_main]

use core::panic::PanicInfo;

extern crate rlibc;

/// Use the alloc crate for heap allocations
extern crate alloc;
use alloc::format;
///use alloc::string::String;
///use alloc::vec::Vec;

/// Import the console module for printing
mod console;
/// Import the custom allocator module
mod allocator;
use allocator::{BumpAllocator, Locked};

/// Define the global allocator using the locked bump allocator
#[global_allocator]
static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());

/// Define the heap size (100 KiB)
const HEAP_SIZE: usize = 100 * 1024; // 100 KiB
static mut HEAP_MEM: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

/// Include the binary disk image as a byte array
static DISK: &[u8] = include_bytes!("../media/data.img");

/// Required language items for no_std environment
#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}
#[unsafe(no_mangle)]
pub extern "C" fn _Unwind_Resume() {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // Initialize the global allocator
    unsafe {
        ALLOCATOR
            .lock()
            .init(core::ptr::addr_of_mut!(HEAP_MEM) as usize, HEAP_SIZE);
    }

    // Validate the MBR of the disk image
    if DISK.len() < 512 {
        console::print("Disk image is too small!");
    }

    // Check MBR signature
    let mbr = &DISK[0..512];
    if mbr[510] != 0x55 || mbr[511] != 0xAA {
        console::print("Invalid MBR signature!");
    } else {
        // If valid, create a message using heap allocation
        let disk_size = DISK.len();
        console::print(&format!(
            "Valid MBR detected. Disk size: {} bytes",
            disk_size
        ));
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
