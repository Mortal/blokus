use std::io::Write;

use rand::Rng;

pub trait GameStateMut {
    fn undo(&mut self);
    fn move_count(&mut self) -> usize;
    fn select_move(&mut self, i: usize);
    fn value(&mut self) -> f64;
}

struct TreeSearch<'a, G: GameStateMut> {
    game_state: &'a mut G,
    // children[i] is None if state is unexplored, or a vec of child indices otherwise.
    children: Vec<Option<Vec<usize>>>,
    // parent[0] is 0 (root special case)
    parent: Vec<usize>,
    value_sums: Vec<f64>,
    value_counts: Vec<usize>,
}

impl <'a, G: GameStateMut> TreeSearch<'a, G> {
    fn select_leaf<R: Rng>(&mut self, rng: &mut R, temperature: f64) -> Option<usize> {
        let mut leaves = Vec::new();
        let mut max = 0.0f64;
        for i in 0..self.children.len() {
            if self.children[i].is_some() {
                continue;
            }
            let value = if self.value_counts[i] > 0 {
                self.value_sums[i] / self.value_counts[i] as f64
            } else {
                0.0
            };
            max = max.max(value);
            leaves.push((i, value));
        }
        if leaves.is_empty() {
            return None;
        }
        let mut sum = 0.0;
        for (_, ref mut value) in leaves.iter_mut() {
            *value = ((*value - max) * temperature).exp();
            sum += *value;
        }
        assert!(sum > 1e-5);
        let v = rng.gen_range(0.0, sum);
        let mut acc = 0.0;
        for &(i, value) in leaves.iter() {
            acc += value;
            if v < acc {
                print!("\r\x1B[KT={:.4} Selected {:8} with prob {:.4}", temperature, i, value / sum);
                std::io::stdout().flush().ok().expect("Could not flush stdout");
                return Some(i);
            }
        }
        panic!();
    }

    fn from_root_to_node(&mut self, mut i: usize) {
        let mut stack = Vec::new();
        while i > 0 {
            stack.push(i);
            i = self.parent[i];
        }
        while let Some(i) = stack.pop() {
            let p: usize = self.parent[i];
            let mut j: usize = 0;
            let c = self.children[p].as_ref().unwrap();
            while j < c.len() && c[j] != i {
                j += 1;
            }
            assert!(j < c.len());
            self.game_state.select_move(j);
        }
    }

    fn from_node_to_root(&mut self, mut i: usize) {
        while i > 0 {
            self.game_state.undo();
            i = self.parent[i];
        }
    }

    fn expand<R: Rng>(&mut self, i: usize, rng: &mut R) -> usize {
        // Assume state is leaf i, goes to and returns random new state
        assert!(self.children[i].is_none());
        let count = self.game_state.move_count();
        let mut c = Vec::new();
        for _ in 0..count {
            c.push(self.children.len());
            self.parent.push(i);
            self.children.push(None);
            self.value_sums.push(0.0);
            self.value_counts.push(0);
        }
        if count > 0 {
            let j = rng.gen_range(0, count);
            let res = c[j];
            self.children[i] = Some(c);
            self.game_state.select_move(j);
            res
        } else {
            self.children[i] = Some(c);
            i
        }
    }

    fn simulate<R: Rng>(&mut self, rng: &mut R) -> f64 {
        let mut depth = 0;
        loop {
            let count = self.game_state.move_count();
            if count == 0 {
                break;
            }
            depth += 1;
            let j = rng.gen_range(0, count);
            self.game_state.select_move(j);
        }
        let res = self.game_state.value();
        for _ in 0..depth {
            self.game_state.undo();
        }
        res
    }

    fn backpropagation(&mut self, mut i: usize, val: f64) {
        loop {
            self.value_sums[i] += val;
            self.value_counts[i] += 1;
            if i == 0 {
                break;
            }
            i = self.parent[i];
        }
    }
}

pub fn run_treesearch<G: GameStateMut, R: Rng>(game_state: &mut G, rng: &mut R, speed: f64) {
    let mut t = TreeSearch {
        game_state: game_state,
        children: vec![None],
        parent: vec![0],
        value_sums: vec![0.0f64],
        value_counts: vec![0],
    };

    let mut temperature = speed;
    loop {
        let i = match t.select_leaf(rng, temperature) {
            Some(i) => i,
            None => return,
        };
        t.from_root_to_node(i);
        let j = t.expand(i, rng);
        let val = t.simulate(rng);
        t.backpropagation(j, val);
        t.from_node_to_root(j);
        temperature += speed;
    }
}
