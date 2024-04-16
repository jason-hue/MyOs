//
// Created by knifefire on 24-4-16.
//
#include "types.h"
#ifndef C_USER_SYSCALL_H
#define C_USER_SYSCALL_H
uint64 syscall(uint64 syscall_id,uint64 arg0,uint64 arg1,uint64 arg2);
void sys_write(uint64 fd,char* buffer, int len);
void sys_exit(uint64 exit_code);
void sys_yield();
#endif //C_USER_SYSCALL_H
