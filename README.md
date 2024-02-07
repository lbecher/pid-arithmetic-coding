# pid-arithmetic-coding 

Trabalho extra da disciplina de Processamento de Imagens Digitais.

## Arquivos de saída

Na codificação, um arquivo com o nome `<nome_do_arquivo_original>`, mais a extensão `.ac`, é gerado no mesmo subdiretório do arquivo que foi codificado. Esse arquivo possui, respectivamente, a região de dados codificados (inteiros não sinalizados de 32 bits com os digitos emitidos durante a codificação), na instância da estrutura de dados ArithmeticCoding, o tamanho em bytes da região de dados codificados (inteiro não sinalizado de 64 bits) e na quantidade de dígitos válidos do último valor da região de dados codificados (inteiro não sinalizado de 8 bits).

Na decodificação, a partir de um arquivo `.ac`, um arquivo como o nome `<nome_do_arquivo_ac>`, menos a extensão `.ac`, mais a extenção `.dec`, é gerado no mesmo subdiretório do arquivo que foi decodificado. Este arquivo, portanto, possui o conteudo do arquivo original utilizado na codificação.

## Compilação e execução a partir do código fonte

Na pasta raiz do projeto (que contém o arquivo `Cargo.toml`), use o comando abaixo para executar em modo debug (neste modo será printada a saída, portanto não funcionará com símbolos não ASCII):

```
cargo run -- <opções>
```

Use o comando abaixo para executar em modo release e poder processar todos os tipos de símbolos:

```
cargo run -r -- <opções>
```

## Instalação de dependências de compilação para Debian/Ubuntu/Linux Mint

Execute os comandos abaixo:

```
sudo apt update
```
```
sudo apt install -y build-essential curl
```
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```