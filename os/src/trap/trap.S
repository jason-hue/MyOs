.altmacro
.macro SAVE_RE n
    sd x\n,\n*8(sp)
.endm
.macro LOAD_RE n
    ld x\n,\n*8(sp)
.endm
    .section .text.trampoline
    .global _alltraps
    .globl _restore
    .align 2

_alltraps:
    csrrw sp,sscratch,sp
    addi sp,sp,-34*8
    sd x1,8(sp)
    sd x3,3*8(sp)
.set n,5
.rept 27
    SAVE_RE %n
    .set n,n+1
.endr

    csrr t0,sstatus
    csrr t1,sepc

    sd t0,32*8(sp)
    sd t1,33*8(sp)
    csrr t2,sscratch
    sd t2,2*8(sp)

    ld t0,34*8(sp)
    ld t1,36*8(sp)
    ld sp,35*8(sp)

    csrw satp,t0
    sfence.vma

    jr t1



_restore:
    csrw satp, a1
    sfence.vma
    csrw sscratch, a0
    mv sp, a0
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
.set n, 5
.rept 27
    LOAD_RE %n
    .set n, n+1
.endr
    ld sp,2*8(sp)
    addi sp, sp, 34*8
    sret





