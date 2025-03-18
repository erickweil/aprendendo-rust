use std::slice::Iter;

use crate::entrada::leia;

pub struct Vec2D<T> {
    width: usize,  
    height: usize, 
    data: Vec<T>
}

impl<T> Vec2D<T> where T: Clone {
    pub fn new(width: usize, height: usize, value: T) -> Vec2D<T> {
        let mut ret = Vec2D { width: width, height: height, data: Vec::new() };
        ret.data.resize(width * height, value);

        return ret;
    }

    pub fn _get(&self, pos: (usize,usize)) -> &T {
         assert!(pos.0 < self.width);
         assert!(pos.1 < self.height);
         return &self.data[pos.1 * self.width + pos.0];
    }

    pub fn set(&mut self, pos: (usize,usize), value: T) {
         assert!(pos.0 < self.width);
         assert!(pos.1 < self.height);
         self.data[pos.1 * self.width + pos.0] = value;
    }

    pub fn values(&self) -> Iter<T> {
        return self.data.iter();
    }

    pub fn _positions(&self) -> Iterator2D {
        return Iterator2D { width: self.width, height: self.height, pos: (0,0) };
    }
}

pub struct Iterator2D {
    width: usize,  
    height: usize, 
    pos: (usize,usize)
}

impl Iterator for Iterator2D {
    type Item = (usize,usize);
    fn next(&mut self) -> Option<Self::Item> {
        let _pos = self.pos;
        if _pos.1 >= self.height {
            return None;
        }

        self.pos.0 += 1;
        if self.pos.0 >= self.width {
            self.pos.0 = 0;
            self.pos.1 += 1;
        }

        return Some(_pos);
    }
}

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
    pub fn iter(profundidade: i32, callback: CB) {
        let mut iterator = Self {
            callback: callback,
            dir: Dir::Right,
            pos: (0,0)
        };
        iterator.iter_a(profundidade);
        iterator.forward();
    }

    fn forward(&mut self) {
        (self.callback)(self.pos, self.dir);

        match self.dir {
            Dir::Right => self.pos.0 += 1,
            Dir::Up => self.pos.1 += 1,
            Dir::Left => self.pos.0 -= 1,
            Dir::Down => self.pos.1 -= 1
        }
    }

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
    let largura: i32 = (2_i32).pow((profundidade+1) as u32);
    
    // Deve ter 2 caracteres de largura cada quadradinho, para desenhar quadrado no terminal
    let mut grade = Vec2D::new((largura*2) as usize, largura as usize, '?');
    
    let mut prev_dir = Dir::Right;
    HIterator::iter(profundidade, |pos,dir| {
        // primeiro caractere é a linha, o segundo pode ser vazio ou continuação da linha '-'
        let grade_pos = ((pos.0 * 2) as usize, (largura - 1 - pos.1) as usize);
        grade.set(grade_pos, get_line_char(prev_dir,dir));
        grade.set((grade_pos.0 + 1, grade_pos.1), 
            if  pos.0 >= largura - 1 { '\n' }
            else if dir == Dir::Right || prev_dir == Dir::Left { '─' }
            else { ' ' }
        );
        
        prev_dir = dir;
    });
    
    // Após construir a grade de caracteres, imprime todos eles na tela
    println!("");
    for c in grade.values() {
        print!("{}",*c);
    }
}