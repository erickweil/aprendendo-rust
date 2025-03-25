use std::{clone, collections::HashSet, io::{self, Stdout}, mem};

use rand::Rng;

use crossterm::{cursor::*, event::*, queue, style::*, terminal::{self, Clear, ClearType}, ExecutableCommand, QueueableCommand};

use crate::{estruturas::{Dir, GraphTraversal, GraphTraversalIter, Iterator2D, LinkedStack, Stack, Vec2D}, utils::{Terminal, TerminalHandler}};

#[derive(Clone)]
enum GradeCell {
    Empty {
        minas: i32,
        explorado: bool
    },
    Mine
}

#[derive(PartialEq)]
#[derive(Clone, Copy)]
enum GameState {
    Running,
    Win,
    Lose
}

struct MineSweeperGame {
    pos: (i32, i32),
    prev_pos: (usize,usize),
    pressed_enter: bool,
    grade: Vec2D<GradeCell>,
    marcacoes: HashSet<(usize,usize)>,
    grade_dirty: Option<HashSet<(usize,usize)>>,
    n_minas: i32,
    n_explorados: i32,
    state: GameState
}

impl GraphTraversal<(usize,usize)> for MineSweeperGame {    
    fn visit<'a>(&mut self, (gx,gy): (usize,usize), neighbors: &'a mut Vec<(usize,usize)>) -> &'a mut Vec<(usize,usize)> {
        let (w,h) = self.grade.size();
        if let GradeCell::Empty { minas, explorado } = &mut self.grade[(gx,gy)] {
            // explorar ele próprio
            if !*explorado {
                *explorado = true;
                self.n_explorados += 1;

                Self::set_dirty(&mut self.grade_dirty, (gx,gy));

                // Se estiver marcado, remove marcação, pois não é mina
                self.marcacoes.remove(&(gx,gy));

                if *minas == 0 { // Só possui vizinhos se for 0
                    let (w,h) = (w as i32, h as i32);
                    let (gx, gy) = (gx as i32, gy as i32);
                    [
                        (gx-1, gy-1), (gx  ,gy-1), (gx+1, gy-1),
                        (gx-1, gy  ),              (gx+1, gy  ),
                        (gx-1, gy+1), (gx  ,gy+1), (gx+1, gy+1)
                    ].into_iter().for_each(|(x,y)| {
                        if x >= 0 && y >= 0 && x < w && y < h {
                            neighbors.push((x as usize, y as usize));
                        }
                    });
                }
            }
        }

        return neighbors;
    }
}

impl MineSweeperGame {
    fn new (size: (i32, i32)) -> MineSweeperGame {
        let center = (size.0 / 2, size.1 / 2);
        let mut grade = Vec2D::new(size.0 as usize, size.1 as usize, GradeCell::Empty { minas: 0, explorado: false });
        let n_minas = MineSweeperGame::inicializar_grade(&mut grade, center);
        MineSweeperGame {
            pos: center,
            prev_pos: (0,0),
            pressed_enter: false,
            grade: grade,
            marcacoes: HashSet::new(),
            grade_dirty: Some(HashSet::new()),
            n_minas: n_minas,
            n_explorados: 0,
            state: GameState::Running
        }
    }

    // =================================================================
    // Métodos da lógica do jogo
    // =================================================================
    fn is_mine(grade: &Vec2D<GradeCell>, (x,y): (i32,i32)) -> bool {
        if let Some(cell) = grade.get((x,y)) {
            match cell {
                GradeCell::Empty { minas, explorado } => false,
                GradeCell::Mine => true,
            }
        } else {
            false
        }
    }

    fn is_mine_or_unexplored(&self, (x,y): (usize,usize)) -> bool {
        if let Some(cell) = self.grade.get((x as i32,y as i32)) {
            match cell {
                GradeCell::Empty { minas, explorado } => !explorado,
                GradeCell::Mine => true,
            }
        } else {
            true
        }        
    }

    fn inicializar_grade(grade: &mut Vec2D<GradeCell>, center: (i32, i32)) -> i32 {
        let mut rng = rand::rng();
        let (w,h) = grade.size();
        let gerar_minas = grade.len() / 20 + 1;
        let mut total_minas = 0;
        for _ in 0..gerar_minas {
            let rdnpos = (
                rng.random_range(0..w),
                rng.random_range(0..h)
            );
            if let GradeCell::Mine = grade[rdnpos] { 
                // já tem mina aqui
                continue; 
            }
            if rdnpos.0 >= (center.0-1) as usize && rdnpos.0 <= (center.0+1) as usize
            && rdnpos.1 >= (center.1-1) as usize && rdnpos.1 <= (center.1+1) as usize {
                // não perder de primeira
                continue;
            }

            grade[rdnpos] = GradeCell::Mine;
            total_minas += 1;
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

        return total_minas;
    }

    fn explorar(&mut self, (gx,gy): (usize,usize)) {
        match &mut self.grade[(gx,gy)] {
            GradeCell::Empty { minas, explorado } => {
                GraphTraversalIter::breadth_first(self, (gx, gy));

                if self.verificar_ganhou() {
                    self.state = GameState::Win;

                    self.set_dirty_all_mines();
                }
            },
            GradeCell::Mine => {
                if self.state == GameState::Running {
                    self.state = GameState::Lose;

                    self.set_dirty_all_mines();
                }
            },
        }
    }

    fn set_dirty_all_mines(&mut self) {
        for (gx,gy) in Iterator2D::xy(self.grade.size()) {
            if let GradeCell::Mine = self.grade[(gx,gy)] {
                Self::set_dirty(&mut self.grade_dirty, (gx,gy));
            }
        }
    }

    fn set_dirty(grade_dirty: &mut Option<HashSet<(usize,usize)>>, pos: (usize,usize)) {
        if let Some(grade_dirty) = grade_dirty.as_mut() {
            grade_dirty.insert(pos);
        }
    }

    // Se não tem mais nenhum explorado, sobrou só as minas, então ganhou!
    fn verificar_ganhou(&self) -> bool {
        return self.n_explorados + self.n_minas >= self.grade.len() as i32;
    }

    // =================================================================
    // Métodos de desenhar no terminal
    // =================================================================

    fn draw_cell(&self, t: &mut Stdout, (x,y): (usize,usize), grade_pos: (usize,usize), highlight: bool) -> io::Result<()> {
        let mut draw_empty = false;
        t.queue(MoveTo(x as u16,y as u16))?;
        if highlight {
            t.queue(SetAttribute(Attribute::Reverse));
        }
        match self.grade[grade_pos] {
            GradeCell::Empty { minas, explorado } => {
                if explorado {
                    t.queue(SetBackgroundColor(Color::Black))?;
                    if minas == 0 {
                        t.queue(SetForegroundColor(Color::White))?.queue(Print(" "))?;
                    } else if minas == 1 {
                        t.queue(SetForegroundColor(Color::Blue))?.queue(Print(minas))?;
                    } else if minas == 2 {
                        t.queue(SetForegroundColor(Color::Green))?.queue(Print(minas))?;
                    } else if minas == 3 {
                        t.queue(SetForegroundColor(Color::Red))?.queue(Print(minas))?;
                    } else if minas == 4 {
                        t.queue(SetForegroundColor(Color::DarkRed))?.queue(Print(minas))?;
                    } else if minas == 5 {
                        t.queue(SetForegroundColor(Color::Magenta))?.queue(Print(minas))?;
                    } else {
                        t.queue(SetForegroundColor(Color::DarkMagenta))?.queue(Print(minas))?;
                    }
                } else {
                   draw_empty = true;
                }
            },
            GradeCell::Mine => {
                if self.state != GameState::Running {
                    t.queue(SetForegroundColor(Color::Black))?
                    .queue(SetBackgroundColor(Color::White))?
                    .queue(Print("¤"))?;
                } else {
                    draw_empty = true;
                }
            },
        }
        if draw_empty {
            if self.marcacoes.contains(&grade_pos) {
                t.queue(SetBackgroundColor(Color::DarkGrey))?
                .queue(SetForegroundColor(Color::Red))?
                .queue(Print("⌖"))?;
            } else {
                t.queue(SetBackgroundColor(Color::DarkGrey))?
                .queue(SetForegroundColor(Color::White))?
                .queue(Print(" "))?;
            }
        }
        if highlight {
            t.queue(SetAttribute(Attribute::NoReverse));
        }

        Ok(())
    }

    fn draw_in_between(&self, t: &mut Stdout, (x,y): (usize,usize), (gx,gy): (usize,usize)) -> io::Result<()> {
        let esquerda = self.is_mine_or_unexplored((gx,   gy));
        let direita =  self.is_mine_or_unexplored((gx+1, gy));

        if !esquerda && !direita {
            t.queue(MoveTo(x as u16,y as u16))?
            .queue(SetForegroundColor(Color::White))?
            .queue(SetBackgroundColor(Color::Black))?
            .queue(Print(" "))?;
        } else if !esquerda && direita {
            t.queue(MoveTo(x as u16,y as u16))?
            .queue(SetBackgroundColor(Color::Black))?
            .queue(SetForegroundColor(Color::DarkGrey))?
            .queue(Print("▐"))?;
        } else if esquerda && !direita {
            t.queue(MoveTo(x as u16,y as u16))?
            .queue(SetBackgroundColor(Color::Black))?
            .queue(SetForegroundColor(Color::DarkGrey))?
            .queue(Print("▌"))?;
        }

        Ok(())
    }
}

impl TerminalHandler for MineSweeperGame {
    fn on_draw(&mut self, term: &mut Terminal) -> io::Result<()> {
        let t = &mut term.stdout;
        let (w,h) = self.grade.size();
        
        let p = (self.pos.0 as usize,self.pos.1 as usize);
        if p != self.prev_pos {
            Self::set_dirty(&mut self.grade_dirty, self.prev_pos);
            Self::set_dirty(&mut self.grade_dirty, p);
        }
        self.prev_pos = p;

        if self.pressed_enter {
            self.pressed_enter = false;
            self.explorar(p);
        }

        // Barra inferior de informações
        t.queue(MoveTo(0,h as u16))?
        .queue(SetForegroundColor(Color::Black))?
        .queue(SetBackgroundColor(Color::White))?
        .queue(Clear(ClearType::CurrentLine))?;

        match self.state {
            GameState::Running => {
                t.queue(MoveTo(0,h as u16))?
                    .queue(Print("minas:"))?
                    .queue(Print(self.marcacoes.len()))?
                    .queue(Print("/"))?
                    .queue(Print(self.n_minas))?
                    .queue(Print("  "))?
                    .queue(Print("ENTER"))?
                    .queue(Print(" revelar  "))?
                    .queue(Print("ESPAÇO"))?
                    .queue(Print(" marcar "))?;
            },
            GameState::Win => {
                t.queue(MoveTo(0,h as u16))?
                .queue(Print("VOCÊ GANHOU!!!!!!!!"))?;
            },
            GameState::Lose => {
                t.queue(MoveTo(0,h as u16))?
                .queue(Print("... KABUM"))?;
            },
        }
        
        // GAMBIARRA para poder atravessar grade_dirty sem dar problema de ownership
        // basicamente durante o processo de iteração vai deixar um None no lugar
        self.grade_dirty = if let Some(mut dirty) = self.grade_dirty.take() {
            // DEBUG
            // t.execute(MoveTo(0,(self.size.1-1) as u16))?;
            // t.execute(SetForegroundColor(Color::Black))?;
            // t.execute(SetBackgroundColor(Color::White))?;
            // print!("DIRTY {}  ", dirty.len());

            for &(gx,gy) in dirty.iter() {
                let highlight = self.pos == (gx as i32, gy as i32);
                
                if gx > 0 && !dirty.contains(&(gx - 1, gy)) {
                    self.draw_in_between(t, (gx * 2 - 1, gy), (gx-1, gy))?;
                }
                self.draw_in_between(t, (gx * 2 + 1, gy), (gx, gy))?;
                self.draw_cell(t, (gx * 2 + 0, gy), (gx, gy), highlight)?;
            }
            dirty.clear();

            Some(dirty)
        } else {
            None // Nunca deveria ocorrer
        };

        Ok(())
    }

    fn on_key_event(&mut self, e: &crossterm::event::KeyEvent) {
        if self.state != GameState::Running {
            return;
        }

        let (x,y) = self.pos;
        let (w,h) = self.grade.size();
        let (w,h) = (w as i32, h as i32);
        match e.code {
            KeyCode::Char(' ') => {
                let pos_marcacao = (self.pos.0 as usize, self.pos.1 as usize);
                if self.is_mine_or_unexplored(pos_marcacao) && !self.marcacoes.contains(&pos_marcacao) {
                    self.marcacoes.insert(pos_marcacao);
                } else {
                    self.marcacoes.remove(&pos_marcacao);
                }
                Self::set_dirty(&mut self.grade_dirty, pos_marcacao);
            }
            KeyCode::Enter => { self.pressed_enter = true; },
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
    t.stdout.execute(SetBackgroundColor(Color::DarkGrey))?;
    t.stdout.execute(SetForegroundColor(Color::White))?;
    t.stdout.execute(terminal::DisableLineWrap)?;
    
    t.main_loop(500, true, MineSweeperGame::new(((size.0/2) as i32, (size.1-1) as i32)))?;

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