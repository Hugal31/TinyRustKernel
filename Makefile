arch ?= i386
kernel = build/kernel-$(arch).bin
iso := build/os-$(arch).iso
target ?= $(arch)-k
rust_target := target/$(target)/debug/librk.a

linker_script = src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
assembly_source_files := $(wildcard src/arch/$(arch)/*.S)
assembly_object_files := $(patsubst src/arch/$(arch)/%.S, \
	build/arch/$(arch)/%.o, $(assembly_source_files))

CFLAGS += -m32

.PHONY: all clean run iso kernel

all: $(kernel)

clean:
	rm -rf build
	xargo clean

run: $(iso)
	qemu-system-i386 -cdrom $(iso) -serial stdio

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	mkdir -p build/isofiles/boot/grub
	cp $(kernel) build/isofiles/boot/kernel.bin
	cp $(grub_cfg) build/isofiles/boot/grub
	grub-mkrescue -o $(iso) build/isofiles 2> /dev/null
	rm -r build/isofiles

$(kernel): kernel $(rust_os) $(assembly_object_files) $(linker_script)
	ld -n --gc-sections -T $(linker_script) -o $(kernel) \
		$(assembly_object_files) $(rust_target)

kernel:
	RUST_TARGET_PATH=$(PWD) xargo build --target $(target)


build/arch/$(arch)/%.o: src/arch/$(arch)/%.S
	mkdir -p $(shell dirname $@)
	$(CC) $(CFLAGS) -c -o $@ $<
