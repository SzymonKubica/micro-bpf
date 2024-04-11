use core::ffi::c_void;

use alloc::ffi::CString;
use riot_wrappers::cstr::cstr;

pub struct HD44780LCD {
    dev: *mut hd44780_t,
}

pub type hd44780_params_t = c_void;
pub type hd44780_t = c_void;
pub type hd44780_state_t = bool;

impl HD44780LCD {
    pub fn new() -> Self {
        unsafe {
            let dev = hd44780_init_default() as *mut c_void;
            HD44780LCD { dev }
        }
    }

    pub fn init_from(dev: &mut hd44780_t, params: &hd44780_params_t) -> Self {
        unsafe {
            let dev_ptr = dev as *mut hd44780_t;
            let params_ptr = params as *const hd44780_params_t;
            let dev = hd44780_init(dev, params) as *mut hd44780_t;
            HD44780LCD { dev }
        }
    }

    pub fn clear(&self) {
        unsafe { hd44780_clear(self.dev as *const c_void) }
    }
    pub fn home(&self) {
        unsafe { hd44780_home(self.dev as *const hd44780_t) }
    }
    pub fn set_cursor(&self, col: u8, row: u8) {
        unsafe { hd44780_set_cursor(self.dev as *const hd44780_t, col, row) }
    }
    pub fn display(&mut self, state: hd44780_state_t) {
        unsafe { hd44780_display(self.dev as *mut hd44780_t, state) }
    }
    pub fn cursor(&mut self, state: hd44780_state_t) {
        unsafe { hd44780_cursor(self.dev as *mut hd44780_t, state) }
    }
    pub fn blink(&mut self, state: hd44780_state_t) {
        unsafe { hd44780_blink(self.dev as *mut hd44780_t, state) }
    }
    pub fn scroll_left(&self) {
        unsafe { hd44780_scroll_left(self.dev as *const hd44780_t) }
    }
    pub fn scroll_right(&self) {
        unsafe { hd44780_scroll_right(self.dev as *const hd44780_t) }
    }
    pub fn left2right(&mut self) {
        unsafe { hd44780_left2right(self.dev as *mut hd44780_t) }
    }
    pub fn right2left(&mut self) {
        unsafe { hd44780_right2left(self.dev as *mut hd44780_t) }
    }
    pub fn autoscroll(&mut self, state: hd44780_state_t) {
        unsafe { hd44780_autoscroll(self.dev as *mut hd44780_t, state) }
    }
    pub fn create_char(&self, location: u8, charmap: &[u8]) {
        unsafe { hd44780_create_char(self.dev as *mut hd44780_t, location, charmap as *const [u8]) }
    }
    pub fn write(&self, value: u8) {
        unsafe { hd44780_write(self.dev as *const hd44780_t, value) }
    }
    pub fn print(&self, data: &str) {
        let c_str = CString::new(data).unwrap();
        unsafe { hd44780_print(self.dev as *const hd44780_t, c_str.as_ptr() as *const u8) }
    }
}

impl From<*mut hd44780_t> for HD44780LCD {
    fn from(dev: *mut hd44780_t) -> Self {
        HD44780LCD { dev }
    }
}

impl Into<*mut hd44780_t> for HD44780LCD {
    fn into(self) -> *mut hd44780_t {
        self.dev
    }
}

extern "C" {
    /// Initializes the HD44780 LCD display with default parameters that are globally
    /// defined in the C device driver. This should only be called once as we
    /// only have one display connected to the microcontroller.
    fn hd44780_init_default() -> i32;
    fn hd44780_init(dev: *mut hd44780_t, params: *const hd44780_params_t) -> i32;
    fn hd44780_clear(dev: *const c_void);
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
    fn hd44780_create_char(dev: *const hd44780_t, location: u8, charmap: *const [u8]);
    fn hd44780_write(dev: *const hd44780_t, value: u8);
    fn hd44780_print(dev: *const hd44780_t, data: *const u8);
}

type gpio_t = u32;
const HD44780_MAX_PINS: usize = 8;
const HD44780_MAX_ROWS: usize = 4;

/*
/// Redefined parameters for the HD44780 LCD display to allow for using it
/// within rust code
#[allow(non_camel_case_types)]
#[repr(C)]
struct hd44780_params_t {
    cols: u8,                                /* number of LCD cols */
    rows: u8,                                /* number of LCD rows */
    rs: gpio_t,                              /* rs gpio pin */
    rw: gpio_t,                              /* rw gpio pin */
    enable: gpio_t,                          /* enable gpio pin */
    data: *const [gpio_t; HD44780_MAX_PINS], /* data gpio pins */
}

/// Redefinition of the HD44780 device descriptor.
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct hd44780_t {
    p: hd44780_params_t,                 /* LCD config parameters */
    flag: u8,                            /* LCD functional flags */
    ctrl: u8,                            /* LCD control flags */
    mode: u8,                            /* LCD mode flags */
    roff: *const [u8; HD44780_MAX_ROWS], /* offsets for LCD rows */
}

/// State of the HD44780 display.
#[allow(non_camel_case_types)]
#[repr(C)]
enum hd44780_state_t {
    HD44780_OFF, /* disable feature */
    HD44780_ON,  /* enable feature */
}
*/
