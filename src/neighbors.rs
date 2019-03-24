pub struct CardinalIterator {
    rows: usize,
    cols: usize,
    x: usize,
    y: usize,
    i: usize,
}

impl CardinalIterator {
    pub fn new(rows: usize, cols: usize, x: usize, y: usize) -> Self {
        CardinalIterator {
            rows: rows,
            cols: cols,
            x: x,
            y: y,
            i: 0,
        }
    }
}

impl Iterator for CardinalIterator {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        if self.i == 0 {
            self.i += 1;
            if self.y > 0 {
                return Some((self.y - 1) * self.cols + self.x);
            }
        }
        if self.i == 1 {
            self.i += 1;
            if self.x > 0 {
                return Some(self.y * self.cols + (self.x - 1));
            }
        }
        if self.i == 2 {
            self.i += 1;
            if self.x + 1 < self.cols {
                return Some(self.y * self.cols + (self.x + 1));
            }
        }
        if self.i == 3 {
            self.i += 1;
            if self.y + 1 < self.rows {
                return Some((self.y + 1) * self.cols + self.x);
            }
        }
        None
    }
}

pub struct DiagonalIterator {
    rows: usize,
    cols: usize,
    x: usize,
    y: usize,
    i: usize,
}

impl DiagonalIterator {
    pub fn new(rows: usize, cols: usize, x: usize, y: usize) -> Self {
        DiagonalIterator {
            rows: rows,
            cols: cols,
            x: x,
            y: y,
            i: 0,
        }
    }
}

impl Iterator for DiagonalIterator {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        if self.i == 0 {
            self.i += 1;
            if self.y > 0 && self.x > 0 {
                return Some((self.y - 1) * self.cols + (self.x - 1));
            }
        }
        if self.i == 1 {
            self.i += 1;
            if self.y + 1 < self.rows && self.x > 0 {
                return Some((self.y + 1) * self.cols + (self.x - 1));
            }
        }
        if self.i == 2 {
            self.i += 1;
            if self.y > 0 && self.x + 1 < self.cols {
                return Some((self.y - 1) * self.cols + (self.x + 1));
            }
        }
        if self.i == 3 {
            self.i += 1;
            if self.y + 1 < self.rows && self.x + 1 < self.cols {
                return Some((self.y + 1) * self.cols + (self.x + 1));
            }
        }
        None
    }
}
