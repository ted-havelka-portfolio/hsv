#![no_std]
#![no_main]

/// Microbit-v2 HSV sample app

// Starting point for HSV assignment which uses
// [i] PWM
// [i] RGB LED
// [x] Rotary encoder
// [x] Buttons A and B press event handling
//
// [ ] Factor PWM channel setup to file other than main.rs

use core::sync::atomic::{AtomicUsize, Ordering::AcqRel};
use core::sync::atomic::Ordering;

use cortex_m_rt::entry;
use critical_section_lock_mut::LockMut;
use rtt_target::{rtt_init_print, rprintln};
use panic_rtt_target as _;

// Embedded HAL traits
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::InputPin;

use microbit::{
    board::Board,
    display::blocking::Display,
    hal::{
        pac::{self, interrupt},
        self,
    },
};

use sb_rotary_encoder::Direction;

///
/// Local-to-project crates:
///
mod buttons;
use crate::buttons::ButtonPress;
use crate::buttons::{init_buttons};

// Hue, Saturation, Value attributes and logic:
mod hsvui;
use crate::hsvui::ColorAttributes;

// Stuff for the Microbit-v2 5x5 LED display:
mod displaydata;
use crate::displaydata::DisplayData;

mod rgbdisplay;
use crate::rgbdisplay::RgbDisplay;

/// --------------------------------------------------------------------
/// - SECTION - constants
/// --------------------------------------------------------------------

// TODO [ ] Check whether 'duty max' constant will be needed to crate
//          `rgbdisplay` or another localt-to-project crate:
// const LED_DUTY_MAX: u16 = 10000;

// 500ms at 1MHz count rate.
const DEV_RGB_TIME: u32 = 500 * 1_000_000 / 1000;
const DEV_RGB_TIME_LONG: u32 = 2000 * 1_000_000 / 1000;

// Ref https://crates.io/crates/sb-rotary-encoder
// Number of pulses required for one step. 4 is a typical value for encoders with detents.
const PULSE_DIVIDER: i32 = 4;
// Update frequency in Hz, used for velocity calculation
const UPDATE_FREQUENCY: i32 = 5;

const HSV_CLAMP_MIN: usize = 1;
const HSV_CLAMP_MAX: usize = 99;

/// --------------------------------------------------------------------
/// - SECTION - Muteces and atomics
/// --------------------------------------------------------------------

// https://docs.rust-embedded.org/discovery-mb2/15-interrupts/index.html
// https://docs.rust-embedded.org/discovery-mb2/15-interrupts/sharing-data-with-globals.html

// TODO [ ] Amend 05-HSV project readme with note on where LockMut code was
//          initially found and adapted for this project.
//
// Example code from online discovery-mb2 book:
// static GPIOTE_PERIPHERAL: LockMut<gpiote::Gpiote> = LockMut::new();
// . . . we replace this with timer peripheral.

static RGB_TIMER_MTX: LockMut<hal::Timer<pac::TIMER1>> = LockMut::new();

static RGB_DISPLAY_MTX: LockMut<crate::rgbdisplay::RgbDisplay> = LockMut::new();

// Ref https://doc.rust-lang.org/core/sync/atomic/#examples
// Ref https://doc.rust-lang.org/core/sync/atomic/struct.AtomicUsize.html
static COUNTER: AtomicUsize = AtomicUsize::new(0);

// TODO [ ] Replace hard-coded 1's with constant:
static HUE: AtomicUsize = AtomicUsize::new(1);
static SAT: AtomicUsize = AtomicUsize::new(1);
static VAL: AtomicUsize = AtomicUsize::new(1);

/// --------------------------------------------------------------------
/// - SECTION - ISRs
/// --------------------------------------------------------------------

#[interrupt]
fn TIMER1() {
    let count = COUNTER.fetch_add(1, AcqRel);
    let hue = HUE.load(Ordering::SeqCst);
    // rprintln!("Timeout event {}", count + 1);
    rprintln!("Timeout event {}, Hue set to {}", count + 1, hue);

    RGB_TIMER_MTX.with_lock(|timer| {
        timer.start(DEV_RGB_TIME_LONG);
        timer.reset_event();
    });

    RGB_DISPLAY_MTX.with_lock(|rgb_led| {
        rgb_led.red_led_off();
    });
}

/// --------------------------------------------------------------------
/// - SECTION - main routine
/// --------------------------------------------------------------------

#[entry]
fn init() -> ! {
    rtt_init_print!();

    // Try to get all the peripherals . . .
    let board = Board::take().expect("Couldn't initialize board.");

    // Configure a timer for use with `delay_ms()` function:
    let mut timer = hal::Timer::new(board.TIMER0);

    // HSV-to-RGB logic and tri-color LED control:
    // TODO [ ] Amend use statements section to shorten reference to `Level`:
    let blu_edge08 = board.edge.e08.into_push_pull_output(hal::gpio::Level::Low);
    let grn_edge09 = board.edge.e09.into_push_pull_output(hal::gpio::Level::Low);
    let red_edge12 = board.edge.e12.into_push_pull_output(hal::gpio::Level::Low);

    let pins = [red_edge12.degrade(), grn_edge09.degrade(), blu_edge08.degrade()];
    let mut rgb_led = RgbDisplay::new(pins);
    RGB_DISPLAY_MTX.init(rgb_led);

    // Set up timer for RGB pulse width modulation.
    let mut rgb_timer = hal::Timer::new(board.TIMER1);
    rgb_timer.disable_interrupt();
    rgb_timer.reset_event();
    rgb_timer.start(DEV_RGB_TIME);
    rgb_timer.enable_interrupt();
    RGB_TIMER_MTX.init(rgb_timer);



    // Set up the NVIC to handle interrupts:
    unsafe { pac::NVIC::unmask(pac::Interrupt::TIMER1) };
    pac::NVIC::unpend(pac::Interrupt::TIMER1);

    init_buttons(board.GPIOTE, board.buttons);

    // Hsvui crate takes button press events and manages current color
    // attribute, updating this attr on each button press.
    let mut ui = crate::hsvui::Hsvui::new();

    // Create a new instance of an encoder:
    let mut encoder = sb_rotary_encoder::RotaryEncoder::new();

    // Configure GPIOs for quadrature encoder:
    let mut input_a = board.edge.e00.into_floating_input().degrade();
    let mut input_b = board.edge.e01.into_floating_input().degrade();

    // Some dummy tick value, can also be `None` if you don't want to use the velocity feature.
    // TODO [ ] Figure out what tick value or range of values make sense, and why:
    let tick = Some(300);

    // 5x5 LED matrix support:
    let mut display = Display::new(board.display_pins);
    let mut cur_attr: ColorAttributes = ColorAttributes::Hue;
    let mut prev_attr = cur_attr;

    // Hue, Saturation, Value (brightness) variables:
    let mut hue = HUE.fetch_add(0, AcqRel);
    let mut sat = SAT.fetch_add(0, AcqRel);
    let mut val = VAL.fetch_add(0, AcqRel);

    loop {
        // Check user input, namely buttons:
        let bp: ButtonPress = buttons::read_buttons(true);
        ui.handle_buttons(bp);

        let image: [[u8; 5]; 5];

        // ui.show_current_hsv_attr();
        cur_attr = ui.current_color_attr();
        if cur_attr != prev_attr {
            match cur_attr {
                ColorAttributes::Hue => { image = DisplayData::show_h_for_hue(); },
                ColorAttributes::Sat => { image = DisplayData::show_s_for_saturation(); },
                ColorAttributes::Val => { image = DisplayData::show_v_for_value(); },
            }
            display.show(&mut timer, image, 100);
        }
        prev_attr = cur_attr;

        // TODO [ ] Clean up this swatch of code to read quadrature encoder:
        let read_qen_a_res = input_a.is_low().unwrap();
        let read_qen_b_res = input_b.is_low().unwrap();
        let val_qen_b = read_qen_b_res;
        let val_qen_a = read_qen_a_res;

        // Ref https://crates.io/crates/sb-rotary-encoder
        if let Some(event) = encoder.update(val_qen_a, val_qen_b, tick, PULSE_DIVIDER) {
            rprintln!("1. {:?}", event);

            rprintln!("2. value: {}", event.value());
            match event.direction() {
                Direction::Clockwise => {
                    rprintln!("3: CW");
                    match cur_attr {
                        ColorAttributes::Hue => {
                            hue = HUE.fetch_add(1, AcqRel);
                            if hue > HSV_CLAMP_MAX {
                                HUE.store(HSV_CLAMP_MAX, Ordering::SeqCst);
                                hue = HUE.load(Ordering::SeqCst);
                            }
                            // TODO [ ] Refactor LED pin control to timer1 interrupt:
                            RGB_DISPLAY_MTX.with_lock(|rgb_led| {
                                rgb_led.red_led_on();
                            });
                        },
                        ColorAttributes::Sat => {
                            sat = SAT.fetch_add(1, AcqRel);
                            if sat > HSV_CLAMP_MAX {
                                SAT.store(HSV_CLAMP_MAX, Ordering::SeqCst);
                                sat = SAT.load(Ordering::SeqCst);
                            }
                        },
                        ColorAttributes::Val => {
                            val = VAL.fetch_add(1, AcqRel);
                            if val > HSV_CLAMP_MAX {
                                VAL.store(HSV_CLAMP_MAX, Ordering::SeqCst);
                                val = VAL.load(Ordering::SeqCst);
                            }
                        },
                    }
                }

                Direction::CounterClockwise => {
                    rprintln!("3: CCW");
                    match cur_attr {
                        ColorAttributes::Hue => {
                            // TODO [ ] Refactor LED pin control to timer1 interrupt:
                            RGB_DISPLAY_MTX.with_lock(|rgb_led| {
                                rgb_led.red_led_off();
                            });

                            hue = HUE.fetch_sub(1, AcqRel);
                            if hue < HSV_CLAMP_MIN {
                                HUE.store(HSV_CLAMP_MIN, Ordering::SeqCst);
                                hue = HUE.load(Ordering::SeqCst);
                            }
                        },
                        ColorAttributes::Sat => {
                            sat = SAT.fetch_sub(1, AcqRel);
                            if sat < HSV_CLAMP_MIN {
                                SAT.store(HSV_CLAMP_MIN, Ordering::SeqCst);
                                sat = SAT.load(Ordering::SeqCst);
                            }
                        },
                        ColorAttributes::Val => {
                            val = VAL.fetch_sub(1, AcqRel);
                            if val < HSV_CLAMP_MIN {
                                VAL.store(HSV_CLAMP_MIN, Ordering::SeqCst);
                                val = VAL.load(Ordering::SeqCst);
                            }
                        },
                    }
                }
            }

            // TODO [ ] Determine why timedelta is always zero after first reading:
            // rprintln!("timedelta: {}", event.timedelta().unwrap());

            rprintln!("- DEV 0311 - Hue {}, Sat {}, Val {}", hue, sat, val);

            // NOTE this code never seems to run:
            if let Some(velocity) = event.velocity(UPDATE_FREQUENCY) {
                rprintln!("{:?}", velocity);

                // The velocity allows to calculate a dynamic step value to
                // accelerate the encoder when moved quickly.
                let acceleration = velocity >> 4;
                let _step = event.step() + (event.step() * acceleration);
            }
        }


        timer.delay_ms(10);
    }
}
