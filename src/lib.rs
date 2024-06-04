//! This Library enables communication between a Microcontroller and the MAX7219 LED Matrix Driver.
//!
//! ## Example:
//! ```no_run
//! #![no_std]
//! #![no_main]
//! 
//! use panic_halt as _;
//! 
//! use maxmatrix_rs::*;
//! 
//! #[arduino_hal::entry]
//! fn main() -> ! {
//!     let dp = arduino_hal::Peripherals::take().unwrap();
//!     let pins = arduino_hal::pins!(dp);
//!    
//!     let din = pins.d3.into_output();
//!     let cs = pins.d5.into_output();
//!     let clk = pins.d6.into_output();
//!
//!     let mut matrix = MaxMatrix::new(din, cs, clk, 4);
//!     matrix.init();
//!
//!     loop {
//!         matrix.set_dot(0, 0, true);
//!         arduino_hal::delay_ms(500);
//!         matrix.set_dot(0, 0, false);
//!         arduino_hal::delay_ms(500);
//!     }
//! }
//! ```

#![no_std]

use embedded_hal::digital::v2::OutputPin;

// const MAX7219_REG_NOOP: u8 = 0x00;
// const MAX7219_REG_DIGIT0: u8 = 0x01;
// const MAX7219_REG_DIGIT1: u8 = 0x02;
// const MAX7219_REG_DIGIT2: u8 = 0x03;
// const MAX7219_REG_DIGIT3: u8 = 0x04;
// const MAX7219_REG_DIGIT4: u8 = 0x05;
// const MAX7219_REG_DIGIT5: u8 = 0x06;
// const MAX7219_REG_DIGIT6: u8 = 0x07;
// const MAX7219_REG_DIGIT7: u8 = 0x08;
const MAX7219_REG_DECODE_MODE: u8 = 0x09;
const MAX7219_REG_INTENSITY: u8 = 0x0a;
const MAX7219_REG_SCAN_LIMIT: u8 = 0x0b;
const MAX7219_REG_SHUTDOWN: u8 = 0x0c;
const MAX7219_REG_DISPLAY_TEST: u8 = 0x0f;

/// The Struct to communicate with the MAX7219 LED Matrix
pub struct MaxMatrix<DataPin, LoadPin, ClockPin> {
    /// The data pin
    data: DataPin,
    /// The load pin
    load: LoadPin,
    /// The clock pin
    clock: ClockPin,
    /// The amount of LED Matrices connected
    num_panels: u8,
    /// Buffer for the Pixel data
    data_buffer: [u8; 80],
}

impl<DataPin: OutputPin, LoadPin: OutputPin, ClockPin: OutputPin>
    MaxMatrix<DataPin, LoadPin, ClockPin>
{
    /// Reload the data from the buffer to the LED Matrix
    pub fn reload(&mut self) {
        for i in 0..8 {
            let mut col: i32 = i;
            let _ = self.load.set_low();
            for _ in 0..self.num_panels {
                shift_out(
                    &mut self.data,
                    &mut self.clock,
                    ShiftOrder::MSBFIRST,
                    i as u8 + 1,
                );
                shift_out(
                    &mut self.data,
                    &mut self.clock,
                    ShiftOrder::MSBFIRST,
                    self.data_buffer[col as usize],
                );
                col += 8;
            }
            let _ = self.load.set_low();
            let _ = self.load.set_high();
        }
    }

    /// Creates a new instance of the MaxMatrix struct<br/>
    /// <italic>data</italic> - The data pin<br/>
    /// <italic>load</italic> - The load pin<br/>
    /// <italic>clock</italic> - The clock pin<br/>
    /// <italic>num</italic> - The amount of LED Matrices connected<br/>
    pub fn new(data: DataPin, load: LoadPin, clock: ClockPin, num: u8) -> Self {
        MaxMatrix {
            data: data,
            load: load,
            clock: clock,
            num_panels: num,
            data_buffer: [0; 80],
        }
    }

    /// Initializes the LED Matrix with default values
    pub fn init(&mut self) {
        let _ = self.clock.set_high();

        self.set_command(MAX7219_REG_SCAN_LIMIT, 0x07);
        self.set_command(MAX7219_REG_DECODE_MODE, 0x00); // using an led matrix (not digits)
        self.set_command(MAX7219_REG_SHUTDOWN, 0x01); // not in shutdown mode
        self.set_command(MAX7219_REG_DISPLAY_TEST, 0x00); // no display test

        self.clear();

        self.set_intensity(0x0f);
    }

    /// Clears the Buffer and the LED Matrix
    pub fn clear(&mut self) {
        for i in 0..8 {
            self.set_column_all(i, 0);
        }

        for i in 0..80 {
            self.data_buffer[i] = 0;
        }
    }

    fn set_command(&mut self, command: u8, value: u8) {
        let _ = self.load.set_low();
        for _ in 0..self.num_panels {
            shift_out(
                &mut self.data,
                &mut self.clock,
                ShiftOrder::MSBFIRST,
                command,
            );
            shift_out(&mut self.data, &mut self.clock, ShiftOrder::MSBFIRST, value);
        }
        let _ = self.load.set_low();
        let _ = self.load.set_high();
    }

    /// Sets the brightness of the LED Matrix
    pub fn set_intensity(&mut self, intensity: u8) {
        self.set_command(MAX7219_REG_INTENSITY, intensity);
    }

    /// Sets an entire column of the LED Matrix <br/>
    /// The value is a byte where each bit represents a pixel
    pub fn set_column(&mut self, col: u8, value: u8) {
        let n: u8 = col / 8;
        let c: u8 = col % 8;
        let _ = self.load.set_low();
        for i in 0..self.num_panels {
            if i == n {
                shift_out(&mut self.data, &mut self.clock, ShiftOrder::MSBFIRST, c + 1);
                shift_out(&mut self.data, &mut self.clock, ShiftOrder::MSBFIRST, value);
            } else {
                shift_out(&mut self.data, &mut self.clock, ShiftOrder::MSBFIRST, 0);
                shift_out(&mut self.data, &mut self.clock, ShiftOrder::MSBFIRST, 0);
            }
        }
        let _ = self.load.set_low();
        let _ = self.load.set_high();

        self.data_buffer[col as usize] = value;
    }

    /// Sets a colum of all connected LED Matrices
    pub fn set_column_all(&mut self, col: u8, value: u8) {
        let _ = self.load.set_low();
        for i in 0..self.num_panels {
            shift_out(
                &mut self.data,
                &mut self.clock,
                ShiftOrder::MSBFIRST,
                col + 1,
            );
            shift_out(&mut self.data, &mut self.clock, ShiftOrder::MSBFIRST, value);
            self.data_buffer[(col * i) as usize] = value;
        }
        let _ = self.load.set_low();
        let _ = self.load.set_high();
    }

    /// Updates the buffer at a specified position <br/>
    /// This does not automatically refresh the displays
    pub fn update_buffer_at(&mut self, col: u8, row: u8, value: bool) {
        bit_write(&mut self.data_buffer[col as usize], row, value);
    }

    /// Sets a pixel at a specified position and refreshes the displays
    pub fn set_dot(&mut self, col: u8, row: u8, value: bool) {
        bit_write(&mut self.data_buffer[col as usize], row, value);

        let n = col / 8;
        let c = col % 8;
        let _ = self.load.set_low();
        for i in 0..self.num_panels {
            if i == n {
                shift_out(&mut self.data, &mut self.clock, ShiftOrder::MSBFIRST, c + 1);
                shift_out(
                    &mut self.data,
                    &mut self.clock,
                    ShiftOrder::MSBFIRST,
                    self.data_buffer[col as usize],
                );
            } else {
                shift_out(&mut self.data, &mut self.clock, ShiftOrder::MSBFIRST, 0);
                shift_out(&mut self.data, &mut self.clock, ShiftOrder::MSBFIRST, 0);
            }
        }
        let _ = self.load.set_low();
        let _ = self.load.set_high();
    }

    #[allow(dead_code, unused_variables)]
    fn write_sprite(&mut self, x: i32, y: i32, sprite: u8) {
        todo!();
    }

    /// Shifts the entire buffer to the left <br/>
    /// <italic>rotate</italic> - If true, the last column will be moved to the first position <br/>
    /// <italic>fill_zero</italic> - If true, the last column will be set to 0 <br/>
    pub fn shift_left(&mut self, rotate: bool, fill_zero: bool) {
        let old: u8 = self.data_buffer[0];
        for i in 0..79 {
            self.data_buffer[i] = self.data_buffer[i + 1];
        }
        if rotate {
            self.data_buffer[(self.num_panels * 8 - 1) as usize] = old;
        } else if fill_zero {
            self.data_buffer[(self.num_panels * 8 - 1) as usize] = 0
        };

        self.reload();
    }

    /// Shifts the entire buffer to the right <br/>
    /// <italic>rotate</italic> - If true, the last column will be moved to the first position <br/>
    /// <italic>fill_zero</italic> - If true, the last column will be set to 0 <br/>
    pub fn shift_right(&mut self, rotate: bool, fill_zero: bool) {
        let last = self.num_panels * 8 - 1;
        let old: u8 = self.data_buffer[last as usize];
        for i in (1..80).rev() {
            self.data_buffer[i] = self.data_buffer[i - 1];
        }
        if rotate {
            self.data_buffer[0] = old;
        } else if fill_zero {
            self.data_buffer[0] = 0;
        }

        self.reload();
    }

    /// Shifts the entire buffer up <br/>
    /// <italic>rotate</italic> - If true, the last row will be moved to the first position <br/>
    pub fn shift_up(&mut self, rotate: bool) {
        for i in 0..(self.num_panels * 8) as usize {
            let b = self.data_buffer[i] & 1 > 0;
            self.data_buffer[i] >>= 1;
            if rotate {
                bit_write(&mut self.data_buffer[i], 7, b);
            }
        }
        self.reload();
    }

    /// Shifts the entire buffer down <br/>
    /// <italic>rotate</italic> - If true, the last row will be moved to the first position <br/>
    pub fn shift_down(&mut self, rotate: bool) {
        for i in 0..(self.num_panels * 8) as usize {
            let b = self.data_buffer[i] & 0x80 > 0;
            self.data_buffer[i] <<= 1;
            if rotate {
                bit_write(&mut self.data_buffer[i], 0, b);
            }
        }
        self.reload();
    }
}

#[derive(Eq, PartialEq)]
enum ShiftOrder {
    LSBFIRST,
    MSBFIRST,
}

fn shift_out<DataPin: OutputPin, ClockPin: OutputPin>(
    data_pin: &mut DataPin,
    clock_pin: &mut ClockPin,
    bit_order: ShiftOrder,
    mut val: u8,
) {
    for _ in 0..8 {
        if bit_order == ShiftOrder::LSBFIRST {
            if (val & 1) == 1 {
                let _ = data_pin.set_high();
            } else {
                let _ = data_pin.set_low();
            }
            val >>= 1;
        } else {
            if (val & 128) != 0 {
                let _ = data_pin.set_high();
            } else {
                let _ = data_pin.set_low();
            }
            val <<= 1;
        }

        let _ = clock_pin.set_high();
        let _ = clock_pin.set_low();
    }
}

fn bit_write(value: &mut u8, bit: u8, bitvalue: bool) {
    if bitvalue {
        *value |= 1u8 << (bit);
    } else {
        *value &= !(1u8 << (bit));
    }
}
