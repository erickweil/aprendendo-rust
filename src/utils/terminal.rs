// https://en.wikipedia.org/wiki/ANSI_escape_code
// https://medium.com/@protiumx/creating-a-text-based-ui-with-rust-2d8eaff7fe8b

use crossterm::cursor::MoveTo;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{BeginSynchronizedUpdate, Clear, ClearType, EndSynchronizedUpdate};
use crossterm::QueueableCommand;
use crossterm::{
    cursor::{Hide, Show},
    style::*,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use std::collections::VecDeque;
use std::io::{stdout, Stdout};
use std::time::Instant;
use std::usize;
use std::{io::{self, Write}, thread::sleep, time::Duration};

use crate::estruturas::Vec2D;

pub trait TerminalHandler {
    fn on_draw(&mut self, _terminal: &mut Terminal) -> io::Result<()> {
        Ok(())
    }

    fn on_key_event(&mut self, _key_event: &KeyEvent) {

    }
}

pub struct Terminal {
    pub stdout: Stdout,
    pub debug: bool
}

// https://blog.stackademic.com/rust-terminal-manipulation-with-crossterm-d14e76617a3d
impl Terminal {
    pub fn new() -> Terminal {
        Terminal { stdout: stdout(), debug: false }
    }

    fn nonblocking_read_event(&self, timeout: u64) -> Option<Event> {
        match event::poll(Duration::from_millis(timeout)) {
            Ok(true) => {
                match event::read() {
                    Ok(event) => Some(event),
                    Err(_) => None
                }
            },
            Ok(false) => None,
            Err(_) => None
        }
    }

    /**
     * A forma que delay funciona é que irá esperar esse tanto em milissegundos até o próximo evento, ou então se tiver evento já não irá esperar
     * Assim pode colocar o delay um valor alto, mas quando houver input a tela vai atualizar mais rápido
     */
    pub fn main_loop<H: TerminalHandler>(&mut self, delay: u64, draw_on_event: bool, mut handler: H) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        self.stdout.execute(EnterAlternateScreen)?;
        self.stdout.execute(Hide)?;

        'main: loop {
            let before = Instant::now();

            self.stdout.execute(BeginSynchronizedUpdate)?;
            // Executa operações no terminal
            handler.on_draw(self)?;

            // Escreve tudo e espera um tempo
            self.stdout.flush()?;

            self.stdout.execute(EndSynchronizedUpdate)?;

            let frame_time = before.elapsed();

            if self.debug {
                // Debug FPS
                self.stdout.queue(SetForegroundColor(Color::Black))?
                .queue(SetBackgroundColor(Color::White))?
                .queue(MoveTo(0,0))?
                .queue(Print("              "))?
                .queue(MoveTo(0,0))?
                .queue(Print("µs:"))?
                .queue(Print(frame_time.as_micros()))?;

                self.stdout.flush()?;
            }


            if !draw_on_event {
                sleep(Duration::from_millis(delay));
            }
            // Consumir todos os eventos
            let mut first_event = true;
            loop {
                let event = self.nonblocking_read_event(if first_event && draw_on_event { delay } else { 0 });
                first_event = false;

                if let Some(event) = event {
                    if let Event::Key(key_event) = event.to_owned() {
                        // CTRL + C
                        if key_event.code == KeyCode::Esc || (
                                key_event.code == KeyCode::Char('c')
                                && key_event.modifiers == KeyModifiers::CONTROL
                        ) {
                            break 'main;
                        }

                        if key_event.code == KeyCode::F(1) {
                            self.debug = !self.debug;
                        }

                        handler.on_key_event(&key_event);
                    }
                } else {
                    // Acabaram os eventos
                    break;
                }
            }
        }
    
        self.stdout.execute(Clear(terminal::ClearType::All))?;
        self.stdout.execute(Show)?;
        self.stdout.execute(LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;

        self.stdout.execute(ResetColor)?;
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StyledChar {
    foreground: Color,
    background: Color,
    character: char,
}

impl StyledChar {
    pub fn new(c: char, b: Color, f: Color) -> StyledChar {
        StyledChar {
            character: c,
            background: b,
            foreground: f
        }
    }
}

/**
 * A ideia é possuir todos os caracteres que devem ser desenhados
 * e só de fato executar os comandos do que mudou em relação à tela anterior
 * 
 * Só vai ser bom caso na maioria das vezes não mudou muito a tela
 */
pub struct TerminalScreen {
    frame: i32,
    drawed: i32,
    screen: Vec2D<StyledChar>,
    active_background: Color,
    active_foreground: Color,
    //to_draw: VecDeque<((usize,usize), StyledChar)>
    //prev_screen: Vec2D<StyledChar>
}

impl TerminalScreen {
    pub fn new(size: (usize,usize)) -> TerminalScreen {
        let screen = Vec2D::new(size.0, size.1, StyledChar::new(' ', Color::Reset, Color::Reset) );
        TerminalScreen {
            frame: 0,
            drawed: 0,
            //prev_screen: screen.clone(),
            active_background: Color::Reset,
            active_foreground: Color::Reset,
            screen: screen,
            //to_draw: VecDeque::new()
        }
    }

    pub fn size(&self) -> (usize, usize) {
        return self.screen.size();
    }

    pub fn clear(&mut self, t: &mut Stdout, color: Color) -> io::Result<()> {
        t.queue(SetBackgroundColor(color))?;
        t.execute(Clear(ClearType::All))?;

        self.active_background = Color::Reset;
        self.active_foreground = Color::Reset;
        for pos in self.screen.positions() {
            self.screen[pos] = StyledChar::new(' ', color, color);
        }

        Ok(())
    }

    pub fn set(&mut self, t: &mut Stdout, pos: (usize, usize), c: StyledChar) -> io::Result<()> {
        // Se já estava, ignora
        if self.screen[pos] != c {
            // só muda a cor se realmente precisar
            if c.background != self.active_background {
                t.queue(SetBackgroundColor(c.background))?;
                self.active_background = c.background;
            }
            if c.foreground != self.active_foreground {
                t.queue(SetForegroundColor(c.foreground))?;
                self.active_foreground = c.foreground;
            }
            
            // Vai lá e desenha
            t.queue(MoveTo(pos.0 as u16,pos.1 as u16))?
            .queue(Print(c.character))?;

            // Registra o que desenhou
            self.screen[pos] = c;
            self.drawed += 1;
            //self.to_draw.push_back((pos,c));
        }

        Ok(())
    }

    pub fn draw(&mut self, t: &mut Stdout) -> io::Result<()> {
        let (w,h) = self.screen.size();

        //let mut drawed = 0;
        //let mut prev_background = Color::Reset;
        //let mut prev_foreground = Color::Reset;
        /*for pos in self.screen.positions() {
            let c = self.screen[pos];

            if c == self.prev_screen[pos] {
                continue; // Não precisa re-desenhar tá igual
            }
            self.prev_screen[pos] = c;

            // só muda a cor se realmente precisar
            if c.background != prev_background {
                t.queue(SetBackgroundColor(c.background))?;
                prev_background = c.background;
            }
            if c.foreground != prev_foreground {
                t.queue(SetForegroundColor(c.foreground))?;
                prev_foreground = c.foreground;
            }

            // alguns caracteres movem mais que 1 pro lado, precisa sempre setar a posição
            t.queue(MoveTo(pos.0 as u16,pos.1 as u16))?
            .queue(Print(c.character))?;
            drawed += 1;
        }*/
        /*
        while let Some((pos,c)) = self.to_draw.pop_front() {
            self.screen[pos] = c;

            // só muda a cor se realmente precisar
            if c.background != prev_background {
                t.queue(SetBackgroundColor(c.background))?;
                prev_background = c.background;
            }
            if c.foreground != prev_foreground {
                t.queue(SetForegroundColor(c.foreground))?;
                prev_foreground = c.foreground;
            }

            // alguns caracteres movem mais que 1 pro lado, precisa sempre setar a posição
            t.queue(MoveTo(pos.0 as u16,pos.1 as u16))?
            .queue(Print(c.character))?;
            drawed += 1;
        }*/

        // Barra inferior de informações
        t.queue(SetForegroundColor(Color::Black))?
        .queue(SetBackgroundColor(Color::White))?
        .queue(MoveTo(0,(h-1) as u16))?
        .queue(Print("               "))?
        .queue(MoveTo(0,(h-1) as u16))?
        .queue(Print(self.frame))?
        .queue(Print("  draw:"))?
        .queue(Print(self.drawed))?;

        self.active_background = Color::Reset;
        self.active_foreground = Color::Reset;

        self.frame += 1;
        self.drawed = 0;

        Ok(())
    }
}