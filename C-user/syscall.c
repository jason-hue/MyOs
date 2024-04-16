//
// Created by knifefire on 24-4-16.
//
#include "syscall.h"
const uint64 SYS_WRITE = 64;
const uint64 SYS_EXIT = 93;
const uint64 SYS_YIELD = 124;
uint64 syscall(uint64 syscall_id, uint64 arg0, uint64 arg1, uint64 arg2) {
    register uint64 a0 asm("a0") = arg0;
    register uint64 a1 asm("a1") = arg1;
    register uint64 a2 asm("a2") = arg2;
    register uint64 a7 asm("a7") = syscall_id;
    asm volatile("ecall"
            : "=r"(a0)
            : "r"(a0), "r"(a1), "r"(a2), "r"(a7)
            : "memory");
    return a0;
}

void sys_write(uint64 fd, char* buffer, int len) {
    syscall(SYS_WRITE, fd, (uint64) buffer, len);
}

void sys_exit(uint64 exit_code) {
    syscall(SYS_EXIT,exit_code,0,0);
}

void sys_yield() {
    syscall(SYS_YIELD,0,0,0);
}
