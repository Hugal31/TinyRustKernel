#define	MAGIC		0x1badb002
#define FLAGS		0x3 /* PAGE_ALIGN | MEMORY_INFO */

	.section .multiboot
	.type multiboot_header, @object
multiboot_header:
	.align		4
	.long		MAGIC
	.long		FLAGS
	.long		-(MAGIC + FLAGS)
	.size multiboot_header, . - multiboot_header

	.section .text
	.global k_entry
	.type k_entry, @function
k_entry:
	push %ebx	/* multiboot info */
	push %eax	/* magic */
	call k_main	/* kernel entry point */
.Lend:
	jmp .Lend
	.size k_entry, . - k_entry
