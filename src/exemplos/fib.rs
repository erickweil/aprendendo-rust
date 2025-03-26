use crate::utils::leia;

pub fn fib() {
    let input = leia("Até quanto contar fibonacci? (padrão 34) \n>");
    
    let maximo: u128 = input.parse::<u128>().unwrap_or(34);

    println!("Sequência de fibonacci, contando até {}:", maximo);
    let mut a: u128 = 0;
    let mut b: u128 = 1;
    loop {
        let c = a;
        a = a + b;
        b = c;

        if a > maximo {
            break;
        }
        println!("{}", a);
    }
}