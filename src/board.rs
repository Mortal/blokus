use pieces::{Piece, Point};
use neighbors::{DiagonalIterator, CardinalIterator};

pub type Color = u8;

struct Translation<'a> {
    indices: &'a Vec<usize>,
    offset: usize,
    i: usize,
}

impl<'a> Iterator for Translation<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.indices.len() {
            return None;
        }
        let res = Some(self.indices[self.i] + self.offset);
        self.i += 1;
        res
    }
}

struct TranslationsIterator<'a> {
    indices: &'a Vec<usize>,
    x_range: usize,
    y_range: usize,
    board_width: usize,
    x: usize,
    y: usize,
}

impl<'a> Iterator for TranslationsIterator<'a> {
    type Item = (usize, Translation<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.y == self.y_range {
            return None;
        }
        let offset = self.x + self.y * self.board_width;
        let res = Some((offset, Translation { indices: self.indices, offset: offset, i: 0 }));
        if self.x + 1 == self.x_range {
            self.x = 0;
            self.y += 1;
        } else {
            self.x += 1;
        }
        res
    }
}

#[derive(Debug)]
struct BoardPieceVariation {
    points: Vec<usize>,
    width: usize,
    height: usize,
}

impl BoardPieceVariation {
    fn new(points: &[(i8, i8)], board_width: usize) -> Self {
        let mut width = 0;
        let mut height = 0;
        let mut indices = Vec::new();
        for &(x, y) in points {
            width = width.max(x as usize + 1);
            height = height.max(y as usize + 1);
            indices.push(x as usize + y as usize * board_width);
        }
        BoardPieceVariation {
            points: indices,
            width: width,
            height: height,
        }
    }

    fn translations(&self, board_width: usize, board_height: usize) -> TranslationsIterator {
        TranslationsIterator {
            indices: &self.points,
            x_range: board_width - self.width + 1,
            y_range: board_height - self.height + 1,
            board_width: board_width,
            x: 0,
            y: 0,
        }
    }

    fn translation(&self, d: usize) -> Translation {
        Translation {
            indices: &self.points,
            offset: d,
            i: 0,
        }
    }
}

#[test]
fn test_translations() {
    let points = &[(0, 0), (0, 1), (0, 2), (1, 0)];
    let board_width = 4;
    let board_height = 5;
    let v = BoardPieceVariation::new(points, board_width);
    assert_eq!(v.width, 2);
    assert_eq!(v.height, 3);
    let translations = v.translations(board_width, board_height).map(|(o, p)| o).collect::<Vec<_>>();
    assert_eq!(translations, vec![0, 1, 2, 4, 5, 6, 8, 9, 10]);
}

type BoardPiece = Vec<BoardPieceVariation>;

#[derive(Debug)]
pub struct Board {
    pieces: Vec<BoardPiece>,
    rows: usize,
    cols: usize,
    /// `positions[c][p]` is `Some((i, offset))`
    /// if color `c` is at `pieces[p][i].translation(offset)`
    positions: Vec<Vec<Option<(usize, usize)>>>,
    board: Vec<Option<Color>>,
    flags: Vec<u8>,
    history: Vec<(Color, usize, usize, usize)>,
    homes: Vec<usize>,
}

const CORNER: u8 = 1;
const BLOCKED: u8 = 2;
const OCCUPIED: u8 = 2 + 8 + 32 + 128;

pub struct Moves<'a> {
    board: &'a mut Board,
    color: Color,
    pub moves: Vec<(usize, usize, usize)>,
}

impl Board {
    pub fn new(pieces: Vec<Piece>, rows: usize, cols: usize, home_points: &[Point]) -> Self {
        let homes = home_points.iter().map(|&(x, y)| x as usize + y as usize * cols).collect::<Vec<_>>();
        assert!(homes.iter().all(|&i| i < rows * cols));
        assert!(homes.len() <= 4);
        let positions = vec![vec![None; pieces.len()]; homes.len()];
        let flags = vec![0; rows * cols];
        let mut res = Self {
            pieces: pieces.into_iter().map(|p| p.variations().into_iter().map(|v| BoardPieceVariation::new(&v.points, cols)).collect()).collect(),
            rows: rows,
            cols: cols,
            positions: positions,
            board: vec![None; rows * cols],
            flags: flags,
            history: Vec::new(),
            homes: homes,
        };
        res.update_home_flags();
        res
    }

    fn update_home_flags(&mut self) {
        for (c, i) in self.homes.iter().enumerate() {
            if (self.flags[*i] & (BLOCKED << (2 * c))) == 0 {
                self.flags[*i] |= CORNER << (2 * c);
            }
        }
    }

    pub fn size(&self) -> usize {
        self.board.len()
    }

    pub fn at(&self, i: usize) -> Option<Color> {
        self.board[i]
    }

    pub fn diagonal_neighbors(&self, position: usize) -> DiagonalIterator {
        DiagonalIterator::new(
            self.rows,
            self.cols,
            position % self.cols,
            position / self.cols,
        )
    }

    pub fn cardinal_neighbors(&self, position: usize) -> CardinalIterator {
        CardinalIterator::new(
            self.rows,
            self.cols,
            position % self.cols,
            position / self.cols,
        )
    }

    fn compute_flag(&self, i: usize) -> u8 {
        if self.board[i] != None {
            return OCCUPIED;
        }
        let mut flag = 0;
        for j in self.cardinal_neighbors(i) {
            match self.board[j] {
                Some(c) => flag |= BLOCKED << (2 * c),
                None => (),
            };
        }
        for j in self.diagonal_neighbors(i) {
            match self.board[j] {
                Some(c) => flag |= CORNER << (2 * c),
                None => (),
            };
        }
        flag
    }

    pub fn moves(&mut self, color: Color) -> Result<Moves, &'static str> {
        if color as usize >= self.positions.len() { return Err("color out of bounds"); }
        let mut moves = Vec::new();
        for (piece, variations) in self.pieces.iter().enumerate() {
            if !self.positions[color as usize][piece].is_none() {
                continue;
            }
            for (variation, p) in variations.iter().enumerate() {
                for (d, translation) in p.translations(self.cols, self.rows) {
                    let mut flag_union = 0;
                    for i in translation {
                        flag_union |= self.flags[i];
                    }
                    let test_flags = (CORNER | BLOCKED) << (2*color);
                    let req_flags = CORNER << (2*color);
                    if (flag_union & test_flags) == req_flags {
                        moves.push((piece, variation, d));
                    }
                }
            }
        }
        Ok(Moves { board: self, color: color, moves: moves })
    }

    fn write_piece(&mut self, piece: usize, variation: usize, offset: usize, prev: Option<Color>, next: Option<Color>) {
        let mut first = None;
        let mut last = 0;
        for i in self.pieces[piece][variation].translation(offset) {
            if first == None {
                first = Some(i);
            }
            last = i;
            assert_eq!(self.board[i], prev);
            self.board[i] = next;
        }

        // Update flags
        let first = first.unwrap();
        let margin = self.cols + 1;
        let a = if first < margin { 0 } else { first - margin };
        let b = (last + margin + 1).min(self.flags.len());
        for i in 0..(self.rows * self.cols) {
            self.flags[i] = self.compute_flag(i);
        }
        self.update_home_flags();
    }

    pub fn undo(&mut self) -> Result<(), &'static str> {
        let (color, piece, variation, offset) = match self.history.pop() {
            Some(x) => x,
            None => return Err("no moves to undo"),
        };
        assert_eq!(self.positions[color as usize][piece], Some((variation, offset)));
        self.positions[color as usize][piece] = None;
        self.write_piece(piece, variation, offset, Some(color), None);
        Ok(())
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for y in 0..self.rows {
            for x in 0..self.cols {
                match self.board[x + y * self.cols] {
                    None => try!(write!(f, "\u{2591}\u{2591}")),
                    Some(c) => try!(write!(f, "\x1B[{}m\u{2588}\u{2588}\x1B[0m", 31 + c)),
                }
            }
            try!(write!(f, "\n"));
        }
        Ok(())
    }
}

impl <'a> Moves<'a> {
    pub fn place(self, move_index: usize) {
        let (piece, variation, offset) = self.moves[move_index];
        self.board.write_piece(piece, variation, offset, None, Some(self.color));
        self.board.positions[self.color as usize][piece] = Some((variation, offset));
        self.board.history.push((self.color, piece, variation, offset));
    }
}
