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
ISO_FULL_OUT	=	build/kernel-full.iso

BOOT		=	./multiboot/header.asm
LINKER		=	linker/linker.ld

FLAGS		=	-fno-builtin -fno-builtin -fno-builtin -nostdlib -nodefaultlibs

# **************************************************************************** #
# 
# **************************************************************************** #

GRUB_MKRESCUE	=	$(shell which grub2-mkrescue 2>/dev/null || which grub-mkrescue 2>/dev/null)
ifeq ($(GRUB_MKRESCUE),)
	$(error "grub-mkrescue not found, please install it.")
endif

GRUB_MODULE_DIR	=	$(shell [ -d /usr/lib/grub/i386-pc ] && echo /usr/lib/grub/i386-pc || ([ -d /usr/lib64/grub/i386-pc ] && echo /usr/lib64/grub/i386-pc))
ifeq ($(GRUB_MODULE_DIR),)
	$(error "GRUB i386-pc modules not found. Please install grub-pc-bin or equivalent package.")
endif

QEMU_SYSTEM	=	$(shell which qemu-system-i386 2>/dev/null || which qemu 2>/dev/null)
ifeq ($(QEMU_SYSTEM),)
	$(error "qemu-system-i386 not found, please install it.")
endif

LD		=	$(shell which ld 2>/dev/null || which ld.bfd 2>/dev/null)
ifeq ($(LD),)
	$(error "ld not found, please install it.")
endif

NASM		=	$(shell which nasm 2>/dev/null || which nasm 2>/dev/null)
ifeq ($(NASM),)
	$(error "nasm not found, please install it.")
endif

CARGO		=	$(shell which cargo 2>/dev/null)
ifeq ($(CARGO),)
	$(error "cargo not found, please install it.")
endif

RUSTC		=	$(shell which rustc 2>/dev/null)
ifeq ($(RUSTC),)
	$(error "rustc not found, please install it.")
endif

# **************************************************************************** #
# 📖 RULES
# **************************************************************************** #

all: run-iso

SRCS = $(shell find src -name "*.rs")

build: 	${SRCS}
	@mkdir -p build
	@${NASM} -f elf32 ${BOOT} -o build/boot.o
	@${CARGO} build --no-default-features --release
	@echo -e "$(BOLD)$(GREEN)[✓] KERNEL BUILD DONE$(RESET)"
	@${LD} -m elf_i386 -T ${LINKER} -o build/kernel.bin build/boot.o  ${KERNEL_OUT}
	@echo -e "$(BOLD)$(GREEN)[✓] KERNEL LINK DONE$(RESET)"

build_debug: ${SRCS}
	@echo -e "$(BOLD)$(YELLOW)[✓] KERNEL DEBUG MODE ON$(RESET)"
	@mkdir -p build
	@${NASM} -f elf32 ${BOOT} -o build/boot.o
	@${CARGO} build
	@echo -e "$(BOLD)$(GREEN)[✓] KERNEL BUILD DONE$(RESET)"
	@${LD} -m elf_i386 -T ${LINKER} -o build/kernel.bin build/boot.o  ${KERNEL_DEBUG_OUT}
	@echo -e "$(BOLD)$(GREEN)[✓] KERNEL LINK DONE$(RESET)"

run: build
	@${QEMU_SYSTEM} -kernel ./build/kernel.bin -monitor stdio
	@echo -e "\n$(BOLD)$(CYAN)[✓] KERNEL EXIT DONE$(RESET)"

debug: build_debug
	@${QEMU_SYSTEM} -kernel ${KERNEL_OUT} -s -S &
	@gdb -x .gdbinit
	@echo -e "\n$(BOLD)$(CYAN)[✓] KERNEL DEBUG EXIT DONE$(RESET)"

iso: build
	@mkdir -p build/iso/boot/grub
	@cp grub/grub.cfg build/iso/boot/grub
	@cp build/kernel.bin build/iso/boot
	@${GRUB_MKRESCUE} -o ${ISO_OUT} build/iso --directory=${GRUB_MODULE_DIR} \
		--modules="multiboot" --locales="" --fonts="" --themes=""
	@echo -e "$(BOLD)$(GREEN)[✓] KERNEL ISO BUILD$(RESET)"

iso-full: build
	@mkdir -p build/iso/boot/grub
	@cp grub/grub.cfg build/iso/boot/grub
	@cp build/kernel.bin build/iso/boot
	@${GRUB_MKRESCUE} -o ${ISO_FULL_OUT} build/iso --directory=${GRUB_MODULE_DIR} --modules="multiboot"
	@echo -e "$(BOLD)$(GREEN)[✓] KERNEL FULL ISO BUILD$(RESET)"

run-iso: iso
	@${QEMU_SYSTEM} -m 4G -cdrom ${ISO_OUT}
	@echo -e "\n$(BOLD)$(CYAN)[✓] KERNEL EXIT DONE$(RESET)"

run-iso-full: iso-full
	@${QEMU_SYSTEM} -m 4G -cdrom ${ISO_FULL_OUT}
	@echo -e "\n$(BOLD)$(CYAN)[✓] KERNEL EXIT DONE$(RESET)"

run-iso-term: iso
	@${QEMU_SYSTEM} -m 4G -cdrom ${ISO_OUT} -boot d -nographic
	@echo -e "\n$(BOLD)$(CYAN)[✓] KERNEL EXIT DONE$(RESET)"

clean:
	@rm -rf build/
	@echo -e "$(BOLD)$(RED)[♻︎] DELETE KERNEL DONE$(RESET)"

fclean: clean
	@${CARGO} clean
	@echo -e "$(BOLD)$(RED)[♻︎] DELETE BUILD/ DONE$(RESET)"

re: clean all

.PHONY: all clean fclean re
