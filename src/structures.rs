use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TableSymbol {
    pub symbol: u8,
    pub occurrence: u32,
    pub probability: f64,
    pub accumulated_probability: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArithmeticCoding {
    pub low: u32,
    pub high: u32,
    pub probability_table: Vec<TableSymbol>,
    pub encoded_data: Vec<u32>,
}