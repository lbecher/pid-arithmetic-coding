use debug_print::debug_print;
use std::fs::File;
use std::io::{
    Read,
    Seek,
    Write,
};
use std::mem::size_of;

use arithmetic_coding::ArithmeticCoding;

#[derive(Debug)]
pub struct ArithmeticDecoder {
    arithmetic_coding: ArithmeticCoding,
}

impl ArithmeticDecoder {
    pub fn new(
        arithmetic_coding: ArithmeticCoding,
    ) -> Self {
        Self {
            arithmetic_coding,
        }
    }

    pub fn decode(
        &mut self,
        encoded_data_len: u64,
        input_file: &mut File,
        output_file: &mut File,
    ) {
        input_file.seek(std::io::SeekFrom::Start(0)).unwrap();

        debug_print!("\n Símbolo |\tLow\tHigh\t| Dígito");
        debug_print!("\n---------|----------------------|---------");

        let mut count: u64 = 0;

        while count < encoded_data_len {
            let mut value_buffer: [u8; 4] = [0,0,0,0];
            if let Err(e) = input_file.read_exact(&mut value_buffer) {
                eprintln!("Erro ao ler os dados codificados: {}", e);
                std::process::exit(1);
            };
            let value = u32::from_le_bytes(value_buffer);

            let decoded_simbols = self.decode_current_value(value);
            self.write_decode_symbols(decoded_simbols, output_file);

            count += size_of::<u32>() as u64;
        }

        debug_print!("\n\n");
    }

    fn decode_current_value(
        &mut self,
        mut value: u32,
    ) -> Vec<u8> {
        let mut decoded_simbols: Vec<u8> = Vec::new();

        let high_divisor = self.arithmetic_coding.get_high_divisor();

        let mut code = value;
        let mut code_digits = (code as f64).log10() as usize + 1;
        let mut code_divisor = 10u32.pow((code_digits - 1) as u32);
        if code_divisor > high_divisor {
            code /= code_divisor / high_divisor;
        }
        value %= code_divisor;

        let mut shift = 1;

        while code > self.arithmetic_coding.get_low() && code < self.arithmetic_coding.get_high() {
            let low = self.arithmetic_coding.get_low();
            let high = self.arithmetic_coding.get_high();
            let range = (high - low + 1) as f64;
            let probability = (((code - low + 1) as f64 * 10.0 - 1.0) / range) / 10.0;

            let symbol = self.arithmetic_coding.get_symbol_by_probability(probability);
            decoded_simbols.push(symbol);

            let emitted_digits = self.arithmetic_coding.calculate_arithmetic_coding(symbol);
            for _digit in emitted_digits {
                code = value;

                code_digits = (code as f64).log10() as usize + 1;
                code_divisor = 10u32.pow((code_digits - 1) as u32);

                if code_divisor >= high_divisor {
                    code /= code_divisor / high_divisor;
                } else {
                    code *= 10u32.pow(shift);
                    shift += 1;
                }

                value %= code_divisor;
            }
        }

        decoded_simbols
    }

    fn write_decode_symbols(
        &self, 
        symbols: Vec<u8>,
        output_file: &mut File,
    ) {
        if let Err(e) = output_file.write(&symbols.as_slice()) {
            eprintln!("Não foi possível gravar no arquivo de saída: {}", e);
            std::process::exit(1);
        }
    }
        /*let mut value_buffer: [u8; 4] = [0,0,0,0];
        if let Err(e) = input_file.read_exact(&mut value_buffer) {
            eprintln!("Erro ao obter o tamanho dos dados codificados: {}", e);
            std::process::exit(1);
        };
        let mut value = u32::from_le_bytes(value_buffer);

        let mut count: u64 = 0;

        while count < encoded_data_len {
            let mut code = value;

            let mut code_digits = (code as f64).log10() as usize + 1;
            let mut code_divisor = 10u32.pow((code_digits - 1) as u32);

            if code_divisor > self.high_divisor {
                code /= code_divisor / self.high_divisor;
            }

            value %= code_divisor;

            let mut shift: u32 = 1;

            while code > self.low && code < self.high {
                let range = (self.high - self.low + 1) as f64;
                let probability = (((code - self.low + 1) as f64 * 10.0 - 1.0) / range) / 10.0;

                let position = self.symbols_table
                    .iter()
                    .position(|s| probability < s.accumulated_probability);

                let index = match position {
                    Some(index) => index,
                    None => {
                        println!("Um símbolo lido não foi encontrado na tabela.");
                        std::process::exit(1);
                    }
                };

                let symbol = self.probability_table.get(index).unwrap();

                if let Err(e) = output_file.write(&[symbol.symbol]) {
                    eprintln!("Não foi possível gravar no arquivo de saída: {}", e);
                    std::process::exit(1);
                }

                let mut new_low = self.low;
                let mut new_high = self.low + ((range * symbol.accumulated_probability) as u32) - 1;

                if index > 0 {
                    if let Some(symbol) = self.probability_table.get(index - 1) {
                        new_low = self.low + ((range * symbol.accumulated_probability) as u32);
                    }
                }

                print!("\n{}\t|\t{}\t{}\t|\t{}\t|",
                    String::from_utf8([symbol.symbol].to_vec()).unwrap(),
                    new_low,
                    new_high,
                    code,
                );

                let mut low_first_digit = new_low / self.high_divisor;
                let mut high_first_digit = new_high / self.high_divisor;

                while low_first_digit == high_first_digit {
                    new_low = (new_low - low_first_digit * self.high_divisor) * 10;
                    new_high = (new_high - high_first_digit * self.high_divisor) * 10 + 9;

                    code = value;

                    code_digits = (code as f64).log10() as usize + 1;
                    code_divisor = 10u32.pow((code_digits - 1) as u32);

                    if code_divisor >= self.high_divisor {
                        code /= code_divisor / self.high_divisor;
                    } else {
                        code *= 10u32.pow(shift);
                        shift += 1;
                    }

                    value %= code_divisor;

                    print!("\t{}\n \t|\t{}\t{}\t|\t{}\t|",
                        low_first_digit,
                        new_low,
                        new_high,
                        code,
                    );

                    low_first_digit = new_low / self.high_divisor;
                    high_first_digit = new_high / self.high_divisor;
                }

                self.low = new_low;
                self.high = new_high;
            }


            if let Err(e) = input_file.read_exact(&mut value_buffer) {
                eprintln!("Erro ao obter o tamanho dos dados codificados: {}", e);
                std::process::exit(1);
            };
            value = u32::from_le_bytes(value_buffer);

            count += size_of::<u32>() as u64;
        }
        print!("\n\n");*/
}