.altmacro
.macro SAVE_RE n
    sd x\n \n*8(sp)
.endm

    .section
    .align 2
    ._alltraps
    ._allstore
_alltraps:
    csrrw sp,sscratch,sp
    addi sp,sp,-34*8
    sd x1,8(sp)
    sd x3,3*8(sp)
.set n 5
.rept 27
    SAVE_RE %n
    .set n,n+1
.endr


