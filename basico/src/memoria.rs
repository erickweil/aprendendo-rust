
pub fn memoria() {
    let mut texto: String = String::new();

    texto.push('H');
    texto.push('e');
    texto.push('l');
    texto.push('l');
    texto.push('o');
    let texto_slice: &str = &texto;

    println!("{}", texto_slice);
}