use crate::utils::leia;
use crate::estruturas::Vec2D;

#[derive(PartialEq)]
#[derive(Clone, Copy)]
pub enum Dir {
    Right,
    Up,
    Left,
    Down
}

impl Dir {
    pub fn counter_clockwise(&self) -> Dir {
        match &self {
            Dir::Right => Dir::Up,
            Dir::Up => Dir::Left,
            Dir::Left => Dir::Down,
            Dir::Down => Dir::Right,
        }
    }

    pub fn clockwise(&self) -> Dir {
        match &self {
            Dir::Right => Dir::Down,
            Dir::Down => Dir::Left,
            Dir::Left => Dir::Up,
            Dir::Up => Dir::Right,
        }
    }
}

/**
https://en.wikipedia.org/wiki/Hilbert_curve
The Hilbert Curve can be expressed by a rewrite system (L-system).

    Alphabet : A, B
    Constants : F + −
    Axiom : A
    Production rules:

        A → +BF−AFA−FB+
        B → −AF+BFB+FA−

Here, "F" means "draw forward", "+" means "turn left 90°", "-" means "turn right 90°" (see turtle graphics), and "A" and "B" are ignored during drawing. 
*/
pub struct HIterator<CB> {
    callback: CB,
    dir: Dir,
    pos: (i32,i32)
}

impl<CB> HIterator<CB>
where 
    CB: FnMut((i32,i32),Dir),
{
    /**
     * Faz uma iteração em uma grade seguindo a curva de hilbert
     * O valor de profundidade indica o tamanho da grade da curva:
     * 0: 2x2
     * 1: 4x4
     * 2: 8x8
     * 3: 16x16
     * ...
     * n: 2^(n+1)
     */
    pub fn iter(profundidade: i32, callback: CB) {
        let mut iterator = Self {
            callback: callback,
            dir: Dir::Right,
            pos: (0,0)
        };
        iterator.iter_a(profundidade);
        iterator.forward();
    }

    /**
     * "F" means "draw forward"
     * Chama a callback e anda para frente de acordo com a direção
     */ 
    fn forward(&mut self) {
        (self.callback)(self.pos, self.dir);

        match self.dir {
            Dir::Right => self.pos.0 += 1,
            Dir::Up => self.pos.1 += 1,
            Dir::Left => self.pos.0 -= 1,
            Dir::Down => self.pos.1 -= 1
        }
    }

    /**
     * A → +BF−AFA−FB+
     */
    fn iter_a(&mut self, depth: i32) {
        self.dir = self.dir.counter_clockwise();     
        if depth > 0 { self.iter_b(depth-1) } 
        self.forward();                              
        self.dir = self.dir.clockwise();            
        if depth > 0 { self.iter_a(depth-1) }
        self.forward();                            
        if depth > 0 { self.iter_a(depth-1) }
        self.dir = self.dir.clockwise();
        self.forward();
        if depth > 0 { self.iter_b(depth-1) }
        self.dir = self.dir.counter_clockwise();
    }

    /**
     * B → −AF+BFB+FA−
     */
    fn iter_b(&mut self, depth: i32) {
        self.dir = self.dir.clockwise();
        if depth > 0 { self.iter_a(depth-1) }
        self.forward();
        self.dir = self.dir.counter_clockwise();
        if depth > 0 { self.iter_b(depth-1) }
        self.forward();
        if depth > 0 { self.iter_b(depth-1) }
        self.dir = self.dir.counter_clockwise();
        self.forward();
        if depth > 0 { self.iter_a(depth-1) }
        self.dir = self.dir.clockwise();
    }
}

fn get_line_char(prev: Dir, next: Dir) -> char {    
    if prev == next {
        if prev == Dir::Right || prev == Dir::Left { return '─'; } else { return '│'; }
    } else {
        if      prev == Dir::Up   && next == Dir::Right || prev == Dir::Left  && next == Dir::Down { return '┌'; }
        else if prev == Dir::Up   && next == Dir::Left  || prev == Dir::Right && next == Dir::Down { return '┐'; }
        else if prev == Dir::Down && next == Dir::Right || prev == Dir::Left  && next == Dir::Up   { return '└'; }
        else if prev == Dir::Down && next == Dir::Left  || prev == Dir::Right && next == Dir::Up   { return '┘'; }
        else { return '?'; }
    }
}

pub fn hilbert() {
    let input = leia("Qual a profundidade da curva de Hilbert? (padrão 3)");
    let profundidade: i32 = input.parse::<i32>().unwrap_or(3);

    if profundidade < 0 || profundidade > 10 {
        println!("Deve escolher um valor entre 0 e 10 para profundidade.");
        return;
    }

    let largura: i32 = (2_i32).pow((profundidade+1) as u32);
    
    // Constrói a grade vazia de caracteres
    // Deve ter 2 caracteres de largura cada quadradinho, para desenhar quadrado no terminal
    let mut grade = Vec2D::new((largura*2) as usize, largura as usize, '?');
    
    // Atravessa a curva preenchendo a grade
    let mut prev_dir = Dir::Right;
    HIterator::iter(profundidade, |(x,y),dir| {
        // primeiro caractere é a linha, o segundo pode ser vazio ou continuação da linha '-'
        let (gx, gy) = ((x * 2) as usize, (largura - 1 - y) as usize);
        grade[(gx, gy)] = get_line_char(prev_dir,dir);
        grade[(gx + 1, gy)] = 
            if  x >= largura - 1 { '\n' }
            else if dir == Dir::Right || prev_dir == Dir::Left { '─' }
            else { ' ' }
        ;
        
        prev_dir = dir;
    });
    
    // Após construir a grade de caracteres, imprime todos eles na tela
    println!("");
    for c in grade.values() {
        print!("{}",*c);
    }
}