OUTPUT_FORMAT("elf32-i386")
OUTPUT_ARCH("i386")
ENTRY(k_entry)

SECTIONS
{
	. = 0x100000 + SIZEOF_HEADERS;

	.text :
	{
		*(.multiboot) *(.text)
	}

	.rodata :
	{
		*(.rodata)
	}

	.data :
	{
		*(.data)
	}

	.bss :
	{
		*(.bss) *(COMMON)
	}

	PROVIDE(_end = .);
}
