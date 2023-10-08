use embedded_hal::adc::{Channel, OneShot};
use stm32f7xx_hal::{
    adc::Adc,
    pac::{ADC1, ADC_COMMON},
    rcc::{Clocks, APB2},
    signature::{VtempCal110, VtempCal30},
};

struct AdcTemperatureSensor;

impl Channel<ADC1> for AdcTemperatureSensor {
    type ID = u8;
    fn channel() -> u8 {
        18_u8 // Temperature sensor is connected to ADC1_IN18
    }
}

pub struct TemperatureSensor {
    adc: Adc<ADC1>,
    adc_temp_sensor: AdcTemperatureSensor,
    cal30: u16,
    cal110: u16,
    first_read: bool,
}

impl TemperatureSensor {
    pub fn new(adc_common: ADC_COMMON, adc: ADC1, apb: &mut APB2, clocks: &Clocks) -> Self {
        // Setup ADC1
        let adc1: Adc<ADC1> = Adc::adc1(adc, apb, &clocks, 12, true);

        // Initialize internal temperature sensor
        // Enable the temperature and vref internal channels
        adc_common.ccr.modify(|_, w| w.vbate().clear_bit()); // Disable VBAT
        adc_common.ccr.modify(|_, w| w.tsvrefe().set_bit()); // Enable TS and VREF

        Self {
            adc: adc1,
            adc_temp_sensor: AdcTemperatureSensor,
            cal30: VtempCal30::get().read(),
            cal110: VtempCal110::get().read(),
            first_read: false,
        }
    }

    fn convert_adc_reading(&mut self, adc_value: u16) -> f32 {
        // Temperature conversion formula
        (110.0 - 30.0) * (adc_value - self.cal30) as f32 / (self.cal110 - self.cal30) as f32 + 30.0
    }

    pub fn read_temperature(&mut self) -> f32 {
        if self.first_read == false {
            self.first_read = true;

            // Discart first ADC reading
            let _: u16 = self.adc.read(&mut self.adc_temp_sensor).unwrap();
        }

        // Read Temperature
        let adc_data: u16 = self.adc.read(&mut self.adc_temp_sensor).unwrap();

        self.convert_adc_reading(adc_data)
    }
}
