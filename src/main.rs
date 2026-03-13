#![no_std]
#![no_main]

// Microbit-v2 HSV sample app

// Starting point for HSV assignment which uses
// [x] PWM
// [x] RGB LED
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
use crate::rgbdisplay::RgbDisplay;

// ---------------------------------------------------------------------
// - SECTION - constants
// ---------------------------------------------------------------------

// 500ms at 1MHz count rate.
const DEV_RGB_TIME: u32 = 500 * 1_000_000 / 1000;

// Ref https://crates.io/crates/sb-rotary-encoder
// Number of pulses required for one step. 4 is a typical value for encoders with detents.
const PULSE_DIVIDER: i32 = 4;

const DUTY_CYCLE_SCALING: u32 = 5;

const FRAME_IS_NEW: usize = 0;
const FRAME_IN_PROGRESS: usize = 1;

// ---------------------------------------------------------------------
// - SECTION - Muteces and atomics
// ---------------------------------------------------------------------

static RGB_TIMER_MTX: LockMut<hal::Timer<pac::TIMER1>> = LockMut::new();

static RGB_DISPLAY_MTX: LockMut<crate::rgbdisplay::RgbDisplay> = LockMut::new();

// Ref https://doc.rust-lang.org/core/sync/atomic/#examples
// Ref https://doc.rust-lang.org/core/sync/atomic/struct.AtomicUsize.html
static COUNTER: AtomicUsize = AtomicUsize::new(0);

static FRAME_STATE: AtomicUsize = AtomicUsize::new(0);

static HUE: AtomicUsize = AtomicUsize::new(1);
static SAT: AtomicUsize = AtomicUsize::new(1);
static VAL: AtomicUsize = AtomicUsize::new(1);

static RED: AtomicUsize = AtomicUsize::new(1);
static GRN: AtomicUsize = AtomicUsize::new(1);
static BLU: AtomicUsize = AtomicUsize::new(1);

// These atomics are scratchpad values updated over the course of a single
// RGB LED frame, composed of two, three or four ticks to support each color's
// duty cycle:
static R1: AtomicUsize = AtomicUsize::new(1);
static G1: AtomicUsize = AtomicUsize::new(1);
static B1: AtomicUsize = AtomicUsize::new(1);

// ---------------------------------------------------------------------
// - SECTION - ISRs
// ---------------------------------------------------------------------

#[interrupt]
fn TIMER1() {
    // TODO [ ] Review and check whether 'count' needed:
    //  Note `count` is helpful to have for diagnostics and debugging
    let _count = COUNTER.fetch_add(1, AcqRel);

    // let frame_state = FRAME_STATE.load(Ordering::SeqCst);
    let frame_state = FRAME_STATE.fetch_add(0, AcqRel);

    let red: usize;
    let grn: usize;
    let blu: usize;

    // When starting a display frame grab the R,G,B values from application:
    if frame_state == FRAME_IS_NEW {
        red = RED.fetch_add(0, AcqRel);
        grn = GRN.fetch_add(0, AcqRel);
        blu = BLU.fetch_add(0, AcqRel);

        RGB_DISPLAY_MTX.with_lock(|rgb_led| {
            rgb_led.calc_down_time([red as u8, grn as u8, blu as u8]);
            rgb_led.red_led_on();
            rgb_led.grn_led_on();
            rgb_led.blu_led_on();
        });

        FRAME_STATE.store(FRAME_IN_PROGRESS, Ordering::SeqCst);
    } else {
    // When frame in progress grab the scratchpad R,G,B values:
        red = R1.fetch_add(0, AcqRel);
        grn = G1.fetch_add(0, AcqRel);
        blu = B1.fetch_add(0, AcqRel);
    }

    // Declare variables to be updated and then read to turn off next LEDs whose
    // duty cycle is ending in present frame:
    let mut schedule: [u8; 4] = [0,0,0,0];
    let mut duty_cycle_remaining = 10;

    RGB_DISPLAY_MTX.with_lock(|rgb_led| {

        schedule = rgb_led.shortest_duty_cycle_of([red as u8, grn as u8, blu as u8]);

        let r1 = schedule[0];
        let g1 = schedule[1];
        let b1 = schedule[2];
        duty_cycle_remaining = schedule[3];

        // Store the scratch pad values for R, G, B duty cycle remainders:
        R1.store(r1 as usize, Ordering::SeqCst);
        G1.store(g1 as usize, Ordering::SeqCst);
        B1.store(b1 as usize, Ordering::SeqCst);

        if r1 == 0 {
            rgb_led.red_led_off();
        }

        if g1 == 0 {
            rgb_led.grn_led_off();
        }

        if b1 == 0 {
            rgb_led.blu_led_off();
        }

        if r1 == 0 && g1 == 0 && b1 == 0 {
            duty_cycle_remaining = rgb_led.down_time();
            FRAME_STATE.store(FRAME_IS_NEW, Ordering::SeqCst);
        }
    });

    RGB_TIMER_MTX.with_lock(|timer| {
        if duty_cycle_remaining < 1 {
            duty_cycle_remaining = 2;
        } else if duty_cycle_remaining > 99 {
            duty_cycle_remaining = 98;
        }
        timer.start(duty_cycle_remaining as u32 * DUTY_CYCLE_SCALING * 100);

        timer.reset_event();
    });
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

        RGB_DISPLAY_MTX.with_lock(|rgb_led| {
            hsv_clamp_min = rgb_led.hsv_clamp_min();
            hsv_clamp_max = rgb_led.hsv_clamp_max();
        });

        // Ref https://crates.io/crates/sb-rotary-encoder
        if let Some(event) = encoder.update(val_qen_a, val_qen_b, tick, PULSE_DIVIDER) {
            rprintln!("1. {:?}", event);
            rprintln!("2. value: {}", event.value());

            let mut hsv_updated: bool = false;

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

            if hsv_updated == true {
                rprintln!("One of H, S and V values changed, now are {} {} {}",
                    hue, sat, val);

                // TODO [ ] Add HSV to RGB conversion call here

                RED.store(hue, Ordering::SeqCst);
                GRN.store(sat, Ordering::SeqCst);
                BLU.store(val, Ordering::SeqCst);
            }

            count = COUNTER.fetch_add(0, AcqRel);

            rprintln!("- DEV 0311 - Hue {}, Sat {}, Val {}", hue, sat, val);
            rprintln!("- DEV 0311 - Timer1 event count {}", count);
        }
        timer.delay_ms(10);
    }
}
