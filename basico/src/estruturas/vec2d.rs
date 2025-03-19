use std::{ops::{Index, IndexMut}, slice::Iter};

/**
 * Facilita utilizar um Vec<> como se fosse 2D, utilizando cálculo de índice a partir das posições x e y
 */
pub struct Vec2D<T> {
    width: usize,  
    height: usize, 
    data: Vec<T>
}

impl<T> Vec2D<T> where T: Clone {
    pub fn new(width: usize, height: usize, value: T) -> Vec2D<T> {
        Self {
            width,
            height,
            data: vec![value; width * height]
        }
    }

    pub fn values(&self) -> Iter<T> {
        return self.data.iter();
    }

    pub fn _positions(&self) -> Iterator2D {
        return Iterator2D { width: self.width, height: self.height, pos: (0,0) };
    }
}

// let value = vec2d[(x, y)];
impl<T> Index<(usize, usize)> for Vec2D<T> {
    type Output = T;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        if x >= self.width || y >= self.height { panic!("Índice fora dos limites: {:?}",(x,y)); }

        &self.data[y * self.width + x]
    }
}

// vec2d[(x, y)] = value;
impl<T> IndexMut<(usize, usize)> for Vec2D<T> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        if x >= self.width || y >= self.height { panic!("Índice fora dos limites: {:?}",(x,y)); }

        &mut self.data[y * self.width + x]
    }
}

pub struct Iterator2D {
    width: usize,  
    height: usize, 
    pos: (usize,usize)
}

impl Iterator2D {
    pub fn xy((width, height): (usize, usize)) -> Iterator2D {
        return Iterator2D { width: width, height: height, pos: (0,0) };
    }
}

impl Iterator for Iterator2D {
    type Item = (usize,usize);
    fn next(&mut self) -> Option<Self::Item> {
        let _pos = self.pos;
        if _pos.1 >= self.height {
            return None;
        }

        self.pos.0 += 1;
        if self.pos.0 >= self.width {
            self.pos.0 = 0;
            self.pos.1 += 1;
        }

        return Some(_pos);
    }
}