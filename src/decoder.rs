use debug_print::debug_print;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::fs::File;

use arithmetic_coding::ArithmeticCoding;

pub struct ArithmeticDecoder {
    ac: ArithmeticCoding,
    code: u32,
    value: u32,
    value_count: u64,
    value_shifts: u32,
    last_value_shifts: u32,
    output: File,
}

impl ArithmeticDecoder {
    pub fn new(ac: ArithmeticCoding, value_count: u64, last_value_shifts: u32, output: File) -> Self {
        let code = 0;
        let value = 0;
        let value_shifts = 0;
        Self {
            ac,
            code,
            value,
            value_count,
            value_shifts,
            last_value_shifts,
            output,
        }
    }
    
    pub fn decode(&mut self, input: &mut File) {
        input.seek(std::io::SeekFrom::Start(0)).unwrap();
        self.read_value_from_file(input);

        for _ in 0..self.ac.precision {
            let bit = self.get_bit_from_value(input);
            self.code = (self.code << 1) | bit;
        }

        debug_print!("\t{:012b}\n", self.code);
        
        debug_print!("\t\t|\t{:012b}\t{:012b}\t|",
            self.ac.low,
            self.ac.high,
        );

        let total = self.ac.symbols.total;
        let mut count: u64 = 0;

        while count < total {
            let low = self.ac.low;
            let high = self.ac.high;

            let range = (high - low + 1) as u64;
            let offset = (self.code - low) as u64;
            let value = ((offset + 1) * (total + 1)) / range;

            let symbol = self.ac.symbols.get_symbol_by_value(value);

            self.write_decoded_symbol(symbol as u8);
            self.update(symbol as u8, input);

            count += 1;
        }

        debug_print!("\n\n");
    }

    fn update(&mut self, symbol: u8, input: &mut File) {
        let (
            low_of_symbol,
            high_of_symbol,
        ) = self.ac.symbols.get_low_and_high(symbol);
    
        let range = (self.ac.high - self.ac.low + 1) as u64;
        let total = self.ac.symbols.total + 1;
        let old_low = self.ac.low;
        
        self.ac.low = old_low + ((low_of_symbol * range) / total) as u32;
        self.ac.high = old_low + ((high_of_symbol * range) / total) as u32 - 1;

        debug_print!("\n\t{}\t|\t{:012b}\t{:012b}\t|",
            symbol,
            self.ac.low,
            self.ac.high,
        );
    
        self.ac.verify_low_and_high();

        while ((self.ac.low ^ self.ac.high) & self.ac.full_bit()) == 0 {
            self.shift(input);

            self.ac.low = (self.ac.low << 1) & self.ac.full_mask();
            self.ac.high = ((self.ac.high << 1) & self.ac.full_mask()) | 1;

            debug_print!("\n\t\t|\t{:012b}\t{:012b}\t|",
                self.ac.low,
                self.ac.high,
            );

            debug_print!("\t{:012b}", self.code);

            self.ac.verify_low_and_high();
        }
        
        while (self.ac.low & !self.ac.high & self.ac.half_bit()) != 0 {
            self.underflow(input);

            self.ac.low = (self.ac.low << 1) & self.ac.half_mask();
            self.ac.high = self.ac.full_bit() | ((self.ac.high << 1) & self.ac.half_mask()) | 1;

            debug_print!("\n\t\t|\t{:012b}\t{:012b}\t|",
                self.ac.low,
                self.ac.high,
            );

            debug_print!("\t{:012b}", self.code);

            self.ac.verify_low_and_high();
        }
    }
    
    fn shift(&mut self, input: &mut File) {
        let bit = self.get_bit_from_value(input);
        self.code = ((self.code << 1) & self.ac.full_mask()) | bit;
    }

    fn underflow(&mut self, input: &mut File) {
        let bit = self.get_bit_from_value(input);
        self.code = (self.code & self.ac.full_bit()) | ((self.code << 1) & self.ac.half_mask()) | bit;
    }

    fn get_bit_from_value(&mut self, input: &mut File) -> u32 {
        let bit: u32;
        if self.value_shifts == 0 {
            bit = self.value;
            self.read_value_from_file(input);
        } else {
            bit = self.value >> self.value_shifts;
            self.value &= u32::MAX >> 32 - self.value_shifts;
            self.value_shifts -= 1;
        }
        bit
    }

    fn read_value_from_file(&mut self, input: &mut File) {
        let mut value_buffer: [u8; 4] = [0,0,0,0];
    
        if let Err(e) = input.read_exact(&mut value_buffer) {
            eprintln!("\nErro ao ler os dados codificados: {}\n", e);
            std::process::exit(1);
        };
        if self.value_count > 0 {
            self.value_count -= 1;
        }

        self.value = u32::from_le_bytes(value_buffer);
        self.value_shifts = if self.is_last_value() {
            self.last_value_shifts - 1
        } else {
            31
        };
    }

    fn is_last_value(&self) -> bool {
        if self.value_count == 0 {
            true
        } else {
            false
        }
    }

    fn write_decoded_symbol(&mut self, symbol: u8) {
        if let Err(e) = self.output.write(&[symbol]) {
            eprintln!("\nNão foi possível gravar no arquivo de saída: {}\n", e);
            std::process::exit(1);
        }
    }
}