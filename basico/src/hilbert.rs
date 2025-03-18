use crate::entrada::leia;

#[derive(PartialEq)]
enum Dir {
    Right,
    Up,
    Left,
    Down
}

impl Dir {
    fn counter_clockwise(&self) -> Dir {
        match &self {
            Dir::Right => Dir::Up,
            Dir::Up => Dir::Left,
            Dir::Left => Dir::Down,
            Dir::Down => Dir::Right,
        }
    }

    fn clockwise(&self) -> Dir {
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
    pos: (i32,i32),
    prev_pos: (i32,i32)
}

impl<CB> HIterator<CB>
where 
    CB: FnMut((i32,i32),(i32,i32),(i32,i32)),
{
    pub fn iter(profundidade: i32, callback: CB) {
        let mut iterator = Self {
            callback: callback,
            dir: Dir::Right,
            pos: (0,0),
            prev_pos: (-1,0)
        };
        iterator.iter_a(profundidade);
        iterator.forward();
    }

    fn forward(&mut self) {
        let _pos = self.pos;

        if self.dir == Dir::Right { self.pos.0 += 1; }
        else if self.dir == Dir::Up { self.pos.1 += 1; }
        else if self.dir == Dir::Left { self.pos.0 -= 1; }
        else if self.dir == Dir::Down { self.pos.1 -= 1; }
        
        (self.callback)(self.prev_pos, _pos, self.pos);

        self.prev_pos = _pos;
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

fn get_dir(a: (i32,i32), b: (i32,i32)) -> Dir {
    let diff = (b.0 - a.0, b.1 - a.1);
    if diff.0 == 1 { return Dir::Right; }
    if diff.1 == 1 { return Dir::Up; }
    if diff.0 == -1 { return Dir::Left; }
    if diff.1 == -1 { return Dir::Down; }
    return Dir::Right;
}

fn get_line_char(a: (i32,i32), b: (i32,i32), c: (i32,i32)) -> char {
    let prev = get_dir(a,b);
    let next = get_dir(b,c);
    
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
    
    let mut grade: Vec<char> = Vec::new();
    grade.resize((largura * largura) as usize, '?');
    
    HIterator::iter(profundidade, |a,b,c| {
        grade[((largura - 1 - b.1) * largura + b.0) as usize] = get_line_char(a,b,c);
    });
    
    println!("");
    
    for y in 0..largura {
        for x in 0..largura {
            let c = grade[(y * largura + x) as usize];
            
            if c == '─' || c == '┌' || c == '└' {
                print!("{}─",c);
            } else {
                print!("{} ",c);
            }
        }
        println!("");
    }
}