// Part of Microbit-v2 HSV sample app

// use microbit::display::blocking::Display;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) struct DisplayData {
    led_array: [[u8; 5]; 5],
}

impl DisplayData {
/*
    pub(crate) fn new() -> Self {
        let led_array = [[0; 5]; 5];
        Self {
            led_array,
        }
    }

    pub(crate) fn clear(&mut self) -> [[u8; 5]; 5] {
        for row in 0..5 {
            for col in 0..5 {
                self.led_array[row][col] = 0;
            }
        }
        self.led_array
    }
*/
    pub(crate) fn show_h_for_hue() -> [[u8; 5]; 5] {
        let pattern_h = [
            [0, 1, 0, 1, 0],
            [0, 1, 0, 1, 0],
            [0, 1, 1, 1, 0],
            [0, 1, 0, 1, 0],
            [0, 1, 0, 1, 0],
        ];
        pattern_h
    }

    pub(crate) fn show_s_for_saturation() -> [[u8; 5]; 5] {
        let pattern_s = [
            [0, 0, 1, 1, 0],
            [0, 1, 0, 0, 0],
            [0, 0, 1, 0, 0],
            [0, 0, 0, 1, 0],
            [0, 1, 1, 0, 0],
        ];
        pattern_s
    }

    pub(crate) fn show_v_for_value() -> [[u8; 5]; 5] {
        let pattern_v = [
            [0, 1, 0, 1, 0],
            [0, 1, 0, 1, 0],
            [0, 1, 0, 1, 0],
            [0, 1, 0, 1, 0],
            [0, 0, 1, 0, 0],
        ];
        pattern_v
    }
}
