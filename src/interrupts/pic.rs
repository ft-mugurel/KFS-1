use crate::startup_config::pic;
use crate::x86::io::outb;

pub(crate) unsafe fn init_pic() {
    // Master PIC: Başlangıç komutları
    outb(pic::MASTER_COMMAND_PORT, pic::ICW1_INIT); // ICW1: Başlangıç
    outb(pic::MASTER_DATA_PORT, pic::MASTER_IRQ_OFFSET); // ICW2: Master PIC için vektör offset'i 0x20 (32)
    outb(pic::MASTER_DATA_PORT, pic::SLAVE_CONNECTED_TO_IRQ); // ICW3: Slave PIC'in IRQ2'ye bağlandığını belirt
    outb(pic::MASTER_DATA_PORT, pic::ICW4_8086); // ICW4: 8086 modunu ayarla

    // Slave PIC: Başlangıç komutları
    outb(pic::SLAVE_COMMAND_PORT, pic::ICW1_INIT); // ICW1: Başlangıç
    outb(pic::SLAVE_DATA_PORT, pic::SLAVE_IRQ_OFFSET); // ICW2: Slave PIC için vektör offset'i 0x28 (40)
    outb(pic::SLAVE_DATA_PORT, pic::CASCADE_IDENTITY); // ICW3: Slave PIC'in IRQ2'ye bağlandığını belirt
    outb(pic::SLAVE_DATA_PORT, pic::ICW4_8086); // ICW4: 8086 modunu ayarla

    // Tüm interruptları maskeler (engeller) - Başlangıçta maskeyi kaldırıyoruz
    outb(pic::MASTER_DATA_PORT, pic::MASK_ALL); // Master PIC'teki tüm interruptları maskeler
    outb(pic::SLAVE_DATA_PORT, pic::MASK_ALL); // Slave PIC'teki tüm interruptları maskeler

    // Klavye interrupt'ını (IRQ1) etkinleştir
    outb(pic::MASTER_DATA_PORT, pic::KEYBOARD_ENABLE_MASK); // Master PIC: IRQ1 (klavye) için maskeyi kaldır
    outb(pic::SLAVE_DATA_PORT, pic::MASK_ALL); // Slave PIC'teki interruptları maskele (gerekirse)
}

