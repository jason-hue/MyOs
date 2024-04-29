FAT_IMG := ../fat.img
ELF_SRC := ../testsuits/
clean:
	cargo clean
qemu_build:clean
	python3 linkchg.py qemu
	cd ../user/&& make build && make copy_to_img
	cargo build --release
k210_build:clean
	python3 linkchg.py k210
	cd ../user/&& make build && make copy_to_img
	cargo build --release
qemu_dump:qemu_build
	make fat-img
	rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/MyOs -O binary target/riscv64gc-unknown-none-elf/release/MyOs.bin
k210_dump:k210_build
	rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/MyOs -O binary target/riscv64gc-unknown-none-elf/release/MyOs.bin
run:qemu_dump
	qemu-system-riscv64 \
        -machine virt \
        -nographic \
        -bios bootloader/rustsbi-qemu.bin \
        -device loader,file=target/riscv64gc-unknown-none-elf/release/MyOs.bin,addr=0x80200000 \
		-drive file=$(FAT_IMG),if=none,format=raw,id=x0 \
        -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \

gdbserver:qemu_dump
	qemu-system-riscv64 \
            -machine virt \
            -nographic \
            -bios bootloader/rustsbi-qemu.bin \
            -device loader,file=target/riscv64gc-unknown-none-elf/release/MyOs.bin,addr=0x80200000 \
            -drive file=$(FAT_IMG),if=none,format=raw,id=x0 \
            -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
            -s -S
gdbclient:
	riscv64-unknown-elf-gdb -ex 'file target/riscv64gc-unknown-none-elf/release/MyOs' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'

k210:k210_dump
	cp bootloader/rustsbi-k210.bin k210-bin/rustsbi-k210-copy.bin
	dd if=target/riscv64gc-unknown-none-elf/release/MyOs.bin of=k210-bin/rustsbi-k210-copy.bin bs=128k seek=1
	mv k210-bin/rustsbi-k210-copy.bin k210-bin/MyOs.bin
	sudo kflash -p /dev/ttyUSB0 -t k210-bin/MyOs.bin

fat-img:
	@rm -f ../$(FAT_IMG)
	@cd ../fat32-fuse && cargo run -- -s $(ELF_SRC) -t $(ELF_SRC) -o $(FAT_IMG)