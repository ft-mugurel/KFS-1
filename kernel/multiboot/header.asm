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
