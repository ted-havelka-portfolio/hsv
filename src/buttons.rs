// This source copied and adapted from
// *  https://github.com/rust-embedded/discovery/blob/master/microbit/src/11-snake-game/src/control.rs

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use microbit::hal::gpiote::Gpiote;

static GPIO: Mutex<RefCell<Option<Gpiote>>> = Mutex::new(RefCell::new(None));
static BUTTONPRESS: Mutex<RefCell<ButtonPress>> = Mutex::new(RefCell::new(ButtonPress::None));

// Other Rust sources of this applicaton may use this button press module by
// adding the line:
// 
//     use crate::level::ButtonPress;

#[derive(Debug, Copy, Clone)]
pub enum ButtonPress {
    ButtonA,
    ButtonB,
    None
}

// (1) Code to initialize Microbit-v2 buttons

use cortex_m::interrupt::free;
use microbit::{
    board::Buttons,
    pac::{self, GPIOTE}
};

/// Initialise the buttons and enable interrupts.
pub(crate) fn init_buttons(board_gpiote: GPIOTE, board_buttons: Buttons) {
    let gpiote = Gpiote::new(board_gpiote);

    let channel0 = gpiote.channel0();
    channel0
        .input_pin(&board_buttons.button_a.degrade())
        .hi_to_lo()
        .enable_interrupt();
    channel0.reset_events();

    let channel1 = gpiote.channel1();
    channel1
        .input_pin(&board_buttons.button_b.degrade())
        .hi_to_lo()
        .enable_interrupt();
    channel1.reset_events();

    free(move |cs| {
        *GPIO.borrow(cs).borrow_mut() = Some(gpiote);

        unsafe {
            pac::NVIC::unmask(pac::Interrupt::GPIOTE);
        }
        pac::NVIC::unpend(pac::Interrupt::GPIOTE);
    });
}

// (2) Create ISR and give it same name as interrupt we want to handle

use microbit::pac::interrupt;

#[interrupt]
fn GPIOTE() {
    free(|cs| {
        if let Some(gpiote) = GPIO.borrow(cs).borrow().as_ref() {
            let a_pressed = gpiote.channel0().is_event_triggered();
            let b_pressed = gpiote.channel1().is_event_triggered();

            let bpress = match (a_pressed, b_pressed) {
                (true, false) => ButtonPress::ButtonA,
                (false, true) => ButtonPress::ButtonB,
                _ => ButtonPress::None
            };

            gpiote.channel0().reset_events();
            gpiote.channel1().reset_events();

            *BUTTONPRESS.borrow(cs).borrow_mut() = bpress;
        }
    });
}

// (3) Define function to "get" next bpress

pub fn read_buttons(reset: bool) -> ButtonPress {
    free(|cs| {
        let bpress = *BUTTONPRESS.borrow(cs).borrow();
        if reset {
            *BUTTONPRESS.borrow(cs).borrow_mut() = ButtonPress::None
        }
        bpress
    })
}
