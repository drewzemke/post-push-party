use std::time::{Duration, Instant};

use rand::RngExt;

pub const SPEED_CELLS_SEC: f64 = 30.0;
pub const INITIAL_BAR_WIDTH: f64 = 20.0;
pub const GAME_WIDTH: usize = 60;
pub const HUE_STEP: f64 = 10.;

const FLASH_COUNT: usize = 6;
const FLASH_DURATION: Duration = Duration::from_millis(800);

/// computes the left and right column bounds of the game area, centered in `cols`
pub fn game_bounds(cols: usize) -> (usize, usize) {
    let left = cols.saturating_sub(GAME_WIDTH) / 2;
    (left, cols - left)
}

pub enum Input {
    Cut,
}

enum GameState {
    Running,
    GameOver,
}

pub enum CutResult {
    Perfect,
    Normal,
    Miss,
}

#[derive(Clone)]
pub struct Bar {
    pub pos: f64,
    pub width: f64,
    pub hue: f64,

    /// used to display how much was deleted due the player missing an exact cut
    /// negative is leftside, positive is right
    pub deleted: f64,
}

impl Bar {
    fn new(pos: f64, width: f64, hue: f64, deleted: f64) -> Self {
        Self {
            pos,
            width,
            hue,
            deleted,
        }
    }

    #[inline]
    fn left(&self) -> f64 {
        self.pos
    }

    #[inline]
    fn right(&self) -> f64 {
        self.pos + self.width
    }

    #[inline]
    pub fn quantized_left(&self) -> f64 {
        (self.left() * 2.).floor() / 2.
    }

    pub fn quantized_right(&self) -> f64 {
        (self.right() * 2.).floor() / 2.
    }

    /// creates the bar that results from the intersection of two bars,
    /// returns None if there's no intersection
    /// takes the hue of self
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        if self.quantized_right() <= other.quantized_left()
            || self.quantized_left() >= other.quantized_right()
        {
            // no overlap (edges merely touching count as a miss, not a zero-width bar)
            return None;
        }

        let left = f64::max(self.quantized_left(), other.quantized_left());
        let right = f64::min(self.quantized_right(), other.quantized_right());

        // NOTE: this computation relies on the fact that `self` is always *smaller* than `other`
        let deleted = if other.quantized_left() - self.quantized_left() > 0. {
            self.quantized_left() - other.quantized_left()
        } else {
            self.quantized_right() - other.quantized_right()
        };

        Some(Self {
            pos: left,
            width: right - left,
            hue: self.hue,
            deleted,
        })
    }
}

pub struct StackGame {
    pub current: Bar,
    pub stack: Vec<Bar>,
    moving_right: bool,
    starting_hue: f64,

    bounds: (usize, usize),

    state: GameState,
    pub perfect_cut_at: Option<Instant>,
    pub perfect_run: usize,

    multiplier: f64,
}

impl StackGame {
    pub fn new(bounds: (usize, usize)) -> Self {
        let hue = rand::rng().random_range(0.0..360.0);
        let first_bar = Bar::new(bounds.0 as f64, INITIAL_BAR_WIDTH, hue, 0.);

        Self {
            current: first_bar,
            stack: Vec::new(),
            moving_right: true,
            starting_hue: hue,
            bounds,
            state: GameState::Running,
            perfect_cut_at: None,
            perfect_run: 0,
            multiplier: 1.0,
        }
    }

    /// updates game state
    pub fn tick(&mut self, dt: Duration, input: Option<Input>) -> Option<CutResult> {
        match self.state {
            GameState::Running => match input {
                Some(Input::Cut) => {
                    let result = self.cut();
                    if matches!(result, CutResult::Miss) {
                        self.state = GameState::GameOver;
                    }
                    Some(result)
                }
                _ => {
                    self.update_position(dt);
                    None
                }
            },
            GameState::GameOver => None,
        }
    }

    fn update_position(&mut self, dt: Duration) {
        let dx = SPEED_CELLS_SEC * dt.as_secs_f64();

        self.current.pos += if self.moving_right { dx } else { -dx };

        // change direction if necessary
        if self.moving_right && self.current.pos + self.current.width > self.bounds.1 as f64 {
            self.moving_right = false;
            self.current.pos = self.bounds.1 as f64 - self.current.width - 0.1;
        } else if !self.moving_right && self.current.pos < self.bounds.0 as f64 {
            self.moving_right = true;
            self.current.pos = self.bounds.0 as f64 + 0.1;
        }
    }

    /// cuts the current bar based on its overlap with the previous one, pushes the overlap onto the stack as a new bar,
    /// and creates a new current bar
    fn cut(&mut self) -> CutResult {
        let top_bar = self
            .stack
            .last()
            .cloned()
            .unwrap_or_else(|| self.default_bar(0));

        let Some(intersection) = self.current.intersect(&top_bar) else {
            return CutResult::Miss;
        };

        let next_hue = self.current.hue + HUE_STEP;
        let new_bar = if self.moving_right {
            Bar::new(
                self.bounds.1 as f64 - intersection.width,
                intersection.width,
                next_hue,
                0.,
            )
        } else {
            Bar::new(self.bounds.0 as f64, intersection.width, next_hue, 0.)
        };

        let is_perfect = top_bar.width == new_bar.width;

        let _ = std::mem::replace(&mut self.current, new_bar);

        self.stack.push(intersection);
        self.moving_right = !self.moving_right;

        if is_perfect {
            self.perfect_cut_at = Some(Instant::now());
            self.perfect_run += 1;
            self.multiplier += 0.1 * self.perfect_run as f64;
            CutResult::Perfect
        } else {
            self.perfect_cut_at = None;
            self.perfect_run = 0;
            CutResult::Normal
        }
    }

    /// returns the bar that's centered in the screen, with the default width
    pub fn default_bar(&self, offset: usize) -> Bar {
        Bar {
            pos: self.bounds.0 as f64 + (self.game_width() as f64 - INITIAL_BAR_WIDTH) / 2.,
            width: INITIAL_BAR_WIDTH,
            hue: self.starting_hue - HUE_STEP * offset as f64,
            deleted: 0.,
        }
    }

    pub fn is_game_over(&self) -> bool {
        matches!(self.state, GameState::GameOver)
    }

    pub fn score(&self) -> u64 {
        (self.raw_points() as f64 * self.multiplier()) as u64
    }

    pub fn raw_points(&self) -> usize {
        self.stack.len()
    }

    pub fn multiplier(&self) -> f64 {
        self.multiplier
    }

    pub fn flash_factor(&self) -> f64 {
        let Some(t) = self.perfect_cut_at else {
            return 0.;
        };
        let elapsed = t.elapsed().as_secs_f64();
        let total = FLASH_DURATION.as_secs_f64();
        if elapsed >= total {
            return 0.;
        }
        let phase = elapsed / total * FLASH_COUNT as f64 * 2. * std::f64::consts::PI;
        phase.sin().max(0.)
    }

    fn game_width(&self) -> usize {
        self.bounds.1 - self.bounds.0
    }
}
