use crate::structures::TableSymbol;

#[derive(Debug)]
pub struct ArithmeticDecoder {
    pub low: u32,
    pub high: u32,
    pub probability_table: Vec<TableSymbol>,
    pub decoded_data: Vec<u8>,
}

impl ArithmeticDecoder {
    pub fn new(low: u32, high: u32, probability_table: Vec<TableSymbol>) -> Self {
        Self {
            low,
            high,
            probability_table,
            decoded_data: Vec::new(),
        }
    }

    pub fn decode(&mut self, data: &[u32]) {

    }

    pub fn get_decoded_data(&self) -> Vec<u8> {
        self.decoded_data.to_vec()
    }
}