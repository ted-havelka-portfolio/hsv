# HSV

Hue, Saturation and Value (brightness) demo application, written by
Theodore M Havelka, with much help from Rust Discovery-MB2 book and Crates.io.
This project represents a bare metal application exercising

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
* Potentiometer or quadrature encoder adjusts selected parameter
* RGB values are calculated from HSV values
* RBG values are normalized on a scale ranging 0..100
* RBG LEDs are driven at 100Hz, each with their respective duty cycle

## What is done

Functions completed so far include:

* Hue, Saturation and Brightness (Value) selection with wrapping via buttons
* Parameter adjustment using quadrature encoder
* A rough timer-only based PWM function

## What is missing

* Conversion of HSV to RGB values
* Fine tuning of timer based interrupt to give 100Hz RGB display update rate

## Most challenging

## New stuff learned
