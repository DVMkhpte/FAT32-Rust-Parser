#![no_std]
#![no_main]

use core::panic::PanicInfo;

extern crate rlibc;

/// Use the alloc crate for heap allocations
extern crate alloc;
use alloc::{format, string::String};
/// Import the custom allocator module
mod allocator;
/// Import the console module for printing
mod console;
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

        let sector_size = u16::from_le_bytes([mbr[11], mbr[12]]) as u32;
        let reserved_sectors = u16::from_le_bytes([mbr[14], mbr[15]]) as u32;
        let nb_FATs = mbr[16] as u32;
        let FAT_sectors = u32::from_le_bytes([mbr[36], mbr[37], mbr[38], mbr[39]]);
        let root_cluster = u32::from_le_bytes([mbr[44], mbr[45], mbr[46], mbr[47]]);

        let root_offset = (reserved_sectors + (nb_FATs * FAT_sectors)) * sector_size;

        let offset = root_offset as usize;
        let data = &DISK[offset..offset + 32]; // Read first 32 bytes of root directory

        // Extract the file name from the directory entry
        let mut name = String::new();
        for i in 0..8 {
            name.push(data[i] as char);
        }

        let mut ext = String::new();
        for i in 8..11 {
            ext.push(data[i] as char);
        }

        let _type = if (data[11] & 0x10) != 0 {
            "FOLDER"
        } else {
            "FILE"
        };

        let info = format!("Trouve : [{}] . [{}] (Type: {})", name, ext, _type);
        console::print(&info);
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
