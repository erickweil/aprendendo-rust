use std::io;

use rand::{distr::Uniform, Rng};

use crossterm::{cursor::MoveTo, event::KeyCode, style::{Color, SetBackgroundColor, SetForegroundColor}, terminal::{size, Clear, ClearType, DisableLineWrap, EnableLineWrap}, ExecutableCommand};

use crate::{estruturas::Dir, utils::{Terminal, TerminalHandler}};
struct Snake {
    pos: (i32,i32),
    body: Vec<(i32,i32)>,
    dir: Dir
}

impl Snake {
    pub fn slither(&mut self, size: (i32,i32)) -> bool {
        let off = self.dir.to_xy();
        let pos = self.pos;
        let new_pos = (pos.0 + off.0, pos.1 - off.1);

        if self.check_collision_self(new_pos) {
            return true;
        }

        if Snake::check_collision_wall(new_pos,size) {
            return true;
        }

        self.pos = new_pos;
        self.body.push(new_pos);
        return false;
    }

    pub fn shrink_tail(&mut self) {
        // A FAZER: implementar fila
        for i in 1..self.body.len() {
            self.body[i-1] = self.body[i];
        }
        self.body.pop();
    }

    pub fn check_collision_wall((x,y): (i32,i32), (w,h): (i32,i32)) -> bool {
        if x < 0 || y < 0 || x >= w || y >= h {
            return true;
        } else {
            return false;
        }
    }

    pub fn check_collision_self(&self, pos: (i32,i32)) -> bool {
        for body_pos in self.body.iter() {
            if *body_pos == pos { return true; }
        }
        return false;
    }
}

struct SnakeGame {
    size: (i32,i32),
    player: Snake,
    fruit: (i32,i32),
    score: i32,
    paused: bool
}

impl SnakeGame {
    pub fn new(size: (i32,i32)) -> SnakeGame {
        let center = ((size.0 / 2) as i32, (size.1 / 2) as i32);
        SnakeGame {
            size: size,
            paused: false,
            score: 0,
            fruit: ((size.0 / 3) as i32, (size.1 / 3) as i32),
            player: Snake {
                pos: center,
                body: vec![center],
                dir: Dir::Left
            }
        }
    }

    pub fn simulate(&mut self) -> bool {
        if self.player.slither(self.size) {
            return true;
        }

        if self.player.pos == self.fruit {
            self.score += 1;
            let mut rng = rand::rng();
            self.fruit = loop {
                let fruit_pos = (
                    rng.random_range(0..self.size.0),
                    rng.random_range(0..self.size.1-1)
                );
                if !self.player.check_collision_self(fruit_pos) {
                    break fruit_pos;
                }
            };
        } else {
            // se não achou fruta, remove 1 da cauda
            self.player.shrink_tail();
        }

        return false;
    }
}

impl TerminalHandler for SnakeGame {
    fn on_draw(&mut self, term: &mut Terminal) -> io::Result<()> {
        let t = &mut term.stdout;

        if !self.paused {
            let collision = self.simulate();
            if collision {
                self.paused = true;
            }
        }
        
        // Reset
        t.execute(SetBackgroundColor(Color::Black))?;
        t.execute(Clear(ClearType::All))?;
        t.execute(MoveTo(0,0))?;

        // Desenha Cobra
        t.execute(SetForegroundColor(Color::Green))?;
        for pos in self.player.body.iter() {
            t.execute(MoveTo((pos.0 * 2) as u16, pos.1 as u16))?;
            print!("██");
        }

        let pos = self.fruit;
        t.execute(MoveTo((pos.0 * 2) as u16, pos.1 as u16))?;
        print!("🍎");

        // Barra inferior de informações
        t.execute(SetForegroundColor(Color::Black))?;
        t.execute(SetBackgroundColor(Color::White))?;
        t.execute(MoveTo(0,(self.size.1-1) as u16))?;
        print!("Pontos {} ",self.score);

        Ok(())
    }

    fn on_key_event(&mut self, e: &crossterm::event::KeyEvent) {
        match e.code {
            KeyCode::Left => self.player.dir = Dir::Left,
            KeyCode::Right => self.player.dir = Dir::Right,
            KeyCode::Up => self.player.dir = Dir::Up,
            KeyCode::Down => self.player.dir = Dir::Down,
            _ => {

            }
        }
    }
}

fn setup_terminal() -> io::Result<()> {
    let mut t = Terminal::new();

    let size = size()?;
    t.stdout.execute(SetBackgroundColor(Color::Black))?;
    t.stdout.execute(SetForegroundColor(Color::Green))?;
    t.stdout.execute(DisableLineWrap)?;
    
    // 15 FPS
    t.main_loop(66, false, SnakeGame::new(((size.0/2) as i32, size.1 as i32)))?;

    t.stdout.execute(EnableLineWrap)?;

    Ok(())
}

pub fn snake() {
    match setup_terminal() {
        Ok(_) => {
            println!();
            println!("Finalizou execução.");
        },
        Err(err) => {
            println!();
            println!("Erro: {}", err);
        },
    }
}