#include "printf.h"
void clear_bss();
void start_main() __attribute__((section(".text.entry")));
void start_main(){
    clear_bss();
    printf("hello wrold!\n");
}
void clear_bss(){
    extern char start_bss();
    extern char end_bss();
    char *p;
    for (p = start_bss; p < end_bss; ++p)
        *p = 0;
}