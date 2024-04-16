//
// Created by knifefire on 24-4-16.
//

#include "libc.h"
#include "syscall.h"
const uint64 STDOUT = 1;
void write(char* buffer) {
    uint buffer_size = 0;
    while (buffer[buffer_size] != '\0') {
        buffer_size++;
    }
    sys_write(STDOUT, buffer, buffer_size);
}

void shoutdown(uint64 exit_code) {
    sys_exit(exit_code);
}

void yield() {
    sys_yield();
}
