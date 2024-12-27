//! 这个项目用于复用链接脚本，并提供依赖内存布局和符号的操作，包括设置启动栈和清零 .bss 节。

#![no_std]
#![deny(warnings, missing_docs)]

/// 链接脚本文本。
pub const SCRIPT: &[u8] = b"
OUTPUT_ARCH(riscv)
ENTRY(_start)
MEMORY { DRAM : ORIGIN = 0x80000000, LENGTH = 2M }
SECTIONS {
    .text : {
        *(.text.entry)
        *(.text .text.*)
    } > DRAM
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    } > DRAM
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    } > DRAM
    .bss (NOLOAD) : {
        __sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        __ebss = .;
    } > DRAM
    .boot (NOLOAD) : ALIGN(8) {
        __boot = .;
        KEEP(*(.boot.stack))
        . = ALIGN(8);
        __end = .;
    } > DRAM
    /DISCARD/ : {
        *(.eh_frame)
    }
}";

/// 定义内核入口。
///
/// 将设置一个启动栈，并在启动栈上调用高级语言入口。
#[macro_export]
macro_rules! boot0 {
    ($entry:ident; stack = $stack:expr) => {
        #[link_section = ".text.entry"]
        #[no_mangle]
        #[naked]
        unsafe extern "C" fn _start() -> ! {
            #[link_section = ".boot.stack"]
            static mut STACK: [u8; $stack] = [0u8; $stack];

            core::arch::naked_asm!(
                "la sp, __end",
                "j  {main}",
                main = sym $entry,
                //options(noreturn),
            )
        }
    };
}

/// 清零 .bss。
///
/// # Safety
///
/// 必须在使用 .bss 内任何东西之前调用。
pub unsafe fn zero_bss() {
    extern "C" {
        fn __sbss();
        fn __ebss();
    }
    unsafe {
        // 必须 volatile，不能用 `slice::fill`，因为需要多核可见
        (__sbss as usize..__ebss as usize).for_each(|addr| {
            (addr as *mut u8).write_volatile(0);
        });
    }
}
