/// Creation date 2026-03-06
/// This crate part of HSV project (Hue, Saturation, Value to RGB)
///
/// Crate `rgbled` encapsulates:
///   * the color control pins to a red-green-blue LED.
///   * local vars for red, green and blue duty cycles.
///   * functions to turn on and turn off individual LEDs of the RGB LED.

use crate::hal::gpio;
use crate::hal::gpio::Output;
use crate::hal::gpio::PushPull;

use embedded_hal::digital::OutputPin;

use rtt_target::rprintln;

// Hue, Saturation, Value attributes and logic:
// use crate::hsvui::Hsvui; // ::LedColors;

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
        }
    }

    // Function to determine current timer countdown value to apply, to achieve
    // correct duty cycle for lowest duty cycle colors.  Note this routine
    // figures two things: (1) the period a.k.a. duration of the active part of
    // the shortest LED color duty cycle and (2) the color or colors channels
    // of the RGB LED to turn off at end of this period.
    pub(crate) fn shortest_on_time(&self, rgb_duty_cycles: [u8; 3]) -> [u8; 4] {
        let mut min1: u8;
        /*
        let mut r1: u8;
        let mut g1: u8;
        let mut b1: u8;
        */
        let [mut r1, mut g1, mut b1] = rgb_duty_cycles.clone();
        // rprintln!("r1, g1, b1 hold {} {} {}", r1, g1, b1);

        // Find the smallest duty cycle among red and green RGB vals:
        if r1 <= g1 {
            min1 = r1;
        } else {
            min1 = g1;
        }

        // Find the smallest duty cycle among blue and previous determination:
        if b1 <= min1 {
            min1 = b1;
        }

        r1 = r1 - min1;
        g1 = g1 - min1;
        b1 = b1 - min1;
        // rprintln!("After min sort r1, g1, b1 hold {} {} {}", r1, g1, b1);

        // After the subtraction of min1 one or more RGB remaining duty cycle
        // periods will be zero.  Those RGB channels with no remaining
        // duty cycle beyond min1 must be turned off at next timer1 timeout.

        [r1, g1, b1, min1]
    }

    // Red LED pin control
    pub(crate) fn red_led_off(&mut self) {
        let pin_red = &mut self.rgb_pins[0];
        pin_red.set_high().unwrap();
    }

    pub(crate) fn red_led_on(&mut self) {
        let pin_red = &mut self.rgb_pins[0];
        pin_red.set_low().unwrap();
    }

    // Green LED pin control
    pub(crate) fn grn_led_off(&mut self) {
        let pin_grn = &mut self.rgb_pins[1];
        pin_grn.set_high().unwrap();
    }

    pub(crate) fn grn_led_on(&mut self) {
        let pin_grn = &mut self.rgb_pins[0];
        pin_grn.set_low().unwrap();
    }

    // Blue LED pin control
    pub(crate) fn blu_led_off(&mut self) {
        let pin_blu = &mut self.rgb_pins[1];
        pin_blu.set_high().unwrap();
    }

    pub(crate) fn blu_led_on(&mut self) {
        let pin_blu = &mut self.rgb_pins[0];
        pin_blu.set_low().unwrap();
    }
}
