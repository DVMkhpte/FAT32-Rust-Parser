pub fn print(s: &str) {
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") 1,               // syscall number for write
            in("rdi") 1,               // file descriptor (stdout)
            in("rsi") s.as_ptr(),      // pointer to the string
            in("rdx") s.len(),         // length of the string
            out("rcx") _,              // clobbered registers
            out("r11") _,
        );

        core::arch::asm!(
            "syscall",
            in("rax") 1,
            in("rdi") 1,
            in("rsi") "\n".as_ptr(),
            in("rdx") 1,
            out("rcx") _,
            out("r11") _,
        );
    }
}
