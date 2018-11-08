isr:
	// Save registers
	pusha
	pushw %ds
	pushw %es
	pushw %fs
	pushw %gs

	//  Put correct value in ds, es, fs, and gs
	movw %ss, %ax
	movw %ax, %ds
	movw %ax, %es
	movw %ax, %fs
	movw %ax, %gs

	pushl %esp 		// Push the argument, wich is the pointer to the context
	movl %esp, %ebp
	call isr_generic_handler
	movl %ebp, %esp
	popl %esp

	// Restore  registers
	popw %ds
	popw %es
	popw %fs
	popw %gs
	popa

	add $8, %esp		// Remove the isr and the error codes

	iret

	// TODO Try to use macro
	.global isr_0
isr_0:
	pushl $0 // Mocked error code
	pushl $0 // Interrupt number
	jmp isr

	.global isr_64
isr_64:
	pushl $0 // Mocked error code
	pushl $64 // Interrupt number
	jmp isr

	.global isr_65
isr_65:
	pushl $0 // Mocked error code
	pushl $65 // Interrupt number
	jmp isr

	.global isr_128
isr_128:
	pushl $0 // Mocked error code
	pushl $128 // Interrupt number
	jmp isr
