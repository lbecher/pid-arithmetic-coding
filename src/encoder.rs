use bincode::serialize_into;
use debug_print::debug_print;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::fs::File;

use arithmetic_coding::ArithmeticCoding;

pub struct ArithmeticEncoder {
    ac: ArithmeticCoding,
    initial_low: u32,
    initial_high: u32,
    value: u32,
    value_shifts: u32,
    underflow_count: u32,
    output: File,
}

impl ArithmeticEncoder {
    pub fn new(low: u32, high: u32, output: File) -> Self {
        let ac = ArithmeticCoding::new(low, high);
        let initial_low = low;
        let initial_high = high;
        let value = 0;
        let value_shifts = 0;
        let underflow_count = 0;
        Self {
            ac,
            initial_low,
            initial_high,
            value,
            value_shifts,
            underflow_count,
            output,
        }
    }
    
    pub fn encode(&mut self, input: &mut File) {
        input.seek(std::io::SeekFrom::Start(0)).unwrap();
        self.generate_table(input);
        input.seek(std::io::SeekFrom::Start(0)).unwrap();

        debug_print!("\n\t\t|\t{:012b}\t{:012b}\t|",
            self.ac.low,
            self.ac.high,
        );
        
        let reader = BufReader::new(input);
        for byte in reader.bytes() {
            match byte {
                Ok(byte) => {
                    self.update(byte);
                }
                Err(e) => {
                    eprintln!("\nErro ao ler o arquivo de entrada: {}\n", e);
                    std::process::exit(1);
                }
            }
        }
        
        self.finish();
        self.add_bit_to_value(1); // gambiarra inexplicável

        debug_print!("\n\n");
    }

    fn generate_table(&mut self, input: &mut File) {
        let reader = BufReader::new(input);
        for byte in reader.bytes() {
            match byte {
                Ok(byte) => {
                    self.ac.symbols.add_symbol(byte);
                }
                Err(e) => {
                    eprintln!("\nErro ao ler o arquivo de entrada: {}\n", e);
                    std::process::exit(1);
                }
            }
        }
        self.ac.symbols.calculate_accumulated_frequency();
    }

    fn update(&mut self, symbol: u8) {
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
            self.shift();

            self.ac.low = (self.ac.low << 1) & self.ac.full_mask();
            self.ac.high = ((self.ac.high << 1) & self.ac.full_mask()) | 1;

            debug_print!("\n\t\t|\t{:012b}\t{:012b}\t|",
                self.ac.low,
                self.ac.high,
            );

            self.ac.verify_low_and_high();
        }
        
        while (self.ac.low & !self.ac.high & self.ac.half_bit()) != 0 {
            self.underflow();

            self.ac.low = (self.ac.low << 1) & self.ac.half_mask();
            self.ac.high = self.ac.full_bit() | ((self.ac.high << 1) & self.ac.half_mask()) | 1;

            debug_print!("\t.\n\t\t|\t{:012b}\t{:012b}\t|",
                self.ac.low,
                self.ac.high,
            );

            self.ac.verify_low_and_high();
        }
    }
    
    fn shift(&mut self) {
        let bit = self.ac.low >> (self.ac.precision - 1);
        self.add_bit_to_value(bit);

        debug_print!("\t{}", bit);
        
        for _ in 0..self.underflow_count {
            let underflow_bit = bit ^ 1;
            self.add_bit_to_value(underflow_bit);
            debug_print!("\n\t\t|\t{:012b}\t{:012b}\t|\t{}",
                self.ac.low,
                self.ac.high,
                underflow_bit,
            );
        }

        self.underflow_count = 0;
    }

    fn underflow(&mut self) {
        self.underflow_count += 1;
    }

    fn add_bit_to_value(&mut self, bit: u32) {
        if self.value_shifts < 32 {
            self.value = (self.value << 1) | bit;
            self.value_shifts += 1;
        } else {
            self.write_value_to_file();
            self.value = bit;
            self.value_shifts = 1;
        }
    }

    fn finish(&mut self) {
        self.write_value_to_file();

        // obtém tamanho da região codificada do arquivo
        let encoded_data_len = match self.output.metadata() {
            Ok(metadata) => metadata.len(),
            Err(e) => {
                eprintln!("\nErro ao obter metadados do arquivo de saída: {}\n", e);
                std::process::exit(1);
            }
        };
        println!("\nTamanho dos dados codificados: {} bytes.", encoded_data_len);

        // grava estrutura de dados no arquivo de saída
        let mut ac = self.ac.clone();
        ac.low = self.initial_low;
        ac.high = self.initial_high;
        if let Err(e) = serialize_into(&mut self.output, &ac) {
            eprintln!("\nErro ao gravar no arquivo de saída: {}\n", e);
            std::process::exit(1);
        };

        // obtém tamanho da estrutura de dados
        let symbols_table_len = match self.output.metadata() {
            Ok(metadata) => metadata.len() - encoded_data_len,
            Err(e) => {
                eprintln!("\nErro ao obter metadados do arquivo de saída: {}\n", e);
                std::process::exit(1);
            }
        };
        println!("Tamanho da tabela de símbolos: {} bytes.\n", symbols_table_len);

        // grava tamanho da região codificada no arquivo de saída
        if let Err(e) = self.output.write_all(&encoded_data_len.to_le_bytes()) {
            eprintln!("\nErro ao gravar no arquivo de saída: {}\n", e);
            std::process::exit(1);
        };

        // grava quantidade de dígitos validos do último byte no arquivo de saída
        if let Err(e) = self.output.write_all(&(self.value_shifts as u8).to_le_bytes()) {
            eprintln!("\nErro ao gravar no arquivo de saída: {}\n", e);
            std::process::exit(1);
        };
    }

    fn write_value_to_file(&mut self) {
        let value_buffer = self.value.to_le_bytes();
        if let Err(e) = self.output.write_all(&value_buffer) {
            eprintln!("\nErro ao gravar no arquivo de saída: {}\n", e);
            std::process::exit(1);
        };
    }

    pub fn verify_file_len(&self, file_len: u64) {
        if file_len >= self.ac.half_bit() as u64 {
            print!("\nO arquivo é muito grande para a precisão escolhida!\n\n");
            std::process::exit(1);
        };
    }
}