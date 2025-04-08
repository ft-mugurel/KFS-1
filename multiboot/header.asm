		bits 32
		section .multiboot
		dd 0x1BADB002	; Magic number
		dd 0x0			; Flags
		dd - (0x1BADB002 + 0x0)	; Checksum

		section .text
		global _start

_start:
		; Kernel entry point
		extern kmain      ; Rust'taki ana fonksiyon
		call kmain        ; Rust kodunu çağır
		hlt                     ; CPU'yu durdur
		

; isr_keyboard.asm
global isr_keyboard

extern keyboard_interrupt_handler  ; Defined in Rust

section .text
isr_keyboard:
    pusha                       ; Save all general-purpose registers
    call keyboard_interrupt_handler
    popa                        ; Restore all general-purpose registers
    iret                        ; Return from interrupt

