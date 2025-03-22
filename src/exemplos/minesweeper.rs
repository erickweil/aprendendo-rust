use std::io;

use rand::Rng;

use crossterm::{cursor::*, event::*, style::*, terminal, ExecutableCommand, QueueableCommand};

use crate::{estruturas::{Dir, Vec2D}, utils::{Terminal, TerminalHandler}};

#[derive(Clone)]
enum GradeCell {
    Empty {
        minas: i32,
        explorado: bool
    },
    Mine
}

impl GradeCell {
    fn empty() -> GradeCell {
        GradeCell::Empty { minas: 0, explorado: false }
    }

    fn mine() -> GradeCell {
        GradeCell::Mine
    }
}

struct MineSweeperGame {
    size: (i32, i32),
    pos: (i32, i32),
    grade: Vec2D<GradeCell>,
    n_minas: i32
}

impl MineSweeperGame {
    fn new (size: (i32, i32)) -> MineSweeperGame {
        let center = (size.0 / 2, size.1 / 2);
        let mut grade = Vec2D::new(size.0 as usize, size.1 as usize, GradeCell::empty());
        let n_minas = MineSweeperGame::inicializar_grade(&mut grade, size.0 + 2);
        MineSweeperGame {
            size: size, 
            pos: center, 
            grade: grade,
            n_minas
        }
    }

    fn is_mine(grade: &mut Vec2D<GradeCell>, (x,y): (i32,i32)) -> bool {
        let (w,h) = grade.size();
        if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
            return false;
        }

        if let GradeCell::Mine = grade[(x as usize,y as usize)] {
            return true;
        } else {
            return false;
        }
    }

    fn inicializar_grade(grade: &mut Vec2D<GradeCell>, minas: i32) -> i32 {
        let mut rng = rand::rng();
        let size = grade.size();
        let n_posicoes = (size.0 * size.1) as u32;
        let mut n_minas = 0;
        for (x,y) in grade.positions() {
            if rng.random_ratio(minas as u32, n_posicoes) {
                grade[(x,y)] = GradeCell::mine();
                n_minas += 1;
            }
        }
        for pos in grade.positions() {
            let grade_value = &grade[pos];
            if let GradeCell::Empty { minas, explorado } = grade_value {
                let mut minas = 0;
                let (x,y) = (pos.0 as i32, pos.1 as i32);
                if Self::is_mine(grade,(x-1, y-1)) { minas += 1; }
                if Self::is_mine(grade,(x  , y-1)) { minas += 1; }
                if Self::is_mine(grade,(x+1, y-1)) { minas += 1; }
                if Self::is_mine(grade,(x-1, y  )) { minas += 1; }
                if Self::is_mine(grade,(x+1, y  )) { minas += 1; }
                if Self::is_mine(grade,(x-1, y+1)) { minas += 1; }
                if Self::is_mine(grade,(x  , y+1)) { minas += 1; }
                if Self::is_mine(grade,(x+1, y+1)) { minas += 1; }

                grade[pos] = GradeCell::Empty { minas: minas, explorado: false };
            }
        }
        return n_minas;
    }
}

impl TerminalHandler for MineSweeperGame {
    fn on_draw(&mut self, term: &mut Terminal) -> io::Result<()> {
        let t = &mut term.stdout;

        t.execute(SetBackgroundColor(Color::Black))?;
        t.execute(SetForegroundColor(Color::Green))?;
        t.execute(terminal::Clear(terminal::ClearType::All))?;

        // Desenha grade
        t.execute(SetForegroundColor(Color::Green))?;
        t.execute(SetBackgroundColor(Color::Black))?;
        
        for (x,y) in self.grade.positions() {
            match self.grade[(x,y)] {
                GradeCell::Empty { minas, explorado } => {
                    t.queue(MoveTo((x*2) as u16,y as u16))?;
                    if explorado { 
                        t.queue(Print("██"))?;
                    } else if minas != 0 {
                        if minas == 1 {
                            t.queue(SetForegroundColor(Color::Blue))?;
                        }
                        if minas == 2 {
                            t.queue(SetForegroundColor(Color::Green))?;
                        }
                        if minas == 3 {
                            t.queue(SetForegroundColor(Color::Red))?;
                        }
                        if minas >= 4 {
                            t.queue(SetForegroundColor(Color::DarkRed))?;
                        }
                        t.queue(Print(minas))?.queue(Print(" "));
                    }
                },
                GradeCell::Mine => {
                    t.queue(MoveTo((x*2) as u16,y as u16))?
                    .queue(SetForegroundColor(Color::DarkRed))?
                    .queue(Print("█ "))?;
                },
            }
        }

        t
        .queue(MoveTo((self.pos.0*2) as u16, self.pos.1 as u16))?
        .queue(Print("? "))?;

        Ok(())
    }

    fn on_key_event(&mut self, e: &crossterm::event::KeyEvent) {
        // Evitar que faça curva de 180º e perca imediatamente o jogo
        let (x,y) = self.pos;
        let (w,h) = self.size;
        match e.code {
            KeyCode::Left =>  if x > 0 { self.pos.0 -= 1; },
            KeyCode::Right => if x < w-1 { self.pos.0 += 1; },
            KeyCode::Up =>    if y > 0 { self.pos.1 -= 1; },
            KeyCode::Down =>  if y < h-1 { self.pos.1 += 1; }
            _ => {

            }
        }
    }
}

fn setup_terminal() -> io::Result<()> {
    let mut t = Terminal::new();

    let size = terminal::size()?;
    t.stdout.execute(SetBackgroundColor(Color::Black))?;
    t.stdout.execute(SetForegroundColor(Color::Green))?;
    t.stdout.execute(terminal::DisableLineWrap)?;
    
    t.main_loop(500, true, MineSweeperGame::new(((size.0/2) as i32, size.1 as i32)))?;

    t.stdout.execute(terminal::EnableLineWrap)?;

    Ok(())
}

pub fn mine_sweeper() {
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