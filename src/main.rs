use std::env;
use std::fs;
use bincode::{
    deserialize_from,
    serialize_into,
};

mod decoder;
mod encoder;
mod structures;

use crate::{
    decoder::ArithmeticDecoder,
    encoder::ArithmeticEncoder,
    structures::ArithmeticCoding,
};

#[derive(Debug, Clone)]
enum Operation {
    Decode,
    Encode,
}

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
            println!("Operação não informada.");
            std::process::exit(1);
        }
    };
    let file_path = file_path.unwrap();

    match operation.clone() {
        Operation::Decode => {
            if !file_path.ends_with(".ac") {
                println!("O arquivo informado não possui a extensão \".ac\"!");
                std::process::exit(1);
            }

            let input_file = match fs::File::open(file_path) {
                Ok(input_file) => input_file,
                Err(e) => {
                    eprintln!("Erro ao abrir o arquivo: {}", e);
                    std::process::exit(1);
                }
            };

            let arithmetic_coding: ArithmeticCoding = match deserialize_from(input_file) {
                Ok(arithmetic_coding) => arithmetic_coding,
                Err(e) => {
                    eprintln!("Erro ao abrir o arquivo: {}", e);
                    std::process::exit(1);
                }
            };
            
            let mut decoder = ArithmeticDecoder::new(
                match low {
                    Some(low) => low,
                    None => arithmetic_coding.low,
                }, 
                match high {
                    Some(high) => high,
                    None => arithmetic_coding.high,
                }, 
                arithmetic_coding.probability_table.to_owned(),
            );
            decoder.decode(arithmetic_coding.encoded_data.as_slice());
            let decoded_data = decoder.get_decoded_data();

            let mut output_file_path = file_path.to_string();
            output_file_path.truncate(file_path.len() - 3);

            match fs::write(output_file_path + ".dec", decoded_data) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Erro ao criar o arquivo de saída: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Operation::Encode => {
            let low = match low {
                Some(low) => low,
                None => {
                    println!("Valor de low não informado.");
                    std::process::exit(1);
                }
            };
            let high = match high {
                Some(high) => high,
                None => {
                    println!("Valor de high não informado.");
                    std::process::exit(1);
                }
            };

            let file_buffer = match fs::read(file_path) {
                Ok(file_buffer) => file_buffer,
                Err(e) => {
                    eprintln!("Erro ao abrir o arquivo: {}", e);
                    std::process::exit(1);
                }
            };

            let mut encoder = ArithmeticEncoder::new(low, high);
            encoder.encode(file_buffer.as_slice());
            let encoded_data = encoder.get_encoded_data();

            let output_file_path = String::from(file_path) + ".ac";

            let output_file = match fs::File::create(output_file_path) {
                Ok(output_file) => output_file,
                Err(e) => {
                    eprintln!("Erro ao criar o arquivo de saída: {}", e);
                    std::process::exit(1);
                }
            };

            if let Err(e) = serialize_into(output_file, &encoded_data) {
                eprintln!("Erro ao gravar no arquivo de saída: {}", e);
                std::process::exit(1);
            };
        }
    }
}
