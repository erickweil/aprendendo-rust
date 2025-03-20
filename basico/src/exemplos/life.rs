use std::io;
use std::mem;

use crossterm::{cursor::*, event::KeyCode, style::*, terminal::*, ExecutableCommand, QueueableCommand};

use crate::{estruturas::Vec2D, utils::{Terminal, TerminalHandler}};

fn wrap_xy((x,y): (i32,i32), (w,h): (usize,usize)) -> (usize,usize) {
    (
        if x < 0 { w-1 }
        else if x >= w as i32 { 0 }
        else { x as usize },
        if y < 0 { h-1 }
        else if y >= h as i32 { 0 }
        else { y as usize }
    )
}

struct Life {
    // Necessário para simulação
    grade: Vec2D<bool>,
    prev_grade: Vec2D<bool>,
    size: (usize,usize),
    counter: i32,

    // Utilizado para estado da interação com terminal
    pos: (usize,usize),
    paused: bool,
    fast: bool
}

impl Life {
    pub fn new(size: (usize,usize)) -> Life {
        Life {
            prev_grade: Vec2D::new(size.0, size.1, false),
            grade: Vec2D::new(size.0, size.1, false),
            size: size,
            pos: (0,0),
            counter: 0,
            paused: true,
            fast: false
        }
    }

    /**
     *  https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life
     *  Any live cell with fewer than two live neighbours dies, as if by underpopulation.
     *  Any live cell with two or three live neighbours lives on to the next generation.
     *  Any live cell with more than three live neighbours dies, as if by overpopulation.
     *  Any dead cell with exactly three live neighbours becomes a live cell, as if by reproduction.
     */
    pub fn process_cell(&self, pos: (usize,usize)) -> bool {
        let mut vizinhos = 0;
        let (x,y) = (pos.0 as i32, pos.1 as i32);
        if self.grade[wrap_xy((x-1, y-1), self.size)] { vizinhos += 1; }
        if self.grade[wrap_xy((x  , y-1), self.size)] { vizinhos += 1; }
        if self.grade[wrap_xy((x+1, y-1), self.size)] { vizinhos += 1; }

        if self.grade[wrap_xy((x-1, y  ), self.size)] { vizinhos += 1; }
        // 0,0
        if self.grade[wrap_xy((x+1, y  ), self.size)] { vizinhos += 1; }

        if self.grade[wrap_xy((x-1, y+1), self.size)] { vizinhos += 1; }
        if self.grade[wrap_xy((x  , y+1), self.size)] { vizinhos += 1; }
        if self.grade[wrap_xy((x+1, y+1), self.size)] { vizinhos += 1; }

        if vizinhos == 3 { 
            return true; 
        } else if vizinhos < 2 || vizinhos > 3 { 
            return false; 
        } else {
            return self.grade[pos];
        }
    }

    pub fn simulate(&mut self) {
        self.counter += 1;

        for (x,y) in self.grade.positions() {
            self.prev_grade[(x,y)] = self.process_cell((x,y));
        }

        // A FAZER: Investigar se isso está copiando ou não...
        mem::swap(&mut self.grade, &mut self.prev_grade);        
    }
}

impl TerminalHandler for Life {
    fn on_draw(&mut self, term: &mut Terminal) -> io::Result<()> {
        let t = &mut term.stdout;

        // Roda uma etapa da simulação
        if !self.paused {
            let steps = if self.fast { self.fast = false; 10 } else { 1 };
            for _ in 0..steps {
                self.simulate();
            }
        }
        
        // Reset
        t.execute(Clear(ClearType::All))?;
        t.execute(MoveTo(0,0))?;

        // Desenha cursor
        t.execute(SetForegroundColor(Color::White))?;
        t.execute(MoveTo((self.pos.0 * 2) as u16,self.pos.1 as u16))?;
        print!("██");

        // Barra inferior de informações
        t.execute(SetForegroundColor(Color::Green))?;
        t.execute(MoveTo(0,(self.size.1-1) as u16))?;
        t.execute(SetBackgroundColor(Color::White))?;
        print!("GEN {}",self.counter);

        t.execute(SetBackgroundColor(Color::Black))?;
        print!("  ");

        t.execute(SetBackgroundColor(Color::White))?;
        print!("ESPACE");
        t.execute(SetBackgroundColor(Color::Black))?;
        print!("{}",if self.paused { " Continuar " } else { " Pausar "});

        t.execute(SetBackgroundColor(Color::White))?;
        print!("ENTER");
        t.execute(SetBackgroundColor(Color::Black))?;
        print!(" Marcar ");

        t.execute(SetBackgroundColor(Color::White))?;
        print!("←↑→↓ ");
        t.execute(SetBackgroundColor(Color::Black))?;
        print!(" Mover ");

        t.execute(SetBackgroundColor(Color::White))?;
        print!("F");
        t.execute(SetBackgroundColor(Color::Black))?;
        print!(" Rápido ");

        // Desenha grade
        t.execute(SetForegroundColor(Color::Green))?;
        t.execute(SetBackgroundColor(Color::Black))?;
        
        for (x,y) in self.grade.positions() {
            let c = self.grade[(x,y)];
            if c { 
                t.queue(MoveTo((x*2) as u16,y as u16))?
                .queue(Print("██"))?;
            }

            /*if c {
                print!("██");
            } else {
                print!("  ");
            }
            if x >= self.size.0 -1 {
                t.execute(MoveToNextLine(1))?;
            }*/
        }

        Ok(())
    }

    fn on_key_event(&mut self, e: &crossterm::event::KeyEvent) {
        match e.code {
            KeyCode::Char('f') => { self.fast = true; }
            KeyCode::Char(' ') => { self.paused = !self.paused; }
            KeyCode::Enter => {
                let index = (self.pos.0 as usize, self.pos.1 as usize);
                self.grade[index] = !self.grade[index];
            },
            KeyCode::Left => self.pos = wrap_xy((self.pos.0 as i32 - 1, self.pos.1 as i32), self.size),
            KeyCode::Right => self.pos = wrap_xy((self.pos.0 as i32 + 1, self.pos.1 as i32), self.size),
            KeyCode::Up => self.pos = wrap_xy((self.pos.0 as i32, self.pos.1 as i32 - 1), self.size),
            KeyCode::Down => self.pos = wrap_xy((self.pos.0 as i32, self.pos.1 as i32 + 1), self.size),
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
    
    t.main_loop(250, true, Life::new(((size.0/2) as usize, size.1 as usize)))?;

    t.stdout.execute(EnableLineWrap)?;

    Ok(())
}

pub fn life() {
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