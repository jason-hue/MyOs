    .section .text.entry
    .globl _start
_start:
     la sp,stack_top
     call start_main

    .section .bss.stack
    .globl stack_low
stack_low:
    .space 4096*16
    .globl stack_top
stack_top: