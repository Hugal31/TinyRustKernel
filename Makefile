# TODO Refactor
arch ?= i386
kernel = build/kernel-$(arch).bin
iso := build/os-$(arch).iso
target ?= $(arch)-k
build_type ?= debug
rust_target := target/$(target)/$(build_type)/librk.a

linker_script = src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
assembly_source_files := $(wildcard src/arch/$(arch)/*.S)
assembly_object_files := $(patsubst src/arch/$(arch)/%.S, \
	build/arch/$(arch)/%.o, $(assembly_source_files))

ROMS	= \
	  roms/a_test \
	  roms/chichehunter \
	  roms/chichepong \
	  roms/chichevaders \
	  roms/perrodlauncher \
	  roms/skate \
	  roms/yakanoid \

SUBDIRS =	libs/libc \
		libs/libk \
		tools/mkkfs

CFLAGS += -m32

ifeq ($(build_type),release)
	CARGOFLAGS += --release
endif

all: $(iso)

iso: $(iso)

$(iso): $(kernel) $(ROMS)
	./tools/create-iso.sh $@ $(kernel)


$(kernel): kernel $(rust_os) $(assembly_object_files) $(linker_script)
	ld -n --gc-sections -T $(linker_script) -o $(kernel) \
		$(assembly_object_files) $(rust_target)

kernel:
	RUST_TARGET_PATH=$(PWD) xargo build --target $(target) $(CARGOFLAGS)

$(ROMS): tools/mkkfs libs/libc libs/libk

$(SUBDIRS):
	$(MAKE) -C $@

build/arch/$(arch)/%.o: src/arch/$(arch)/%.S
	mkdir -p $(shell dirname $@)
	$(CC) $(CFLAGS) -c -o $@ $<

run: $(iso)
	qemu-system-i386 -cdrom $(iso) -serial stdio -soundhw pcspk

run-debug: $(iso)
	qemu-system-i386 -cdrom $(iso) -serial stdio -s -S

clean:
	for I in $(SUBDIRS);			\
	do					\
		$(MAKE) -C $$I $@ || exit 1;	\
	done
	rm -rf build
	xargo clean

.PHONY: all clean run iso kernel $(SUBDIRS)
