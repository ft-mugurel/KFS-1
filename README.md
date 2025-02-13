# KFS-1 - Kernel From Scratch

## Description
KFS-1 (Kernel From Scratch) is a project aimed at building a simple operating system kernel from scratch. This project is part of the curriculum at Ecole 42 and serves as an introduction to low-level programming, operating systems, and kernel development.

The goal of this project is to understand the fundamental concepts of operating systems, including boot processes, memory management, process scheduling, and hardware interaction.

## Features
- [x] **Bootloader**: A custom bootloader written in Assembly to load the kernel into memory.
- [ ] **Kernel**: A minimalistic kernel written in C, providing basic functionalities such as:
  - [ ] **Memory Management**: Simple memory allocation and management.
  - [ ] **Process Scheduling**: Basic round-robin scheduling.
  - [ ] **Hardware Interaction**: Interaction with hardware devices such as the keyboard and screen.
- [ ] **Shell**: A basic shell to interact with the kernel.

## Requirements
- **NASM**: Netwide Assembler for assembling the bootloader.
- **GCC**: GNU Compiler Collection for compiling the kernel.
- **QEMU**: Quick Emulator for testing the kernel in a virtual environment.
- **Make**: Build automation tool for compiling and linking the project.

## Installation
1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/kfs-1.git
   cd kfs-1
   ```
