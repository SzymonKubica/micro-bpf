use core::ffi::c_void;

pub struct Hd44780Lcd {
    dev: *mut c_void,
}

impl Hd44780Lcd {
    pub fn new() -> Self {
        unsafe {
            let dev = hd44780_init_default() as *mut c_void;
            Hd44780Lcd { dev }
        }
    }

    pub fn clear(&self) {
        unsafe {
            hd44780_clear(self.dev as *const c_void);
        }
    }

    pub fn print(&self, data: &str) {
        unsafe {
            hd44780_print(self.dev as *const c_void, data.as_ptr() as *const char);
        }
    }
}

type hd44780_t = c_void;
type hd44780_params_t = c_void;
type hd44780_state_t = c_void;

extern "C" {
    fn hd44780_init_default() -> i32;
    fn hd44780_init(dev: *mut hd44780_t, params: *const hd44780_params_t) -> i32;
    fn hd44780_clear(dev: *const hd44780_t);
    fn hd44780_home(dev: *const hd44780_t);
    fn hd44780_set_cursor(dev: *const hd44780_t, col: u8, row: u8);
    fn hd44780_display(dev: *const hd44780_t, state: hd44780_state_t);
    fn hd44780_cursor(dev: *const hd44780_t, state: hd44780_state_t);
    fn hd44780_blink(dev: *const hd44780_t, state: hd44780_state_t);
    fn hd44780_scroll_left(dev: *const hd44780_t);
    fn hd44780_scroll_right(dev: *const hd44780_t);
    fn hd44780_left2right(dev: *const hd44780_t);
    fn hd44780_right2left(dev: *const hd44780_t);
    fn hd44780_autoscroll(dev: *const hd44780_t, state: hd44780_state_t);
    fn hd44780_create_char(dev: *const hd44780_t, location: u8, charmap: &[u8]);
    fn hd44780_write(dev: *const hd44780_t, value: u8);
    fn hd44780_print(dev: *const hd44780_t, data: *const char);
}
