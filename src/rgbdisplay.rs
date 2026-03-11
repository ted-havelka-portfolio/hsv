/// Creation date 2026-03-06
/// This crate part of HSV project (Hue, Saturation, Value to RGB)
///
/// Crate `rgbled` encapsulates:
///   * the color control pins to a red-green-blue LED.
///   * local vars for hue, saturation, and value (brightness).
///   * local vars for red, green and blue duty cycles.
///   * functions to turn on and turn off individual LEDs of the RGB LED.

use microbit::pac;

use crate::hal::gpio;
// use crate::hal::gpio::Pin;
use crate::hal::gpio::Output;
use crate::hal::gpio::PushPull;
use crate::hal::Timer;

// Hue, Saturation, Value attributes and logic:
// mod hsvui;
// use crate::hsvui::ColorAttributes;
use crate::hsvui::Hsvui;

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
    rgb_timer: Timer<pac::TIMER1>,
}

impl RgbDisplay {

    pub(crate) fn new(pins: [gpio::Pin<Output<PushPull>>; 3], timer1: Timer<pac::TIMER1>) -> Self {
        let tick = 0u32;
        // Schedule may grow to be a 2-d array to hold in one
        // dimension between 0 and 3 leds to turn off.
        let schedule = [0; 3];
        //
        let next_schedule: Option<[u32; 3]> = Some([0u32; 3]);

        // RGB pins
        let rgb_pins = pins;

        // TODO [ ] Consider renaming parameter `timer1` to something more general
        let rgb_timer = timer1;
        Self {
            tick,
            schedule,
            next_schedule,
            rgb_pins,
            rgb_timer,
        }
    }

    /// Terminology:
    ///
    /// frame: a 10 millisecon period in which R, G, B LEDs realize their
    ///        respective duty cycles.
    ///
    /// step:  time at which one or more LEDs turn off after the start of
    ///        present frame.

    /// Create frame schedule from present RGB values
    ///
    /// 1. Disable TIMER1 interrupt
    /// 2. Clear any pending interrupts
    /// 3. Call timer.start(countdown_value) to begin period 
    /// 4. Iterate over R, G, B vals to find transition periods
    /// 5. turn on LEDs

    pub(crate) fn calc_schedule(&mut self, _hsvui: &Hsvui) {
        self.rgb_timer.disable_interrupt();
        self.rgb_timer.reset_event();
        self.rgb_timer.start(DEV_RGB_RED_DUTY_MS);
        self.rgb_timer.enable_interrupt();
    }

    /// Take the next frame update step. Called at startup
    /// and then from the timer interrupt handler.

    pub(crate) fn take_step(&mut self) {
        // todo!()
        self.rgb_timer.disable_interrupt();
    }
}
