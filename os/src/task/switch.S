.altmacro
.macro SAVE_SRE n
    sd s\n,(\n+2)*8(a0)
.endm
.macro LOAD_SRE n
    ld s\n,(\n+2)*8(a1)
.endm

    .section .text
    .globl _switch
_switch:
    sd ra,0(a0)
    sd sp,8(a0)
    .set n,0
    .rept 12
        SAVE_SRE %n
        .set n,n+1
    .endr

    ld ra,0(a1)
    .set n,0
    .rept 12
        LOAD_SRE %n
        .set n,n+1
    .endr
    ld sp,8(a1)
    ret
