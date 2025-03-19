use crossterm::style::Color;

use crate::{estruturas::Iterator2D, utils::Terminal};

pub fn life() {
    let mut t = Terminal::new();

    let size = t.get_size();
    t.set_background_color(Color::Black);
    t.set_foreground_color(Color::Green);
    t.main_loop(100, |t| {
        t.clear();

        for (x,y) in Iterator2D::xy((size.0 as usize, size.1 as usize)) {
            t.cursor_position((x as u16,y as u16));
            print!("@");
        }
    });  

    println!("");
}