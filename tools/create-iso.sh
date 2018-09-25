#!/bin/sh

iso_filename=$1
kernel_filename=$2
base_dir=build/iso

mkdir -p $base_dir/
mkdir -p $base_dir/roms/
mkdir -p $base_dir/boot/grub/

cp $kernel_filename $base_dir/boot/kernel.bin

find roms -name "*.rom" -exec cp {} $base_dir/roms/ \;

for rom in $(find $base_dir/roms -name "*.rom") ; do
	name=$(basename $rom .rom)
	cat <<EOF
menuentry "k - $name" {
	multiboot /boot/kernel.bin /$name
	module /roms/$name.rom
}
EOF
done > $base_dir/boot/grub/grub.cfg

grub-mkrescue -o $iso_filename $base_dir
