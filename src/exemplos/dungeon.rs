use std::{cmp::{max, min}, io::{self, Stdout}};

use crossterm::{cursor::*, event::KeyCode, style::*, terminal::*, ExecutableCommand, QueueableCommand};
use rand::{seq::SliceRandom, Rng};

use crate::{estruturas::{Dir, GraphIter, GraphIterState, GraphSearch, Stack, Vec2D}, utils::{StyledChar, Terminal, TerminalHandler, TerminalScreen}};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tile {
    Void,
    WallV,
    WallH,
    WallNE,
    WallNW,
    WallSW,
    WallSE,
    WallCross,
    WallUJointN,
    WallUJointW,
    WallUJointS,
    WallUJointE,
    Ground
}

impl Tile {
    fn get_char(&self, back: Color) -> StyledChar {
        match self {
            Tile::Void   => StyledChar::new(' ', back, Color::White),
            Tile::WallV  => StyledChar::new('║', back, Color::DarkYellow),
            Tile::WallH  => StyledChar::new('═', back, Color::DarkYellow),
            Tile::WallNE => StyledChar::new('╗', back, Color::DarkYellow),
            Tile::WallNW => StyledChar::new('╔', back, Color::DarkYellow),
            Tile::WallSW => StyledChar::new('╚', back, Color::DarkYellow),
            Tile::WallSE => StyledChar::new('╝', back, Color::DarkYellow),
            Tile::WallUJointN => StyledChar::new('╦', back, Color::DarkYellow),
            Tile::WallUJointW => StyledChar::new('╠', back, Color::DarkYellow),
            Tile::WallUJointS => StyledChar::new('╩', back, Color::DarkYellow),
            Tile::WallUJointE => StyledChar::new('╣', back, Color::DarkYellow),
            Tile::WallCross => StyledChar::new('╬', back, Color::DarkYellow),
            Tile::Ground => StyledChar::new(' ', back, Color::DarkYellow)
        }
    }

    fn get_wall(n: bool, w: bool, s: bool, e: bool) -> Tile {
        if n && w && s && e {
            Tile::WallCross
        } else if !n && w && s && e {
            Tile::WallUJointN
        } else if n && !w && s && e {
            Tile::WallUJointW
        } else if n && w && !s && e {
            Tile::WallUJointS
        } else if n && w && s && !e {
            Tile::WallUJointE
        } else if !n && w && s && !e {
            Tile::WallNE
        } else if !n && !w && s && e {
            Tile::WallNW
        } else if n && !w && !s && e {
            Tile::WallSW
        } else if n && w && !s && !e {
            Tile::WallSE
        } else if s || n {
            Tile::WallV
        } else if e || w {
            Tile::WallH
        } else {
            Tile::Ground
        }
    }
}

#[derive(Clone)]
struct MapTile {
    tile: Tile,
    explored: bool,
    visible: bool
}

impl MapTile {
    fn new(tile: Tile) -> MapTile {
        MapTile { tile: tile, explored: false, visible: false }
    }
    
    fn mark_visible(&mut self) {
        self.explored = true;
        self.visible = true;
    }
}

struct DungeonGame {
    screen: TerminalScreen,
    tiles: Vec2D<MapTile>,
    pos: (i32, i32)
}

impl DungeonGame {
    fn new (size: (usize, usize)) -> DungeonGame {       
        let mut g = DungeonGame {
            screen: TerminalScreen::new(size),
            tiles: Vec2D::new(size.0, size.1, MapTile::new(Tile::Void)),
            pos: (0,0)
        };

        g.generate_maze();
        g.open_rooms();
        g.fill_walls();

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
    fn open_rooms(&mut self) {
        let (w,h) = self.tiles.size();
        let mut rng = rand::rng();

        for room in 0..(w/10+2) {
            let a = (
                rng.random_range(1..w-1),
                rng.random_range(1..h-1)
            );

            let b = (
                rng.random_range(1..w-1),
                rng.random_range(1..h-1)
            );

            let mut start = (min(a.0, b.0), min(a.1, b.1));
            let mut end = (max(a.0, b.0), max(a.1, b.1));

            while end.0 - start.0 > 10 {
                start.0 += 1;
                end.0 -= 1;
            }

            while end.1 - start.1 > 10 {
                start.1 += 1;
                end.1 -= 1;
            }

            for x in start.0..end.0 {
                for y in start.1..end.1 {
                    self.tiles[(x,y)].tile = Tile::Ground;
                }
            }

            self.pos = (((start.0 + end.0)/2) as i32,((start.1 + end.1)/2) as i32);
        }
    }

    fn generate_maze(&mut self) {
        let mut iter = GraphSearch::<(i32, i32)>::depth_first((0,0));
        let (w,h) = self.screen.size();
        let (mw,mh) = (w as i32 / 2, h as i32 / 2);
        while let Some(path) = iter.next() {
            let mut path = path.clone();

            let (mx,my) = path.pop().unwrap();
            let (m1x,m1y) = path.pop().unwrap_or((0,0));

            self.tiles[((mx * 2) as usize, (my * 2) as usize)].tile = Tile::Ground;
            self.tiles[(((m1x * 2 + mx * 2) / 2) as usize, ((m1y * 2 + my * 2) / 2) as usize)].tile = Tile::Ground;

            // Add neighbors
            let mut neighs = [(mx+1,my),(mx,my-1),(mx-1,my),(mx,my+1)];
            let mut rng = rand::rng();
            neighs.shuffle(&mut rng);
            
            for neigh in neighs {
                if neigh.0 >= 0 && neigh.0 < mw && neigh.1 >= 0 && neigh.1 < mh {
                    iter.push_neighbor(neigh);
                }
            }
        }
    }

    fn fill_walls(&mut self) {
        for pos in self.tiles.positions() {
            let tile = self.tiles[pos].tile;
            if tile != Tile::Void {
                continue;
            }

            let (x,y) = (pos.0 as i32, pos.1 as i32);
            let gnd = MapTile::new(Tile::Ground);
            let n = self.tiles.get((x  , y-1)).unwrap_or(&gnd).tile != Tile::Ground;
            let w = self.tiles.get((x-1, y  )).unwrap_or(&gnd).tile != Tile::Ground;
            let s = self.tiles.get((x  , y+1)).unwrap_or(&gnd).tile != Tile::Ground;
            let e = self.tiles.get((x+1, y  )).unwrap_or(&gnd).tile != Tile::Ground;

            self.tiles[pos].tile = Tile::get_wall(n, w, s, e);
        }
    }

    fn draw_map(&mut self, t: &mut Stdout) -> io::Result<()> {
        for pos in self.tiles.positions() {
            let maptile = &mut self.tiles[pos];
            let tile = maptile.tile;
            if tile == Tile::Void {
                continue;
            }
            if !maptile.explored {
                self.screen.set(t, pos, StyledChar::new(' ', Color::Grey, Color::DarkYellow))?;
            } else {
                if tile == Tile::Ground {
                    if maptile.visible {
                        self.screen.set(t, pos, StyledChar::new(' ', Color::Black, Color::DarkYellow))?;
                    } else {
                        self.screen.set(t, pos, StyledChar::new(' ', Color::DarkGrey, Color::DarkYellow))?;
                    }
                } else {
                    self.screen.set(t, pos, tile.get_char(if maptile.visible { Color::Black } else { Color::DarkGrey }))?;
                }
            }

            maptile.visible = false;
        }

        Ok(())
    }

    fn is_visible(&self, (x0,y0): (i32, i32), (x1, y1): (i32, i32)) -> bool {
        // https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm
        let mut dx = (x1 - x0).abs();
        let mut dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;
        let mut x = x0;
        let mut y = y0;

        while x != x1 || y != y1 {
            if let Some(maptile) = self.tiles.get((x,y)) {
                if maptile.tile != Tile::Ground {
                    return false;
                }
            } else {
                return false;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }

        return true;
    }
}

impl TerminalHandler for DungeonGame {
    fn on_draw(&mut self, term: &mut Terminal) -> io::Result<()> {
        let t = &mut term.stdout;
        let (w,h) = self.screen.size();
        let (w,h) = (w as i32, h as i32);

        // Mark surroundings as visible and explored
        let (x,y) = self.pos;
        let mut iter = GraphSearch::<(i32, i32)>::breadth_first((x,y));
        while let Some(path) = iter.next() {
            let (mx,my) = path.peek().unwrap().clone();
            //let dist = (mx - x)*(mx - x) + (my - y)*(my - y);
            
            // Add neighbors
            let mut neighs = [(mx+1,my),(mx,my-1),(mx-1,my),(mx,my+1), (mx+1,my+1),(mx-1,my-1),(mx-1,my+1),(mx+1,my-1)];
            for neigh in neighs {
                if neigh.0 >= 0 && neigh.0 < w && neigh.1 >= 0 && neigh.1 < h {
                    let maptile = &mut self.tiles[(neigh.0 as usize, neigh.1 as usize)];
                    maptile.mark_visible();

                    if maptile.tile == Tile::Ground && self.is_visible( neigh, self.pos) {
                        iter.push_neighbor(neigh);
                    }                    
                }
            }
        }

        self.draw_map(t)?;
        //self.screen.clear(t, Color::Black);
        {
            
            self.screen.set(t, (x as usize, y as usize), StyledChar::new('*', Color::Black, Color::Green))?;
        }

        /*
        
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
        let (mut x,mut y) = self.pos;
        let (w,h) = self.screen.size();
        let (w,h) = (w as i32, h as i32);
        match e.code {
            KeyCode::Left =>  if x > 0 { x -= 1; },
            KeyCode::Right => if x < w-1 { x += 1; },
            KeyCode::Up =>    if y > 0 { y -= 1; },
            KeyCode::Down =>  if y < h-1 { y += 1; }
            _ => {

            }
        }

        if let Some(maptile) = self.tiles.get((x,y)) {
            if maptile.tile == Tile::Ground {
                self.pos = (x,y);
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