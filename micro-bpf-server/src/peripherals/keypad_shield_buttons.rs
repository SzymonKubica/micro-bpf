use alloc::{format, string::String};


pub struct KeypadShieldButtons {
    adc_index: u8,
}

/// Encodes available direction buttons that can be chosen on the keypad.
/// The select button doesn't work (hardware might not be connected), so we
/// only have those 4 options.

pub enum KeypadDirection {
    Right = 0,
    Up = 1,
    Down = 2,
    Left = 3,
    NoInput = 4,
}

const RIGHT_THRESHOLD: u32 = 200;
const UP_THRESHOLD: u32 = 500;
const DOWN_THRESHOLD: u32 = 700;
const LEFT_THRESHOLD: u32 = 900;

impl KeypadShieldButtons {
    pub fn new(adc_index: u8) -> Result<Self, String> {
        unsafe {
            initialise_adc(adc_index);
            let result = initialise_adc(adc_index);
            if result != 0 {
                return Err(format!("Failed to initialise ADC line: {}", adc_index));
            }
        }
        Ok(KeypadShieldButtons { adc_index })
    }

    pub fn read_direction(&self) -> KeypadDirection {
        let reading = unsafe { read_adc(self.adc_index) };
        if reading < RIGHT_THRESHOLD {
            KeypadDirection::Right
        } else if reading < UP_THRESHOLD {
            KeypadDirection::Up
        } else if reading < DOWN_THRESHOLD {
            KeypadDirection::Down
        } else if reading < LEFT_THRESHOLD {
            KeypadDirection::Left
        } else {
            KeypadDirection::NoInput
        }
    }
}

extern "C" {
    fn initialise_adc(adc_index: u8) -> u32;
    fn read_adc(adc_index: u8) -> u32;
}
