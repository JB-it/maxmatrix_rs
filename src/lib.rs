#![no_std]

use embedded_hal::digital::v2::OutputPin;

pub mod maxmatrix {

    pub const MAX7219_REG_NOOP: u8 = 0x00;
    pub const MAX7219_REG_DIGIT0: u8 = 0x01;
    pub const MAX7219_REG_DIGIT1: u8 = 0x02;
    pub const MAX7219_REG_DIGIT2: u8 = 0x03;
    pub const MAX7219_REG_DIGIT3: u8 = 0x04;
    pub const MAX7219_REG_DIGIT4: u8 = 0x05;
    pub const MAX7219_REG_DIGIT5: u8 = 0x06;
    pub const MAX7219_REG_DIGIT6: u8 = 0x07;
    pub const MAX7219_REG_DIGIT7: u8 = 0x08;
    pub const MAX7219_REG_DECODE_MODE: u8 = 0x09;
    pub const MAX7219_REG_INTENSITY: u8 = 0x0a;
    pub const MAX7219_REG_SCAN_LIMIT: u8 = 0x0b;
    pub const MAX7219_REG_SHUTDOWN: u8 = 0x0c;
    pub const MAX7219_REG_DISPLAY_TEST: u8 = 0x0f;

    pub struct MaxMatrix<DataPin, LoadPin, ClockPin> {
        data: DataPin,
        load: LoadPin,
        clock: ClockPin,
        num: u8,
        buffer: [u8; 80],
    }

    impl<DataPin: OutputPin, LoadPin: OutputPin, ClockPin: OutputPin> 
    MaxMatrix<DataPin, LoadPin, ClockPin> {
        pub fn reload(&mut self) {
            for i in 0..8
            {
                let mut col: i32 = i;
                let _ = self.load.set_low();    
                for _ in 0..self.num
                {
                    shift_out(&mut self.data, &mut self.clock, ShiftOrder::MSBFIRST, i as u8 + 1);
                    shift_out(&mut self.data, &mut self.clock, ShiftOrder::MSBFIRST, self.buffer[col as usize]);
                    col += 8;
                }
                let _ = self.load.set_low();
                let _ = self.load.set_high();
            }
        }

        pub fn new(data: DataPin, load: LoadPin, clock: ClockPin, num: u8) -> Self {
            MaxMatrix {
                data: data,
                load: load,
                clock: clock,
                num: num,
                buffer: [0; 80],
            }
        }

        pub fn init(&mut self) {
            let _ = self.clock.set_high();
            
            self.set_command(MAX7219_REG_SCAN_LIMIT, 0x07);
            self.set_command(MAX7219_REG_DECODE_MODE, 0x00);  // using an led matrix (not digits)
            self.set_command(MAX7219_REG_SHUTDOWN, 0x01);    // not in shutdown mode
            self.set_command(MAX7219_REG_DISPLAY_TEST, 0x00); // no display test
            
            // empty registers, turn all LEDs off
            self.clear();
            
            self.set_intensity(0x0f); 
        }

        pub fn clear(&mut self) {
            for i in 0..8 {
                self.set_column_all(i, 0);
            }

            for i in 0..80 {
                self.buffer[i] = 0;
            }
        }

        pub fn set_command(&mut self, command: u8, value: u8) {
            let _ = self.load.set_low();  
            for _ in 0..self.num
            {
                shift_out(&mut self.data, &mut self.clock, ShiftOrder::MSBFIRST, command);
                shift_out(&mut self.data, &mut self.clock, ShiftOrder::MSBFIRST, value);
            }
            let _ = self.load.set_low();
            let _ = self.load.set_high();
        }

        pub fn set_intensity(&mut self, intensity: u8) {
            self.set_command(MAX7219_REG_INTENSITY, intensity);
        }

        pub fn set_column(&mut self, col: u8, value: u8) {
            let n: u8 = col / 8;
            let c: u8 = col % 8;
            let _ = self.load.set_low();  
            for i in 0..self.num
            {
                if i == n
                {
                    shift_out(&mut self.data, &mut self.clock, ShiftOrder::MSBFIRST, c + 1);
                    shift_out(&mut self.data, &mut self.clock, ShiftOrder::MSBFIRST, value);
                }
                else
                {
                    shift_out(&mut self.data, &mut self.clock, ShiftOrder::MSBFIRST, 0);
                    shift_out(&mut self.data, &mut self.clock, ShiftOrder::MSBFIRST, 0);
                }
            }
            let _ = self.load.set_low();
            let _ = self.load.set_high();
            
            self.buffer[col as usize] = value;
        }

        pub fn set_column_all(&mut self, col: u8, value: u8) {
            let _ = self.load.set_low();    
            for i in 0..self.num
            {
                shift_out(&mut self.data, &mut self.clock, ShiftOrder::MSBFIRST, col + 1);
                shift_out(&mut self.data, &mut self.clock, ShiftOrder::MSBFIRST, value);
                self.buffer[(col * i) as usize] = value;
            }
            let _ = self.load.set_low();
            let _ = self.load.set_high();
        }

        pub fn update_buffer_at(&mut self, col: u8, row: u8, value: bool) {
            bit_write(&mut self.buffer[col as usize], row, value);
        }

        pub fn set_dot(&mut self, col: u8, row: u8, value: bool) {
            bit_write(&mut self.buffer[col as usize], row, value);

            let n = col / 8;
            let c = col % 8;
            let _ = self.load.set_low();    
            for i in 0..self.num
            {
                if i == n
                {
                    shift_out(&mut self.data, &mut self.clock, ShiftOrder::MSBFIRST, c + 1);
                    shift_out(&mut self.data, &mut self.clock, ShiftOrder::MSBFIRST, self.buffer[col as usize]);
                }
                else
                {
                    shift_out(&mut self.data, &mut self.clock, ShiftOrder::MSBFIRST, 0);
                    shift_out(&mut self.data, &mut self.clock, ShiftOrder::MSBFIRST, 0);
                }
            }
            let _ = self.load.set_low();
            let _ = self.load.set_high();
        }

        fn write_sprite(&mut self, x: i32, y: i32, sprite: u8) {
            todo!();
        }

        pub fn shift_left(&mut self, rotate: bool, fill_zero: bool) {
            let old: u8 = self.buffer[0];
            for i in 0..79 {
                self.buffer[i] = self.buffer[i+1];
            }
            if rotate {self.buffer[(self.num*8-1) as usize] = old;}
            else if fill_zero {self.buffer[(self.num*8-1) as usize] = 0};
            
            self.reload();
        }

        pub fn shift_right(&mut self, rotate: bool, fill_zero: bool) {
            let last = self.num*8-1;
            let old: u8 = self.buffer[last as usize];
            for i in (1..80).rev() {
                self.buffer[i] = self.buffer[i-1];
            }
            if rotate {self.buffer[0] = old;}
            else if fill_zero {self.buffer[0] = 0;}
            
            self.reload();
        }

        pub fn shift_up(&mut self, rotate: bool) {
            for i in 0..(self.num*8) as usize
            {
                let b = self.buffer[i] & 1 > 0;
                self.buffer[i] >>= 1;
                if rotate {bit_write(&mut self.buffer[i], 7, b);}
            }
            self.reload();
        }

        pub fn shift_down(&mut self, rotate: bool) {        
            for i in 0..(self.num*8) as usize
            {
                let b = self.buffer[i] & 0x80 > 0;
                self.buffer[i] <<= 1;
                if rotate {bit_write(&mut self.buffer[i], 0, b);}
            }
            self.reload();
        }
    }

    #[derive(Eq, PartialEq)]
    pub enum ShiftOrder {
        LSBFIRST,
        MSBFIRST,
    }

    fn shift_out<DataPin: OutputPin, ClockPin: OutputPin>(data_pin: &mut DataPin, clock_pin: &mut ClockPin, bit_order: ShiftOrder, mut val: u8)
    {
        for i in 0..8 {
            if bit_order == ShiftOrder::LSBFIRST {
                if (val & 1) == 1 {
                    data_pin.set_high();
                } else {
                    data_pin.set_low();
                }
                val >>= 1;
            } else {	
                if (val & 128) != 0 {
                    data_pin.set_high();
                } else {
                    data_pin.set_low();
                }
                val <<= 1;
            }
                
            clock_pin.set_high();
            clock_pin.set_low();		
        }
    }

    fn bit_write(value: &mut u8, bit: u8, bitvalue: bool) {
        if bitvalue {
            *value |= 1u8 << (bit);
        } else {
            *value &= !(1u8 << (bit));
        }
    }

    fn bit_write_u16(value: &mut u16, bit: u8, bitvalue: bool) {
        if bitvalue {
            *value |= 1u16 << (bit);
        } else {
            *value &= !(1u16 << (bit));
        }
    }
}