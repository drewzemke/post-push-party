use std::collections::VecDeque;

use rand::RngExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    pub fn turn(&self, dir: Self) -> Self {
        match (self, dir) {
            (Dir::Up, Dir::Left) => Dir::Left,
            (Dir::Up, Dir::Right) => Dir::Right,
            (Dir::Down, Dir::Left) => Dir::Left,
            (Dir::Down, Dir::Right) => Dir::Right,
            (Dir::Left, Dir::Up) => Dir::Up,
            (Dir::Left, Dir::Down) => Dir::Down,
            (Dir::Right, Dir::Up) => Dir::Up,
            (Dir::Right, Dir::Down) => Dir::Down,
            (s, _) => *s,
        }
    }

    pub fn is_perpendicular(&self, other: Self) -> bool {
        matches!(
            (self, other),
            (Dir::Right | Dir::Left, Dir::Up | Dir::Down)
                | (Dir::Up | Dir::Down, Dir::Right | Dir::Left)
        )
    }
}

pub struct SnakeGame {
    pub width: i64,
    pub height: i64,
    pub food: (i64, i64),
    pub snake: VecDeque<(i64, i64)>,
    dir: Dir,

    /// a queue of turn directions. one is consuemd on each call to `advance()`
    pending_turns: VecDeque<Dir>,
}

impl SnakeGame {
    pub fn new(width: i64, height: i64) -> Self {
        let center = (width / 2, height / 2);
        let snake = vec![center, (center.0 - 1, center.1), (center.0 - 2, center.1)].into();
        Self {
            width,
            height,
            food: Self::gen_food_loc(width, height, &snake),
            snake,
            dir: Dir::Right,
            pending_turns: VecDeque::new(),
        }
    }

    fn gen_food_loc(width: i64, height: i64, exclude: &VecDeque<(i64, i64)>) -> (i64, i64) {
        let mut pt = (
            rand::rng().random_range(0..width),
            rand::rng().random_range(0..height),
        );

        while exclude.contains(&pt) {
            pt = (
                rand::rng().random_range(0..width),
                rand::rng().random_range(0..height),
            );
        }

        pt
    }

    fn head_pos(&self) -> (i64, i64) {
        self.snake[0]
    }

    pub fn advance(&mut self) {
        // turn if there's a pending turn
        if let Some(dir) = self.pending_turns.pop_front() {
            self.dir = self.dir.turn(dir);
        }

        // move forward
        let mut new_head = self.head_pos();
        match self.dir {
            Dir::Up => new_head.1 -= 1,
            Dir::Down => new_head.1 += 1,
            Dir::Left => new_head.0 -= 1,
            Dir::Right => new_head.0 += 1,
        }

        let last = self.snake.pop_back();
        self.snake.push_front(new_head);

        // eat?
        if self.head_pos() == self.food {
            self.food = Self::gen_food_loc(self.width, self.height, &self.snake);
            if let Some(last) = last {
                self.snake.push_back(last);
            }
        }
    }

    pub fn turn(&mut self, dir: Dir) {
        // don't queue too many turns at once
        const MAX_PENDING_TURNS: usize = 2;
        if self.pending_turns.len() >= MAX_PENDING_TURNS {
            return;
        }

        // make sure incoming turn is perpendicular to what will be the
        // snake's direction when this is applied
        let effective_dir = self.pending_turns.back().copied().unwrap_or(self.dir);

        // only queue this if it's a real turn
        if dir.is_perpendicular(effective_dir) {
            self.pending_turns.push_back(dir);
        }
    }

    pub fn is_dead(&self) -> bool {
        self.head_pos().0 < 0
            || self.head_pos().0 >= self.width
            || self.head_pos().1 < 0
            || self.head_pos().1 >= self.height
            || self.snake.iter().skip(1).any(|&x| x == self.head_pos())
    }

    pub fn score(&self) -> u64 {
        self.snake.len().saturating_sub(3) as u64
    }
}
