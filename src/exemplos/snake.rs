use std::io;

use rand::Rng;

use crossterm::{cursor::MoveTo, event::KeyCode, style::{Color, Print, SetBackgroundColor, SetForegroundColor}, terminal::{size, DisableLineWrap, EnableLineWrap}, ExecutableCommand, QueueableCommand};

use crate::{estruturas::{Dir, LinkedList, Queue}, utils::{Terminal, TerminalHandler}};
struct Snake {
    prev_tail: (i32,i32),
    body: LinkedList<(i32,i32)>,
    dir: Dir,
    prev_dir: Dir
}

impl Snake {
    pub fn slither(&mut self, size: (i32,i32)) -> bool {
        let off = self.dir.to_xy();
        let pos = self.snake_head();
        let new_pos = (pos.0 + off.0, pos.1 - off.1);

        if self.check_collision_self(new_pos) {
            return true;
        }

        if Snake::check_collision_wall(new_pos,size) {
            return true;
        }

        self.body.enqueue(new_pos);
        self.prev_dir = self.dir;
        return false;
    }

    pub fn shrink_tail(&mut self) {
        self.prev_tail = self.body.dequeue().expect("Deve chamar shrink_tail só depois de slither");
    }

    pub fn check_collision_wall((x,y): (i32,i32), (w,h): (i32,i32)) -> bool {
        if x < 0 || y < 0 || x >= w || y >= h {
            return true;
        } else {
            return false;
        }
    }

    pub fn check_collision_self(&self, pos: (i32,i32)) -> bool {
        for value in self.body.iter() {
            if *value == pos { 
                return true;
            }
        }
        return false;
    }

    pub fn snake_head(&self) -> (i32,i32) {
        return *self.body.tail().expect("Cobra não existe?")
    }
}

#[derive(PartialEq)]
#[derive(Clone, Copy)]
enum GameState {
    Running,
    End
}

struct SnakeGame {
    size: (i32,i32),
    player: Snake,
    fruit: (i32,i32),
    score: i32,
    state: GameState
}

impl SnakeGame {
    pub fn new(size: (i32,i32)) -> SnakeGame {
        let snake_pos = (1, (size.1 / 2) as i32);
        SnakeGame {
            size: size,
            state: GameState::Running,
            score: 0,
            fruit: ((size.0 / 3) as i32, (size.1 / 3) as i32),
            player: Snake {
                prev_tail: (snake_pos.0-1,snake_pos.1),
                body: LinkedList::from([snake_pos]),
                dir: Dir::Right,
                prev_dir: Dir::Right
            }
        }
    }

    pub fn gen_fruit_pos(&self) -> Option<(i32,i32)> {
        let mut tentativas = 1_000_000; // irá no máximo fazer 1 milhão de tentativas de gerar uma fruta (para evitar possível loop infinito)
        let mut rng = rand::rng();
        loop {
            tentativas -= 1;
            if tentativas <= 0 {
                return None; 
            }
            let fruit_pos = (
                rng.random_range(0..self.size.0),
                rng.random_range(0..self.size.1-1)
            );
            if !self.player.check_collision_self(fruit_pos) {
                return Some(fruit_pos);
            }
        }
    }

    pub fn simulate(&mut self) -> GameState {
        // anda para frente, crescendo temporariamente
        if self.player.slither(self.size) {
            return GameState::End; // FIM DO JOGO, colidiu
        }

        if self.player.snake_head() == self.fruit {
            self.score += 1;            
            self.fruit = if let Some(pos) = self.gen_fruit_pos() { pos } else {
                return GameState::End; // FIM DO JOGO, NÃO CONSEGUIU GERAR FRUTA, A COBRA COBRIU O MAPA INTEIRO
            };
        } else {
            // se não comeu fruta, remove 1 da cauda para não crescer
            self.player.shrink_tail();
        }

        // Continua o jogo
        return GameState::Running;
    }
}

impl TerminalHandler for SnakeGame {
    fn on_draw(&mut self, term: &mut Terminal) -> io::Result<()> {
        let t = &mut term.stdout;

        if self.state == GameState::Running {
            self.state = self.simulate();
        } else if self.state == GameState::End {
            let mensagem = "  FIM DO JOGO  ";
            let center = ((self.size.0 - (mensagem.len()/2) as i32) as u16, (self.size.1/2) as u16);
            t
            .queue(SetBackgroundColor(Color::White))?
            .queue(SetForegroundColor(Color::White))?
            .queue(MoveTo(center.0,center.1-1))?
            .queue(Print("               "))?
            .queue(MoveTo(center.0,center.1+1))?
            .queue(Print("               "))?
            .queue(MoveTo(center.0,center.1))?
            .queue(SetForegroundColor(Color::Black))?
            .queue(Print(mensagem))?;
        }
        
        let fruit_pos = self.fruit;
        let head_pos = self.player.snake_head();
        let prev_tail_pos = self.player.prev_tail;

        // Barra inferior de informações
        t.queue(SetForegroundColor(Color::Black))?
        .queue(SetBackgroundColor(Color::White))?
        .queue(MoveTo(0,(self.size.1-1) as u16))?
        .queue(Print("Pontos "))?
        .queue(Print(self.score))?
        .queue(SetBackgroundColor(Color::Black))?

        // fruta
        .queue(MoveTo((fruit_pos.0 * 2) as u16, fruit_pos.1 as u16))?
        .queue(Print("🍎"))?

        // Desenhar Cobra (irá apenas desenhar a 'cabeça' e apagar a última 'cauda' que foi removida)
        .queue(SetForegroundColor(Color::Green))?
        .queue(MoveTo((prev_tail_pos.0 * 2) as u16, prev_tail_pos.1 as u16))?
        .queue(Print("  "))?
        .queue(MoveTo((head_pos.0 * 2) as u16, head_pos.1 as u16))?
        .queue(Print("██"))?;

        Ok(())
    }

    fn on_key_event(&mut self, e: &crossterm::event::KeyEvent) {
        // Evitar que faça curva de 180º e perca imediatamente o jogo
        let prev = self.player.prev_dir;
        match e.code {
            KeyCode::Left =>  if prev != Dir::Right { self.player.dir = Dir::Left; },
            KeyCode::Right => if prev != Dir::Left  { self.player.dir = Dir::Right; },
            KeyCode::Up =>    if prev != Dir::Down  { self.player.dir = Dir::Up; },
            KeyCode::Down =>  if prev != Dir::Up    { self.player.dir = Dir::Down; },
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
    
    // 14-15 FPS
    t.main_loop(67, false, SnakeGame::new(((size.0/2) as i32, size.1 as i32)))?;

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