ORG 0
BITS 16

jmp 0x7c0:start

start:
		cli ; clear intrupts
		mov ax, 0x7c0
		mov ds, ax
		mov es, ax
		mov ax, 0x00
		mov ss, ax
		mov sp, 0x7c00
		sti ; enable intrupts


times 510-($ - $$) db 0
dw 0xAA55
