use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::io::Write;

use crate::structures::{
    ArithmeticCodingInfo,
    TableSymbol,
};

#[derive(Debug)]
pub struct ArithmeticEncoder {
    initial_low: u32,
    initial_high: u32,
    low: u32,
    high: u32,
    cumulative_probability: f64,
    probability_table: Vec<TableSymbol>,
    value: u32,
}

impl ArithmeticEncoder {
    pub fn new(
        low: u32, 
        high: u32,
    ) -> Self {
        let mut encoded_data = Vec::new();
        encoded_data.push(0);
        Self {
            initial_low: low,
            initial_high: high,
            low,
            high,
            cumulative_probability: 0.0,
            probability_table: Vec::new(),
            value: 0,
        }
    }

    pub fn encode(
        &mut self, 
        input_file: &mut File, 
        output_file: &mut File,
    ) {
        let bytes = self.inc_or_add_symbol(input_file);
        self.calc_symbol_prob(bytes);
        input_file.seek(std::io::SeekFrom::Start(0)).unwrap();
        self.update_low_and_high(input_file, output_file);
    }

    pub fn get_arithmetic_coding_info(&self) -> ArithmeticCodingInfo {
        ArithmeticCodingInfo {
            low: self.initial_low,
            high: self.initial_high,
            probability_table: self.probability_table.to_vec(),
        }
    }

    fn inc_or_add_symbol(
        &mut self, 
        input_file: &File,
    ) -> usize {
        let reader = BufReader::new(input_file);

        let mut bytes_count = 0;

        for byte in reader.bytes() {
            if let Ok(symbol) = byte {
                let position = self.probability_table
                    .iter()
                    .position(|s| s.symbol == symbol);

                if let Some(index) = position {
                    self.probability_table[index].occurrence += 1;
                } else {
                    self.probability_table.push(TableSymbol {
                        symbol: symbol,
                        occurrence: 1,
                        probability: 0.0,
                        accumulated_probability: 0.0,
                    });
                }

                bytes_count += 1;
            } else {
                break;
            }
        }

        bytes_count
    }

    fn calc_symbol_prob(&mut self, bytes: usize) {
        print!("\n");

        let n = bytes as f64;

        for symbol in self.probability_table.iter_mut() {
            let n_sigma = symbol.occurrence as f64;
            let probability = n_sigma / n;

            symbol.probability = probability;
            self.cumulative_probability += probability;
            symbol.accumulated_probability = self.cumulative_probability;

            println!("Símbolo: {}; Quantidade: {}; Probabilidade: {}; Probabilidade acumulada: {}", 
                String::from_utf8(vec![symbol.symbol]).unwrap(), 
                symbol.occurrence,
                symbol.probability,
                symbol.accumulated_probability,
            );
        }
    }

    fn update_low_and_high(&mut self, input_file: &File, output_file: &mut File) {
        let reader = BufReader::new(input_file);

        for byte in reader.bytes() {
            if let Ok(symbol) = byte {
                let position = self.probability_table
                    .iter()
                    .position(|s| s.symbol == symbol);

                if let Some(index) = position {
                    let symbol = self.probability_table.get(index).unwrap();

                    let range = (self.high - self.low + 1) as f64;

                    let mut new_low = self.low;
                    let mut new_high = self.low + ((range * symbol.accumulated_probability) as u32) - 1;

                    if index > 0 {
                        if let Some(symbol) = self.probability_table.get(index - 1) {
                            new_low = self.low + ((range * symbol.accumulated_probability) as u32);
                        }
                    }

                    print!("\n{} |\t{}\t{} |",
                        String::from_utf8([symbol.symbol].to_vec()).unwrap(),
                        new_low,
                        new_high,
                    );

                    let high_digits = (new_high as f64).log10() as usize + 1;
                    let divisor = 10u32.pow((high_digits - 1) as u32);

                    let mut low_first_digit = new_low / divisor;
                    let mut high_first_digit = new_high / divisor;

                    while low_first_digit == high_first_digit {
                        new_low = (new_low - low_first_digit * divisor) * 10;
                        new_high = (new_high - high_first_digit * divisor) * 10 + 9;

                        print!(" {}\n  |\t{}\t{} |",
                            low_first_digit,
                            new_low,
                            new_high,
                        );

                        match 10u32.checked_mul(self.value) {
                            Some(mul) => {
                                //print!("mul: {}, ", mul);
                                match mul.checked_add(low_first_digit) {
                                    Some(add) => {
                                        //print!("add: {}", add);
                                        self.value = add;
                                    }
                                    None => {
                                        if let Err(e) = output_file.write_all(&self.value.to_le_bytes()) {
                                            eprintln!("Erro ao gravar no arquivo de saída: {}", e);
                                            std::process::exit(1);
                                        };
                                        self.value = low_first_digit;
                                    }
                                }
                            }
                            None => {
                                if let Err(e) = output_file.write_all(&self.value.to_le_bytes()) {
                                    eprintln!("Erro ao gravar no arquivo de saída: {}", e);
                                    std::process::exit(1);
                                };
                                self.value = low_first_digit;
                            }
                        }

                        low_first_digit = new_low / divisor;
                        high_first_digit = new_high / divisor;
                    }

                    self.low = new_low;
                    self.high = new_high;
                }
            } else {
                break;
            }
        }

        if let Err(e) = output_file.write_all(&self.value.to_le_bytes()) {
            eprintln!("Erro ao gravar no arquivo de saída: {}", e);
            std::process::exit(1);
        };

        print!("\n\n");
    }
}
