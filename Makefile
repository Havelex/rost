TARGET_JSON = targets/x86_64-rost.json

.PHONY: all kernel userland copy iso install

all: kernel userland copy iso install

kernel:
	cargo build -Z build-std=core,compiler_builtins \
		-Z json-target-spec \
		--target $(TARGET_JSON) \
		-p x86_64

userland:
	cargo build -p shell -p init

copy:
	cp ./target/x86_64-rost/debug/x86_64 ./iso_root/boot/kernel.elf

iso:
	xorriso -as mkisofs \
		-R -r -J \
		-b boot/limine/limine-bios-cd.bin \
		-no-emul-boot \
		-boot-load-size 4 \
		-boot-info-table \
		--efi-boot boot/limine/limine-uefi-cd.bin \
		-efi-boot-part \
		--efi-boot-image \
		--protective-msdos-label \
		iso_root \
		-o rost.iso

install:
	limine bios-install rost.iso
