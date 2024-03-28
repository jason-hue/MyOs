build:
	cargo build --release
dump:build
	rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/MyOs -O binary target/riscv64gc-unknown-none-elf/release/MyOs.bin
run:dump
	qemu-system-riscv64 \
        -machine virt \
        -nographic \
        -bios bootloader/rustsbi-qemu.bin \
        -device loader,file=target/riscv64gc-unknown-none-elf/release/MyOs.bin,addr=0x80200000

gdbserver:dump
	qemu-system-riscv64 \
            -machine virt \
            -nographic \
            -bios bootloader/rustsbi-qemu.bin \
            -device loader,file=target/riscv64gc-unknown-none-elf/release/MyOs.bin,addr=0x80200000 \
            -s -S
gdbclient:
	~/下载/riscv64-unknown-elf-gcc-8.3.0-2020.04.1-x86_64-linux-ubuntu14/bin/riscv64-unknown-elf-gdb -ex 'file target/riscv64gc-unknown-none-elf/release/MyOs' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'