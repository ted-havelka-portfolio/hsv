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

// Hue, Sat, Val parameters have an integral range inclusive of:
const HSV_CLAMP_MIN: u8 = 1;
const HSV_CLAMP_MAX: u8 = 50;

pub(crate) struct RgbDisplay {
    hsv_clamp_min: u8,
    hsv_clamp_max: u8,
    down_time: u8,
    // R, G, and B pins.
    rgb_pins: [gpio::Pin<Output<PushPull>>; 3],
}

impl RgbDisplay {

    pub(crate) fn new(pins: [gpio::Pin<Output<PushPull>>; 3]) -> Self {
        let hsv_clamp_min = HSV_CLAMP_MIN;
        let hsv_clamp_max = HSV_CLAMP_MAX;
        // Last time period of an RGB display "frame" during which all LEDs off
        let down_time = 1u8;
        // RGB pins
        let rgb_pins = pins;

        Self {
            hsv_clamp_min,
            hsv_clamp_max,
            down_time,
            rgb_pins,
        }
    }

    pub(crate) fn hsv_clamp_min(&self) -> u8 {
        self.hsv_clamp_min
    }

    pub(crate) fn hsv_clamp_max(&self) -> u8 {
        self.hsv_clamp_max
    }

    // Calculate a frame's "down time", the latter time during which no LEDs are
    // turned on.
    pub(crate) fn calc_down_time(&mut self, rgb_duty_cycles: [u8; 3]) {
        let mut max1: u8;
        let [mut r1, mut g1, mut b1] = rgb_duty_cycles.clone();

        if r1 > HSV_CLAMP_MAX {
            r1 = HSV_CLAMP_MAX;
        }

        if g1 > HSV_CLAMP_MAX {
            g1 = HSV_CLAMP_MAX;
        }

        if b1 > HSV_CLAMP_MAX {
            b1 = HSV_CLAMP_MAX;
        }

        if r1 >= g1 {
            max1 = r1;
        } else {
            max1 = g1;
        }

        if b1 >= max1 {
            max1 = b1;
        }

        // Assure clock gets some non-zero period:
        if max1 >= HSV_CLAMP_MAX {
            max1 = HSV_CLAMP_MAX -1;
        }

        self.down_time = HSV_CLAMP_MAX - max1;
    }

    pub(crate) fn down_time(&self) -> u8 {
        self.down_time
    }

    // Function to determine current frame partial period.
    pub(crate) fn shortest_duty_cycle_of(&self, rgb_duty_cycles: [u8; 3]) -> [u8; 4] {
        let mut min1: u8 = HSV_CLAMP_MAX;
        let [mut r1, mut g1, mut b1] = rgb_duty_cycles.clone();

        // Find minimun duty cycle among red and green
        if r1 > 0 {
            if r1 <= g1 || g1 == 0 {
                min1 = r1;
            } else if g1 > 0 {
                min1 = g1;
            }
        } else if g1 > 0 {
            // Find minimum duty cycle among green and blue
            if g1 <= b1 || b1 == 0 {
                min1 = g1;
            } else {
                min1 = b1;
            }
        } else {
            min1 = b1;
        }

        if r1 >= min1 {
            r1 = r1 - min1;
        }

        if g1 >= min1 {
            g1 = g1 - min1;
        }

        if b1 >= min1 {
            b1 = b1 - min1;
        }

        if r1 == 0 && g1 == 0 && b1 == 0 {
            min1 = self.down_time;
        }

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
        let pin_grn = &mut self.rgb_pins[1];
        pin_grn.set_low().unwrap();
    }

    // Blue LED pin control
    pub(crate) fn blu_led_off(&mut self) {
        let pin_blu = &mut self.rgb_pins[2];
        pin_blu.set_high().unwrap();
    }

    pub(crate) fn blu_led_on(&mut self) {
        let pin_blu = &mut self.rgb_pins[2];
        pin_blu.set_low().unwrap();
    }
}
