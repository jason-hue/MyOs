cmake_minimum_required(VERSION 3.28)
project(C_user C)

set(CMAKE_C_STANDARD 11)

add_executable(C_user main.c
        types.h
        syscall.h
        syscall.c
        libc.c
        libc.h
        printf.c
        printf.h)
