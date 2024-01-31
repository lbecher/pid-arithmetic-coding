use crate::structures::{
    ArithmeticCoding,
    TableSymbol,
};

#[derive(Debug)]
pub struct ArithmeticEncoder {
    low: u32,
    high: u32,
    cumulative_probability: f64,
    probability_table: Vec<TableSymbol>,
    encoded_data: Vec<u32>,
}

impl ArithmeticEncoder {
    pub fn new(low: u32, high: u32) -> Self {
        let mut encoded_data = Vec::new();
        encoded_data.push(0);
        Self {
            low,
            high,
            cumulative_probability: 0.0,
            probability_table: Vec::new(),
            encoded_data,
        }
    }

    pub fn encode(&mut self, data: &[u8]) {
        self.inc_or_add_symbol(&data);
        self.calc_symbol_prob( data.len());
        self.update_low_and_high(&data);
    }

    pub fn get_encoded_data(&self) -> ArithmeticCoding {
        ArithmeticCoding {
            low: self.low,
            high: self.high,
            probability_table: self.probability_table.to_vec(),
            encoded_data: self.encoded_data.to_vec(),
        }
    }

    fn inc_or_add_symbol(&mut self, data: &[u8]) {
        for symbol in data.iter() {
            let position = self.probability_table
                .iter()
                .position(|s| s.symbol == *symbol);

            if let Some(index) = position {
                self.probability_table[index].occurrence += 1;
            } else {
                self.probability_table.push(TableSymbol {
                    symbol: *symbol,
                    occurrence: 1,
                    probability: 0.0,
                    accumulated_probability: 0.0,
                });
            }
        }
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

    fn update_low_and_high(&mut self, data: &[u8]) {
        for symbol in data.iter() {
            let position = self.probability_table
                .iter()
                .position(|s| s.symbol == *symbol);

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

                    let last_u32 = self.encoded_data[0];
                    //print!("\nlast_u32: {}, ", last_u32);

                    match 10u32.checked_mul(last_u32) {
                        Some(mul) => {
                            //print!("mul: {}, ", mul);
                            match mul.checked_add(low_first_digit) {
                                Some(add) => {
                                    //print!("add: {}", add);
                                    self.encoded_data[0] = add;
                                }
                                None => {
                                    self.encoded_data.insert(0, low_first_digit);
                                }
                            }
                        }
                        None => {
                            self.encoded_data.insert(0, low_first_digit);
                        }
                    }

                    low_first_digit = new_low / divisor;
                    high_first_digit = new_high / divisor;
                }

                self.low = new_low;
                self.high = new_high;
            }
        }

        print!("\n\n");
    }
}
