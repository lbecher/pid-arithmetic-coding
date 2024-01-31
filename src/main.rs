use std::env;
use std::fs;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::mem::size_of;
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
    structures::ArithmeticCodingInfo,
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

            let mut input_file = match fs::File::open(file_path) {
                Ok(input_file) => input_file,
                Err(e) => {
                    eprintln!("Erro ao abrir o arquivo: {}", e);
                    std::process::exit(1);
                }
            };

            let input_file_len = match input_file.metadata() {
                Ok(metadata) => metadata.len(),
                Err(e) => {
                    eprintln!("Erro ao obter metadados do arquivo de entrada: {}", e);
                    std::process::exit(1);
                }
            };

            let encoded_data_len_position = input_file_len - size_of::<u64>() as u64;
            input_file.seek(std::io::SeekFrom::Start(encoded_data_len_position)).unwrap();

            let mut encoded_data_len_buffer: [u8; 8] = [0,0,0,0,0,0,0,0];
            if let Err(e) = input_file.read_exact(&mut encoded_data_len_buffer) {
                eprintln!("Erro ao obter o tamanho dos dados codificados: {}", e);
                std::process::exit(1);
            };
            let encoded_data_len = u64::from_le_bytes(encoded_data_len_buffer);

            input_file.seek(std::io::SeekFrom::Start(encoded_data_len)).unwrap();

            let arithmetic_coding_info: ArithmeticCodingInfo = match deserialize_from(&mut input_file) {
                Ok(arithmetic_coding_info) => arithmetic_coding_info,
                Err(e) => {
                    eprintln!("Erro ao ler o arquivo de entrada: {}", e);
                    std::process::exit(1);
                }
            };

            input_file.seek(std::io::SeekFrom::Start(0)).unwrap();

            let mut output_file_path = file_path.to_string();
            output_file_path.truncate(file_path.len() - 3);

            let mut output_file = match fs::File::create(output_file_path) {
                Ok(output_file) => output_file,
                Err(e) => {
                    eprintln!("Erro ao criar o arquivo de saída: {}", e);
                    std::process::exit(1);
                }
            };
            
            let mut decoder = ArithmeticDecoder::new(
                match low {
                    Some(low) => low,
                    None => arithmetic_coding_info.low,
                }, 
                match high {
                    Some(high) => high,
                    None => arithmetic_coding_info.high,
                }, 
                arithmetic_coding_info.probability_table.to_owned(),
            );
            decoder.decode(encoded_data_len, &mut input_file, &mut output_file);
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

            let mut input_file = match fs::File::open(file_path) {
                Ok(input_file) => input_file,
                Err(e) => {
                    eprintln!("Erro ao abrir o arquivo: {}", e);
                    std::process::exit(1);
                }
            };

            let output_file_path = String::from(file_path) + ".ac";

            let mut output_file = match fs::File::create(output_file_path) {
                Ok(output_file) => output_file,
                Err(e) => {
                    eprintln!("Erro ao criar o arquivo de saída: {}", e);
                    std::process::exit(1);
                }
            };

            let mut encoder = ArithmeticEncoder::new(low, high);
            encoder.encode(&mut input_file, &mut output_file);
            let arithmetic_coding_info = encoder.get_arithmetic_coding_info();

            let encoded_data_len = match output_file.metadata() {
                Ok(metadata) => metadata.len(),
                Err(e) => {
                    eprintln!("Erro ao obter metadados do arquivo de saída: {}", e);
                    std::process::exit(1);
                }
            };

            if let Err(e) = serialize_into(&mut output_file, &arithmetic_coding_info) {
                eprintln!("Erro ao gravar no arquivo de saída: {}", e);
                std::process::exit(1);
            };

            if let Err(e) = output_file.write_all(&encoded_data_len.to_le_bytes()) {
                eprintln!("Erro ao gravar no arquivo de saída: {}", e);
                std::process::exit(1);
            };
        }
    }
}
