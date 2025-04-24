**Code Crafters**
- https://app.codecrafters.io/collections/rust-primer	
- https://app.codecrafters.io/tracks/rust	

**Livros**
- https://rust-book.cs.brown.edu/ - The Rust Programming Language (Experiment, interactive)
- https://rust-unofficial.github.io/too-many-lists/ Learn Rust With Entirely Too Many Linked Lists (Linked List in rust Overview)
- https://rust-lang.github.io/async-book/ Async Book (Guide to asynchronous programming in Rust)
- https://doc.rust-lang.org/nomicon/ The Rustonomicon (The Dark Arts of Unsafe Rust)
- ? Rust Atomics and Locks - by @m_ou_se
- ? Rust in Action - by @timClicks
- ? Zero to production in Rust - by @algo_luca
- ? Rust for Rustaceans - by @jonhoo

**Blogs**
- https://os.phil-opp.com/ - Writing a OS in Rust

**Prática**
- https://github.com/rust-lang/rustlings Exercícios práticos para aprender Rust
- https://practice.course.rs/why-exercise.html Rust By Practice (Practice Rust with challenging examples, exercises and projects)
- https://doc.rust-lang.org/rust-by-example/ Rust By Example (Learn Rust with examples)


**Youtube**
- **Tech With Tim** https://www.youtube.com/watch?v=T_KrYLW4jw8 (Série de vídeos descontinuada)
- **Bek Brace** https://www.youtube.com/watch?v=rQ_J9WH6CGk (Curso completo duração 03:05:04)

# Instalação
- https://doc.rust-lang.org/stable/book/ch01-01-installation.html

Hello world

`main.rs`
```rust
fn main() {
    println!("Hello, world!");
}

```

Executar via terminal: `rustc main.rs && ./main`

## Cargo
Gerencia as dependências

- Criar um novo projeto: 	`cargo new projeto`
- Fazer o build do projeto:	`cargo build`
- Build + Executar:		    `cargo run`
- Verificar erros:			`cargo check`

```sh
~/rust/basico$ ls -lhas
total 32K
4,0K drwxrwxr-x 5 erick erick 4,0K mar 16 19:29 .
4,0K drwxrwxr-x 3 erick erick 4,0K mar 16 19:22 ..
4,0K -rw-rw-r-- 1 erick erick  150 mar 16 19:29 Cargo.lock
4,0K -rw-rw-r-- 1 erick erick   77 mar 16 19:22 Cargo.toml
4,0K drwxrwxr-x 6 erick erick 4,0K mar 16 19:23 .git
4,0K -rw-rw-r-- 1 erick erick	8  mar 16 19:22 .gitignore
4,0K drwxrwxr-x 2 erick erick 4,0K mar 16 19:22 src
4,0K drwxrwxr-x 3 erick erick 4,0K mar 16 19:29 target
```

Um repositório git local é inicializado por padrão, .gitignore apenas o diretório /target
- Cargo.toml e Cargo.lock → contém a definição do projeto e as dependências
- src → Código
- target →contém o Build

## RustFMT
Formata o código, basta executar `rustfmt main.rs`


# Linguagem

- Compilada
- Fortemente tipada
- Sintaxe semelhante? ao C, `{} () ; =`

## Variáveis

- Variáveis são imutáveis por padrão (`let mut` para mutável)

Geralmente implicitamente infere o tipo
```rust
let x = 4;
```
Mas pode informar o tipo explicitamente
```rust
let x: u32 = 4;
```
Exibindo variáveis no println!
```rust
let x = 5;
println!("X é: {}", x);
```

Isso produz erro, pois não é mutável
```rust
let x = 5;
println!("X é: {}", x);

x = 4;
println!("X é depois: {}", x);
```

Mas isso não produz erro, redeclarou a variável, até para outro tipo!
```rust
let x = 5;
println!("X é: {}", x);

let x = "A";
println!("X é depois: {}", x);
```

Variáveis só existem dentro do seu próprio escopo, shadowing
```rust
let x = 5;
println!("A: {}", x);

{
    let x = x + 1;
    println!("B: {}", x);
}

let x = x + 1;
println!("C: {}", x);

// A: 5
// B: 6
// C: 6
```

Para definir valores que não mudam e não podem ser re-declarados usa const
```rust
const CONSTANTE: i32 = 5;
```

**Tipos de dados**

**scalar**
```rust
// Signed integers
i8, i16, i32, i64, i128 // (default is i32)
// Unsigned integers
u8, u16, u32, u64, u128
// Float 64 (default is double)
let numero: f64 = 1.00000001;
// Float 32
let pi: f32 = 3.13f32;
// Boolean
let ativo: bool = false;
// Caracteres
let letter: char = 'A';
```
**compound**
```rust
// tuple
let tup: (i32, bool, char) = (1, true, 's');
println!("0:{} 1:{} 2:{}", tup.0, tup.1, tup.2);
// array (O tamanho faz parte da definição do tipo)
let arr: [i32; 4] = [1, 2, 3, 4];
println!("{}", arr[0]);
```

**Convertendo**

```rust
// `as` conversão explícita entre tipos primitivos
let x: i32 = 5_i32;
let y: i64 = x as i64;

// .parse em uma string converte para número
let mut input = String::new();
// ... ler dados?
let numero: i64 = input.trim().parse::<i64>().unwrap();
```

## Entrada de dados

```rust
use std::io;

fn main() {
    let mut input = String::new();

    println!("Insira uma linha:");
    io::stdin().read_line(&mut input).expect("Não foi possível ler");

    println!("{}", input);
}
```

# Memória, Ownership e Borrow Checker

# Exemplos
Para executar os exemplos, execute `cargo run <nome>`
- **hilbert** [Gerador Curva de Hilbert](src/exemplos/hilbert.rs)
- **life** [Jogo da Vida (Conway Game of Life)](src/exemplos/life.rs)
- **snake** [Jogo da 'Cobrinha' (Snake Game)](src/exemplos/snake.rs)
- **minesweeper** [Campo Minado (Minewsweeper)](src/exemplos/minesweeper.rs)
- **expression** [Parser expressões matemáticas](src/exemplos/expression.rs)

# Ideias, Trabalho futuro

- **Objetivo Final** Gerador de horário

Porém, antes de chegar e fazer algo complexo como um gerador de horário, vamos com outros desafios primeiro, que ajudarão a entender melhor rust:

## Jogos/Simulações/Terminal Interativo
- Jogo da forca – Adivinhe letras de uma palavra oculta antes de ficar sem tentativas.
- Jogo do dinossauro – Um jogo de rolagem lateral onde você evita obstáculos pressionando uma tecla.
- Text-based Adventure – Um jogo baseado em escolhas, com descrições em texto e caminhos alternativos.
- Jogo da memória – Lembre-se e recorde sequências de números ou palavras.
- Tetris – Implemente um jogo clássico de Tetris em modo texto, usando caracteres para os blocos e a rotação das peças.
- 2048 – Una números digitando comandos de movimento (WASD).
- Sudoku Solver – Implemente uma versão em texto do Sudoku com um modo solucionador.
- Rogue like, Dungeon Crawler – Navegue por um labirinto enquanto encontra monstros e tesouros.
- Xadrez/Damas – Jogo simples 1v1 onde os jogadores inserem movimentos em notação algébrica.
- Simulação de Formigas (Langton's Ant) – Modele um autômato celular baseado em formigas que seguem regras simples para criar padrões emergentes.
- Gerador de Labirintos – Implemente um algoritmo como Prim ou Recursive Backtracking para criar labirintos aleatórios.

## Algoritmos e Computação
- Simulador de Máquina de Turing – Crie uma simulação básica de uma Máquina de Turing com regras configuráveis.
- Algoritmos Genéticos – Resolva um problema simples (como encontrar uma string alvo) usando um algoritmo genético.
- Solução de Sudoku – Escreva um solver eficiente usando Backtracking ou Constraint Propagation.