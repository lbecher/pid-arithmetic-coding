use std::env;
use std::fs;
use std::io::Read;
use std::io::Seek;
use std::mem::size_of;
use bincode::deserialize_from;

mod decoder;
mod encoder;

use arithmetic_coding::Operation;
use arithmetic_coding::ArithmeticCoding;
use crate::{
    decoder::ArithmeticDecoder,
    encoder::ArithmeticEncoder,
};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("\nUso: {} <parâmetros> <operação>\n", args[0]);
        println!("Operações suportadas:");
        println!("  -e, --encode <arquivo>    Codificar o conteúdo do arquivo informado.");
        println!("  -d, --decode <arquivo>    Decodificar o conteúdo do arquivo informado.\n");
        println!("Parâmetros de codificação:");
        println!("  -l, --low <valor>         Define o valor de low.");
        println!("  -h, --high <valor>        Define o valor de high.\n");
        std::process::exit(1);
    }

    let mut low: Option<u32> = None;
    let mut high: Option<u32> = None;
    let mut operation: Option<Operation> = None;
    let mut file_path: Option<&str> = None;

    let mut iter = args.iter().skip(1);

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--low" | "-l" => {
                if let Some(value) = iter.next() {
                    if let Ok(value) = value.parse::<u32>() {
                        low = Some(value);
                    } else {
                        println!("Valor de low inválido.");
                        std::process::exit(1);
                    }
                } else {
                    println!("Valor de low não fornecido.");
                    std::process::exit(1);
                }
            }
            "--high" | "-h" => {
                if let Some(value) = iter.next() {
                    if let Ok(value) = value.parse::<u32>() {
                        high = Some(value);
                    } else {
                        println!("Valor de high inválido.");
                        std::process::exit(1);
                    }
                } else {
                    println!("Valor de high não fornecido.");
                    std::process::exit(1);
                }
            }
            "--decode" | "-d" | "--encode" | "-e" => {
                operation = Some(match arg.as_str() {
                    "--decode" | "-d" => Operation::Decode,
                    "--encode" | "-e" => Operation::Encode,
                    _ => unreachable!(),
                });
                if let Some(value) = iter.next() {
                    file_path = Some(value.as_str());
                } else {
                    println!("Arquivo de entrada não fornecido.");
                    std::process::exit(1);
                }
            }
            _ => {
                println!("\nUso: {} <parâmetros> <operação>\n", args[0]);
                println!("Operações suportadas:");
                println!("  -e, --encode <arquivo>    Codificar o conteúdo do arquivo informado.");
                println!("  -d, --decode <arquivo>    Decodificar o conteúdo do arquivo informado.\n");
                println!("Parâmetros de codificação:");
                println!("  -l, --low <valor>         Define o valor de low.");
                println!("  -h, --high <valor>        Define o valor de high.\n");
                std::process::exit(1);
            }
        }
    }

    let operation = match operation {
        Some(operation) => operation,
        None => {
            println!("\nOperação não informada.\n");
            std::process::exit(1);
        }
    };
    let file_path = file_path.unwrap();

    if let Some(low) = low {
        if !verify_gp2_minus_one(low) {
            println!("\nLow não pertence à PG(2) - 1!\n");
            std::process::exit(1);
        }
    }
    if let Some(high) = high {
        if !verify_gp2_minus_one(high) {
            println!("\nHigh não pertence à PG(2) - 1!\n");
            std::process::exit(1);
        }
    }

    match operation.clone() {
        Operation::Decode => {
            if !file_path.ends_with(".ac") {
                println!("\nO arquivo informado não possui a extensão \".ac\"!\n");
                std::process::exit(1);
            }

            // abre arquivo de entrada
            let mut input_file = match fs::File::open(file_path) {
                Ok(input_file) => input_file,
                Err(e) => {
                    eprintln!("\nErro ao abrir o arquivo: {}\n", e);
                    std::process::exit(1);
                }
            };

            // obtém o número de dígitos do último valor gravado
            let input_file_len = match input_file.metadata() {
                Ok(metadata) => metadata.len(),
                Err(e) => {
                    eprintln!("\nErro ao obter metadados do arquivo de entrada: {}\n", e);
                    std::process::exit(1);
                }
            };

            let last_value_shifts_position = input_file_len - size_of::<u8>() as u64;
            input_file.seek(std::io::SeekFrom::Start(last_value_shifts_position)).unwrap();

            let mut last_value_shifts_buffer: [u8; 1] = [0];
            if let Err(e) = input_file.read_exact(&mut last_value_shifts_buffer) {
                eprintln!("\nErro ao obter o tamanho dos dados codificados: {}\n", e);
                std::process::exit(1);
            };
            let last_value_shifts = u8::from_le_bytes(last_value_shifts_buffer);

            // obtém o tamanho em bytes dos dados codificados
            let encoded_data_len_position = input_file_len - (size_of::<u64>() + size_of::<u8>()) as u64;
            input_file.seek(std::io::SeekFrom::Start(encoded_data_len_position)).unwrap();

            let mut encoded_data_len_buffer: [u8; 8] = [0,0,0,0,0,0,0,0];
            if let Err(e) = input_file.read_exact(&mut encoded_data_len_buffer) {
                eprintln!("\nErro ao obter o tamanho dos dados codificados: {}\n", e);
                std::process::exit(1);
            };
            let encoded_data_len = u64::from_le_bytes(encoded_data_len_buffer);

            // lê estrutura de dados principal
            input_file.seek(std::io::SeekFrom::Start(encoded_data_len)).unwrap();

            let mut arithmetic_coding: ArithmeticCoding = match deserialize_from(&mut input_file) {
                Ok(arithmetic_coding) => arithmetic_coding,
                Err(e) => {
                    eprintln!("\nErro ao ler o arquivo de entrada: {}\n", e);
                    std::process::exit(1);
                }
            };

            if let Some(low) = low {
                arithmetic_coding.low = low;
            }
            if let Some(high) = high {
                arithmetic_coding.high = high;
            }

            // cria arquivo de saída
            let mut output_file_path = file_path.to_string();
            output_file_path.truncate(file_path.len() - 3);
            output_file_path += ".dec";

            let output_file = match fs::File::create(output_file_path) {
                Ok(output_file) => output_file,
                Err(e) => {
                    eprintln!("\nErro ao criar o arquivo de saída: {}\n", e);
                    std::process::exit(1);
                }
            };

            // decodifica
            let mut decoder = ArithmeticDecoder::new(
                arithmetic_coding,
                encoded_data_len / size_of::<u32>() as u64,
                last_value_shifts as u32,
                output_file,
            );
            decoder.decode(&mut input_file);
        }
        Operation::Encode => {
            let low = match low {
                Some(low) => low,
                None => {
                    println!("\nValor de low não informado.\n");
                    std::process::exit(1);
                }
            };
            let high = match high {
                Some(high) => high,
                None => {
                    println!("\nValor de high não informado.\n");
                    std::process::exit(1);
                }
            };

            // abre arquivo de entrada
            let mut input_file = match fs::File::open(file_path) {
                Ok(input_file) => input_file,
                Err(e) => {
                    eprintln!("\nErro ao abrir o arquivo: {}\n", e);
                    std::process::exit(1);
                }
            };
            let input_file_len = match input_file.metadata() {
                Ok(metadata) => metadata.len(),
                Err(e) => {
                    eprintln!("\nErro ao obter metadados do arquivo de entrada: {}\n", e);
                    std::process::exit(1);
                }
            };

            // cria arquivo de saída
            let output_file_path = String::from(file_path) + ".ac";
            let output_file = match fs::File::create(output_file_path) {
                Ok(output_file) => output_file,
                Err(e) => {
                    eprintln!("\nErro ao criar o arquivo de saída: {}\n", e);
                    std::process::exit(1);
                }
            };

            // codifica
            let mut encoder = ArithmeticEncoder::new(low, high, output_file);
            encoder.verify_file_len(input_file_len);
            encoder.encode(&mut input_file);
        }
    }
}

fn verify_gp2_minus_one(number: u32) -> bool {
    let number = number as u64;
    let mut gp: u64 = 1;
    for _ in 0..32 {
        if number == gp - 1 {
            return true;
        }
        gp <<= 1;
    }
    false
}