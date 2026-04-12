// pub struct Hpet {
//     base_addr: usize,
//     pub clk_period: u32, // Period in femtoseconds
// }
//
// impl Hpet {
//     pub unsafe fn init(physical_addr: usize) -> Self {
//         // 1. Map physical_addr into virtual memory (use your Mapper)
//         let virtual_addr = map_hpet_region(physical_addr);
//
//         let mut hpet = Self {
//             base_addr: virtual_addr,
//             clk_period: 0,
//         };
//
//         // 2. Read Period (Top 32 bits of offset 0x00)
//         let capabilities = hpet.read_reg(0x00);
//         hpet.clk_period = (capabilities >> 32) as u32;
//
//         // 3. Enable the counter (Offset 0x10, Bit 0)
//         let config = hpet.read_reg(0x10);
//         unsafe {
//             hpet.write_reg(0x10, config | 1);
//         }
//
//         hpet
//     }
//
//     pub fn get_counter(&self) -> u64 {
//         unsafe { self.read_reg(0x0F0) }
//     }
//
//     fn read_reg(&self, offset: usize) -> u64 {
//         unsafe { core::ptr::read_volatile((self.base_addr + offset) as *const u64) }
//     }
//
//     unsafe fn write_reg(&self, offset: usize, val: u64) {
//         unsafe { core::ptr::write_volatile((self.base_addr + offset) as *mut u64, val) }
//     }
// }
