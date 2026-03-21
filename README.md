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

## Application design requirements

The application is expected to manage and adjust Hue, Saturation and Brightness
values for the user, reading button press events and one of either potentiometer
input or quadrature encoder input.  These three values must be converted to Red,
Green and Blue duty cycles, applied to the individual color LED lines of an RBG
LED.

A wiring diagram accompanies project sources in a `notes` directory.

In list form the HSV application must support:

* User adjustments to color parameters Hue, Saturation and Brightness (Value)
* Microbit buttons A and B to cycle and wrap selected parameter
* MB2 red LED matrix to show H, S or V after parameter selection changes
* Potentiometer or quadrature encoder to adjust selected parameter
* Calculation of RGB values from HSV values
* Normalizing RBG values on a scale ranging 0..100
* An RBG update rate of 100Hz, purely timer driver (no hardware PWM)

## What is done

Functions completed so far include:

* Hue, Saturation and Brightness (Value) selection with wrapping via buttons
* Selected parameter is highlighted by one of 'H', 'S' and 'V' on 5x5 LED matrix
* Parameter adjustment using quadrature encoder
* Conversion of HSV to RGB values using a community provided crate
* A timer-only based PWM function
* Some rather ugly logic to calculate timer periods and effect R, G, and B
   PWM signals

## What remains to do

* Fine tuning of timer based interrupt to give a more exact 100Hz RGB display
   update rate
* Button debouncing
* A better factoring of partial duty cycle calculations, moving these to
   `rgbdisplay` project crate and out of timer1 interrupt service routine
* A revit to the atomic data types used in `main.rs`, some of which may be no
   longer necessary with a proper encapsulation of RGB duty cycle logic in
   its own module
* Make use of rotary encoder push button as a "zero all" feature

2026-03-20 Note: branch `pwm-calc-improvement` implements a better duty cycle
calculation, factored into the RgbDisplay crate.  The timer1 interrupt is
updateing and simplified to make use of this.  Mid-way color glitches are no
longer showing across Hue adjustments.  There is however some condition
causing a panic when Hue reaches its default upper clamped bound of 99.

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

To share stateful data between the app and the `timer1` interrupt service
routine, atomic variables are used.  The first working example in this HSV
project involved atomics of type `usize`.  This was workable, but later involved
some klunky data conversion between this integer type and floating point data.

In contrast to the largely C based Zephyr RTOS, where Zephyr's atomic type
variable is one of unsigned 32-bit integer and Boolean, it appears that Rust
supports a wider range of atomic data types.  It may make sense to change this
project's Hue, Sat, and Val atomic variables from type `usize` to `f32` or
floating point.  Further, a better factoring of duty cycle and RGB frame
schedule may obviate the need for some of the atomics in use now.

The most pressing improvement this project demands is a reworking of the
RgbDisplay module.  At least half of the "next change in frame" logic to
support arbitrary duty cycles per color is implemented in the timer1 ISR.  This
is an anti-pattern.  It makes the code harder to understand and not re-usable
outside of the sample app.

## Hardware limitations

In this demo application a rotary encoder was selected over a potentiometer,
for value adjustments over ranges.  The thinking was that an encoder would
provide finer control.  It may in the sense that it is slow to traverse the
hue, saturation and value ranges which span a hundred steps.  But the encoder
turns out to be a little too slow for convenient use.  It was very difficult
to show much LED output change in a video less than fifteen seconds long.

The other hardware shortfall arises from using a non-logarithmic scale of values
for H, S and V settings.  Going from dimmest LED output to the next step is a
huge jump, and most of the higher steps are hard to distinguish.

## Other documents

There is a short video (about seven seconds long) of the HSV application
running on the Microbit-v2 board with external parts.  The video is located in
this project's `notes` directory.

Also in the notes directory there is an ASCII text wiring diagram of the simple
hardware used to run this tri-color LED demonstration.

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

A later reward in the project comes from seeing the effects of sweeping through
the range of Hue values.  As a programmer and not a graphic artist, over years
the author has most often encountered color in terms of R, G, and B parameters.
Hue adjustments span the rainbow in a manner which single R, G and B
adjustments do not.  Getting to handle and adjust the hardware in person
really brings home this kind of information and meaning.

Final note, `cargo clippy --no-deps` did not do the author wrong!
