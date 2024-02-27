use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone)]
pub enum Operation {
    Decode,
    Encode,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Symbols {
    pub table: Vec<(u8, u64)>,
    pub total: u64,
}

impl Clone for Symbols {
    fn clone(&self) -> Self {
        let table = self.table.to_vec();
        let total = self.total;
        Self {
            table,
            total,
        }
    }
}

impl Symbols {
    pub fn new() -> Self {
        let table: Vec<(u8, u64)> = Vec::new();
        let total: u64 = 0;
        Self {
            table,
            total,
        }
    }

    pub fn add_symbol(&mut self, symbol: u8) {
        let position = self.table.iter().position(|s| {
            s.0 == symbol
        });

        match position {
            Some(index) => {
                self.table[index].1 += 1;
            }
            None => {
                self.table.push((symbol, 1));
            }
        }

        self.total += 1;
    }
    
    pub fn calculate_accumulated_frequency(&mut self) {
        self.table.sort_by(|a, b| a.0.cmp(&b.0));
        let mut accumulated_frequency: u64 = 0;
        for item in self.table.iter_mut() {
            item.1 += accumulated_frequency;
            accumulated_frequency = item.1;
        }
    }

    pub fn get_low_and_high(&self, symbol: u8) -> (u64, u64) {
        let position = self.table.iter().position(|s| {
            s.0 == symbol
        });

        match position {
            Some(index) => {
                let low = if index == 0 {
                    0
                } else {
                    self.table[index - 1].1
                };
                let high = self.table[index].1;
                (low, high)
            }
            None => {
                println!("\n\nERRO: Símbolo não encontrado!\n");
                std::process::exit(1);
            }
        }
    }

    pub fn get_symbol_by_value(&mut self, value: u64) -> u8 {
        let position = self.table.iter().position(|s| {
            value < s.1
        });

        match position {
            Some(index) => {
                self.table[index].0
            }
            None => {
                println!("\n\nERRO: Símbolo não encontrado!\n");
                std::process::exit(1);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArithmeticCoding {
    pub low: u32,
    pub high: u32,
    pub precision: u32,
    pub symbols: Symbols,
}

impl ArithmeticCoding {
    pub fn new(mut low: u32, mut high: u32) -> Self {
        if low >= high {
            println!("\n\nERRO: Low maior ou igual a high!\n");
            std::process::exit(1);
        }

        let shift: u32 = 32 - low.leading_zeros();
        low >>= shift;
        high >>= shift;

        let precision: u32 = 32 - high.leading_zeros();

        if precision < 4 {
            println!("\n\nERRO: Precisão muito baixa!\n");
            std::process::exit(1);
        }

        let symbols: Symbols = Symbols::new();

        Self {
            low,
            high,
            precision,
            symbols,
        }
    }

    pub fn verify_low_and_high(&self) {
        if self.low >= self.high {
            println!("\n\nERRO: Low maior ou igual a high!\n");
            std::process::exit(1);
        }
    }

    pub fn full_bit(&self) -> u32 {
        1 << (self.precision - 1)
    }

    pub fn full_mask(&self) -> u32 {
        u32::MAX >> (32 - self.precision)
    }

    pub fn half_bit(&self) -> u32 {
        1 << (self.precision - 2)
    }

    pub fn half_mask(&self) -> u32 {
        u32::MAX >> (32 - (self.precision - 1))
    }
}