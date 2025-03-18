
/**
 * Regras de Ownership:
 * 1- Cada valor em rust possui uma variável que é seu dono
 * 2- Só é possível ter um dono por vez
 * 3- Quando o dono sair do escopo, o valor deixa de existir
 * 
 * Regras de referências &:
 * 1- Só pode ter uma referência mutável, ou várias referências imutáveis, mas não os dois
 */
pub fn memoria() {

    // PRIMITIVOS
    // -------------------
    let mut c: i32 = 0;
    {
        let contador = &mut c;

        *contador += 1;

        print!("contador {} == ", contador);
    }
    println!("{}", c);

    let mut a: i32 = 5;
    let mut b: i32 = 3;

    print!("swap a:{} b:{} --> ", a, b);
    swap(&mut a, &mut b);
    println!("a:{} b:{}", a, b);

    // STRINGS
    // -------------------
    let mut texto: String = String::from("Hello");
    texto.push('!');

    // Slice de um texto
    let texto_slice: &str = &texto[0..4];
    println!("Slice de texto str: '{}' str[0..4]: '{}'", texto, texto_slice);

    // Emprestando
    emprestar(&texto);

    // Passando o dono
    roubar(texto);

    // println!("{}",texto); ERR -> borrow of moved value: `texto`, value borrowed here after move
}

pub fn swap(a: &mut i32, b: &mut i32) {
    let c = *a;
    *a = *b;
    *b = c;
}

/**
 * Borrowing
 * Apenas é dono da variável durante a execução da função
 */
pub fn emprestar(texto: &String) {
    println!("Peguei emprestado só o texto... {}", texto);
}

/** 
 * 
 * Desse jeito a função toma a variável para ela
 */
pub fn roubar(texto: String) {
    println!("Roubei o texto! {}", texto);
}

