# Microbit-v2 HSV Demo Application wiring

The HSB demonstration written in Rust runs on a Microbit-v2 board, attached to
a Microbit "T type" GPIO board.  The specifications for this breakout board
are difficult to locate online.  At time of writing (2026 Q1) one site selling
the breakout board is:

* https://ielectrony.com/en/product/micro-bit-t-type-gpio-board/

For the Hue, Saturation, Value demonstration the following wiring is employed
to connect an RGB light emitting diode and a rotary encoder (part number not
known) to the MB2.  Ascii text is used to convery the wiring:


```
                                                                               .
               3V3                                                             .
                ^                                                              .
                |                                                              .
                |    +------------------------------------+                    .
                \    |                                    |                    .
             1K /    +-------------+        +-------------+                    .
                \                  |5V   3V3|                                  .
    --------    |                  |GND  GND|                                  .
  /       5V \--+      +-----------|P0   GND|                                  .
 |        KEY |--------+  +--------|P1   P20|                                  .
 |        S2  |-----------+        |P2   P19|                                  .
 |        S1  |                    |P3   P16|                                  .
  \       GND/--+                  |P4   P15|                                  .
    --------    |                  |P5   P14|                                  .
    Rotary      |                  |P6   P13|   1K                             .
   encoder     ---         1K      |P7   P12|---/\/--+                         .
                -   +------/\/-----|P8   P11|        |                         .
                    |     +--/\/---|P9   P10|        |                         .
                    |     |  1K    |5V   3V3|        |            3V3          .
                    |     |        |        |        |             ^           .
                    |     |        +--------+        |             |           .
                    |     |                          |             |           .
                ^^  | ^^  |                      ^^  |             |           .
           CR1.A \\--- \\--- CR1.B                \\--- CR1.C  CR1.Anode       .
            BLUE   / \   / \ GREEN                  / \  RED                   .
                   ---   ---                        ---                        .
                    |     |                          |                         .
                   ---   ---                        ---                        .
                    -     -                          -                         .
                                                                               .
```



Notes:

1. It is Ok to connect rotary encoder 5V pin to 3V3 as the encoder is only a
mechanical switch, and we need its output connected to the MB2 to agree with
the typical 3.3 volt power rail and GPIO pin voltage ratings.

2. CR1 designates an RGB LED, a single four-lead package.  The sub-designations
`.A`, `.B`, `.C` may differ on the datasheet.  They are here on the diagram to
emphasize that the three electrical LEDs come housed in one package.

3. No datasheet found at ielectrony.com site (2026 Q1) but an accurate photo of
the GPIO breakout board appeared there at time of writing.
