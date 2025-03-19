// https://en.wikipedia.org/wiki/ANSI_escape_code
// https://medium.com/@protiumx/creating-a-text-based-ui-with-rust-2d8eaff7fe8b

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::Clear;
use crossterm::{
    cursor::{Hide, Show},
    style::*,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use std::io::{stdout, Stdout};
use std::{io::{self, Write}, thread::sleep, time::Duration};

pub trait TerminalHandler {
    fn on_draw(&mut self, _terminal: &mut Terminal) -> io::Result<()> {
        Ok(())
    }

    fn on_key_event(&mut self, _key_event: &KeyEvent) {

    }
}

pub struct Terminal {
    pub stdout: Stdout
}

// https://blog.stackademic.com/rust-terminal-manipulation-with-crossterm-d14e76617a3d
impl Terminal {
    pub fn new() -> Terminal {
        Terminal { stdout: stdout() }
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
                        handler.on_key_event(&key_event);
                    }
                } else {
                    // Acabaram os eventos
                    break;
                }
            }
            // Executa operações no terminal
            handler.on_draw(self)?;

            // Escreve tudo e espera um tempo
            self.stdout.flush()?;
            if !draw_on_event {
                sleep(Duration::from_millis(delay));
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