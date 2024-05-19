pub struct SoundSensor {
    adc_index: u8,
}

impl SoundSensor {
    pub fn new(adc_index: u8) -> Self {
        unsafe {
            initialise_adc(adc_index);
        }
        SoundSensor { adc_index }
    }
    pub fn read_db(&self) -> u32 {
        unsafe { read_db(self.adc_index) }
    }
}

extern "C" {
    fn initialise_adc(adc_index: u8) -> u32;
    fn read_db(adc_index: u8) -> u32;
}
