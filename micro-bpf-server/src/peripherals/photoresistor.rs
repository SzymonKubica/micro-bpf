use alloc::{format, string::String};

/// Allows for reading the light intensity value in % of the detectable range,
/// which is spanned by roughly 200-1023 ADC analog input values.
pub struct Photoresistor {
    adc_index: u8,
}


#[allow(dead_code)]
impl Photoresistor {
    pub fn new(adc_index: u8) -> Result<Self, String> {
        unsafe {
            let result = initialise_adc(adc_index);
            if result != 0 {
                return Err(format!("Failed to initialise ADC line: {}", adc_index));
            }
        }
        Ok(Photoresistor { adc_index })
    }
    // Returns the light intensity in % of the measurable range.
    pub fn read_intensity(&self) -> u32 {
        unsafe { read_light_intensity(self.adc_index) }
    }
}

extern "C" {
    fn initialise_adc(adc_index: u8) -> u32;
    fn read_light_intensity(adc_index: u8) -> u32;
}
