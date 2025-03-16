**Code Crafters**
- https://app.codecrafters.io/collections/rust-primer	
- https://app.codecrafters.io/tracks/rust	

**Livro**
- https://doc.rust-lang.org/stable/book/	

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
