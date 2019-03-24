extern crate std;

pub type Point = (i8, i8);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Piece {
    pub points: Vec<Point>,
}

impl Piece {
    fn min_corner(&self) -> Point {
        if self.points.is_empty() {
            (0, 0)
        } else {
            self.points.iter().fold(self.points[0], |(a, b), &(c, d)| (a.min(c), b.min(d)))
        }
    }

    pub fn max_corner(&self) -> Point {
        if self.points.is_empty() {
            (0, 0)
        } else {
            self.points.iter().fold(self.points[0], |(a, b), &(c, d)| (a.max(c), b.max(d)))
        }
    }

    fn map<F>(mut self, mut f: F) -> Self where F: FnMut(Point) -> Point {
        for p in self.points.iter_mut() {
            *p = f(*p);
        }
        self
    }

    fn sorted(mut self) -> Self {
        self.points.sort_unstable();
        self
    }

    fn translate(self, (dx, dy): Point) -> Self {
        self.map(|(x, y)| (x + dx, y + dy))
    }

    fn translate_origin(self) -> Self {
        let (x, y) = self.min_corner();
        self.translate((-x, -y))
    }

    fn mirror_x(self) -> Self {
        self.map(|(x, y)| (-x, y))
    }

    fn rot_ccw(self) -> Self {
        self.map(|(x, y)| (-y, x))
    }

    fn vary<F>(&self, mut f: F) where F: FnMut(Piece) -> () {
        f(self.clone().translate_origin());
        f(self.clone().rot_ccw().translate_origin());
        f(self.clone().rot_ccw().rot_ccw().translate_origin());
        f(self.clone().rot_ccw().rot_ccw().rot_ccw().translate_origin());
        f(self.clone().mirror_x().translate_origin());
        f(self.clone().mirror_x().rot_ccw().translate_origin());
        f(self.clone().mirror_x().rot_ccw().rot_ccw().translate_origin());
        f(self.clone().mirror_x().rot_ccw().rot_ccw().rot_ccw().translate_origin());
    }

    pub fn variations(&self) -> Vec<Piece> {
        let mut res = Vec::new();
        self.vary(|p| res.push(p.sorted()));
        res.sort();
        res.dedup();
        res
    }

    fn canonical(&self) -> Piece {
        let mut res = vec![self.clone().translate_origin()];
        self.vary(|p| {let r = res[0].clone().min(p.sorted()); res[0] = r;});
        res[0].clone()
    }

    fn expand<F>(&self, mut f: F) where F: FnMut(Piece) -> () {
        let mut try = |p| {
            if !self.points.contains(&p) {
                let mut ps = self.points.clone();
                ps.push(p);
                f(Piece {points: ps});
            }
        };
        for &(x, y) in self.points.iter() {
            try((x - 1, y));
            try((x + 1, y));
            try((x, y - 1));
            try((x, y + 1));
        }
    }
}

pub fn compute_pieces(max_piece_size: usize) -> Vec<Piece> {
    assert!(max_piece_size >= 1);
    let mut result = Vec::new();
    let mut pieces = vec![Piece {points: vec![(0, 0)]}];
    for _ in 1..max_piece_size {
        let mut pieces2 = Vec::new();
        std::mem::swap(&mut pieces, &mut pieces2);
        for piece in pieces2.iter() {
            piece.expand(|piece| {
                pieces.push(piece.canonical().sorted());
            });
        }
        result.append(&mut pieces2);
        pieces.sort_unstable();
        pieces.dedup();
    }
    result.append(&mut pieces);
    result
}

#[allow(unused)]
pub fn print_pieces(pieces: &[Piece]) {
    for piece in pieces {
        let piece = piece.clone().rot_ccw().mirror_x().translate_origin();
        let (x, y) = piece.max_corner();
        for yy in 0..(y + 1) {
            for xx in 0..(x + 1) {
                if piece.points.contains(&(xx, yy)) {
                    print!("x");
                } else {
                    print!(" ");
                }
            }
            println!("");
        }
        println!("");
    }
}

#[test]
fn test_pieces() {
    assert_eq!(compute_pieces(3), vec![Piece { points: vec![(0, 0)] }, Piece { points: vec![(0, 0), (0, 1)] }, Piece { points: vec![(0, 0), (0, 1), (0, 2)] }, Piece { points: vec![(0, 0), (0, 1), (1, 0)] }]);
}

#[test]
fn test_min_corner() {
    assert_eq!(Piece {points: vec![(1, 2), (2, 1)]}.min_corner(), (1, 1));
}

#[test]
fn test_translate() {
    let p = Piece {points: vec![(1, 2), (2, 1)]};
    assert_eq!(p.translate_origin().min_corner(), (0, 0));
}

#[test]
fn test_mirror_x() {
    let p = Piece {points: vec![(1, 2), (2, 1)]};
    assert_eq!(p.mirror_x().min_corner(), (-2, 1));
}

#[test]
fn test_variations_o() {
    let o = Piece {points: vec![(0, 0), (0, 1), (1, 0), (1, 1)]};
    assert_eq!(o.variations().len(), 1);
}

#[test]
fn test_variations_i() {
    let i = Piece {points: vec![(0, 0), (0, 1), (0, 2), (0, 3)]};
    assert_eq!(i.variations().len(), 2);
}
