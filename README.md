# pid-arithmetic-coding 

Trabalho extra da disciplina de Processamento de Imagens Digitais.

## Arquivos de saída

Na codificação, um arquivo com o nome `<nome_do_arquivo_original>`, mais a extensão `.ac`, é gerado no mesmo subdiretório do arquivo que foi codificado. Esse arquivo possui, respectivamente, a região de dados codificados (inteiros não sinalizados de 32 bits com os dígitos emitidos durante a codificação), a instância da estrutura de dados ArithmeticCoding, o tamanho em bytes da região de dados codificados (inteiro não sinalizado de 64 bits) e a quantidade de dígitos válidos do último valor da região de dados codificados (inteiro não sinalizado de 8 bits).

Na decodificação, a partir de um arquivo `.ac`, um arquivo como o nome `<nome_do_arquivo_ac>`, menos a extensão `.ac`, mais a extensão `.dec`, é gerado no mesmo subdiretório do arquivo que foi decodificado. Este arquivo, portanto, possui o conteúdo do arquivo original utilizado na codificação.

## Compilação e execução a partir do código fonte

Na pasta raiz do projeto (que contém o arquivo `Cargo.toml`), use o comando abaixo para executar em modo *debug* (neste modo será printada a saída, portanto não funcionará com símbolos não ASCII):

```
cargo run -- <opções>
```

Use o comando abaixo para executar em modo *release* e poder processar todos os tipos de símbolos:

```
cargo run -r -- <opções>
```

### Opções

Para executar uma codificação, use a opção `--encode` (ou `-e`) seguida por um caminho de arquivo, mais as opções `--low` (ou `-l`) e `--high` (ou `-h`). Tanto `--low` quanto `--high` devem ser seguidos por um valor inteiro não sinalizado. Lembre-se que `--low` deve ser menor que `--high` e a diferença entre ambos deve ser maior que o tamanho em bytes do arquivo a ser codificado. Para `--low`, recomenda-se 0, ou então um número na progressão geométrica na base dois subtraido por 1. Para `--high`, recomenda-se um número na progressão geométrica na base dois subtraido por 1.

Para executar uma decodificação, use a opção `--decode` (ou `-d`) seguida por um caminho de um arquivo codificado, com extensão `.ac`.

### Exemplos

```
cargo run -r -- --encode flag.bmp --low 0 --high 16777215 
```

```
cargo run -r -- --decode flag.bmp.ac
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
