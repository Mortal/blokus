extern crate rand;

mod neighbors;
mod pieces;
use pieces::{compute_pieces};
mod board;
use board::Board;
mod treesearch;
use treesearch::{GameStateMut, run_treesearch};

struct BoardStateMut {
    board: Board,
    count: usize,
    homes: Vec<(i8, i8)>,
    best: f64,
}

impl GameStateMut for BoardStateMut {
    fn undo(&mut self) {
        self.board.undo().unwrap();
        self.count -= 1;
    }

    fn move_count(&mut self) -> usize {
        let c = (self.count % self.homes.len()) as u8;
        self.board.moves(c).unwrap().moves.len()
    }

    fn select_move(&mut self, i: usize) {
        let c = (self.count % self.homes.len()) as u8;
        let moves = self.board.moves(c).unwrap();
        if i >= moves.moves.len() {
            panic!("Only {} moves but tried to select number {}", moves.moves.len(), i);
        }
        moves.place(i);
        self.count += 1;
    }

    fn value(&mut self) -> f64 {
        let mut flags = vec![0; self.board.size()];
        let occupied = 1u8;
        let visited = 2u8;
        let mut stack = Vec::new();
        let mut eightway_stack = Vec::new();
        let mut cardinal_stack = Vec::new();
        let mut occupied_count = 0;
        for i in 0..flags.len() {
            if self.board.at(i) == None {
                stack.push(i);
            } else {
                occupied_count += 1;
                flags[i] |= occupied;
            }
        }
        let mut eightway_component_count = 0;
        let mut cardinal_component_count = 0;
        while let Some(i) = stack.pop() {
            if flags[i] != 0 {
                continue;
            }
            eightway_component_count += 1;
            eightway_stack.push(i);
            while let Some(i) = eightway_stack.pop() {
                if flags[i] != 0 {
                    continue;
                }
                cardinal_component_count += 1;
                cardinal_stack.push(i);
                while let Some(i) = cardinal_stack.pop() {
                    if flags[i] != 0 {
                        continue;
                    }
                    flags[i] |= visited;
                    for j in self.board.cardinal_neighbors(i) {
                        if flags[j] == 0 {
                            cardinal_stack.push(j);
                        }
                    }
                    for j in self.board.diagonal_neighbors(i) {
                        if flags[j] == 0 {
                            eightway_stack.push(j);
                        }
                    }
                }
            }
        }
        let value = (occupied_count as f64) - 3.0 * (eightway_component_count as f64) - (cardinal_component_count as f64);
        if value >= self.best - 2.0 {
            self.best = self.best.max(value);
            let s = format!("occupied = {}, eightway = {}, cardinal = {}, value = {}\n{}\n", occupied_count, eightway_component_count, cardinal_component_count, value, self.board);
            println!("\r\x1B[K{}", s);
        }
        value
    }
}

fn main() {
    let pieces = compute_pieces(5);
    let width = 20;
    let height = 20;
    let needed_tiles = 4 * pieces.iter().map(|p| p.points.len()).sum::<usize>();
    if width * height < needed_tiles {
        panic!("Need {} tiles but have only {}*{}", needed_tiles, width, height);
    }
    let homes = vec![(0, 0), (width as i8 - 1, 0), (width as i8 - 1, height as i8 - 1), (0, height as i8 - 1)];
    let mut rng = rand::thread_rng();
    let b = Board::new(pieces.clone(), height, width, &homes);
    //println!("{:?}", b);
    let mut s = BoardStateMut {board: b, count: 0, homes: homes, best: 0.0};
    run_treesearch(&mut s, &mut rng, 0.00001);
}
