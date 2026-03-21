#![no_std]
#![no_main]

// Microbit-v2 HSV sample app

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

// Local-to-project crates:
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
use crate::rgbdisplay::{FrameElement, RgbDisplay};

// Library crate from https://github.com/pdx-cs-rust-embedded/hsv/blob/main/src/lib.rs
mod hsv;
use crate::hsv::{Hsv, Rgb};

// ---------------------------------------------------------------------
// - SECTION - constants
// ---------------------------------------------------------------------

// 500ms at 1MHz count rate.
const DEV_RGB_TIME: u32 = 500 * 1_000_000 / 1000;

// Ref https://crates.io/crates/sb-rotary-encoder
// Number of pulses required for one step. 4 is a typical value for encoders with detents.
const PULSE_DIVIDER: i32 = 4;

// const DUTY_CYCLE_SCALING: u32 = 2;
const DUTY_CYCLE_SCALING: u32 = 2;

// ---------------------------------------------------------------------
// - SECTION - Muteces and atomics
// ---------------------------------------------------------------------

static RGB_TIMER_MTX: LockMut<hal::Timer<pac::TIMER1>> = LockMut::new();

static RGB_DISPLAY_MTX: LockMut<crate::rgbdisplay::RgbDisplay> = LockMut::new();

// Ref https://doc.rust-lang.org/core/sync/atomic/#examples
// Ref https://doc.rust-lang.org/core/sync/atomic/struct.AtomicUsize.html
static COUNTER: AtomicUsize = AtomicUsize::new(0);

static HUE: AtomicUsize = AtomicUsize::new(1);
static SAT: AtomicUsize = AtomicUsize::new(1);
static VAL: AtomicUsize = AtomicUsize::new(1);

static RED: AtomicUsize = AtomicUsize::new(1);
static GRN: AtomicUsize = AtomicUsize::new(1);
static BLU: AtomicUsize = AtomicUsize::new(1);

// Frames have between one and four elements, allow timer ISR to track
// which element in the sequence of time periods in a frame we're on:
static FRAME_ELEMENT: AtomicUsize = AtomicUsize::new(0);

// ---------------------------------------------------------------------
// - SECTION - ISRs
// ---------------------------------------------------------------------

#[interrupt]
fn TIMER1() {

    let mut felement = FRAME_ELEMENT.fetch_add(0, AcqRel);
    // rprintln!("working with felement {}", felement);

    let mut frame_element = FrameElement::new();

    RGB_DISPLAY_MTX.with_lock(|rgb_led| {
        match felement {
            0 => { frame_element = rgb_led.duty_cycle_timing.fe0.clone(); },
            1 => { frame_element = rgb_led.duty_cycle_timing.fe1.clone(); },
            2 => { frame_element = rgb_led.duty_cycle_timing.fe2.clone(); },
            3 => { frame_element = rgb_led.duty_cycle_timing.fe3.clone(); },
            4_usize.. => todo!(),
        }
    });

    /*
    rprintln!("states Per, R, G, B: {} {} {} {}", frame_element.period,
        frame_element.rstate,
        frame_element.gstate,
        frame_element.bstate
        );
    */

    // When frame element period is non-zero, set LEDs and update timer:
    if frame_element.period > 0 {
        RGB_DISPLAY_MTX.with_lock(|rgb_led| {
            if frame_element.rstate > 0 {
                rgb_led.red_led_on();
            } else {
                rgb_led.red_led_off();
            }

            if frame_element.gstate > 0 {
                rgb_led.grn_led_on();
            } else {
                rgb_led.grn_led_off();
            }

            if frame_element.bstate > 0 {
                rgb_led.blu_led_on();
            } else {
                rgb_led.blu_led_off();
            }
        });

        // Update timer period:
        RGB_TIMER_MTX.with_lock(|timer| {
            let mut period = frame_element.period as u32;
            if period < 1000 {
                period = 1000;
            }

            timer.start(frame_element.period as u32 * DUTY_CYCLE_SCALING * 20);
            timer.reset_event();
        });

        felement += 1;
        // Should not reach this state but seems to happen:
        if felement > 3 {
            felement = 0;
        }

    // When frame element period is zero that indicates we've reached end of
    // present frame.  Reset frame element index value used with `match`:
    } else if frame_element.period <= 0 {
        felement = 0;
    }

    // rprintln!("storing felement {}", felement);
    FRAME_ELEMENT.store(felement, Ordering::SeqCst);
}

/// --------------------------------------------------------------------
/// - SECTION - functions
/// --------------------------------------------------------------------

#[entry]
fn init() -> ! {
    rtt_init_print!();

    // Try to get all the peripherals . . .
    let board = Board::take().expect("Couldn't initialize board.");

    // Configure a timer for use with `delay_ms()` function:
    let mut timer = hal::Timer::new(board.TIMER0);

    // tri-color LED control pins:
    // TODO [ ] Amend use statements section to shorten reference to `Level`:

    let blu_edge08 = board.edge.e08.into_push_pull_output(hal::gpio::Level::High);
    let grn_edge09 = board.edge.e09.into_push_pull_output(hal::gpio::Level::Low);
    let red_edge12 = board.edge.e12.into_push_pull_output(hal::gpio::Level::High);

    let pins = [red_edge12.degrade(), grn_edge09.degrade(), blu_edge08.degrade()];
    let rgb_led = RgbDisplay::new(pins);
    RGB_DISPLAY_MTX.init(rgb_led);

    // Set up timer for RGB pulse width modulation
    let mut rgb_timer = hal::Timer::new(board.TIMER1);
    rgb_timer.disable_interrupt();
    rgb_timer.reset_event();
    rgb_timer.start(DEV_RGB_TIME);
    rgb_timer.enable_interrupt();
    RGB_TIMER_MTX.init(rgb_timer);

    // Set up the NVIC to handle interrupts:
    unsafe { pac::NVIC::unmask(pac::Interrupt::TIMER1) };
    pac::NVIC::unpend(pac::Interrupt::TIMER1);

    /*
    // Instantiate a duty cycle timing structure
    let mut dc_timing = DutyCycleTiming::new();
    DUTY_CYCLE_TIMING.init(dc_timing);
    */

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

    let mut count;

    loop {
        // Check user input, namely buttons:
        let bp: ButtonPress = buttons::read_buttons(true);
        ui.handle_buttons(bp);

        let image: [[u8; 5]; 5];

        cur_attr = ui.current_color_attr();
        if cur_attr != prev_attr {
            match cur_attr {
                ColorAttributes::Hue => { image = DisplayData::show_h_for_hue(); },
                ColorAttributes::Sat => { image = DisplayData::show_s_for_saturation(); },
                ColorAttributes::Val => { image = DisplayData::show_v_for_value(); },
            }
            display.show(&mut timer, image, 300);
        }
        prev_attr = cur_attr;

        let val_qen_a = input_a.is_low().unwrap();
        let val_qen_b = input_b.is_low().unwrap();

        let mut hsv_clamp_min: u8 = 1;
        let mut hsv_clamp_max: u8 = 1;

        // Clamp our rotary encoder incremented/decremented color attribute
        // values with limits from local rgbdisplay crate:
        RGB_DISPLAY_MTX.with_lock(|rgb_led| {
            hsv_clamp_min = rgb_led.hsv_clamp_min();
            hsv_clamp_max = rgb_led.hsv_clamp_max();
        });

        // Ref https://crates.io/crates/sb-rotary-encoder
        if let Some(event) = encoder.update(val_qen_a, val_qen_b, tick, PULSE_DIVIDER) {
            rprintln!("1. {:?}", event);
            rprintln!("2. value: {}", event.value());

            let hsv_updated: bool;

            match event.direction() {
                Direction::Clockwise => {
                    rprintln!("3: CW");
                    hsv_updated = true;
                    match cur_attr {
                        ColorAttributes::Hue => {
                            hue = HUE.fetch_add(1, AcqRel);
                            if hue > hsv_clamp_max as usize {
                                HUE.store(hsv_clamp_max as usize, Ordering::SeqCst);
                                hue = HUE.fetch_add(0, AcqRel);
                            }
                        },
                        ColorAttributes::Sat => {
                            sat = SAT.fetch_add(1, AcqRel);
                            if sat > hsv_clamp_max as usize {
                                SAT.store(hsv_clamp_max as usize, Ordering::SeqCst);
                                sat = SAT.fetch_add(0, AcqRel);
                            }
                        },
                        ColorAttributes::Val => {
                            val = VAL.fetch_add(1, AcqRel);
                            if val > hsv_clamp_max as usize {
                                VAL.store(hsv_clamp_max as usize, Ordering::SeqCst);
                                val = VAL.fetch_add(0, AcqRel);
                            }
                        },
                    }
                }

                Direction::CounterClockwise => {
                    rprintln!("3: CCW");
                    hsv_updated = true;
                    match cur_attr {
                        ColorAttributes::Hue => {
                            hue = HUE.fetch_sub(1, AcqRel);
                            if hue < hsv_clamp_min as usize {
                                HUE.store(hsv_clamp_min as usize, Ordering::SeqCst);
                                hue = HUE.fetch_sub(0, AcqRel);
                            }
                        },
                        ColorAttributes::Sat => {
                            sat = SAT.fetch_sub(1, AcqRel);
                            if sat < hsv_clamp_min as usize {
                                SAT.store(hsv_clamp_min as usize, Ordering::SeqCst);
                                sat = SAT.fetch_sub(0, AcqRel);
                            }
                        },
                        ColorAttributes::Val => {
                            val = VAL.fetch_sub(1, AcqRel);
                            if val < hsv_clamp_min as usize {
                                VAL.store(hsv_clamp_min as usize, Ordering::SeqCst);
                                val = VAL.fetch_sub(0, AcqRel);
                            }
                        },
                    }
                }
            }

            if hsv_updated {
                rprintln!("One of H, S and V values changed, now are {} {} {}",
                    hue, sat, val);

                // TODO [ ] Look at changing HUE, SAT and VAL atomics to type f32,
                //          to avoid these tedious type conversions:
                let hue1: f32 = hue as f32 / 100.0;
                let sat1: f32 = hue as f32 / 100.0;
                let val1: f32 = hue as f32 / 100.0;

                let hsv_vals = Hsv {h: hue1, s: sat1, v: val1};

                // HSV to RGB conversion call here:
                let rgb_vals: Rgb = hsv_vals.into();

                let red1: usize = (rgb_vals.r * 100.0) as usize;
                let grn1: usize = (rgb_vals.g * 100.0) as usize;
                let blu1: usize = (rgb_vals.b * 100.0) as usize;

                RED.store(red1, Ordering::SeqCst);
                GRN.store(grn1, Ordering::SeqCst);
                BLU.store(blu1, Ordering::SeqCst);

                // Call the later RgbDisplay side frame periods calculation:
                RGB_DISPLAY_MTX.with_lock(|rgb_led| {
                    rgb_led.calc_display_frame_periods([red1 as u8, grn1 as u8, blu1 as u8]);
                });
            }

            count = COUNTER.fetch_add(0, AcqRel);

            rprintln!("- DEV 0311 - Hue {}, Sat {}, Val {}", hue, sat, val);
            rprintln!("- DEV 0311 - Timer1 event count {}", count);
        }
        timer.delay_ms(10);
    }
}
