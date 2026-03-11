# HSV

Hue, Saturation and Value (brightness) demo application, written by
Theodore M Havelka, with much help from
[Rust Discovery-MB2 book](https://docs.rust-embedded.org/discovery-mb2/) and
[Crates.io](https://crates.io/).  This project represents a bare metal
application exercising:

* Microbit-v2 buttons
* the board's 5x5 LED array
* an off board quadrature encoder
* an analog RGB LED

## Build and Run

With a current Rust compiler and toolchain installed, build and run this demo
application with:

$ cargo run

The project file `.cargo/config.toml` provides target hardware specific settings
to support simpler invocations of `cargo`.

## Requirements

The application is expected to manage and adjust Hue, Saturation and Brightness
values for the user, reading button press events and one of either potentiometer
input or quadrature encoder input.  These three values must be converted to Red,
Green and Blue duty cycles, applied to the individual color LED lines of the RBG
LED.

In list form the requirements include:

* Supported color parameters include Hue, Saturation and Brightness (Value)
* Microbit buttons A and B cycle and wrap selected parameter
* MB2 red LED matrix shows H, S or V for 100 ms after select parameter changes
* Potentiometer or quadrature encoder adjusts selected parameter
* RGB values are calculated from HSV values
* RBG values are normalized on a scale ranging 0..100
* RBG LEDs are driven at 100Hz, each with their respective duty cycle

## What is done

Functions completed so far include:

* Hue, Saturation and Brightness (Value) selection with wrapping via buttons
* Selected parameter is highlighted by one of 'H', 'S' and 'V' on 5x5 LED martix
* Parameter adjustment using quadrature encoder
* A rough timer-only based PWM function

## What remains to do

* Conversion of HSV to RGB values
* The variable duty cycle and variable timer countdown "schedule" calculation
* Fine tuning of timer based interrupt to give 100Hz RGB display update rate

## Most challenging

The most difficult element of HSV application development came in the effort to
implement an RgbDisplay module.  The module is intended to factor together the
hardware peripherals and control logic for driving an multi-color, in our case
three color LED.  An experienced Rust developer could knock out this module in
half an hour, maybe a little longer.  It took a few hours of reading and trial
and error to arrive at a working RgbDisplay parameter expression for a main
application to pass in GPIO pins to the LED.

A more difficult development aspect lay in the inclusion of the timer peripheral
in the RgbDisplay module.  Though there are clearly straightforward ways to
accomplish this factoring of specific peripheral code, the author Ted was not
able to find a correct syntax or program constructs to correctly update the
timer in the non-main.rs module.  There was question also about where the, in
this case TIMER1 interrupt should live.  Should it be in `main.rs` or in the
module which owns the timer?

## New stuff learned

From a late night Zulip conversation learned about `cargo tree`, which was
helpful to find a crate which was included in the developing HSV application but
not needed.  In fact that crate was causing some build time conflicts.

The investigation into project crate dependency tree led to an even more
interesting find.  The local rust toolchain and friends installation creates
a `$HOME/.cargo` configuration and cache directory.  Within 
`$HOME/.cargo/registry` is a collection of many or all of the Rust crate
sources used across the projects on the localhost.  There is a ton of Rust
souce code to study in this registry!  This is helpful since some of the
links at [Crates.io](https://crates.io/) which say "source" do not lead to
source code nor to Github code repositories.
