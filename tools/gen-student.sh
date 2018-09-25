#!/bin/sh

files="\
libs/ \
tools/ \
roms/chichehunter/ \
roms/chichepong/ \
roms/chichevaders/ \
roms/perrodlauncher/ \
roms/roms.lds \
roms/roms.mk \
roms/skate/ \
roms/yakanoid/ \
config.mk \
Makefile \
README.md \
k/Makefile \
k/compiler.h \
k/crt0.S \
k/elf.h \
k/include \
k/include/k \
k/include/k/kfs.h \
k/include/k/kstd.h \
k/include/k/types.h \
k/io.h \
k/k.c \
k/k.lds \
k/libvga.c \
k/libvga.h \
k/multiboot.h \
"

if [ -d "$1" ]; then
	echo "Usage: $0 <outdir>" >&2
	echo "Generates a student archive into <outdir>" >&2
	echo "I am a coward, <outdir> must not exist an will be created." >&2

	exit 1
fi

install_file() {
	mkdir -p "$2"/$(dirname "$1")
	cp "$1" "$2/$1"
	unifdef -DSTUDENT "$2/$1" -o "$2/$1"
}

install_dir() {
find "$1" -type f -print0 | while IFS= read -r -d $'\0' file; do
	install_file "$file" "$2"
done
}

for f in $files; do
	if [ -d "$f" ]; then
		install_dir "$f" "$1"
	else
		install_file "$f" "$1"
	fi
done
