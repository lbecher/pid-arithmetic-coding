use std::fs::File;
use std::io::{Read, Write};
use std::mem::size_of;

use crate::structures::TableSymbol;

#[derive(Debug)]
pub struct ArithmeticDecoder {
    low: u32, 
    high: u32, 
    high_divisor: u32, 
    probability_table: Vec<TableSymbol>,
}

impl ArithmeticDecoder {
    pub fn new(
        low: u32, 
        high: u32, 
        probability_table: Vec<TableSymbol>,
    ) -> Self {
        let high_digits = (high as f64).log10() as usize + 1;
        let high_divisor = 10u32.pow((high_digits - 1) as u32);
        Self {
            low,
            high,
            high_divisor,
            probability_table,
        }
    }

    pub fn decode(
        &mut self, 
        encoded_data_len: u64, 
        input_file: &mut File, 
        output_file: &mut File,
    ) {
        let mut value_buffer: [u8; 4] = [0,0,0,0];
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

                let position = self.probability_table
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
        print!("\n\n");
    }
}