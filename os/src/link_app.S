    .align 3
    .section .data
    .globl _num_app
_num_app:
    .quad 5
    .quad app0_start
    .quad app1_start
    .quad app2_start
    .quad app3_start
    .quad app4_start

    .section .data
    .globl app0_start
    .globl app0_end
app0_start:
    .incbin "/home/knifefire/RustroverProjects/MyOs/user/target/riscv64gc-unknown-none-elf/release/00hello_world.bin"
app0_end:

    .globl app1_start
    .globl app1_end
app1_start:
    .incbin "/home/knifefire/RustroverProjects/MyOs/user/target/riscv64gc-unknown-none-elf/release/01store_fault.bin"
app1_end:

    .globl app2_start
    .globl app2_end
app2_start:
    .incbin "/home/knifefire/RustroverProjects/MyOs/user/target/riscv64gc-unknown-none-elf/release/02power.bin"
app2_end:

    .globl app3_start
    .globl app3_end
app3_start:
    .incbin "/home/knifefire/RustroverProjects/MyOs/user/target/riscv64gc-unknown-none-elf/release/03priv_inst.bin"
app3_end:

        .globl app4_start
        .globl app4_end
    app4_start:
        .incbin "/home/knifefire/RustroverProjects/MyOs/user/target/riscv64gc-unknown-none-elf/release/04priv_csr.bin"
    app4_end: