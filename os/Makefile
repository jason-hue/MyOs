clean:
	cargo clean
build:clean
	cargo build --release
dump:build
	rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/MyOs -O binary target/riscv64gc-unknown-none-elf/release/MyOs.bin
run:dump
	echo "\033[33m警告:请将linker.ld中的基址改为0x80200000!!!否则在qemu上会ERROR\033[0m"
	sleep 1
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

k210:dump
	echo "\033[33m警告:请将linker.ld中的基址改为0x80020000!!!否则在K210开发板上会ERROR\033[0m"
	sleep 1
	cp bootloader/rustsbi-k210.bin k210-bin/rustsbi-k210-copy.bin
	dd if=target/riscv64gc-unknown-none-elf/release/MyOs.bin of=k210-bin/rustsbi-k210-copy.bin bs=128k seek=1
	mv k210-bin/rustsbi-k210-copy.bin k210-bin/MyOs.bin
	sudo kflash -p /dev/ttyUSB0 -t k210-bin/MyOs.bin