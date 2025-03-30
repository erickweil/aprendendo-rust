use std::io;

use crossterm::{cursor::*, event::KeyCode, style::*, terminal::*, ExecutableCommand, QueueableCommand};
use rand::Rng;

use crate::{estruturas::Vec2D, utils::{StyledChar, Terminal, TerminalHandler, TerminalScreen}};


struct DungeonGame {
    screen: TerminalScreen,
    pos: (i32, i32)
}

impl DungeonGame {
    fn new (size: (usize, usize)) -> DungeonGame {       
        DungeonGame {
            screen: TerminalScreen::new(size),
            pos: (0,0)
        }
    }
}

impl TerminalHandler for DungeonGame {
    fn on_draw(&mut self, term: &mut Terminal) -> io::Result<()> {
        let t = &mut term.stdout;
        
        //self.screen.clear(t, Color::Black);
        {
            let (x,y) = self.pos;
            self.screen.set(t, (x as usize, y as usize), StyledChar::new('▮', Color::Black, Color::Yellow));
        }

        /*let (w,h) = self.screen.size();
        let mut rng = rand::rng();
        for i in 0..100 {
            let mut rdnpos = (
                rng.random_range(0..w),
                rng.random_range(0..h)
            );
            for x in 0..100 {
                self.screen.set(t, rdnpos, StyledChar::new('.', Color::Black, Color::Green));
                rdnpos.1 += 1;
                if rdnpos.1 >= h { break; }
            }
        }*/

        self.screen.draw(t);

        Ok(())
    }

    fn on_key_event(&mut self, e: &crossterm::event::KeyEvent) {
        let (x,y) = self.pos;
        let (w,h) = self.screen.size();
        let (w,h) = (w as i32, h as i32);
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

    let size = size()?;
    t.stdout.execute(SetBackgroundColor(Color::Black))?;
    t.stdout.execute(SetForegroundColor(Color::Green))?;
    t.stdout.execute(DisableLineWrap)?;
    
    t.main_loop(60000, true, DungeonGame::new((size.0 as usize, size.1 as usize)))?;

    t.stdout.execute(EnableLineWrap)?;

    Ok(())
}

pub fn dungeon() {
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