# Renk Tanımlamaları
GREY    	=	\033[030m
RED     	=	\033[031m
GREEN   	=	\033[032m
YELLOW  	=	\033[033m
BLUE    	=	\033[034m
MAGENTA 	=	\033[035m
CYAN		=	\033[036m
BOLD		=	\033[1m
RESET   	=	\033[0m

# **************************************************************************** #
# 💾 VARIABLES
# **************************************************************************** #

KERNEL_OUT	=	 ./target/i686-kernel/release/libkernel.a
KERNEL_DEBUG_OUT	=	 ./target/i686-kernel/debug/libkernel.a

ISO_OUT		=	build/kernel.iso

BOOT		=	./multiboot/header.asm
LINKER		=	linker/linker.ld

FLAGS		=	-fno-builtin -fno-builtin -fno-builtin -nostdlib -nodefaultlibs

# **************************************************************************** #
# 📖 RULES
# **************************************************************************** #

all: build

SRCS = $(shell find src -name "*.rs")

build: 	${SRCS}
	@mkdir -p build
	@nasm -f elf32 ${BOOT} -o build/boot.o
	@cargo build --release
	@echo -e "$(BOLD)$(GREEN)[✓] KERNEL BUILD DONE$(RESET)"
	@ld -m elf_i386 -T ${LINKER} -o build/kernel.bin build/boot.o  ${KERNEL_OUT}
	@echo -e "$(BOLD)$(GREEN)[✓] KERNEL LINK DONE$(RESET)"

build_debug: ${SRCS} 
	@echo -e "$(BOLD)$(YELLOW)[✓] KERNEL DEBUG MODE ON$(RESET)"
	@mkdir -p build
	@nasm -f elf32 ${BOOT} -o build/boot.o
	@cargo build
	@echo -e "$(BOLD)$(GREEN)[✓] KERNEL BUILD DONE$(RESET)"
	@ld -m elf_i386 -T ${LINKER} -o build/kernel.bin build/boot.o  ${KERNEL_DEBUG_OUT}
	@echo -e "$(BOLD)$(GREEN)[✓] KERNEL LINK DONE$(RESET)"

run: build
	@qemu-system-i386 -kernel ./build/kernel.bin -monitor stdio
	@echo -e "\n$(BOLD)$(CYAN)[✓] KERNEL EXIT DONE$(RESET)"

debug: build_debug
	@qemu-system-i386 -kernel ${KERNEL_OUT} -s -S &
	@gdb -x .gdbinit
	@echo -e "\n$(BOLD)$(CYAN)[✓] KERNEL DEBUG EXIT DONE$(RESET)"

iso: build
	@mkdir -p build/iso/boot/grub
	@cp grub/grub.cfg build/iso/boot/grub
	@cp build/kernel.bin build/iso/boot
	@grub-mkrescue -o ${ISO_OUT} build/iso --modules="multiboot"
	@echo -e "$(BOLD)$(GREEN)[✓] KERNEL ISO BUILD$(RESET)"

run-iso: iso
	@qemu-system-i386 -cdrom ${ISO_OUT}
	@echo -e "\n$(BOLD)$(CYAN)[✓] KERNEL EXIT DONE$(RESET)"

clean:
	@cargo clean
	@echo -e "$(BOLD)$(RED)[♻︎] DELETE KERNEL DONE$(RESET)"

fclean:
	clear
	@rm -rf build/
	@echo -e "$(BOLD)$(RED)[♻︎] DELETE BUILD/ DONE$(RESET)"

re: fclean all

.PHONY: all clean fclean re
