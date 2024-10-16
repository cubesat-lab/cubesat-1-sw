use stm32f7xx_hal::signature::Uid;

pub struct McuUid {
    pub x: u16,           // X coordinate on wafer
    pub y: u16,           // Y coordinate on wafer
    pub waf_num: u8,      // Wafer number
    pub lot_num: [u8; 7], // Lot number
}

impl McuUid {
    pub fn new() -> Self {
        // Copy lot number slice from UID
        let mut lot_num: [u8; 7] = Default::default();
        lot_num.clone_from_slice(Uid::get().lot_num().as_bytes());

        Self {
            x: Uid::get().x(),
            y: Uid::get().y(),
            waf_num: Uid::get().waf_num(),
            lot_num,
        }
    }
}

impl Default for McuUid {
    fn default() -> Self {
        Self::new()
    }
}
