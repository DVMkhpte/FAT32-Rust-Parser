#![no_std]
#![no_main]

/// https://sgibala.com/01-06-single-syscall-hello-world-part-2/
/// https://www.sobyte.net/post/2022-01/rust-fat32

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

fn scan(cluster: u32, data_start: u32, cluster_size: u32, level: usize) {
    let offset_start = data_start + ((cluster - 2) * cluster_size);

    let mut i = 0;
    loop {
        let offset = (offset_start as usize) + (i * 32);

        if offset + 32 > DISK.len() {
            break;
        }

        let data = &DISK[offset..offset + 32]; // Read first 32 bytes of root director

        if data[0] == 0x00 {
            break;
        } // No more entries
        if data[0] == 0xE5 {
            i += 1;
            continue;
        } // Deleted entry
        if data[11] == 0x0F {
            i += 1;
            continue;
        } // Long file name entr

        let mut name = String::new();
        for k in 0..8 {
            if data[k] != 0x20 {
                name.push(data[k] as char);
            }
        }

        let mut ext = String::new();
        for k in 8..11 {
            if data[k] != 0x20 {
                ext.push(data[k] as char);
            }
        }

        // Starting cluster number
        let high_cluster = u16::from_le_bytes([data[20], data[21]]) as u32;
        let low_cluster = u16::from_le_bytes([data[26], data[27]]) as u32;
        let starting_cluster = (high_cluster << 16) | low_cluster;

        let _type = if (data[11] & 0x10) != 0 {
            "FOLDER"
        } else {
            "FILE"
        };

        if _type == "FOLDER" && starting_cluster == 0 {
            i += 1;
            continue;
        } // Skip invalid folder

        if _type == "FILE" {
            let size = u32::from_le_bytes([data[28], data[29], data[30], data[31]]);
            if ext.len() > 0 {
                console::print(&format!(
                    "File Name : {}.{} / Size : {} bytes",
                    name, ext, size
                ));
            } else {
                console::print(&format!("File Name : {} / Size : {} bytes", name, size));
            }

            if size > 0 {
                let file_offset = data_start + ((starting_cluster - 2) * cluster_size); // File data start byte offset

                let read_len = if size > 64 { 64 } else { size as usize }; // Read up to 64 bytes
                let start = file_offset as usize;
                let end = start + read_len;

                if end <= DISK.len() {
                    let content = &DISK[start..end];

                    // Display content as ASCII,
                    let mut text = String::new();
                    for &b in content {
                        if b >= 32 && b <= 126 {
                            text.push(b as char);
                        } else {
                            text.push(' ');
                        }
                    }
                    console::print(&format!("      Content: \"{}\"", text));
                }
            }
        } else {
            let info = format!("Folder Name : {}", name);
            console::print(&info);

            if name != "." && name != ".." {
                scan(starting_cluster, data_start, cluster_size, level + 1); // Recursive scan
            }
        }
        i += 1;
    }
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

        let sector_size = u16::from_le_bytes([mbr[11], mbr[12]]) as u32; // Bytes per sector
        let reserved_sectors = u16::from_le_bytes([mbr[14], mbr[15]]) as u32; // Reserved sectors
        let sectors_per_cluster = mbr[13] as u32; // Sectors per cluster
        let nb_FATs = mbr[16] as u32; // Number of FATs
        let FAT_sectors = u32::from_le_bytes([mbr[36], mbr[37], mbr[38], mbr[39]]); // Sectors per FAT
        let root_cluster = u32::from_le_bytes([mbr[44], mbr[45], mbr[46], mbr[47]]); // Root directory starting cluster

        let cluster_size = sectors_per_cluster * sector_size;
        let fat_size_bytes = nb_FATs * FAT_sectors * sector_size;

        let data_start = (reserved_sectors * sector_size) + fat_size_bytes; // Data region start byte offset

        scan(root_cluster, data_start, cluster_size, 0);
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
