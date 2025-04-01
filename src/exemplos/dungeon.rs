use std::io::{self, Stdout};

use crossterm::{cursor::*, event::KeyCode, style::*, terminal::*, ExecutableCommand, QueueableCommand};
use rand::Rng;

use crate::{estruturas::Vec2D, utils::{StyledChar, Terminal, TerminalHandler, TerminalScreen}};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tile {
    Void,
    WallV,
    WallH,
    WallNE,
    WallNW,
    WallSW,
    WallSE,
    Ground
}

impl Tile {
    fn get_char(&self) -> StyledChar {
        match self {
            Tile::Void   => StyledChar::new(' ', Color::Black, Color::White),
            Tile::WallV  => StyledChar::new('║', Color::Black, Color::DarkYellow),
            Tile::WallH  => StyledChar::new('═', Color::Black, Color::DarkYellow),
            Tile::WallNE => StyledChar::new('╗', Color::Black, Color::DarkYellow),
            Tile::WallNW => StyledChar::new('╔', Color::Black, Color::DarkYellow),
            Tile::WallSW => StyledChar::new('╚', Color::Black, Color::DarkYellow),
            Tile::WallSE => StyledChar::new('╝', Color::Black, Color::DarkYellow),
            Tile::Ground => StyledChar::new('.', Color::Black, Color::Green)
        }
    }
}

struct DungeonGame {
    screen: TerminalScreen,
    tiles: Vec2D<Tile>,
    pos: (i32, i32)
}

impl DungeonGame {
    fn new (size: (usize, usize)) -> DungeonGame {       
        let mut g = DungeonGame {
            screen: TerminalScreen::new(size),
            tiles: Vec2D::new(size.0, size.1, Tile::Void),
            pos: (0,0)
        };

        g.generate();

        return g;
    }

    /** https://gamedev.stackexchange.com/questions/50570/creating-and-connecting-rooms-for-a-roguelike
     * 
     * https://www.reddit.com/r/roguelikedev/comments/99b5fe/original_rogue_1980_duneon_generation/?rdt=51669
     * https://web.archive.org/web/20131025132021/http://kuoi.org/~kamikaze/GameDesign/art07_rogue_dungeon.php
     * By Mark Damon Hughes <kamikaze@kuoi.asui.uidaho.edu>

    The original Rogue algorithm is pretty nifty. Any time you need a random dungeon, give this a try:

        1. Divide the map into a grid (Rogue uses 3x3, but any size will work).
        2. Give each grid a flag indicating if it's "connected" or not, and an array of which grid numbers it's connected to.
        3. Pick a random room to start with, and mark it "connected".
        4. While there are unconnected neighbor rooms, connect to one of them, make that the current room, mark it "connected", and repeat.
        5. While there are unconnected rooms, try to connect them to a random connected neighbor (if a room has no connected neighbors yet, just keep cycling, you'll fill out to it eventually).
        6. All rooms are now connected at least once.
        7. Make 0 or more random connections to taste; I find rnd(grid_width) random connections looks good.
        8. Draw the rooms onto the map, and draw a corridor from the center of each room to the center of each connected room, changing wall blocks into corridors. If your rooms fill most or all of the space of the grid, your corridors will very short - just holes in the wall.
        9. Scan the map for corridor squares with 2 bordering walls, 1-2 bordering rooms, and 0-1 bordering corridor, and change those to doors.
        10. Place your stairs up in the first room you chose, and your stairs down in the last room chosen in step 5. This will almost always be a LONG way away.
        11. All done!

    Rogue also has "gone rooms", which just put a corridor space instead of the room, and draws L-shaped corridors instead of straight lines, but those are just flavor.

    This algorithm also has the virtues of being extremely fast (even on MUCH bigger grid arrays than 3x3), and guaranteed to succeed.
    */
    fn generate(&mut self) {
        let (w,h) = self.tiles.size();
        let mut rng = rand::rng();

        let mut start = (
            rng.random_range(1..w-10),
            rng.random_range(1..h-10)
        );

        let mut end = (
            rng.random_range(start.0..w-1),
            rng.random_range(start.1..h-1)
        );

        for x in start.0..end.0 {
            for y in start.1..end.1 {
                self.tiles[(x,y)] = 
                if x == start.0 && y == start.1 {
                    Tile::WallNW
                } else if x == start.0 && y == end.1-1 {
                    Tile::WallSW
                } else if x == end.0-1 && y == end.1-1 {
                    Tile::WallSE
                } else if x == end.0-1 && y == start.1 {
                    Tile::WallNE
                } else if x == start.0 || x == end.0-1 {
                    Tile::WallV
                } else if y == start.1 || y == end.1-1 {
                    Tile::WallH
                } else {
                    Tile::Ground
                };
            }
        }

        self.pos = (((start.0 + end.0)/2) as i32,((start.1 + end.1)/2) as i32);
    }

    fn draw_map(&mut self, t: &mut Stdout) -> io::Result<()> {
        for pos in self.tiles.positions() {
            let tile = self.tiles[pos];
            if tile == Tile::Void {
                continue;
            }

            self.screen.set(t, pos, tile.get_char())?;
        }

        Ok(())
    }
}

impl TerminalHandler for DungeonGame {
    fn on_draw(&mut self, term: &mut Terminal) -> io::Result<()> {
        let t = &mut term.stdout;
        
        self.draw_map(t)?;
        //self.screen.clear(t, Color::Black);
        {
            let (x,y) = self.pos;
            self.screen.set(t, (x as usize, y as usize), StyledChar::new('▮', Color::Black, Color::Yellow))?;
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

        self.screen.draw(t)?;

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