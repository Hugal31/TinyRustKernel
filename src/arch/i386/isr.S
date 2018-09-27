isr:
	pusha
	pushl %esp
	movl %esp, %ebp
	call isr_generic_handler
	movl %ebp, %esp
	popl %esp
	popa
	add $8, %esp
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
