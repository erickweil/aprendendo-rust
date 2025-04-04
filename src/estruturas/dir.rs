#[derive(PartialEq, Eq)]
#[derive(Clone, Copy)]
pub enum Dir {
    Right,
    Up,
    Left,
    Down
}

impl Dir {
    pub fn counter_clockwise(&self) -> Dir {
        match &self {
            Dir::Right => Dir::Up,
            Dir::Up => Dir::Left,
            Dir::Left => Dir::Down,
            Dir::Down => Dir::Right,
        }
    }

    pub fn clockwise(&self) -> Dir {
        match &self {
            Dir::Right => Dir::Down,
            Dir::Down => Dir::Left,
            Dir::Left => Dir::Up,
            Dir::Up => Dir::Right,
        }
    }

    pub fn to_xy(&self) -> (i32,i32) {
        match &self {
            Dir::Right => (1,0),
            Dir::Up => (0,1),
            Dir::Left => (-1,0),
            Dir::Down => (0,-1)
        }
    }
}