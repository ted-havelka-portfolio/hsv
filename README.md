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
* Conversion of HSV to RGB values
* A timer-only based PWM function
* Some rather ugly logic to calculate timer periods and effect R, G, and B
   PWM signals.

## What remains to do

* Fine tuning of timer based interrupt to give 100Hz RGB display update rate
* Button debouncing
* A better factoring of partial duty cycle calculations, moving these to
   `rgbdisplay` project crate and out of timer1 interrupt service routine.

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
accomplish this, author Ted H was not able to find a correct syntax to update
the timer in the RgbDisplay module.

The most pressing improvement this project demands is a reworking of the
RgbDisplay module.  At least half of the "next change in frame" logic to
support arbitrary duty cycles per color is implemented in the timer1 ISR.  This
is an anti-pattern.  It makes the code harder to understand and not re-usable
outside of the sample app.

## Hardware limitations

In this demo application a rotary encoder was selected over a potentiometer,
for value adustments over ranges.  The thinking was that an encoder would
provide finer control.  It may in the sense that it is slow to traverse the
hue, saturation and value ranges which span a hundred steps.  But the encoder
turns out to be a little too slow for convenient use.  It was very difficult
to show much LED output change in a video less than fifteen seconds long.

The other hardware shortfall, is in using a non-logarithmic scale of values
for H, S and V settings.  Going from dimmest LED output to the next step is a
huge jump, and most of the higher steps are hard to distinguish.

## New stuff learned

New topics of interest from the HSV demo development:

- `cargo tree` to analyze crate dependencies on local host

- Local dot cargo registry

- "Hue" parameter in action

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

The a later reward in the project comes from seeing the effects of sweeping
through the range of Hue values.  As a programmer and not a graphic artist, over
years the author has more often encountered color online interms of R, G, and B
parameters.  Hue come close to spanning the rainbow in a manner which to adjust
R, G and B values one at a time does not.  Getting touch and adjust the hardware
in person really brings this home!
