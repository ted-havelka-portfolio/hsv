/// Creation date 2026-03-06
/// This crate part of HSV project (Hue, Saturation, Value to RGB)
///
/// Crate `rgbled` encapsulates:
///   * the color control pins to a red-green-blue LED.
///   * local vars for hue, saturation, and value (brightness).
///   * local vars for red, green and blue duty cycles.
///   * functions to turn on and turn off individual LEDs of the RGB LED.

use crate::hal::gpio;
use crate::hal::gpio::Output;
use crate::hal::gpio::PushPull;

use embedded_hal::digital::OutputPin;

// Hue, Saturation, Value attributes and logic:
// use crate::hsvui::Hsvui;

const DEV_RGB_RED_DUTY_MS: u32 = 2 * 1_000_000 / 1000;
const DEV_RGB_GRN_DUTY_MS: u32 = 5 * 1_000_000 / 1000;
const DEV_RGB_BLU_DUTY_MS: u32 = 7 * 1_000_000 / 1000;

pub(crate) struct RgbDisplay {
    // What tick of the frame are we currently on?
    // Setting to 0 starts a new frame.
    tick: u32,
    // What ticks should R, G, B LEDs turn off at?
    schedule: [u32; 3],
    // Schedule to start at next frame.
    next_schedule: Option<[u32; 3]>,
    // R, G, and B pins.
    rgb_pins: [gpio::Pin<Output<PushPull>>; 3],
    // Timer used to reach next tick.
    // pub rgb_timer: Timer<pac::TIMER1>,
}

impl RgbDisplay {

    // pub(crate) fn new(pins: [gpio::Pin<Output<PushPull>>; 3], timer1: Timer<pac::TIMER1>) -> Self {
    pub(crate) fn new(pins: [gpio::Pin<Output<PushPull>>; 3]) -> Self {
        let tick = 0u32;
        // Schedule may grow to be a 2-d array to hold in one
        // dimension between 0 and 3 leds to turn off.
        let schedule = [0; 3];
        //
        let next_schedule: Option<[u32; 3]> = Some([0u32; 3]);

        // RGB pins
        let rgb_pins = pins;

        // TODO [ ] Consider renaming parameter `timer1` to something more general
        // let rgb_timer = timer1;
        Self {
            tick,
            schedule,
            next_schedule,
            rgb_pins,
            // rgb_timer,
        }
    }

    /*
    pub(crate) fn calc_schedule(&mut self, _hsvui: &Hsvui) {
        self.rgb_timer.disable_interrupt();
        self.rgb_timer.start(DEV_RGB_RED_DUTY_MS);
        self.rgb_timer.reset_event();
        self.rgb_timer.enable_interrupt();
    }

    pub(crate) fn take_step(&mut self) {
        // todo!()
        self.rgb_timer.disable_interrupt();
    }

    pub(crate) fn start_timer(&mut self, period: u32) {
        self.rgb_timer.disable_interrupt();
        self.rgb_timer.reset_event();
        self.rgb_timer.start(period);
        self.rgb_timer.enable_interrupt();
    }
    */

    pub(crate) fn red_led_off(&mut self) {
        let pin_red = &mut self.rgb_pins[0];
        pin_red.set_high().unwrap();
    }

    pub(crate) fn red_led_on(&mut self) {
        let pin_red = &mut self.rgb_pins[0];
        pin_red.set_low().unwrap();
    }
}
