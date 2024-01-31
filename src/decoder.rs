use std::fs::File;

use crate::structures::TableSymbol;

#[derive(Debug)]
pub struct ArithmeticDecoder {
    low: u32,
    high: u32,
    probability_table: Vec<TableSymbol>,
}

impl ArithmeticDecoder {
    pub fn new(low: u32, high: u32, probability_table: Vec<TableSymbol>) -> Self {
        Self {
            low,
            high,
            probability_table,
        }
    }

    pub fn decode(&mut self, file: &File) {

    }
}