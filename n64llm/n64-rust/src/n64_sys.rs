// n64_sys.rs
// System definitions for Nintendo 64 hardware


// PI (Peripheral Interface) Registers
pub const PI_BASE_REG: usize = 0xA4600000;
pub const PI_DRAM_ADDR_REG: usize = 0xA4600000;
pub const PI_CART_ADDR_REG: usize = 0xA4600004;
pub const PI_RD_LEN_REG: usize = 0xA4600008;
pub const PI_WR_LEN_REG: usize = 0xA460000C;
pub const PI_STATUS_REG: usize = 0xA4600010;
pub const PI_BSD_DOM1_LAT_REG: usize = 0xA4600014;
pub const PI_BSD_DOM1_PWD_REG: usize = 0xA4600018;
pub const PI_BSD_DOM1_PGS_REG: usize = 0xA460001C;
pub const PI_BSD_DOM1_RLS_REG: usize = 0xA4600020;

// PI Status Register Flags
pub const PI_STATUS_DMA_BUSY: u32 = 0x01;
pub const PI_STATUS_IO_BUSY: u32 = 0x02;
pub const PI_STATUS_ERROR: u32 = 0x04;
pub const PI_STATUS_RESET: u32 = 0x01000000;

// VI (Video Interface) Registers
pub const VI_BASE_REG: usize = 0xA4400000;
pub const VI_STATUS_REG: usize = 0xA4400000;
pub const VI_ORIGIN_REG: usize = 0xA4400004;
pub const VI_WIDTH_REG: usize = 0xA4400008;
pub const VI_V_INTR_REG: usize = 0xA440000C;
pub const VI_CURRENT_REG: usize = 0xA4400010;
pub const VI_BURST_REG: usize = 0xA4400014;
pub const VI_V_SYNC_REG: usize = 0xA4400018;
pub const VI_H_SYNC_REG: usize = 0xA440001C;
pub const VI_H_SYNC_LEAP_REG: usize = 0xA4400020;
pub const VI_H_VIDEO_REG: usize = 0xA4400024;
pub const VI_V_VIDEO_REG: usize = 0xA4400028;
pub const VI_V_BURST_REG: usize = 0xA440002C;
pub const VI_X_SCALE_REG: usize = 0xA4400030;
pub const VI_Y_SCALE_REG: usize = 0xA4400034;

// AI (Audio Interface) Registers
pub const AI_BASE_REG: usize = 0xA4500000;
pub const AI_DRAM_ADDR_REG: usize = 0xA4500000;
pub const AI_LEN_REG: usize = 0xA4500004;
pub const AI_CONTROL_REG: usize = 0xA4500008;
pub const AI_STATUS_REG: usize = 0xA450000C;
pub const AI_DACRATE_REG: usize = 0xA4500010;
pub const AI_BITRATE_REG: usize = 0xA4500014;

// SI (Serial Interface) Registers
pub const SI_BASE_REG: usize = 0xA4800000;
pub const SI_DRAM_ADDR_REG: usize = 0xA4800000;
pub const SI_PIF_ADDR_RD64B_REG: usize = 0xA4800004;
pub const SI_PIF_ADDR_WR64B_REG: usize = 0xA4800010;
pub const SI_STATUS_REG: usize = 0xA4800018;

// Memory map
pub const RDRAM_BASE: usize = 0x80000000;
pub const RDRAM_SIZE: usize = 0x400000; // 4MB (8MB with expansion pak)
pub const CART_ROM_BASE: usize = 0x10000000;
pub const CART_ROM_PHYSICAL: usize = 0xB0000000;

// Controller definitions
pub const CONTROLLER_1: usize = 0;
pub const CONTROLLER_2: usize = 1;
pub const CONTROLLER_3: usize = 2;
pub const CONTROLLER_4: usize = 3;

#[repr(C)]
pub struct ControllerData {
    pub buttons: u16,
    pub stick_x: i8,
    pub stick_y: i8,
}

// Controller button bits
pub const A_BUTTON: u16 = 0x8000;
pub const B_BUTTON: u16 = 0x4000;
pub const Z_BUTTON: u16 = 0x2000;
pub const START_BUTTON: u16 = 0x1000;
pub const UP_BUTTON: u16 = 0x0800;
pub const DOWN_BUTTON: u16 = 0x0400;
pub const LEFT_BUTTON: u16 = 0x0200;
pub const RIGHT_BUTTON: u16 = 0x0100;
pub const L_BUTTON: u16 = 0x0020;
pub const R_BUTTON: u16 = 0x0010;
pub const C_UP: u16 = 0x0008;
pub const C_DOWN: u16 = 0x0004;
pub const C_LEFT: u16 = 0x0002;
pub const C_RIGHT: u16 = 0x0001;

// DMA functions
pub unsafe fn pi_read(ram_address: *mut u8, rom_address: u32, length: u32) {
    // Wait for any previous DMA to complete
    while (*(PI_STATUS_REG as *const u32) & PI_STATUS_DMA_BUSY) != 0 {}
    
    // Set up DMA
    *(PI_DRAM_ADDR_REG as *mut u32) = ram_address as u32;
    *(PI_CART_ADDR_REG as *mut u32) = rom_address;
    *(PI_RD_LEN_REG as *mut u32) = length - 1;
    
    // Wait for DMA to complete
    while (*(PI_STATUS_REG as *const u32) & PI_STATUS_DMA_BUSY) != 0 {}
}

// Read controller data
pub unsafe fn read_controller(controller: usize) -> ControllerData {
    // In a real implementation, this would read from the controller
    // For now, return a placeholder
    ControllerData {
        buttons: 0,
        stick_x: 0,
        stick_y: 0,
    }
}