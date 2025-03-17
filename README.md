**Code Crafters**
- https://app.codecrafters.io/collections/rust-primer	
- https://app.codecrafters.io/tracks/rust	

**Livro**
- https://doc.rust-lang.org/stable/book/	

**Youtube**
- **Tech With Tim** https://www.youtube.com/watch?v=T_KrYLW4jw8 (Série de vídeos)

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