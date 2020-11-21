use itertools::Itertools;
use num::{One, ToPrimitive, Zero};
use pyo3::prelude::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::Rng;
use std::convert::TryInto;
use std::fmt;
use std::hash::Hash;
use std::ops::AddAssign;

// rust-cpython aware function. All of our python interface could be
// declared in a separate module.
// Note that the py_fn!() macro automatically converts the arguments from
// Python objects to Rust values; and the Rust return value back into a Python object.
#[pymodule]
fn r2048(_py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfn(m, "new")]
    fn new_248_py() -> PyResult<Vec<usize>> {
        let board: [usize; 16] = State::new();
        Ok(Box::new(board).to_vec())
    }

    #[pyfn(m, "step")]
    fn step_2048_py(state: Vec<usize>, action: usize) -> PyResult<(Vec<usize>, u64, bool)> {
        let boxed_slice = state.into_boxed_slice();
        let boxed_array: Box<[usize; 16]> = match boxed_slice.try_into() {
            Ok(ba) => ba,
            Err(o) => panic!("Expected a Vec of length {} but it was {}", 16, o.len()),
        };
        let mut input_state = *boxed_array;
        let old_state = input_state;
        let act = match action {
            0 => Some(Action::Down),
            1 => Some(Action::Left),
            2 => Some(Action::Right),
            3 => Some(Action::Up),
            _ => None,
        };
        let mut done = false;
        match act {
            Some(a) => {
                done = input_state.advance_state(&a);
            }
            None => {
                panic!("Action outside of [0,1,2,3]",);
            }
        };
        let reward = input_state.score() - old_state.score();
        Ok((Box::new(input_state).to_vec(), reward, done))
    }

    Ok(())
}

#[derive(Copy, Clone)]
enum Action {
    Up,
    Down,
    Left,
    Right,
}

trait State<T> {
    fn new() -> Self
    where
        Self: Sized;
    fn advance_state(&mut self, act: &Action) -> bool;
    fn to_string(&self) -> String;

    // util
    fn add_random_tile(&mut self);
    fn score(&self) -> u64;

    // slides
    fn slide_left(&mut self);
    fn slide_right(&mut self);
    fn slide_up(&mut self);
    fn slide_down(&mut self);

    // merges
    fn merge_left(&mut self);
    fn merge_right(&mut self);
    fn merge_up(&mut self);
    fn merge_down(&mut self);
}

impl<T> fmt::Display for dyn State<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string: String = self
            .to_string()
            .as_str()
            .chars()
            .interleave("   \n   \n   \n   ".chars())
            .collect();
        write!(f, "{}", string)
    }
}

impl<
        T: Default + One + Zero + ToPrimitive + Eq + Hash + AddAssign + Copy + ToString + Copy + Clone,
    > State<T> for [T; 16]
{
    fn new() -> Self {
        let mut state = Self::default();
        let mut rng = rand::thread_rng();
        let i = rng.gen_range(0, 16);
        let mut j = rng.gen_range(0, 16);
        while i == j {
            j = rng.gen_range(0, 16);
        }
        state[i] = T::one();
        state[j] = T::one();
        state
    }

    fn advance_state(&mut self, act: &Action) -> bool {
        let original_board = *self;
        match act {
            Action::Up => {
                self.slide_up();
                self.merge_up();
                self.slide_up();
            }
            Action::Down => {
                self.slide_down();
                self.merge_down();
                self.slide_down();
            }
            Action::Left => {
                self.slide_left();
                self.merge_left();
                self.slide_left();
            }
            Action::Right => {
                self.slide_right();
                self.merge_right();
                self.slide_right();
            }
        }
        if *self == original_board {
            return false;
        }
        self.add_random_tile();
        let game_over = self.iter().all(|x| !x.is_zero()) && {
            let mut horizontal_board = *self;
            horizontal_board.merge_left();
            let mut vertical_board = *self;
            vertical_board.merge_up();
            (horizontal_board == *self) && (vertical_board == *self)
        };
        game_over
    }

    fn to_string(&self) -> String {
        self.iter().map(|x| x.to_string()).collect()
    }

    fn score(&self) -> u64 {
        const SCORE_LOOKUP: [u64; 18] = [
            0, 0, 4, 12, 28, 60, 124, 252, 508, 1020, 2044, 4092, 8188, 16380, 32764, 65532,
            131068, 262140,
        ];
        self.iter()
            .map(|x| SCORE_LOOKUP[x.to_usize().unwrap()])
            .sum()
    }

    fn slide_left(&mut self) {
        for base in [0, 4, 8, 12].iter() {
            if self[base + 2].is_zero() {
                self.swap(base + 2, base + 3);
            }
            if self[base + 1].is_zero() {
                self.swap(base + 1, base + 2);
                self.swap(base + 2, base + 3);
            }
            if self[base + 0].is_zero() {
                self.swap(base + 0, base + 1);
                self.swap(base + 1, base + 2);
                self.swap(base + 2, base + 3);
            }
        }
    }

    fn merge_left(&mut self) {
        for base in [0, 4, 8, 12].iter() {
            if self[base + 0] == self[base + 1] && !self[base + 0].is_zero() {
                self[base + 0] += T::one();
                self[base + 1] = T::zero();
            }
            if self[base + 1] == self[base + 2] && !self[base + 1].is_zero() {
                self[base + 1] += T::one();
                self[base + 2] = T::zero();
            }
            if self[base + 2] == self[base + 3] && !self[base + 2].is_zero() {
                self[base + 2] += T::one();
                self[base + 3] = T::zero();
            }
        }
    }

    fn slide_right(&mut self) {
        for base in [3, 7, 11, 15].iter() {
            if self[base - 2].is_zero() {
                self.swap(base - 2, base - 3);
            }
            if self[base - 1].is_zero() {
                self.swap(base - 1, base - 2);
                self.swap(base - 2, base - 3);
            }
            if self[base - 0].is_zero() {
                self.swap(base - 0, base - 1);
                self.swap(base - 1, base - 2);
                self.swap(base - 2, base - 3);
            }
        }
    }
    fn merge_right(&mut self) {
        for base in [3, 7, 11, 15].iter() {
            if self[base - 0] == self[base - 1] && !self[base - 0].is_zero() {
                self[base - 0] += T::one();
                self[base - 1] = T::zero();
            }
            if self[base - 1] == self[base - 2] && !self[base - 1].is_zero() {
                self[base - 1] += T::one();
                self[base - 2] = T::zero();
            }
            if self[base - 2] == self[base - 3] && !self[base - 2].is_zero() {
                self[base - 2] += T::one();
                self[base - 3] = T::zero();
            }
        }
    }

    fn slide_up(&mut self) {
        for base in [0, 1, 2, 3].iter() {
            if self[base + 8].is_zero() {
                self.swap(base + 8, base + 12);
            }
            if self[base + 4].is_zero() {
                self.swap(base + 4, base + 8);
                self.swap(base + 8, base + 12);
            }
            if self[base + 0].is_zero() {
                self.swap(base + 0, base + 4);
                self.swap(base + 4, base + 8);
                self.swap(base + 8, base + 12);
            }
        }
    }
    fn merge_up(&mut self) {
        for base in [0, 1, 2, 3].iter() {
            if self[base + 0] == self[base + 4] && !self[base + 0].is_zero() {
                self[base + 0] += T::one();
                self[base + 4] = T::zero();
            }
            if self[base + 4] == self[base + 8] && !self[base + 4].is_zero() {
                self[base + 4] += T::one();
                self[base + 8] = T::zero();
            }
            if self[base + 8] == self[base + 12] && !self[base + 8].is_zero() {
                self[base + 8] += T::one();
                self[base + 12] = T::zero();
            }
        }
    }

    fn slide_down(&mut self) {
        for base in [12, 13, 14, 15].iter() {
            if self[base - 8].is_zero() {
                self.swap(base - 8, base - 12);
            }
            if self[base - 4].is_zero() {
                self.swap(base - 4, base - 8);
                self.swap(base - 8, base - 12);
            }
            if self[base - 0].is_zero() {
                self.swap(base - 0, base - 4);
                self.swap(base - 4, base - 8);
                self.swap(base - 8, base - 12);
            }
        }
    }
    fn merge_down(&mut self) {
        for base in [12, 13, 14, 15].iter() {
            if self[base - 0] == self[base - 4] && !self[base - 0].is_zero() {
                self[base - 0] += T::one();
                self[base - 4] = T::zero();
            }
            if self[base - 4] == self[base - 8] && !self[base - 4].is_zero() {
                self[base - 4] += T::one();
                self[base - 8] = T::zero();
            }
            if self[base - 8] == self[base - 12] && !self[base - 8].is_zero() {
                self[base - 8] += T::one();
                self[base - 12] = T::zero();
            }
        }
    }
    fn add_random_tile(&mut self) {
        let mut rng = rand::thread_rng();
        let mut i = rng.gen_range(0, 16);
        while !self[i].is_zero() {
            i = rng.gen_range(0, 16);
        }
        self[i] = if rng.gen_bool(0.5) {
            T::one()
        } else {
            T::one() + T::one()
        }
    }
}

fn main() {
    let mut rng = thread_rng();
    let move_choices = [Action::Down, Action::Left, Action::Right, Action::Up];
    let moves: Vec<Action> = (0..100000000)
        .map(|_x| *move_choices.choose(&mut rng).unwrap())
        .collect();
    let mut steps = 0;
    let now = std::time::Instant::now();

    let mut total_games = 0;
    for _ in 0..1000000 {
        let mut test: [u8; 16] = State::new();
        total_games += 1;
        let mut is_game_over = false;
        while !is_game_over {
            is_game_over = test.advance_state(&moves[steps]);
            steps += 1;
        }
    }
    println!(
        "{} boards over {} moves averaging {} moves per board",
        total_games,
        steps,
        steps / total_games
    );
    println!("{:?}", now.elapsed());
}
