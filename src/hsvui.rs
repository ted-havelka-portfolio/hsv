use rtt_target::rprintln;

// mod buttons;
use crate::buttons::ButtonPress;

// Ref .../mytechnotalent/rust_embassy_microbit_project/examples/display/src/display.rs
/// An effect filter to apply for an animation
#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum ColorAttributes {
    // None,
    Hue,
    Sat,
    Val,
}

/*
#[derive(Clone, Copy, PartialEq)]
pub enum LedColors{
    None,
    Red,
    Green,
    Blue,
}
*/

// #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[derive(Copy, Clone, PartialEq)]
struct HsvuiData {
    // pub color_attr: u8,
    pub color_attr: ColorAttributes,
    pub hue_value: u16,
    pub sat_value: u16,
    pub val_value: u16,
}

impl HsvuiData {
    fn new() -> Self {
        let color_attr = ColorAttributes::Hue;
        let hue_value = 85; // Hue
        let sat_value = 85; // Saturation
        let val_value = 85; // Brightness
        Self {
            color_attr,
            hue_value,
            sat_value,
            val_value,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub(crate) struct Hsvui {
    hsvui_data: HsvuiData,
}

impl Hsvui {

    pub(crate) fn new() -> Self { 
        let hsvui_data = HsvuiData::new();
        Self {
            hsvui_data,
        }
    }

    pub(crate) fn current_color_attr(&self) -> ColorAttributes {
        self.hsvui_data.color_attr
    }

    /*
    /// Development routine, may be removed for production code:
    pub(crate) fn show_current_hsv_attr(&self) {
        if self.hsvui_data.color_attr == ColorAttributes::Hue {
            rprintln!("Ready to adjust Hue.")
        } else if self.hsvui_data.color_attr == ColorAttributes::Sat {
            rprintln!("Ready to adjust Sat.")
        } else if self.hsvui_data.color_attr == ColorAttributes::Val {
            rprintln!("Ready to adjust Val.")
        }
    }
    */

    // These two functions are only called internally, no need to qualify them
    // with public crate syntax:
    fn handle_button_a(&mut self) {
        // TODO [ ] Remove dev message.
        rprintln!("- DEV - button A pressed");
        match self.hsvui_data.color_attr {
            ColorAttributes::Hue => self.hsvui_data.color_attr = ColorAttributes::Sat,
            ColorAttributes::Sat => self.hsvui_data.color_attr = ColorAttributes::Val,
            ColorAttributes::Val => self.hsvui_data.color_attr = ColorAttributes::Hue,
        }
    }

    fn handle_button_b(&mut self) {
        // TODO [ ] Remove dev message.
        rprintln!("- DEV - button B pressed");
        match self.hsvui_data.color_attr {
            ColorAttributes::Hue => self.hsvui_data.color_attr = ColorAttributes::Val,
            ColorAttributes::Val => self.hsvui_data.color_attr = ColorAttributes::Sat,
            ColorAttributes::Sat => self.hsvui_data.color_attr = ColorAttributes::Hue
        }
    }

    pub(crate) fn handle_buttons(&mut self, binput: crate::buttons::ButtonPress) {
        match binput {
            ButtonPress::ButtonA => { self.handle_button_a() },
            ButtonPress::ButtonB => { self.handle_button_b() },
            // ButtonPress::None => { self.handle_buttons_released() }
            ButtonPress::None => { }
        }
    }
}
