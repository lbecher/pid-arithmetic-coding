use debug_print::debug_print;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Symbol {
    pub symbol: u8,
    pub count: u32,
    pub probability: f64,
    pub accumulated_probability: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolsTable {
    symbols: Vec<Symbol>,
    symbols_count: u64,
}

impl Clone for SymbolsTable {
    fn clone(&self) -> Self {
        Self {
            symbols: self.symbols.to_vec(),
            symbols_count: self.symbols_count,
        }
    }
}

impl SymbolsTable {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
            symbols_count: 0,
        }
    }

    pub fn get_symbol_by_probability(
        &self,
        probability: f64,
    ) -> u8 {
        let position = self.symbols.iter()
            .position(|s| probability < s.accumulated_probability);
        let index = match position {
            Some(index) => index,
            None => {
                println!("Um símbolo lido não foi encontrado na tabela.");
                std::process::exit(1);
            }
        };
        self.symbols[index].symbol
    }

    pub fn get_accumulated_probabilities(
        &self,
        symbol: u8,
    ) -> (f64, Option<f64>) {
        let index = match self.find_symbol(symbol) {
            Some(index) => index,
            None => {
                println!("Um símbolo lido não foi registrado na tabela.");
                std::process::exit(1);
            }
        };
        let accumulated_probability = self.symbols.get(index).unwrap().accumulated_probability;
        let previous_accumulated_probability = if index > 0 {
            Some(self.symbols.get(index - 1).unwrap().accumulated_probability)
        } else {
            None
        };
        (accumulated_probability, previous_accumulated_probability)
    }

    pub fn get_symbols_count(&self) -> u64 {
        self.symbols_count
    }

    pub fn add_or_increment_symbol(
        &mut self, 
        symbol: u8,
    ) {
        let index = self.find_symbol(symbol);
        match index {
            Some(position) => {
                self.increment_symbol(position);
            }
            None => {
                self.add_symbol(symbol);
            }
        }
        self.symbols_count += 1;
    }

    pub fn calculate_probabilities(
        &mut self,
    ) {
        let mut cumulative_probability = 0.0;
        let n = self.symbols_count as f64;
        for symbol in self.symbols.iter_mut() {
            let n_sigma = symbol.count as f64;
            let probability = n_sigma / n;

            symbol.probability = probability;
            cumulative_probability += probability;
            symbol.accumulated_probability = cumulative_probability;

            debug_print!("\nSímbolo: {}; Quantidade: {}; Probabilidade: {}; Probabilidade acumulada: {}", 
                String::from_utf8(vec![symbol.symbol]).unwrap(), 
                symbol.count,
                symbol.probability,
                symbol.accumulated_probability,
            );
        }
        debug_print!("\n\n");
    }

    fn find_symbol(
        &self, 
        symbol: u8,
    ) -> Option<usize> {
        self.symbols.iter().position(|s| {
            s.symbol == symbol
        })
    }

    fn add_symbol(
        &mut self, 
        symbol: u8,
    ) {
        self.symbols.push(Symbol {
            symbol,
            count: 1,
            probability: 0.0,
            accumulated_probability: 0.0,
        });
    }

    fn increment_symbol(
        &mut self, 
        index: usize,
    ) {
        if let Some(symbol) = self.symbols.get_mut(index) {
            symbol.count += 1;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArithmeticCoding {
    low: u32,
    high: u32,
    high_divisor: u32,
    symbols_table: SymbolsTable,
}

impl ArithmeticCoding {
    pub fn new(
        low: u32,
        high: u32,
    ) -> Self {
        let high_digits = (high as f64).log10() as usize + 1;
        let high_divisor = 10u32.pow((high_digits - 1) as u32);

        let symbols_table = SymbolsTable::new();

        Self {
            low,
            high,
            high_divisor,
            symbols_table,
        }
    }

    pub fn set_low(
        &mut self,
        low: u32,
    ) {
        self.low = low;
    }

    pub fn set_high(
        &mut self,
        high: u32,
    ) {
        self.high = high;
    }

    pub fn get_low(
        &self,
    ) -> u32 {
        self.low
    }

    pub fn get_high(
        &self,
    ) -> u32 {
        self.high
    }

    pub fn get_high_divisor(
        &self,
    ) -> u32 {
        self.high_divisor
    }

    pub fn set_symbols_table(
        &mut self,
        symbols_table: SymbolsTable,
    ) {
        self.symbols_table = symbols_table;
    }

    pub fn add_or_increment_symbol(
        &mut self,
        symbol: u8,
    ) {
        self.symbols_table.add_or_increment_symbol(symbol);
    }

    pub fn calculate_probabilities(
        &mut self,
    ) {
        self.symbols_table.calculate_probabilities();
    }

    pub fn get_symbol_by_probability(
        &self,
        probability: f64,
    ) -> u8 {
        self.symbols_table.get_symbol_by_probability(probability)
    }

    pub fn calculate_arithmetic_coding(
        &mut self,
        symbol: u8,
    ) -> Vec<u32> {
        let (
            accumulated_probability,
            previous_accumulated_probability,
        ) = self.symbols_table.get_accumulated_probabilities(symbol);

        let range = (self.high - self.low + 1) as f64;
        
        let mut new_low = match previous_accumulated_probability {
            Some(previous_accumulated_probability) => {
                self.low + ((range * previous_accumulated_probability) as u32)
            }
            None => {
                self.low
            }
        };
        let mut new_high = self.low + ((range * accumulated_probability) as u32) - 1;

        debug_print!("\n       {} |\t{}\t{}\t|",
            String::from_utf8([symbol].to_vec()).unwrap(),
            new_low,
            new_high,
        );

        let mut emitted_digits: Vec<u32> = Vec::new();

        let mut low_first_digit = new_low / self.high_divisor;
        let mut high_first_digit = new_high / self.high_divisor;

        while low_first_digit == high_first_digit {
            new_low = (new_low - low_first_digit * self.high_divisor) * 10;
            new_high = (new_high - high_first_digit * self.high_divisor) * 10 + 9;

            debug_print!(" {}\n         |\t{}\t{}\t|",
                low_first_digit,
                new_low,
                new_high,
            );

            emitted_digits.push(low_first_digit);

            low_first_digit = new_low / self.high_divisor;
            high_first_digit = new_high / self.high_divisor;
        }

        self.low = new_low;
        self.high = new_high;

        emitted_digits
    }
}