// https://en.wikipedia.org/wiki/ANSI_escape_code
// https://medium.com/@protiumx/creating-a-text-based-ui-with-rust-2d8eaff7fe8b

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{size, Clear, SetSize};
use crossterm::{
    cursor::{Hide, Show, MoveTo},
    style::*,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use std::io::{stdout, Stdout};
use std::{io::{self, Write}, thread::sleep, time::Duration};

pub struct Terminal {
    pub stdout: Stdout
}

// https://blog.stackademic.com/rust-terminal-manipulation-with-crossterm-d14e76617a3d
impl Terminal {

    pub fn new() -> Terminal {
        let mut out = stdout();
        terminal::enable_raw_mode().unwrap();
        out.execute(EnterAlternateScreen).unwrap();
        out.execute(Hide).unwrap();
        
        Terminal { stdout: out }
    }

    pub fn reset(&mut self) {
        self.stdout.execute(Show).unwrap();
        self.stdout.execute(LeaveAlternateScreen).unwrap();
        terminal::disable_raw_mode().unwrap();
    }

    pub fn get_size(&mut self) -> (u16, u16) {
        return size().unwrap();
    }

    /**
     * https://github.com/rust-lang/rust/issues/74964
     * Vai para (1,1) e limpa a tela, sem histórico de linhas
     */
    pub fn clear(&mut self) {
        //print!("{ESC}[H{ESC}[2J{ESC}[3J");
        self.stdout.execute(Clear(terminal::ClearType::All)).unwrap();
    }

    /**
     * Moves the cursor to row n, column m. The values are 1-based, (1,1) is top left corner
     */
    pub fn cursor_position(&mut self, (x,y): (u16,u16)) {
        //if x < 1 || y < 1 { panic!("Deve ser maior ou igual à 1"); }
        //print!("{ESC}[{x};{y}H");
        self.stdout.execute(MoveTo(x,y)).unwrap();
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.stdout.execute(SetBackgroundColor(color)).unwrap();
    }

    pub fn set_foreground_color(&mut self, color: Color) {
        self.stdout.execute(SetForegroundColor(color)).unwrap();
    }

    pub fn flush(&self) {
        io::stdout().flush().unwrap();
    }

    pub fn read_event(&self) -> Option<Event> {
        match event::poll(Duration::from_secs(0)) {
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

    pub fn should_exit(&self, event: &Event) -> bool {
        if let Event::Key(key_event) = event.to_owned() {
            if key_event.code == KeyCode::Esc
                || (key_event.code == KeyCode::Char('c')
                    && key_event.modifiers == KeyModifiers::CONTROL)
            {
                return true;
            }
        }
    
        return false;
    }

    pub fn main_loop<CB: FnMut(&mut Terminal)>(&mut self, delay: u64, mut callback: CB) {
        loop {
            let event = self.read_event();
            if let Some(event) = event {
                if self.should_exit(&event) == true {
                    break;
                }
            }
            callback(self);

            self.flush();
            sleep(Duration::from_millis(delay));
        }
    
        self.clear();
        self.reset();
    }
}

pub fn _exemplo() {
    let mut t = Terminal::new();

    let mut pos = (1,1);
    let size = t.get_size();
    t.set_background_color(Color::Black);
    t.set_foreground_color(Color::Green);
    t.main_loop(100, |t| {
        t.clear();

        pos.0 += 1;
        if pos.0 >= size.0 { pos.0 = 0; }
        pos.1 += 1;
        if pos.1 >= size.1 { pos.1 = 0; }
        t.cursor_position(pos);
        print!("HELLO ");
        print!("WORLD");
        //println!("HELLO");
    });  

    println!("");
    println!("");
    println!("");
    println!("");
    println!("");
}