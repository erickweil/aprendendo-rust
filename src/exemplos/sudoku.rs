use std::ops::{Index, IndexMut};

use crate::estruturas::Vec2D;

pub struct QuadroSudoku {
    table: [usize; 81]
}

impl QuadroSudoku {
    pub fn new() -> Self {
        Self {
            table: [0; 81]
        }
    }

    pub fn from_slice(table: &[usize]) -> Self {
        let mut quadro = Self::new();
        quadro.table.copy_from_slice(table);
        quadro
    }

    pub fn print(&self) {
        for (i, &v) in self.table.iter().enumerate() {
            if v == 0 {
                print!(". ");
            } else {
                print!("{} ", v);
            }
            if (i+1) % 3 == 0 {
                print!("\t");
            }
            if (i+1) % 9 == 0 {
                println!();
            }
            if (i+1) % 27 == 0 {
                println!();
            }
        }
    }
}

pub struct Possib<const N: usize> {
    index: usize, // índice da possibilidade no quadro
    p: [bool; N] // possibilidades, true se for possível
}

impl<const N: usize> Possib<N> {
    pub fn new(index: usize) -> Self {
        Self {
            index: index,
            p: [true; N]
        }
    }

    pub fn contar(&self) -> i32 {
        let mut cont = 0;
        for i in 0..N {
            if self.p[i] { cont += 1; }
        }
        cont
    }

    pub fn resetar(&mut self, valor: bool) {
        for i in 0..N {
            self.p[i] = valor;
        }
    }
}

trait PencilMark<const N: usize> {
    fn solucionar(depth: usize, quadro: &mut [usize]) -> bool {
        let possib = Self::obter_melhor(quadro);
        if let None = possib {
            // Se não tem mais possibilidades, não tem como continuar
            return false;
        }
        // Atravessa cada possibilidade
        let mut possib = possib.unwrap();
        // DEPOIS: atravessar aleatoriamente as possibilidades
        for k in 0..N {
            if !possib.p[k] { continue; } // Se não é possível, ignora

            // Marca a possibilidade
            quadro[possib.index] = k + 1;

            if Self::checar_solucionado(quadro) {
                return true;
            }

            if Self::solucionar(depth+1, quadro) {
                return true;
            }

            // Desmarca a possibilidade, para tentar a próxima
            quadro[possib.index] = 0;
        }

	    // Nenhuma escolha foi válida para este quadrado, ou seja, é ímpossível solucionar nesta configuração
        return false;
    }

    // Se não tem nenhum quadrado sem escolher, está solucionado
    fn checar_solucionado(quadro: &[usize]) -> bool {
        for i in 0..quadro.len() {
            if quadro[i] == 0 { return false; }
        }
        true
    }

    // Ao mesmo tempo que analisa as possibilidades, encontra a com menor entropia
    fn obter_melhor(quadro: &[usize]) -> Option<Possib<N>>
    {
        let mut min_p: Option<Possib<N>> = None;
        let mut min_cont: i32 = -1;
        for index in 0..quadro.len() {
            if quadro[index] != 0 { continue; } // Se já foi escolhido ignora

            let mut p = Possib::new(index);
            Self::regras_quadro(quadro, &mut p);

            let cont = p.contar();
            if min_cont == -1 || cont < min_cont {
                min_cont = cont;
                min_p = Some(p);
            }
        }

        min_p
    }

    fn regras_quadro(quadro: &[usize], p: &mut Possib<N>);
}

impl PencilMark<9> for QuadroSudoku {
    fn regras_quadro(quadro: &[usize], possibs: &mut Possib<9>) {
        let index = possibs.index;
        // Se já foi escolhido no quadro, só tem aquela opção disponível
        if quadro[index] != 0 {
            possibs.resetar(false);
            possibs.p[quadro[index]-1] = true;
            return
        }

        let py = index / 9;
        let px = index % 9;

        // Verifica colunas
        for y in 0..9 {
            if y == py { continue; }
            let quadro_v = quadro[y * 9 + px];
            if quadro_v == 0 { continue; }
            
            possibs.p[quadro_v - 1] = false;
        }

        // Verifica linhas
        for x in 0..9 {
            if x == px { continue; }
            let quadro_v = quadro[py * 9 + x];
            if quadro_v == 0 { continue; }
            
            possibs.p[quadro_v - 1] = false;
        }

        // verificar quadrado
        let quadx = (px / 3) * 3;
        let quady = (py / 3) * 3;
        for x in quadx..(quadx+3) {
            for y in quady..(quady+3) {
                if x == px || y == py { continue; }
                let quadro_v = quadro[y*9 + x];
                if quadro_v == 0 { continue; }

                possibs.p[quadro_v-1] = false;
            }
        }
    }
}


pub fn sudoku() {

    let mut table = QuadroSudoku::from_slice(&[
        0, 8, 0,   0, 5, 0,   0, 2, 0,
        3, 0, 5,   0, 0, 0,   1, 0, 6,
        4, 0, 0,   6, 3, 0,   0, 0, 7,

        5, 6, 1,   0, 0, 0,   9, 0, 4,
        0, 0, 0,   0, 0, 0,   0, 0, 0,
        9, 0, 8,   0, 0, 0,   5, 3, 1,

        8, 0, 0,   0, 1, 7,   0, 0, 5,
        2, 0, 7,   0, 0, 0,   3, 0, 8,
        0, 1, 0,   0, 8, 0,   0, 4, 0,
    ]);
    
    table.print();

    QuadroSudoku::solucionar(0, &mut table.table);

    println!("---");
    table.print();
}