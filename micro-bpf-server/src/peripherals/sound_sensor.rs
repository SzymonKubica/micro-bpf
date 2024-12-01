use alloc::{string::String, format};

pub struct SoundSensor {
    adc_index: u8,
}

#[allow(dead_code)]
impl SoundSensor {
    pub fn new(adc_index: u8) -> Result<Self, String> {
        unsafe {
            initialise_adc(adc_index);
            let result = initialise_adc(adc_index);
            if result != 0 {
                return Err(format!("Failed to initialise ADC line: {}", adc_index));
            }
        }
        Ok(SoundSensor { adc_index })
    }
    pub fn read_db(&self) -> u32 {
        unsafe { read_db(self.adc_index) }
    }
}

extern "C" {
    fn initialise_adc(adc_index: u8) -> u32;
    fn read_db(adc_index: u8) -> u32;
}
