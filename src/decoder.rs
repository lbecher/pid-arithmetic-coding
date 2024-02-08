use debug_print::debug_print;
use std::fs::File;
use std::io::{
    Read,
    Seek,
    Write,
};

use arithmetic_coding::ArithmeticCoding;

#[derive(Debug)]
pub struct ArithmeticDecoder {
    arithmetic_coding: ArithmeticCoding,
    last_value_digits: u8,
    bytes_count: u64,
    current_bytes_count: u64,
    code: u32,
    current_value: Vec<u32>,
}

impl ArithmeticDecoder {
    pub fn new(
        arithmetic_coding: ArithmeticCoding,
        last_value_digits: u8,
        bytes_count: u64,
    ) -> Self {
        Self {
            arithmetic_coding,
            last_value_digits,
            bytes_count,
            current_bytes_count: 0,
            code: 0,
            current_value: Vec::new(),
        }
    }

    pub fn decode(
        &mut self,
        input_file: &mut File,
        output_file: &mut File,
    ) {
        input_file.seek(std::io::SeekFrom::Start(0)).unwrap();

        for _ in 0..self.arithmetic_coding.get_high_digits() {
            self.code *= 10;
            self.code += self.get_next_digit_from_current_value(input_file);
        }

        debug_print!("\n\tu8\t|\tLow\tHigh\t|");
        debug_print!("\n\t\t|\t\t\t|");

        let symbols_count = self.arithmetic_coding.get_symbols_count();
        let mut decoded_simbols_count: u64 = 0;

        while decoded_simbols_count < symbols_count {
            let low = self.arithmetic_coding.get_low();
            let high = self.arithmetic_coding.get_high();

            let range = (high - low + 1) as f64;

            let mut probability = (self.code - low + 1) as f64;
            probability *= symbols_count as f64;
            probability -= 1.0;
            probability /= range;
            probability /= symbols_count as f64;

            let symbol = self.arithmetic_coding.get_symbol_by_probability(probability);
            self.write_decode_symbol(symbol, output_file);

            let emitted_digits = self.arithmetic_coding
                .calculate_arithmetic_coding(symbol);
            self.handle_emitted_digits(emitted_digits, input_file);

            decoded_simbols_count += 1;
        }

        debug_print!("\n\n");
    }

    fn handle_emitted_digits(
        &mut self,
        emitted_digits: Vec<u32>,
        input_file: &mut File,
    ) {
        for _digit in emitted_digits {
            self.code %= self.arithmetic_coding.get_high_divisor();
            self.code *= 10;
            self.code += self.get_next_digit_from_current_value(input_file);
        }
    }

    fn get_next_digit_from_current_value(
        &mut self,
        input_file: &mut File,
    ) -> u32 {
        if self.current_value.len() > 0 {
            self.current_value.remove(0)
        } else {
            self.read_next_value(input_file);
            self.current_value.remove(0)
        }
    }

    fn read_next_value(
        &mut self,
        input_file: &mut File,
    ) {
        let mut value_buffer: [u8; 4] = [0,0,0,0];
        
        if self.current_bytes_count < self.bytes_count {
            if let Err(e) = input_file.read_exact(&mut value_buffer) {
                eprintln!("\nErro ao ler os dados codificados: {}\n", e);
                std::process::exit(1);
            };
            self.current_bytes_count += std::mem::size_of::<u32>() as u64;
        }

        let mut value = u32::from_le_bytes(value_buffer);

        let max = if self.current_bytes_count < self.bytes_count {
            9
        } else {
            self.last_value_digits
        };

        for _ in 0..max {
            self.current_value.insert(0, value % 10);
            value /= 10;
        }
    }

    fn write_decode_symbol(
        &self, 
        symbol: u8,
        output_file: &mut File,
    ) {
        if let Err(e) = output_file.write(&[symbol]) {
            eprintln!("\nNão foi possível gravar no arquivo de saída: {}\n", e);
            std::process::exit(1);
        }
    }
}