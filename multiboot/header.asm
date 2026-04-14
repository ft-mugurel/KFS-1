		bits 32
		section .multiboot
		dd 0x1BADB002					; Magic number
		dd 0x00000003					; Flags: align modules + request memory info
		dd - (0x1BADB002 + 0x00000003)	; Checksum

		section .text
		global _start

_start:
		cli				; Disable interrupts immediately to avoid pre-kmain IRQs
		; Kernel entry point
		extern kmain	; Rust'taki ana fonksiyon
		push ebx		; multiboot info pointer
		push eax		; multiboot magic
		call kmain		; Rust kodunu çağır
		hlt				; CPU'yu durdur
		

; isr_keyboard.asm
global isr_keyboard

extern keyboard_interrupt_handler  ; Defined in Rust
extern exception_common_handler

section .text
isr_keyboard:
    pusha                       ; Save all general-purpose registers
    call keyboard_interrupt_handler
    popa                        ; Restore all general-purpose registers
    iret                        ; Return from interrupt

%macro EXC_NOERR 1
global isr_exception_%1
isr_exception_%1:
	pusha
	push dword %1
	call exception_common_handler
	add esp, 4
	popa
	iret
%endmacro

%macro EXC_ERR 1
global isr_exception_%1
isr_exception_%1:
	pusha
	push dword %1
	call exception_common_handler
	add esp, 4
	popa
	add esp, 4
	iret
%endmacro

EXC_NOERR 0
EXC_NOERR 1
EXC_NOERR 2
EXC_NOERR 3
EXC_NOERR 4
EXC_NOERR 5
EXC_NOERR 6
EXC_NOERR 7
EXC_ERR 8
EXC_NOERR 9
EXC_ERR 10
EXC_ERR 11
EXC_ERR 12
EXC_ERR 13
EXC_ERR 14
EXC_NOERR 15
EXC_NOERR 16
EXC_ERR 17
EXC_NOERR 18
EXC_NOERR 19
EXC_NOERR 20
EXC_NOERR 21
EXC_NOERR 22
EXC_NOERR 23
EXC_NOERR 24
EXC_NOERR 25
EXC_NOERR 26
EXC_NOERR 27
EXC_NOERR 28
EXC_NOERR 29
EXC_NOERR 30
EXC_NOERR 31

