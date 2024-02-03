use debug_print::debug_print;
use std::fs::File;
use std::io::{
    BufReader, 
    Read, 
    Seek, 
    Write,
};

use arithmetic_coding::ArithmeticCoding;

#[derive(Debug)]
pub struct ArithmeticEncoder {
    initial_low: u32,
    initial_high: u32,
    arithmetic_coding: ArithmeticCoding,
    current_encoded_value: Option<u32>,
}

impl ArithmeticEncoder {
    pub fn new(
        low: u32, 
        high: u32,
    ) -> Self {
        let arithmetic_coding = ArithmeticCoding::new(low, high);
        Self {
            initial_low: low,
            initial_high: high,
            arithmetic_coding,
            current_encoded_value: None,
        }
    }

    pub fn get_arithmetic_coding(
        &self,
    ) -> ArithmeticCoding {
        let mut arithmetic_coding = self.arithmetic_coding.clone();
        arithmetic_coding.set_low(self.initial_low);
        arithmetic_coding.set_high(self.initial_high);
        arithmetic_coding
    }

    pub fn encode(
        &mut self, 
        input_file: &mut File, 
        output_file: &mut File,
    ) {
        input_file.seek(std::io::SeekFrom::Start(0)).unwrap();
        self.generate_symbol_table(input_file);
        input_file.seek(std::io::SeekFrom::Start(0)).unwrap();

        debug_print!("\n Símbolo |\tLow\tHigh\t| Dígito");
        debug_print!("\n---------|----------------------|---------");

        let reader = BufReader::new(input_file);
        for byte in reader.bytes() {
            match byte {
                Ok(byte) => {
                    let emitted_digits = self.arithmetic_coding.calculate_arithmetic_coding(byte);
                    self.handle_emitted_digits(emitted_digits, output_file);
                }
                Err(e) => {
                    eprintln!("Erro ao ler o arquivo de entrada: {}", e);
                    std::process::exit(1);
                }
            }
        }
        self.write_current_encoded_value(output_file);

        debug_print!("\n\n");
    }

    fn generate_symbol_table(
        &mut self, 
        input_file: &File,
    ) {
        let reader = BufReader::new(input_file);
        for byte in reader.bytes() {
            match byte {
                Ok(byte) => {
                    self.arithmetic_coding.add_or_increment_symbol(byte);
                }
                Err(e) => {
                    eprintln!("Erro ao ler o arquivo de entrada: {}", e);
                    std::process::exit(1);
                }
            }
        }
        self.arithmetic_coding.calculate_probabilities();
    }

    fn handle_emitted_digits(
        &mut self,
        emitted_digits: Vec<u32>,
        output_file: &mut File,
    ) {
        for digit in emitted_digits {
            if let Some(current_encoded_value) = self.current_encoded_value {
                match 10u32.checked_mul(current_encoded_value) {
                    Some(mul) => {
                        match mul.checked_add(digit) {
                            Some(add) => {
                                self.current_encoded_value = Some(add);
                            }
                            None => {
                                self.write_current_encoded_value(output_file);
                                self.current_encoded_value = Some(digit);
                            }
                        }
                    }
                    None => {
                        self.write_current_encoded_value(output_file);
                        self.current_encoded_value = Some(digit);
                    }
                }
            } else {
                self.current_encoded_value = Some(digit);
            }
        }
    }

    fn write_current_encoded_value(
        &self, 
        output_file: &mut File,
    ) {
        if let Some(encoded_value) = self.current_encoded_value {
            let encoded_value_buffer = encoded_value.to_le_bytes();
            if let Err(e) = output_file.write_all(&encoded_value_buffer) {
                eprintln!("Erro ao gravar no arquivo de saída: {}", e);
                std::process::exit(1);
            };
        }
    }
}
